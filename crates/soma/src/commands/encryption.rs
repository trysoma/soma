use std::path::PathBuf;

use clap::{Args, Subcommand};
use shared::error::CommonError;
use tracing::info;

use crate::utils::CliConfig;

#[derive(Args, Debug, Clone)]
pub struct EncryptionParams {
    #[command(subcommand)]
    pub command: EncryptionCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EncryptionCommands {
    /// Manage encryption keys
    #[command(name = "enc-key")]
    EncKey {
        #[command(subcommand)]
        command: EncKeyCommands,
    },
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

pub async fn cmd_encryption(
    params: EncryptionParams,
    _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {
    match params.command {
        EncryptionCommands::EncKey { command } => cmd_enc_key(command).await,
    }
}

pub async fn cmd_enc_key(command: EncKeyCommands) -> Result<(), CommonError> {
    match command {
        EncKeyCommands::Add { key_type } => cmd_enc_key_add(key_type).await,
        EncKeyCommands::Rm { key_type } => cmd_enc_key_rm(key_type).await,
        EncKeyCommands::Migrate { from, to } => cmd_enc_key_migrate(from, to).await,
    }
}

pub async fn cmd_enc_key_add(key_type: AddKeyType) -> Result<(), CommonError> {
    match key_type {
        AddKeyType::Aws { arn, region } => {
            info!("Adding AWS KMS encryption key: {} in region {}", arn, region);
            // TODO: Call bridge API to add AWS KMS key
            // For now, just log the command
            info!("AWS KMS key would be added here");
            Ok(())
        }
        AddKeyType::Local { location } => {
            // Convert relative path to absolute path
            let absolute_location = resolve_location(&location)?;
            info!(
                "Adding local encryption key at location: {}",
                absolute_location.display()
            );
            // TODO: Call bridge API to add local key
            // For now, just log the command
            info!("Local key would be added here");
            Ok(())
        }
    }
}

pub async fn cmd_enc_key_rm(key_type: RmKeyType) -> Result<(), CommonError> {
    match key_type {
        RmKeyType::Aws { arn } => {
            info!("Removing AWS KMS encryption key: {}", arn);
            // TODO: Call bridge API to remove AWS KMS key using ARN as ID
            // For now, just log the command
            info!("AWS KMS key would be removed here");
            Ok(())
        }
        RmKeyType::Local { location } => {
            // Convert relative path to absolute path for consistency
            let absolute_location = resolve_location(&location)?;
            info!(
                "Removing local encryption key at location: {}",
                absolute_location.display()
            );
            // TODO: Call bridge API to remove local key using location as ID
            // For now, just log the command
            info!("Local key would be removed here");
            Ok(())
        }
    }
}

pub async fn cmd_enc_key_migrate(from: String, to: String) -> Result<(), CommonError> {
    info!("Migrating encrypted data from '{}' to '{}'", from, to);

    // Determine if from/to are ARNs or local paths and resolve local paths
    let from_id = if from.starts_with("arn:aws:kms:") {
        from.clone()
    } else {
        resolve_location(&from)?.to_string_lossy().to_string()
    };

    let to_id = if to.starts_with("arn:aws:kms:") {
        to.clone()
    } else {
        resolve_location(&to)?.to_string_lossy().to_string()
    };

    info!("Resolved from: {}", from_id);
    info!("Resolved to: {}", to_id);

    // TODO: Call bridge API migration endpoint with from_id and to_id
    // The endpoint should:
    // 1. Look up all values encrypted with data encryption keys where parent is from_id
    // 2. Decrypt using typed controllers (ApiKeyController, etc.)
    // 3. Re-encrypt with new key and persist to db
    // 4. Trigger bridge on change

    info!("Migration would be performed here");
    Ok(())
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
