use std::path::PathBuf;

use clap::{Args, Subcommand};
use inquire::{Select, Text};
use shared::error::CommonError;
use shared::soma_agent_definition::{
    GroupToRoleMappingYaml, JwtMappingConfigYaml, JwtTemplateConfigYaml, JwtValidationConfigYaml,
    SomaAgentDefinitionLike, StsConfigYaml, TokenLocationYaml, YamlSomaAgentDefinition,
};
use tracing::info;

use crate::utils::{construct_cwd_absolute, CliConfig};

#[derive(Args, Debug, Clone)]
pub struct StsParams {
    #[command(subcommand)]
    pub command: StsCommands,

    /// Path to the project directory (defaults to current directory)
    #[arg(long)]
    pub cwd: Option<PathBuf>,
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

/// Load the soma definition from the project directory
fn load_soma_definition(cwd: Option<PathBuf>) -> Result<YamlSomaAgentDefinition, CommonError> {
    let project_dir = construct_cwd_absolute(cwd)?;
    let path_to_soma_definition = project_dir.join("soma.yaml");

    if !path_to_soma_definition.exists() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Soma definition not found at {}. Run 'soma init' first.",
            path_to_soma_definition.display()
        )));
    }

    YamlSomaAgentDefinition::load_from_file(path_to_soma_definition)
}

pub async fn cmd_sts(params: StsParams, _cli_config: &mut CliConfig) -> Result<(), CommonError> {
    let soma_definition = load_soma_definition(params.cwd.clone())?;

    match params.command {
        StsCommands::Add { add_command } => match add_command {
            StsAddCommands::FromTemplate => cmd_sts_add_from_template(&soma_definition).await,
        },
        StsCommands::Remove { id } => cmd_sts_remove(id, &soma_definition).await,
        StsCommands::List => cmd_sts_list(&soma_definition).await,
    }
}

async fn cmd_sts_add_from_template(
    soma_definition: &YamlSomaAgentDefinition,
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
    println!("Validation Settings");
    println!("-------------------");

    // Issuer (optional)
    let issuer_input = Text::new("Issuer (iss claim):")
        .with_help_message("Expected issuer value (leave empty to skip validation)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let issuer = if issuer_input.trim().is_empty() {
        None
    } else {
        Some(issuer_input)
    };

    // Valid Audiences (optional)
    let audiences_input = Text::new("Valid audiences (comma-separated):")
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

    // Required Scopes (optional)
    let scopes_input = Text::new("Required scopes (comma-separated):")
        .with_help_message("Scopes that must be present in the token (leave empty to skip)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let required_scopes = if scopes_input.trim().is_empty() {
        None
    } else {
        Some(
            scopes_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };

    // Required Groups (optional)
    let groups_input = Text::new("Required groups (comma-separated):")
        .with_help_message("Groups the user must belong to (leave empty to skip)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let required_groups = if groups_input.trim().is_empty() {
        None
    } else {
        Some(
            groups_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };

    println!();
    println!("Claim Mapping");
    println!("-------------");

    // Subject field
    let sub_field = Text::new("Subject field:")
        .with_default("sub")
        .with_help_message("JWT claim containing the user's unique identifier")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    // Email field (optional)
    let email_field_input = Text::new("Email field (optional):")
        .with_help_message("JWT claim containing the user's email (leave empty to skip)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let email_field = if email_field_input.trim().is_empty() {
        None
    } else {
        Some(email_field_input)
    };

    // Groups field (optional)
    let groups_field_input = Text::new("Groups field (optional):")
        .with_help_message("JWT claim containing the user's groups (leave empty to skip)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let groups_field = if groups_field_input.trim().is_empty() {
        None
    } else {
        Some(groups_field_input)
    };

    // Scopes field (optional)
    let scopes_field_input = Text::new("Scopes field (optional):")
        .with_help_message("JWT claim containing the user's scopes (leave empty to skip)")
        .prompt()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;
    let scopes_field = if scopes_field_input.trim().is_empty() {
        None
    } else {
        Some(scopes_field_input)
    };

    println!();
    println!("Group to Role Mappings");
    println!("----------------------");

    let mut group_to_role_mappings: Vec<GroupToRoleMappingYaml> = Vec::new();

    let add_mappings = Select::new(
        "Would you like to add group-to-role mappings?",
        vec!["Yes", "No"],
    )
    .prompt()
    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

    if add_mappings == "Yes" {
        let roles = vec![
            "admin",
            "maintainer",
            "read-only-maintainer",
            "agent",
            "user",
        ];

        loop {
            let group = Text::new("Group name:")
                .with_help_message("The group name from the JWT to match")
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

            if group.trim().is_empty() {
                break;
            }

            let role = Select::new("Role to assign:", roles.clone())
                .prompt()
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read input: {e}")))?;

            group_to_role_mappings.push(GroupToRoleMappingYaml {
                group,
                role: role.to_string(),
            });

            println!(
                "Added mapping. Enter another group name or press Enter to finish."
            );
        }
    }

    // Build the configuration
    let config = StsConfigYaml::JwtTemplate(JwtTemplateConfigYaml {
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
    });

    // Add to soma.yaml
    soma_definition.add_sts_config(id.clone(), config).await?;

    info!("STS configuration '{}' added to soma.yaml", id);
    println!();
    println!("Successfully added STS configuration: {}", id);
    println!("The configuration has been saved to soma.yaml.");
    println!("Run 'soma dev' to start the server with the new configuration.");

    Ok(())
}

async fn cmd_sts_remove(
    id: String,
    soma_definition: &YamlSomaAgentDefinition,
) -> Result<(), CommonError> {

    // Check if the config exists
    let definition = soma_definition.get_definition().await?;
    let exists = definition
        .identity
        .as_ref()
        .and_then(|i| i.sts_configurations.as_ref())
        .map(|configs| configs.contains_key(&id))
        .unwrap_or(false);

    if !exists {
        return Err(CommonError::NotFound {
            msg: format!("STS configuration '{}' not found", id),
            lookup_id: id,
            source: None,
        });
    }

    soma_definition.remove_sts_config(id.clone()).await?;

    info!("STS configuration '{}' removed from soma.yaml", id);
    println!("Successfully removed STS configuration: {}", id);

    Ok(())
}

async fn cmd_sts_list(soma_definition: &YamlSomaAgentDefinition) -> Result<(), CommonError> {
    let definition = soma_definition.get_definition().await?;

    let sts_configs = definition
        .identity
        .as_ref()
        .and_then(|i| i.sts_configurations.as_ref());

    match sts_configs {
        Some(configs) if !configs.is_empty() => {
            println!("STS Configurations:");
            println!("===================");
            for (id, config) in configs {
                println!();
                println!("ID: {}", id);
                match config {
                    StsConfigYaml::JwtTemplate(jwt_config) => {
                        println!("  Type: JWT Template");
                        println!("  JWKS URI: {}", jwt_config.jwks_uri);
                        match &jwt_config.token_location {
                            TokenLocationYaml::Header { name } => {
                                println!("  Token Location: Header ({})", name);
                            }
                            TokenLocationYaml::Cookie { name } => {
                                println!("  Token Location: Cookie ({})", name);
                            }
                        }
                        if let Some(issuer) = &jwt_config.validation.issuer {
                            println!("  Issuer: {}", issuer);
                        }
                        if let Some(audiences) = &jwt_config.validation.valid_audiences {
                            println!("  Audiences: {}", audiences.join(", "));
                        }
                        if let Some(mappings) = &jwt_config.group_to_role_mappings {
                            if !mappings.is_empty() {
                                println!("  Group Mappings:");
                                for mapping in mappings {
                                    println!("    {} -> {}", mapping.group, mapping.role);
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            println!("No STS configurations found.");
            println!();
            println!("Use 'soma sts add from-template' to add a JWT template configuration.");
        }
    }

    Ok(())
}
