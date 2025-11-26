use clap::{Args, Subcommand};
use comfy_table::{Cell, Table};
use shared::error::CommonError;
use soma_api_client::apis::default_api;
use soma_api_client::models;
use tracing::info;

use crate::utils::{create_and_wait_for_api_client, CliConfig};

const DEFAULT_DEK_ALIAS: &str = "default";
const DEFAULT_PAGE_SIZE: i64 = 100;

#[derive(Args, Debug, Clone)]
pub struct SecretParams {
    #[command(subcommand)]
    pub command: SecretCommands,

    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SecretCommands {
    /// Set a secret value (creates or updates)
    Set {
        /// The secret key
        key: String,
        /// The secret value (will be encrypted)
        value: String,
    },
    /// Remove (delete) a secret
    #[command(name = "rm")]
    Remove {
        /// The secret key to delete
        key: String,
    },
    /// List all secrets with their decrypted values
    List,
}

pub async fn cmd_secret(
    params: SecretParams,
    _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {
    match params.command {
        SecretCommands::Set { key, value } => {
            cmd_secret_set(key, value, &params.api_url, params.timeout_secs).await
        }
        SecretCommands::Remove { key } => {
            cmd_secret_rm(key, &params.api_url, params.timeout_secs).await
        }
        SecretCommands::List => cmd_secret_list(&params.api_url, params.timeout_secs).await,
    }
}

/// Check if the default DEK alias exists
async fn default_dek_alias_exists(
    api_config: &soma_api_client::apis::configuration::Configuration,
) -> Result<bool, CommonError> {
    match default_api::get_dek_by_alias_or_id(api_config, DEFAULT_DEK_ALIAS).await {
        Ok(_) => Ok(true),
        Err(soma_api_client::apis::Error::ResponseError(resp)) if resp.status.as_u16() == 404 => {
            Ok(false)
        }
        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to check for default DEK alias: {e:?}"
        ))),
    }
}

/// Check if a secret exists by key
async fn get_secret_by_key(
    api_config: &soma_api_client::apis::configuration::Configuration,
    key: &str,
) -> Result<Option<models::Secret>, CommonError> {
    match default_api::get_secret_by_key(api_config, key).await {
        Ok(secret) => Ok(Some(secret)),
        Err(soma_api_client::apis::Error::ResponseError(resp)) if resp.status.as_u16() == 404 => {
            Ok(None)
        }
        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to get secret by key: {e:?}"
        ))),
    }
}

pub async fn cmd_secret_set(
    key: String,
    value: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Check if default DEK alias exists
    if !default_dek_alias_exists(&api_config).await? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "No default DEK alias found. Please add an encryption key first using:\n  soma enc-key add local --location <path>\n  or\n  soma enc-key add aws --arn <arn> --region <region>"
        )));
    }

    // Check if secret already exists
    let existing_secret = get_secret_by_key(&api_config, &key).await?;

    if let Some(secret) = existing_secret {
        // Update existing secret
        info!("Updating existing secret: {}", key);
        let update_req = models::UpdateSecretRequest { raw_value: value };
        default_api::update_secret(&api_config, &secret.id.to_string(), update_req)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to update secret '{key}': {e:?}"))
            })?;
        println!("Updated secret: {key}");
    } else {
        // Create new secret
        info!("Creating new secret: {}", key);
        let create_req = models::CreateSecretRequest {
            key: key.clone(),
            raw_value: value,
            dek_alias: DEFAULT_DEK_ALIAS.to_string(),
        };
        default_api::create_secret(&api_config, create_req)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to create secret '{key}': {e:?}"))
            })?;
        println!("Created secret: {key}");
    }

    Ok(())
}

pub async fn cmd_secret_rm(
    key: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Check if default DEK alias exists
    if !default_dek_alias_exists(&api_config).await? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "No default DEK alias found. Please add an encryption key first."
        )));
    }

    // Check if secret exists
    let existing_secret = get_secret_by_key(&api_config, &key).await?;

    match existing_secret {
        Some(secret) => {
            info!("Deleting secret: {}", key);
            default_api::delete_secret(&api_config, &secret.id.to_string())
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to delete secret '{key}': {e:?}"))
                })?;
            println!("Deleted secret: {key}");
            Ok(())
        }
        None => Err(CommonError::NotFound {
            msg: format!("Secret with key '{key}' not found"),
            lookup_id: key,
            source: None,
        }),
    }
}

pub async fn cmd_secret_list(api_url: &str, timeout_secs: u64) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Check if default DEK alias exists
    if !default_dek_alias_exists(&api_config).await? {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "No default DEK alias found. Please add an encryption key first."
        )));
    }

    // Fetch all decrypted secrets with pagination
    let mut all_secrets: Vec<models::DecryptedSecret> = Vec::new();
    let mut next_page_token: Option<String> = None;

    loop {
        let response = default_api::list_decrypted_secrets(
            &api_config,
            DEFAULT_PAGE_SIZE,
            next_page_token.as_deref(),
        )
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to list decrypted secrets: {e:?}"))
        })?;

        all_secrets.extend(response.secrets);

        // Handle doubly wrapped Option<Option<String>> from generated API client
        match response.next_page_token.flatten() {
            Some(token) if !token.is_empty() => {
                next_page_token = Some(token);
            }
            _ => break,
        }
    }

    if all_secrets.is_empty() {
        println!("No secrets found.");
        return Ok(());
    }

    // Create and display the table
    let mut table = Table::new();
    table.set_header(vec![Cell::new("Key"), Cell::new("Decrypted Value")]);

    for secret in all_secrets {
        table.add_row(vec![Cell::new(&secret.key), Cell::new(&secret.decrypted_value)]);
    }

    println!("{table}");

    Ok(())
}
