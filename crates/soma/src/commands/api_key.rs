use clap::{Args, Subcommand};
use comfy_table::{Cell, Table};
use shared::error::CommonError;
use soma_api_client::apis::identity_api;
use soma_api_client::models;
use tracing::debug;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

const DEFAULT_PAGE_SIZE: i64 = 100;

#[derive(Args, Debug, Clone)]
pub struct ApiKeyParams {
    #[command(subcommand)]
    pub command: ApiKeyCommands,

    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ApiKeyCommands {
    /// Create a new API key
    Add {
        /// Unique ID for the API key (lowercase letters, numbers, and hyphens only)
        id: String,
        /// Role for the API key (admin, maintainer, read-only-maintainer, agent, user)
        #[arg(long)]
        role: String,
        /// Optional description for the API key
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete an API key
    #[command(name = "rm")]
    Remove {
        /// The API key ID to delete
        id: String,
    },
    /// List all API keys
    List,
}

pub async fn cmd_api_key(
    params: ApiKeyParams,
    _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {
    match params.command {
        ApiKeyCommands::Add {
            id,
            role,
            description,
        } => cmd_api_key_add(id, role, description, &params.api_url, params.timeout_secs).await,
        ApiKeyCommands::Remove { id } => {
            cmd_api_key_rm(id, &params.api_url, params.timeout_secs).await
        }
        ApiKeyCommands::List => cmd_api_key_list(&params.api_url, params.timeout_secs).await,
    }
}

pub async fn cmd_api_key_add(
    id: String,
    role: String,
    description: Option<String>,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Validate role
    let valid_roles = [
        "admin",
        "maintainer",
        "read-only-maintainer",
        "agent",
        "user",
    ];
    if !valid_roles.contains(&role.as_str()) {
        return Err(CommonError::InvalidRequest {
            msg: format!(
                "Invalid role '{}'. Valid roles are: {}",
                role,
                valid_roles.join(", ")
            ),
            source: None,
        });
    }

    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Convert role string to enum
    let role_enum = match role.to_lowercase().as_str() {
        "admin" => models::Role::Admin,
        "maintainer" => models::Role::Maintainer,
        "read-only-maintainer" => models::Role::ReadOnlyMaintainer,
        "agent" => models::Role::Agent,
        "user" => models::Role::User,
        _ => {
            return Err(CommonError::InvalidRequest {
                msg: format!("Invalid role: {role}"),
                source: None,
            });
        }
    };

    debug!("Creating new API key '{}' with role: {}", id, role);
    let create_req = models::CreateApiKeyParams {
        id,
        description: Some(description),
        role: role_enum,
    };
    let response = identity_api::route_create_api_key(&api_config, create_req)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create API key: {e:?}")))?;

    println!("Created API key:");
    println!("  ID: {}", response.id);
    println!("  Key: {}", response.api_key);
    println!();
    println!("IMPORTANT: Save the API key now! It will not be shown again.");

    Ok(())
}

pub async fn cmd_api_key_rm(
    id: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    debug!("Deleting API key: {}", id);
    identity_api::route_delete_api_key(&api_config, &id)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to delete API key '{id}': {e:?}"))
        })?;

    println!("Deleted API key: {id}");
    Ok(())
}

pub async fn cmd_api_key_list(api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Fetch all API keys with pagination
    let mut all_api_keys: Vec<models::HashedApiKey> = Vec::new();
    let mut next_page_token: Option<String> = None;

    loop {
        let response = identity_api::route_list_api_keys(
            &api_config,
            DEFAULT_PAGE_SIZE,
            next_page_token.as_deref(),
        )
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to list API keys: {e:?}")))?;

        all_api_keys.extend(response.items);

        // Handle doubly wrapped Option<Option<String>> from generated API client
        match response.next_page_token.flatten() {
            Some(token) if !token.is_empty() => {
                next_page_token = Some(token);
            }
            _ => break,
        }
    }

    if all_api_keys.is_empty() {
        println!("No API keys found.");
        return Ok(());
    }

    // Create and display the table
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("ID"),
        Cell::new("User ID"),
        Cell::new("Description"),
        Cell::new("Created At"),
    ]);

    for api_key in all_api_keys {
        // Handle potentially doubly-wrapped Option from generated API client
        let description_opt = api_key.description.flatten();
        let description = description_opt.as_deref().unwrap_or("-");
        table.add_row(vec![
            Cell::new(&api_key.id),
            Cell::new(&api_key.user_id),
            Cell::new(description),
            Cell::new(&api_key.created_at),
        ]);
    }

    println!("{table}");

    Ok(())
}
