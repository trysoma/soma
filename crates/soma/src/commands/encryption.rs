use clap::{Args, Subcommand};
use shared::error::CommonError;
use soma_api_client::apis::encryption_api;
use soma_api_client::models;
use tracing::debug;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

const DEFAULT_ALIAS: &str = "default";

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
    /// Migrate all DEKs from one envelope encryption key to another
    Migrate {
        /// Source encryption key ID (ARN for AWS, file name for local)
        from: String,
        /// Target encryption key ID (ARN for AWS, file name for local)
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
        /// Local key file name (relative to cwd / project root)
        #[arg(long)]
        file_name: String,
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
        /// Local key file name (relative to cwd / project root, used as ID)
        file_name: String,
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

/// Check if a DEK alias exists
async fn default_alias_exists(
    api_config: &soma_api_client::apis::configuration::Configuration,
) -> Result<bool, CommonError> {
    match encryption_api::get_dek_by_alias_or_id(api_config, DEFAULT_ALIAS).await {
        Ok(_) => Ok(true),
        Err(soma_api_client::apis::Error::ResponseError(resp)) if resp.status.as_u16() == 404 => {
            Ok(false)
        }
        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to check for default alias: {e:?}"
        ))),
    }
}

/// Create a DEK for an envelope key and set it as default
async fn create_default_dek(
    api_config: &soma_api_client::apis::configuration::Configuration,
    envelope_id: &str,
) -> Result<(), CommonError> {
    // Create the DEK
    let dek_params = models::CreateDataEncryptionKeyParamsRoute {
        id: None,
        encrypted_dek: None,
    };

    let dek = encryption_api::create_data_encryption_key(api_config, envelope_id, dek_params)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create DEK: {e:?}")))?;

    debug!("Created data encryption key: {}", dek.id);

    // Create the default alias for the DEK
    let alias_params = models::CreateDekAliasRequest {
        dek_id: dek.id.clone(),
        alias: DEFAULT_ALIAS.to_string(),
    };

    encryption_api::create_dek_alias(api_config, alias_params)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to create default alias: {e:?}"))
        })?;

    debug!("Created '{}' alias for DEK: {}", DEFAULT_ALIAS, dek.id);

    Ok(())
}

pub async fn cmd_enc_key_add(
    key_type: AddKeyType,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    // Check if default alias already exists
    let has_default_alias = default_alias_exists(&api_config).await?;

    match key_type {
        AddKeyType::Aws { arn, region } => {
            debug!(
                "Adding AWS KMS envelope encryption key: {} in region {}",
                arn, region
            );

            // Create the envelope encryption key using the API client
            let envelope_key = models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(
                models::EnvelopeEncryptionKeyOneOf::new(
                    arn.clone(),
                    region.clone(),
                    models::envelope_encryption_key_one_of::Type::AwsKms,
                ),
            );

            let created_key =
                encryption_api::create_envelope_encryption_key(&api_config, envelope_key)
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to create envelope encryption key: {e:?}"
                        ))
                    })?;

            let envelope_id = match &created_key {
                models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(key) => key.arn.clone(),
                models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(key) => {
                    key.file_name.clone()
                }
            };

            debug!(
                "Successfully created envelope encryption key: {}",
                envelope_id
            );

            // If no default alias exists, create a DEK and set it as default
            if !has_default_alias {
                debug!("No default DEK alias found, creating default DEK...");
                create_default_dek(&api_config, &envelope_id).await?;
            } else {
                debug!("Default DEK alias already exists, skipping DEK creation");
            }

            Ok(())
        }
        AddKeyType::Local { file_name } => {
            debug!(
                "Adding local envelope encryption key at file name: {}",
                file_name
            );

            // Create the envelope encryption key using the API client
            let envelope_key = models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(
                models::EnvelopeEncryptionKeyOneOf1::new(
                    file_name.clone(),
                    models::envelope_encryption_key_one_of_1::Type::Local,
                ),
            );

            let created_key =
                encryption_api::create_envelope_encryption_key(&api_config, envelope_key)
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to create envelope encryption key: {e:?}"
                        ))
                    })?;

            let envelope_id = match &created_key {
                models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(key) => key.arn.clone(),
                models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(key) => {
                    key.file_name.clone()
                }
            };

            debug!(
                "Successfully created envelope encryption key: {}",
                envelope_id
            );

            // If no default alias exists, create a DEK and set it as default
            if !has_default_alias {
                debug!("No default DEK alias found, creating default DEK...");
                create_default_dek(&api_config, &envelope_id).await?;
            } else {
                debug!("Default DEK alias already exists, skipping DEK creation");
            }

            Ok(())
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

    let envelope_id = match key_type {
        RmKeyType::Aws { arn } => {
            debug!("Checking AWS KMS encryption key: {}", arn);
            arn
        }
        RmKeyType::Local { file_name } => {
            debug!("Checking local encryption key at file name: {}", file_name);
            file_name
        }
    };

    // List DEKs tied to this envelope key
    let deks_response = encryption_api::list_data_encryption_keys_by_envelope(
        &api_config,
        &envelope_id,
        100, // page size
        None,
    )
    .await
    .map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Failed to list DEKs for envelope key: {e:?}"
        ))
    })?;

    let dek_count = deks_response.items.len();

    if dek_count > 0 {
        println!("Cannot delete envelope encryption key: {envelope_id}");
        println!();
        println!("There are {dek_count} data encryption key(s) tied to this envelope key:");
        for dek in &deks_response.items {
            println!("  - {}", dek.id);
        }
        println!();
        println!("You must first migrate these DEKs to another envelope key using:");
        println!("  soma enc-key migrate {envelope_id} <new-envelope-key-id>");
        println!();
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Cannot delete envelope key with {dek_count} associated DEK(s). Run migrate first."
        )));
    }

    // No DEKs tied to this envelope key - we can delete it
    // Note: The API doesn't have a delete endpoint for envelope keys yet
    // For now, we'll just inform the user that the key has no DEKs
    debug!(
        "Envelope encryption key {} has no associated DEKs",
        envelope_id
    );
    println!(
        "Envelope encryption key {envelope_id} has no associated DEKs and can be safely removed."
    );
    println!("Note: Direct deletion of envelope keys is not yet implemented in the API.");
    println!("You may remove the key configuration manually from your soma.yaml file.");

    Ok(())
}

pub async fn cmd_enc_key_migrate(
    from: String,
    to: String,
    api_url: &str,
    timeout_secs: u64,
) -> Result<(), CommonError> {
    debug!("Migrating all DEKs from '{}' to '{}'", from, to);

    // Create API client and wait for server to be ready
    let api_config = create_and_wait_for_api_client(api_url, timeout_secs).await?;

    debug!("Source envelope key: {}", from);
    debug!("Target envelope key: {}", to);

    // Call the migrate_all_data_encryption_keys endpoint
    let migrate_params = models::MigrateAllDataEncryptionKeysParamsRoute {
        to_envelope_encryption_key_id: to.clone(),
    };

    encryption_api::migrate_all_data_encryption_keys(&api_config, &from, migrate_params)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to migrate DEKs: {e:?}")))?;

    debug!("Migration completed successfully!");
    println!("Successfully migrated all DEKs from '{from}' to '{to}'");

    Ok(())
}
