use clap::{Args, Subcommand};
use inquire::{Select, Text};
use shared::error::CommonError;
use shared::soma_agent_definition::{
    GroupToRoleMappingYaml, JwtMappingConfigYaml, JwtTemplateConfigYaml, JwtValidationConfigYaml,
    ScopeToGroupMappingYaml, ScopeToRoleMappingYaml, TokenLocationYaml,
};
use soma_api_client::apis::identity_api;
use tracing::info;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

#[derive(Args, Debug, Clone)]
pub struct StsParams {
    #[command(subcommand)]
    pub command: StsCommands,

    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StsCommands {
    /// Add a new STS configuration
    Add {
        #[command(subcommand)]
        add_command: StsAddCommands,
    },
    /// Remove an STS configuration
    #[command(name = "rm")]
    Remove {
        /// The STS configuration ID to remove
        id: String,
    },
    /// List all STS configurations
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StsAddCommands {
    /// Add a JWT template configuration using guided prompts
    #[command(name = "from-template")]
    FromTemplate,
}

pub async fn cmd_sts(params: StsParams, _cli_config: &mut CliConfig) -> Result<(), CommonError> {
    match params.command {
        StsCommands::Add { add_command } => match add_command {
            StsAddCommands::FromTemplate => {
                cmd_sts_add_from_template(&params.api_url, params.timeout_secs).await
            }
        },
        StsCommands::Remove { id } => cmd_sts_remove(id, &params.api_url, params.timeout_secs).await,
        StsCommands::List => cmd_sts_list(&params.api_url, params.timeout_secs).await,
    }
}

async fn cmd_sts_add_from_template(
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    println!("Add STS JWT Template Configuration");
    println!("===================================");
    println!();

    // Configuration ID
    let id = Text::new("Configuration ID:")
        .with_help_message("A unique identifier for this STS configuration (e.g., 'clerk', 'auth0')")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if id.trim().is_empty() {
        return Err(CommonError::InvalidRequest {
            msg: "Configuration ID cannot be empty".to_string(),
            source: None,
        });
    }

    // JWKS URI
    let jwks_uri = Text::new("JWKS URI:")
        .with_help_message("URL to fetch the JSON Web Key Set (e.g., 'https://your-domain.clerk.accounts.dev/.well-known/jwks.json')")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if !jwks_uri.starts_with("https://") && !jwks_uri.starts_with("http://") {
        return Err(CommonError::InvalidRequest {
            msg: "JWKS URI must be a valid HTTP(S) URL".to_string(),
            source: None,
        });
    }

    // Token Location
    let token_location_options = vec!["Header", "Cookie"];
    let token_location_choice = Select::new("Where should the token be read from?", token_location_options)
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let token_location = if token_location_choice == "Header" {
        let header_name = Text::new("Header name:")
            .with_default("authorization")
            .with_help_message("The HTTP header containing the token (e.g., 'authorization' for 'Authorization: Bearer <token>')")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        TokenLocationYaml::Header { name: header_name }
    } else {
        let cookie_name = Text::new("Cookie name:")
            .with_help_message("The name of the cookie containing the token")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        TokenLocationYaml::Cookie { name: cookie_name }
    };

    println!();
    println!("Token Claims Configuration");
    println!("--------------------------");

    // Subject field
    let sub_field = Text::new("What is the user ID / subject field name in the token?")
        .with_default("sub")
        .with_help_message("JWT claim containing the user's unique identifier")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    // Email field (optional)
    let email_field_input = Text::new("What is the email field name? (leave empty to skip)")
        .with_help_message("JWT claim containing the user's email")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let email_field = if email_field_input.trim().is_empty() {
        None
    } else {
        Some(email_field_input)
    };

    println!();
    println!("Token Validation");
    println!("----------------");

    // Issuer (optional)
    let issuer_input = Text::new("What is the expected issuer (iss) value to validate against?")
        .with_help_message("Expected issuer value (leave empty to skip validation)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let issuer = if issuer_input.trim().is_empty() {
        None
    } else {
        Some(issuer_input)
    };

    // Valid Audiences (optional)
    let audiences_input = Text::new("What are valid audience (aud) values to validate against? (comma-separated)")
        .with_help_message("Expected audience values (leave empty to skip validation)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let valid_audiences = if audiences_input.trim().is_empty() {
        None
    } else {
        Some(
            audiences_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };

    // --- Scopes Section ---
    println!();
    println!("Scopes Configuration");
    println!("--------------------");

    let has_scopes = Select::new(
        "Are there scopes in the token you'd like to validate against?",
        vec!["Yes", "No"],
    )
    .prompt()
    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let mut scopes_field: Option<String> = None;
    let mut required_scopes: Option<Vec<String>> = None;
    let mut scope_to_role_mappings: Vec<ScopeToRoleMappingYaml> = Vec::new();
    let mut scope_to_group_mappings: Vec<ScopeToGroupMappingYaml> = Vec::new();

    if has_scopes == "Yes" {
        // Scopes field name
        let scopes_field_input = Text::new("What is the name of the scopes field?")
            .with_default("scope")
            .with_help_message("JWT claim containing the scopes (often 'scope' or 'scp')")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        scopes_field = Some(scopes_field_input);

        // Required scopes
        let required_scopes_input = Text::new("What scopes need to be present? (comma-separated, leave empty to skip)")
            .with_help_message("Scopes that must be present in the token for access")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        required_scopes = if required_scopes_input.trim().is_empty() {
            None
        } else {
            Some(
                required_scopes_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            )
        };

        // Scope to role mappings
        let map_scopes_to_roles = Select::new(
            "Would you like to map scopes to Soma roles?",
            vec!["No", "Yes"],
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

        if map_scopes_to_roles == "Yes" {
            let roles = vec![
                ("admin", "Admin"),
                ("maintainer", "Maintainer"),
                ("read-only-maintainer", "Read-Only Maintainer"),
                ("agent", "Agent"),
                ("user", "User"),
            ];

            for (role_value, role_display) in roles {
                let scopes_for_role = Text::new(&format!(
                    "Which scopes should map to {} role? (comma-separated, leave empty to skip)",
                    role_display
                ))
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if !scopes_for_role.trim().is_empty() {
                    for scope in scopes_for_role.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        scope_to_role_mappings.push(ScopeToRoleMappingYaml {
                            scope: scope.to_string(),
                            role: role_value.to_string(),
                        });
                    }
                }
            }
        }

        // Scope to group mappings
        let map_scopes_to_groups = Select::new(
            "Would you like to map scopes to groups?",
            vec!["No", "Yes"],
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

        if map_scopes_to_groups == "Yes" {
            loop {
                let scope = Text::new("Scope name (press Enter to finish):")
                    .with_help_message("The scope value to match")
                    .prompt()
                    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if scope.trim().is_empty() {
                    break;
                }

                let group = Text::new("Group name:")
                    .with_help_message("The internal group to assign when this scope is present")
                    .prompt()
                    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if !group.trim().is_empty() {
                    scope_to_group_mappings.push(ScopeToGroupMappingYaml {
                        scope,
                        group,
                    });
                    println!("Added scope-to-group mapping.");
                }

                let add_another = Select::new("Add another scope-to-group mapping?", vec!["Yes", "No"])
                    .prompt()
                    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if add_another == "No" {
                    break;
                }
            }
        }
    }

    // --- Groups Section ---
    println!();
    println!("Groups Configuration");
    println!("--------------------");

    let has_groups = Select::new(
        "Are there user groups present in the token?",
        vec!["No", "Yes"],
    )
    .prompt()
    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    let mut groups_field: Option<String> = None;
    let mut required_groups: Option<Vec<String>> = None;
    let mut group_to_role_mappings: Vec<GroupToRoleMappingYaml> = Vec::new();

    if has_groups == "Yes" {
        // Groups field name
        let groups_field_input = Text::new("What is the name of the groups field?")
            .with_help_message("JWT claim containing the user's groups")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        groups_field = Some(groups_field_input);

        // Required groups (optional)
        let required_groups_input = Text::new("What groups must be present? (comma-separated, leave empty to skip)")
            .with_help_message("Groups the user must belong to for access")
            .prompt()
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
        required_groups = if required_groups_input.trim().is_empty() {
            None
        } else {
            Some(
                required_groups_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            )
        };

        // Group to role mappings
        let map_groups_to_roles = Select::new(
            "Would you like to map groups to Soma roles?",
            vec!["No", "Yes"],
        )
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

        if map_groups_to_roles == "Yes" {
            let roles = vec![
                ("admin", "Admin"),
                ("maintainer", "Maintainer"),
                ("read-only-maintainer", "Read-Only Maintainer"),
                ("agent", "Agent"),
                ("user", "User"),
            ];

            for (role_value, role_display) in roles {
                let groups_for_role = Text::new(&format!(
                    "Which groups should map to {} role? (comma-separated, leave empty to skip)",
                    role_display
                ))
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

                if !groups_for_role.trim().is_empty() {
                    for group in groups_for_role.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        group_to_role_mappings.push(GroupToRoleMappingYaml {
                            group: group.to_string(),
                            role: role_value.to_string(),
                        });
                    }
                }
            }
        }
    }

    // Build the JWT template configuration as JSON value
    let jwt_template_config = JwtTemplateConfigYaml {
        jwks_uri,
        token_location,
        validation: JwtValidationConfigYaml {
            issuer,
            valid_audiences,
            required_scopes,
            required_groups,
        },
        mapping: JwtMappingConfigYaml {
            issuer_field: "iss".to_string(),
            audience_field: "aud".to_string(),
            sub_field,
            email_field,
            groups_field,
            scopes_field,
        },
        group_to_role_mappings: if group_to_role_mappings.is_empty() {
            None
        } else {
            Some(group_to_role_mappings)
        },
        scope_to_role_mappings: if scope_to_role_mappings.is_empty() {
            None
        } else {
            Some(scope_to_role_mappings)
        },
        scope_to_group_mappings: if scope_to_group_mappings.is_empty() {
            None
        } else {
            Some(scope_to_group_mappings)
        },
    };

    // Wait for API server and create config
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Convert JwtTemplateConfigYaml to the API model
    let mapping_template = serde_json::to_value(&jwt_template_config)
        .and_then(|v| serde_json::from_value(v))
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to convert JWT template config: {e}")))?;

    let params = soma_api_client::models::StsTokenConfig::StsTokenConfigOneOf(
        soma_api_client::models::StsTokenConfigOneOf {
            jwt_template: soma_api_client::models::JwtTemplateModeConfig {
                id: id.clone(),
                mapping_template,
                validation_template: soma_api_client::models::JwtTokenTemplateValidationConfig {
                    issuer: jwt_template_config.validation.issuer.clone().map(Some),
                    valid_audiences: jwt_template_config.validation.valid_audiences.clone().map(Some),
                    required_groups: jwt_template_config.validation.required_groups.clone().map(Some),
                    required_scopes: jwt_template_config.validation.required_scopes.clone().map(Some),
                },
            },
        },
    );

    identity_api::route_create_sts_config(&api_config, params)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create STS config: {e:?}")))?;

    info!("STS configuration '{}' created", id);
    println!();
    println!("Successfully added STS configuration: {}", id);
    println!("The configuration has been synced to soma.yaml.");

    Ok(())
}

async fn cmd_sts_remove(
    id: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    identity_api::route_delete_sts_config(&api_config, &id)
        .await
        .map_err(|e| {
            if let soma_api_client::apis::Error::ResponseError(resp) = &e {
                if resp.status.as_u16() == 404 {
                    return CommonError::NotFound {
                        msg: format!("STS configuration '{}' not found", id),
                        lookup_id: id.clone(),
                        source: None,
                    };
                }
            }
            CommonError::Unknown(anyhow::anyhow!("Failed to delete STS config: {e:?}"))
        })?;

    info!("STS configuration '{}' removed", id);
    println!("Successfully removed STS configuration: {}", id);

    Ok(())
}

async fn cmd_sts_list(api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    let result = identity_api::route_list_sts_configs(&api_config, 100, None)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to list STS configs: {e:?}")))?;

    if result.items.is_empty() {
        println!("No STS configurations found.");
        println!();
        println!("Use 'soma sts add from-template' to add a JWT template configuration.");
        println!("Use 'soma sts add-dev' to add a dev mode configuration (development only).");
    } else {
        println!("STS Configurations:");
        println!("===================");
        for config in result.items {
            let (config_id, config_type) = match &config {
                soma_api_client::models::StsTokenConfig::StsTokenConfigOneOf(c) => {
                    (c.jwt_template.id.clone(), "JWT Template")
                }
                soma_api_client::models::StsTokenConfig::StsTokenConfigOneOf1(c) => {
                    (c.dev_mode.id.clone(), "Dev Mode")
                }
            };
            println!();
            println!("ID: {}", config_id);
            println!("  Type: {}", config_type);

            // Display config-specific details
            match &config {
                soma_api_client::models::StsTokenConfig::StsTokenConfigOneOf(c) => {
                    let validation = &c.jwt_template.validation_template;
                    if let Some(Some(issuer)) = &validation.issuer {
                        println!("  Issuer: {}", issuer);
                    }
                    if let Some(Some(audiences)) = &validation.valid_audiences {
                        println!("  Audiences: {}", audiences.join(", "));
                    }
                    if let Some(Some(required_scopes)) = &validation.required_scopes {
                        if !required_scopes.is_empty() {
                            println!("  Required Scopes: {}", required_scopes.join(", "));
                        }
                    }
                    if let Some(Some(required_groups)) = &validation.required_groups {
                        if !required_groups.is_empty() {
                            println!("  Required Groups: {}", required_groups.join(", "));
                        }
                    }
                }
                soma_api_client::models::StsTokenConfig::StsTokenConfigOneOf1(_) => {
                    println!("  (Allows unauthenticated access for development)");
                }
            }
        }
    }

    Ok(())
}
