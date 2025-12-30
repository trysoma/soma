use clap::{Args, Subcommand};
use comfy_table::{Cell, Table};
use shared::error::CommonError;
use soma_api_client::apis::environment_api;
use soma_api_client::models;
use tracing::debug;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

const DEFAULT_PAGE_SIZE: i64 = 100;

#[derive(Args, Debug, Clone)]
pub struct EnvironmentParams {
    #[command(subcommand)]
    pub command: EnvironmentCommands,

    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EnvironmentCommands {
    /// Set an environment variable value (creates or updates)
    Set {
        /// The environment variable key
        key: String,
        /// The environment variable value
        value: String,
    },
    /// Remove (delete) an environment variable
    #[command(name = "rm")]
    Remove {
        /// The environment variable key to delete
        key: String,
    },
    /// List all environment variables
    List,
}

pub async fn cmd_environment(
    params: EnvironmentParams,
    _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {
    match params.command {
        EnvironmentCommands::Set { key, value } => {
            cmd_env_set(key, value, &params.api_url, params.timeout_secs).await
        }
        EnvironmentCommands::Remove { key } => {
            cmd_env_rm(key, &params.api_url, params.timeout_secs).await
        }
        EnvironmentCommands::List => cmd_env_list(&params.api_url, params.timeout_secs).await,
    }
}

/// Check if an environment variable exists by key
async fn get_env_var_by_key(
    api_config: &soma_api_client::apis::configuration::Configuration,
    key: &str,
) -> Result<Option<models::Variable>, CommonError> {
    match environment_api::get_variable_by_key(api_config, key).await {
        Ok(variable) => Ok(Some(variable)),
        Err(soma_api_client::apis::Error::ResponseError(resp)) if resp.status.as_u16() == 404 => {
            Ok(None)
        }
        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to get environment variable by key: {e:?}"
        ))),
    }
}

pub async fn cmd_env_set(
    key: String,
    value: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs, None).await?;

    // Check if environment variable already exists
    let existing_env_var = get_env_var_by_key(&api_config, &key).await?;

    if let Some(env_var) = existing_env_var {
        // Update existing environment variable
        debug!("Updating existing environment variable: {}", key);
        let update_req = models::UpdateVariableRequest { value };
        environment_api::update_variable(&api_config, &env_var.id.to_string(), update_req)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to update environment variable '{key}': {e:?}"
                ))
            })?;
        println!("Updated environment variable: {key}");
    } else {
        // Create new environment variable
        debug!("Creating new environment variable: {}", key);
        let create_req = models::CreateVariableRequest {
            key: key.clone(),
            value,
        };
        environment_api::create_variable(&api_config, create_req)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!(
                    "Failed to create environment variable '{key}': {e:?}"
                ))
            })?;
        println!("Created environment variable: {key}");
    }

    Ok(())
}

pub async fn cmd_env_rm(key: String, api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs, None).await?;

    // Check if environment variable exists
    let existing_env_var = get_env_var_by_key(&api_config, &key).await?;

    match existing_env_var {
        Some(env_var) => {
            debug!("Deleting environment variable: {}", key);
            environment_api::delete_variable(&api_config, &env_var.id.to_string())
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to delete environment variable '{key}': {e:?}"
                    ))
                })?;
            println!("Deleted environment variable: {key}");
            Ok(())
        }
        None => Err(CommonError::NotFound {
            msg: format!("Environment variable with key '{key}' not found"),
            lookup_id: key,
            source: None,
        }),
    }
}

pub async fn cmd_env_list(api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs, None).await?;

    // Fetch all environment variables with pagination
    let mut all_env_vars: Vec<models::Variable> = Vec::new();
    let mut next_page_token: Option<String> = None;

    loop {
        let response = environment_api::list_variables(
            &api_config,
            DEFAULT_PAGE_SIZE,
            next_page_token.as_deref(),
        )
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to list environment variables: {e:?}"
            ))
        })?;

        all_env_vars.extend(response.variables);

        // Handle doubly wrapped Option<Option<String>> from generated API client
        match response.next_page_token.flatten() {
            Some(token) if !token.is_empty() => {
                next_page_token = Some(token);
            }
            _ => break,
        }
    }

    if all_env_vars.is_empty() {
        println!("No environment variables found.");
        return Ok(());
    }

    // Create and display the table
    let mut table = Table::new();
    table.set_header(vec![Cell::new("Key"), Cell::new("Value")]);

    for env_var in all_env_vars {
        table.add_row(vec![Cell::new(&env_var.key), Cell::new(&env_var.value)]);
    }

    println!("{table}");

    Ok(())
}
