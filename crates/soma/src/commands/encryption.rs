use std::path::PathBuf;

use clap::{Args, Subcommand};
use shared::error::CommonError;
use soma_api_client::apis::default_api;
use soma_api_client::models;
use tracing::info;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

#[derive(Args, Debug, Clone)]
pub struct EncKeyParams {
    #[command(subcommand)]
    pub command: EncKeyCommands,

    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EncKeyCommands {
    /// Add an encryption key
    Add {
        #[command(subcommand)]
        key_type: AddKeyType,
    },
    /// Remove an encryption key
    Rm {
        #[command(subcommand)]
        key_type: RmKeyType,
    },
    /// Migrate encrypted data from one key to another
    Migrate {
        /// Source encryption key ID (ARN for AWS, location for local)
        from: String,
        /// Target encryption key ID (ARN for AWS, location for local)
        to: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddKeyType {
    /// Add an AWS KMS encryption key
    Aws {
        /// AWS KMS ARN
        #[arg(long)]
        arn: String,
        /// AWS region
        #[arg(long)]
        region: String,
    },
    /// Add a local encryption key
    Local {
        /// Local key location (relative to cwd or absolute path)
        #[arg(long)]
        location: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RmKeyType {
    /// Remove an AWS KMS encryption key
    Aws {
        /// AWS KMS ARN (used as ID)
        arn: String,
    },
    /// Remove a local encryption key
    Local {
        /// Local key location (relative to cwd or absolute path, used as ID)
        location: String,
    },
}

pub async fn cmd_enc_key(
    params: EncKeyParams,
    _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {
    match params.command {
        EncKeyCommands::Add { key_type } => {
            cmd_enc_key_add(key_type, &params.api_url, params.timeout_secs).await
        }
        EncKeyCommands::Rm { key_type } => {
            cmd_enc_key_rm(key_type, &params.api_url, params.timeout_secs).await
        }
        EncKeyCommands::Migrate { from, to } => {
            cmd_enc_key_migrate(from, to, &params.api_url, params.timeout_secs).await
        }
    }
}

pub async fn cmd_enc_key_add(
    key_type: AddKeyType,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    match key_type {
        AddKeyType::Aws { arn, region } => {
            info!("Adding AWS KMS encryption key: {} in region {}", arn, region);

            // Call the API endpoint directly with the envelope encryption key identifier and region
            // The API client may not have the updated schema yet, so we'll construct the request manually
            let create_url = format!("{}/api/v1/bridge/encryption/data-encryption-key", api_url);
            let client = &api_config.client;
            let response = client
                .post(&create_url)
                .json(&serde_json::json!({
                    "id": null,
                    "encrypted_data_envelope_key": null,
                    "envelope_encryption_key_identifier": arn,
                    "aws_region": region
                }))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to call create endpoint: {e:?}"
                    ))
                })?;

            if response.status().is_success() {
                let dek: models::DataEncryptionKey = response.json().await.map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to parse create response: {e:?}"
                    ))
                })?;
                info!("Successfully created data encryption key: {}", dek.id);
                info!("Envelope encryption key ID: {:?}", dek.envelope_encryption_key_id);
                Ok(())
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(CommonError::Unknown(anyhow::anyhow!(
                    "Create failed with status {}: {}",
                    status,
                    error_text
                )))
            }
        }
        AddKeyType::Local { location } => {
            // Convert relative path to absolute path
            let absolute_location = resolve_location(&location)?;
            let location_str = absolute_location.to_string_lossy().to_string();
            info!(
                "Adding local encryption key at location: {}",
                absolute_location.display()
            );

            // Call the API endpoint directly with the envelope encryption key identifier
            let create_url = format!("{}/api/v1/bridge/encryption/data-encryption-key", api_url);
            let client = &api_config.client;
            let response = client
                .post(&create_url)
                .json(&serde_json::json!({
                    "id": null,
                    "encrypted_data_envelope_key": null,
                    "envelope_encryption_key_identifier": location_str
                }))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to call create endpoint: {e:?}"
                    ))
                })?;

            if response.status().is_success() {
                let dek: models::DataEncryptionKey = response.json().await.map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to parse create response: {e:?}"
                    ))
                })?;
                info!("Successfully created data encryption key: {}", dek.id);
                info!("Envelope encryption key ID: {:?}", dek.envelope_encryption_key_id);
                Ok(())
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(CommonError::Unknown(anyhow::anyhow!(
                    "Create failed with status {}: {}",
                    status,
                    error_text
                )))
            }
        }
    }
}

pub async fn cmd_enc_key_rm(
    key_type: RmKeyType,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    match key_type {
        RmKeyType::Aws { arn } => {
            info!("Removing AWS KMS encryption key: {}", arn);
            info!("Note: Bridge will look up the key by ARN to find region and other details");

            // Call delete by identifier endpoint
            // Note: API client needs to be regenerated to include this endpoint
            // For now, we'll construct the request manually
            let delete_url = format!("{}/api/v1/bridge/encryption/data-encryption-key/by-identifier", api_url);
            let client = &api_config.client;
            let response = client
                .delete(&delete_url)
                .json(&serde_json::json!({
                    "identifier": arn
                }))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to call delete endpoint: {e:?}"
                    ))
                })?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await.map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to parse delete response: {e:?}"
                    ))
                })?;
                let deleted_count = result.get("deleted_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                info!("Successfully deleted {} data encryption key(s)", deleted_count);
                Ok(())
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(CommonError::Unknown(anyhow::anyhow!(
                    "Delete failed with status {}: {}",
                    status,
                    error_text
                )))
            }
        }
        RmKeyType::Local { location } => {
            // Convert relative path to absolute path for consistency
            let absolute_location = resolve_location(&location)?;
            let location_str = absolute_location.to_string_lossy().to_string();
            info!(
                "Removing local encryption key at location: {}",
                absolute_location.display()
            );
            info!("Note: Bridge will look up the key by location to find other details");

            // Call delete by identifier endpoint
            let delete_url = format!("{}/api/v1/bridge/encryption/data-encryption-key/by-identifier", api_url);
            let client = &api_config.client;
            let response = client
                .delete(&delete_url)
                .json(&serde_json::json!({
                    "identifier": location_str
                }))
                .send()
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to call delete endpoint: {e:?}"
                    ))
                })?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await.map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to parse delete response: {e:?}"
                    ))
                })?;
                let deleted_count = result.get("deleted_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                info!("Successfully deleted {} data encryption key(s)", deleted_count);
                Ok(())
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(CommonError::Unknown(anyhow::anyhow!(
                    "Delete failed with status {}: {}",
                    status,
                    error_text
                )))
            }
        }
    }
}

pub async fn cmd_enc_key_migrate(
    from: String,
    to: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    info!("Migrating encrypted data from '{}' to '{}'", from, to);
    info!("Note: Bridge will look up keys by identifier to find region and other details");

    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Resolve local paths to absolute paths
    let from_identifier = if from.starts_with("arn:aws:kms:") {
        from.clone()
    } else {
        let absolute_location = resolve_location(&from)?;
        absolute_location.to_string_lossy().to_string()
    };

    let to_identifier = if to.starts_with("arn:aws:kms:") {
        to.clone()
    } else {
        let absolute_location = resolve_location(&to)?;
        absolute_location.to_string_lossy().to_string()
    };

    // Call migrate by identifier endpoint
    // Note: API client needs to be regenerated to include this endpoint
    // For now, we'll construct the request manually
    let migrate_url = format!("{}/api/v1/bridge/encryption/migrate-by-identifier", api_url);
    let client = &api_config.client;
    let response = client
        .post(&migrate_url)
        .json(&serde_json::json!({
            "from": from_identifier,
            "to": to_identifier
        }))
        .send()
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call migrate endpoint: {e:?}"
            ))
        })?;

    if response.status().is_success() {
        let result: models::MigrateEncryptionKeyResponse = response.json().await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to parse migrate response: {e:?}"
            ))
        })?;
        info!("Migration completed successfully!");
        info!(
            "Migrated {} data encryption keys",
            result.migrated_data_encryption_keys
        );
        info!(
            "Migrated {} resource server credentials",
            result.migrated_resource_server_credentials
        );
        info!(
            "Migrated {} user credentials",
            result.migrated_user_credentials
        );
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(CommonError::Unknown(anyhow::anyhow!(
            "Migration failed with status {}: {}",
            status,
            error_text
        )))
    }
}

/// Resolve a location string to an absolute path
/// If the path is already absolute, return it as is
/// If the path is relative, resolve it relative to the current working directory
fn resolve_location(location: &str) -> Result<PathBuf, CommonError> {
    let path = PathBuf::from(location);

    if path.is_absolute() {
        Ok(path)
    } else {
        let current_dir = std::env::current_dir().map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to get current working directory: {}",
                e
            ))
        })?;
        Ok(current_dir.join(path))
    }
}

