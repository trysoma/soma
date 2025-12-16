use clap::{Args, Subcommand};
use comfy_table::{Cell, Table};
use inquire::{Select, Text};
use shared::error::CommonError;
use soma_api_client::apis::identity_api;
use soma_api_client::models;
use tracing::debug;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

#[derive(Args, Debug, Clone)]
pub struct AuthParams {
    #[command(subcommand)]
    pub command: AuthCommands,

    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AuthCommands {
    /// Add a new user authentication flow configuration
    Add {
        #[command(subcommand)]
        add_command: AuthAddCommands,
    },
    /// Remove a user authentication flow configuration
    #[command(name = "rm")]
    Remove {
        /// The configuration ID to remove
        id: String,
    },
    /// List all user authentication flow configurations
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AuthAddCommands {
    /// Add an OAuth authorization code flow configuration
    #[command(name = "oauth")]
    Oauth {
        /// Unique ID for this configuration (lowercase letters, numbers, and hyphens only)
        id: String,
        /// Use PKCE (Proof Key for Code Exchange) for enhanced security
        #[arg(long)]
        pkce: bool,
    },
    /// Add an OIDC authorization code flow configuration
    #[command(name = "oidc")]
    Oidc {
        /// Unique ID for this configuration (lowercase letters, numbers, and hyphens only)
        id: String,
        /// Use PKCE (Proof Key for Code Exchange) for enhanced security
        #[arg(long)]
        pkce: bool,
    },
}

pub async fn cmd_auth(params: AuthParams, _cli_config: &mut CliConfig) -> Result<(), CommonError> {
    match params.command {
        AuthCommands::Add { add_command } => match add_command {
            AuthAddCommands::Oauth { id, pkce } => {
                cmd_auth_add_oauth(id, pkce, &params.api_url, params.timeout_secs).await
            }
            AuthAddCommands::Oidc { id, pkce } => {
                cmd_auth_add_oidc(id, pkce, &params.api_url, params.timeout_secs).await
            }
        },
        AuthCommands::Remove { id } => {
            cmd_auth_remove(id, &params.api_url, params.timeout_secs).await
        }
        AuthCommands::List => cmd_auth_list(&params.api_url, params.timeout_secs).await,
    }
}

async fn cmd_auth_add_oauth(
    id: String,
    use_pkce: bool,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    let flow_type = if use_pkce {
        "OAuth Authorization Code Flow with PKCE"
    } else {
        "OAuth Authorization Code Flow"
    };

    println!("Add {flow_type} Configuration");
    println!("{}", "=".repeat(flow_type.len() + 20));
    println!();

    // Collect OAuth configuration details
    let oauth_config = collect_oauth_config(&id).await?;
    let mapping = collect_token_mapping().await?;

    // Build the request
    let oauth_value = models::OauthConfig {
        id: id.clone(),
        authorization_endpoint: oauth_config.authorization_endpoint,
        token_endpoint: oauth_config.token_endpoint,
        jwks_endpoint: oauth_config.jwks_endpoint,
        client_id: oauth_config.client_id,
        client_secret: oauth_config.client_secret,
        scopes: oauth_config.scopes,
        introspect_url: None,
        mapping,
    };

    let config = if use_pkce {
        models::UserAuthFlowConfig::UserAuthFlowConfigOneOf3(models::UserAuthFlowConfigOneOf3 {
            r#type: models::user_auth_flow_config_one_of_3::Type::OauthAuthorizationCodePkceFlow,
            value: oauth_value,
        })
    } else {
        models::UserAuthFlowConfig::UserAuthFlowConfigOneOf1(models::UserAuthFlowConfigOneOf1 {
            r#type: models::user_auth_flow_config_one_of_1::Type::OauthAuthorizationCodeFlow,
            value: oauth_value,
        })
    };

    // Wait for API server and create config
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    let params = models::CreateUserAuthFlowConfigParams { config };

    identity_api::route_create_user_auth_flow_config(&api_config, params)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to create user auth flow config: {e:?}"
            ))
        })?;

    debug!("User auth flow configuration '{}' created", id);
    println!();
    println!("Successfully added user auth flow configuration: {id}");
    println!("The configuration has been synced to soma.yaml.");

    Ok(())
}

async fn cmd_auth_add_oidc(
    id: String,
    use_pkce: bool,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    let flow_type = if use_pkce {
        "OIDC Authorization Code Flow with PKCE"
    } else {
        "OIDC Authorization Code Flow"
    };

    println!("Add {flow_type} Configuration");
    println!("{}", "=".repeat(flow_type.len() + 20));
    println!();

    // Collect OIDC-specific configuration
    let discovery_endpoint = Text::new("Discovery endpoint URL (optional):")
        .with_help_message(
            "OpenID Connect discovery URL (e.g., 'https://provider.com/.well-known/openid-configuration')",
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let discovery_endpoint = if discovery_endpoint.trim().is_empty() {
        None
    } else {
        Some(discovery_endpoint)
    };

    let userinfo_endpoint = Text::new("Userinfo endpoint URL (optional):")
        .with_help_message(
            "URL to fetch user profile information (e.g., 'https://provider.com/oauth/userinfo')",
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let userinfo_endpoint = if userinfo_endpoint.trim().is_empty() {
        None
    } else {
        Some(userinfo_endpoint)
    };

    let introspect_url = Text::new("Token introspection URL (optional):")
        .with_help_message(
            "RFC 7662 token introspection endpoint for opaque access tokens (e.g., 'https://provider.com/oauth/introspect')",
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let introspect_url = if introspect_url.trim().is_empty() {
        None
    } else {
        Some(introspect_url)
    };

    // Collect base OAuth configuration
    let oauth_config = collect_oauth_config(&id).await?;
    let mapping = collect_token_mapping().await?;

    // Build the request - use the same mapping for both base OAuth and OIDC
    let base_oauth = models::OauthConfig {
        id: id.clone(),
        authorization_endpoint: oauth_config.authorization_endpoint,
        token_endpoint: oauth_config.token_endpoint,
        jwks_endpoint: oauth_config.jwks_endpoint,
        client_id: oauth_config.client_id,
        client_secret: oauth_config.client_secret,
        scopes: oauth_config.scopes,
        introspect_url: None,
        mapping: mapping.clone(),
    };

    let oidc_value = models::OidcConfig {
        id: id.clone(),
        base_config: base_oauth,
        discovery_endpoint: discovery_endpoint.map(Some),
        userinfo_endpoint: userinfo_endpoint.map(Some),
        introspect_url: introspect_url.map(Some),
        mapping,
    };

    let config = if use_pkce {
        models::UserAuthFlowConfig::UserAuthFlowConfigOneOf2(models::UserAuthFlowConfigOneOf2 {
            r#type: models::user_auth_flow_config_one_of_2::Type::OidcAuthorizationCodePkceFlow,
            value: oidc_value,
        })
    } else {
        models::UserAuthFlowConfig::UserAuthFlowConfigOneOf(models::UserAuthFlowConfigOneOf {
            r#type: models::user_auth_flow_config_one_of::Type::OidcAuthorizationCodeFlow,
            value: oidc_value,
        })
    };

    // Wait for API server and create config
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    let params = models::CreateUserAuthFlowConfigParams { config };

    identity_api::route_create_user_auth_flow_config(&api_config, params)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to create user auth flow config: {e:?}"
            ))
        })?;

    debug!("User auth flow configuration '{}' created", id);
    println!();
    println!("Successfully added user auth flow configuration: {id}");
    println!("The configuration has been synced to soma.yaml.");

    Ok(())
}

async fn cmd_auth_remove(id: String, api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    identity_api::route_delete_user_auth_flow_config(&api_config, &id)
        .await
        .map_err(|e| {
            if let soma_api_client::apis::Error::ResponseError(resp) = &e {
                if resp.status.as_u16() == 404 {
                    return CommonError::NotFound {
                        msg: format!("User auth flow configuration '{id}' not found"),
                        lookup_id: id.clone(),
                        source: None,
                    };
                }
            }
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to delete user auth flow config: {e:?}"
            ))
        })?;

    debug!("User auth flow configuration '{}' removed", id);
    println!("Successfully removed user auth flow configuration: {id}");

    Ok(())
}

async fn cmd_auth_list(api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    let result = identity_api::route_list_user_auth_flow_configs(&api_config, None, None, None)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to list user auth flow configs: {e:?}"
            ))
        })?;

    if result.items.is_empty() {
        println!("No user authentication flow configurations found.");
        println!();
        println!("Use 'soma auth add oauth <id>' to add an OAuth configuration.");
        println!("Use 'soma auth add oidc <id>' to add an OIDC configuration.");
        println!("Add --pkce flag for PKCE-enabled flows.");
    } else {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("ID"),
            Cell::new("Type"),
            Cell::new("Client ID"),
            Cell::new("Created At"),
        ]);

        for item in result.items {
            let (config_type, client_id, config_id) = match &item.config {
                models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf(c) => (
                    "OIDC",
                    c.oidc_authorization_code_flow.base_config.client_id.clone(),
                    c.oidc_authorization_code_flow.id.clone(),
                ),
                models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf1(c) => (
                    "OAuth",
                    c.oauth_authorization_code_flow.client_id.clone(),
                    c.oauth_authorization_code_flow.id.clone(),
                ),
                models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf2(c) => (
                    "OIDC + PKCE",
                    c.oidc_authorization_code_pkce_flow
                        .base_config
                        .client_id
                        .clone(),
                    c.oidc_authorization_code_pkce_flow.id.clone(),
                ),
                models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf3(c) => (
                    "OAuth + PKCE",
                    c.oauth_authorization_code_pkce_flow.client_id.clone(),
                    c.oauth_authorization_code_pkce_flow.id.clone(),
                ),
            };

            table.add_row(vec![
                Cell::new(&config_id),
                Cell::new(config_type),
                Cell::new(&client_id),
                Cell::new(&item.created_at),
            ]);
        }

        println!("{table}");
    }

    Ok(())
}

// Helper struct for collecting OAuth config
struct OauthConfigInput {
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_endpoint: String,
    client_id: String,
    client_secret: String,
    scopes: Vec<String>,
}

async fn collect_oauth_config(_id: &str) -> Result<OauthConfigInput, CommonError> {
    println!("OAuth Endpoints");
    println!("---------------");

    let authorization_endpoint = Text::new("Authorization endpoint URL:")
        .with_help_message("URL where users are redirected to authenticate (e.g., 'https://provider.com/oauth/authorize')")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if !authorization_endpoint.starts_with("https://")
        && !authorization_endpoint.starts_with("http://")
    {
        return Err(CommonError::InvalidRequest {
            msg: "Authorization endpoint must be a valid HTTP(S) URL".to_string(),
            source: None,
        });
    }

    let token_endpoint = Text::new("Token endpoint URL:")
        .with_help_message("URL to exchange authorization code for tokens (e.g., 'https://provider.com/oauth/token')")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if !token_endpoint.starts_with("https://") && !token_endpoint.starts_with("http://") {
        return Err(CommonError::InvalidRequest {
            msg: "Token endpoint must be a valid HTTP(S) URL".to_string(),
            source: None,
        });
    }

    let jwks_endpoint = Text::new("JWKS endpoint URL:")
        .with_help_message("URL to fetch JSON Web Key Set for token validation (e.g., 'https://provider.com/.well-known/jwks.json')")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if !jwks_endpoint.starts_with("https://") && !jwks_endpoint.starts_with("http://") {
        return Err(CommonError::InvalidRequest {
            msg: "JWKS endpoint must be a valid HTTP(S) URL".to_string(),
            source: None,
        });
    }

    println!();
    println!("Client Credentials");
    println!("------------------");

    let client_id = Text::new("Client ID:")
        .with_help_message("OAuth client ID from your identity provider")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if client_id.trim().is_empty() {
        return Err(CommonError::InvalidRequest {
            msg: "Client ID cannot be empty".to_string(),
            source: None,
        });
    }

    let client_secret = Text::new("Client Secret:")
        .with_help_message("OAuth client secret from your identity provider (will be encrypted)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if client_secret.trim().is_empty() {
        return Err(CommonError::InvalidRequest {
            msg: "Client secret cannot be empty".to_string(),
            source: None,
        });
    }

    println!();
    println!("Scopes");
    println!("------");

    let scopes_input = Text::new("Scopes (comma-separated):")
        .with_default("openid,profile,email")
        .with_help_message("OAuth scopes to request (e.g., 'openid,profile,email')")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let scopes: Vec<String> = scopes_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(OauthConfigInput {
        authorization_endpoint,
        token_endpoint,
        jwks_endpoint,
        client_id,
        client_secret,
        scopes,
    })
}

async fn collect_token_mapping() -> Result<models::TokenMapping, CommonError> {
    println!();
    println!("Token Mapping Configuration");
    println!("---------------------------");

    // Subject field
    let sub_source = select_mapping_source("user ID / subject")?;
    let sub_field = Text::new("Subject field name:")
        .with_default("sub")
        .with_help_message("Field containing the user's unique identifier")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let sub_field = create_mapping_source(&sub_source, sub_field);

    // Issuer field
    let issuer_source = select_mapping_source("issuer")?;
    let issuer_field = Text::new("Issuer field name:")
        .with_default("iss")
        .with_help_message("Field containing the token issuer")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let issuer_field = create_mapping_source(&issuer_source, issuer_field);

    // Audience field
    let audience_source = select_mapping_source("audience")?;
    let audience_field = Text::new("Audience field name:")
        .with_default("aud")
        .with_help_message("Field containing the token audience")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let audience_field = create_mapping_source(&audience_source, audience_field);

    // Email field (optional)
    let has_email = Select::new("Does the token contain an email field?", vec!["Yes", "No"])
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let email_field = if has_email == "Yes" {
        let email_source = select_mapping_source("email")?;
        let field = Text::new("Email field name:")
            .with_default("email")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        Some(Some(create_mapping_source(&email_source, field)))
    } else {
        None
    };

    // Scopes field (optional)
    let has_scopes = Select::new("Does the token contain scopes?", vec!["No", "Yes"])
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let scopes_field = if has_scopes == "Yes" {
        let scopes_source = select_mapping_source("scopes")?;
        let field = Text::new("Scopes field name:")
            .with_default("scope")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        Some(Some(create_mapping_source(&scopes_source, field)))
    } else {
        None
    };

    // Groups field (optional)
    let has_groups = Select::new("Does the token contain groups?", vec!["No", "Yes"])
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let groups_field = if has_groups == "Yes" {
        let groups_source = select_mapping_source("groups")?;
        let field = Text::new("Groups field name:")
            .with_default("groups")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        Some(Some(create_mapping_source(&groups_source, field)))
    } else {
        None
    };

    // Role mappings
    let mut scope_to_role_mappings: Vec<models::ScopeToRoleMapping> = Vec::new();
    let mut group_to_role_mappings: Vec<models::GroupToRoleMapping> = Vec::new();
    let mut scope_to_group_mappings: Vec<models::ScopeToGroupMapping> = Vec::new();

    // Scope to role mappings
    if has_scopes == "Yes" {
        let map_scopes_to_roles = Select::new(
            "Would you like to map scopes to Soma roles?",
            vec!["No", "Yes"],
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

        if map_scopes_to_roles == "Yes" {
            let roles = vec![
                ("Admin", "admin"),
                ("Maintainer", "maintainer"),
                ("ReadOnlyMaintainer", "read-only-maintainer"),
                ("Agent", "agent"),
                ("User", "user"),
            ];

            for (role_variant, role_display) in roles {
                let scopes_for_role = Text::new(&format!(
                    "Which scopes should map to {role_display} role? (comma-separated, leave empty to skip)"
                ))
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if !scopes_for_role.trim().is_empty() {
                    for scope in scopes_for_role
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                    {
                        scope_to_role_mappings.push(models::ScopeToRoleMapping {
                            scope: scope.to_string(),
                            role: string_to_role(role_variant),
                        });
                    }
                }
            }
        }

        // Scope to group mappings
        let map_scopes_to_groups =
            Select::new("Would you like to map scopes to groups?", vec!["No", "Yes"])
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

        if map_scopes_to_groups == "Yes" {
            loop {
                let scope = Text::new("Scope name (press Enter to finish):")
                    .prompt()
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}"))
                    })?;

                if scope.trim().is_empty() {
                    break;
                }

                let group = Text::new("Group name:").prompt().map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}"))
                })?;

                if !group.trim().is_empty() {
                    scope_to_group_mappings.push(models::ScopeToGroupMapping { scope, group });
                    println!("Added scope-to-group mapping.");
                }

                let add_another =
                    Select::new("Add another scope-to-group mapping?", vec!["Yes", "No"])
                        .prompt()
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}"))
                        })?;

                if add_another == "No" {
                    break;
                }
            }
        }
    }

    // Group to role mappings
    if has_groups == "Yes" {
        let map_groups_to_roles = Select::new(
            "Would you like to map groups to Soma roles?",
            vec!["No", "Yes"],
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

        if map_groups_to_roles == "Yes" {
            let roles = vec![
                ("Admin", "admin"),
                ("Maintainer", "maintainer"),
                ("ReadOnlyMaintainer", "read-only-maintainer"),
                ("Agent", "agent"),
                ("User", "user"),
            ];

            for (role_variant, role_display) in roles {
                let groups_for_role = Text::new(&format!(
                    "Which groups should map to {role_display} role? (comma-separated, leave empty to skip)"
                ))
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if !groups_for_role.trim().is_empty() {
                    for group in groups_for_role
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                    {
                        group_to_role_mappings.push(models::GroupToRoleMapping {
                            group: group.to_string(),
                            role: string_to_role(role_variant),
                        });
                    }
                }
            }
        }
    }

    Ok(models::TokenMapping {
        r#type: models::token_mapping::Type::JwtTemplate,
        value: models::JwtTokenMappingConfig {
            issuer_field,
            audience_field,
            scopes_field,
            sub_field,
            email_field,
            groups_field,
            group_to_role_mappings,
            scope_to_role_mappings,
            scope_to_group_mappings,
        },
    })
}

fn select_mapping_source(field_name: &str) -> Result<String, CommonError> {
    let options = vec!["ID Token", "Userinfo", "Access Token"];
    Select::new(
        &format!("Where is the {field_name} field located?"),
        options,
    )
    .prompt()
    .map(|s| s.to_string())
    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))
}

fn create_mapping_source(source: &str, field: String) -> models::MappingSourceString {
    match source {
        "ID Token" => models::MappingSourceString::MappingSourceStringOneOf(
            models::MappingSourceStringOneOf {
                field,
                r#type: models::mapping_source_string_one_of::Type::IdToken,
            },
        ),
        "Userinfo" => models::MappingSourceString::MappingSourceStringOneOf1(
            models::MappingSourceStringOneOf1 {
                field,
                r#type: models::mapping_source_string_one_of_1::Type::Userinfo,
            },
        ),
        "Access Token" => models::MappingSourceString::MappingSourceStringOneOf2(
            models::MappingSourceStringOneOf2 {
                field,
                r#type: models::mapping_source_string_one_of_2::Type::AccessToken,
            },
        ),
        _ => models::MappingSourceString::MappingSourceStringOneOf(
            models::MappingSourceStringOneOf {
                field,
                r#type: models::mapping_source_string_one_of::Type::IdToken,
            },
        ),
    }
}

fn string_to_role(role: &str) -> models::Role {
    match role {
        "Admin" => models::Role::Admin,
        "Maintainer" => models::Role::Maintainer,
        "ReadOnlyMaintainer" => models::Role::ReadOnlyMaintainer,
        "Agent" => models::Role::Agent,
        "User" => models::Role::User,
        _ => models::Role::User,
    }
}
