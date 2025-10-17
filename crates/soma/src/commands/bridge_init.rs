use std::collections::HashMap;
use std::path::PathBuf;
use bridge::DEFAULT_DATA_ENCRYPTION_KEY_ID;
use clap::Parser;
use tokio_graceful_shutdown::SubsystemHandle;
use tracing::{info, warn};
use shared::error::CommonError;
use crate::utils::config::CliConfig;
use crate::utils::{construct_src_dir_absolute, get_api_config};
use shared::soma_agent_definition::{BridgeConfig, BridgeEncryptionConfig, EncryptionConfiguration, EnvelopeEncryptionKeyId, SomaAgentDefinition};

#[derive(Debug, Clone, Parser)]
pub struct BridgeInitParams {
    #[arg(long)]
    pub src_dir: Option<PathBuf>,
    #[arg(long)]
    pub envelope_encryption_key: String,
}

pub async fn cmd_bridge_init(_subsys: &SubsystemHandle, params: BridgeInitParams, config: &mut CliConfig) -> Result<(), CommonError> {
    // bridge_init(params, config).await
    Ok(())
}


async fn bridge_init(params: BridgeInitParams) -> Result<(), CommonError> {
    info!("Initializing bridge configuration");

    // Construct the absolute src_dir path
    let src_dir = construct_src_dir_absolute(params.src_dir.clone())?;
    let soma_yaml_path = src_dir.join("soma.yaml");

    // Read the current soma.yaml file
    let soma_yaml_str = std::fs::read_to_string(&soma_yaml_path)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read soma.yaml: {}", e)))?;

    // Parse the soma.yaml file
    let mut soma_definition = SomaAgentDefinition::from_yaml(&soma_yaml_str)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse soma.yaml: {}", e)))?;
    
    // Check if bridge encryption is already initialized
    let encrypted_data_envelope_key = match &soma_definition.bridge {
        Some(bridge) => match bridge.encryption.0.get(DEFAULT_DATA_ENCRYPTION_KEY_ID) {
            Some(encryption) => Some(encryption.encrypted_data_envelope_key.clone()),
            None => None,
        }
        None => None,
    };

    if let Some(_) = &encrypted_data_envelope_key {
        info!("Data encryption key already initialized with ID: {}. Running again will overwrite the existing key. Do you want to continue? (y/n)", DEFAULT_DATA_ENCRYPTION_KEY_ID);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to read line: {}", e)))?;
        if input.trim() != "y" {
            return Ok(());
        }
    }

    // Validate the envelope encryption key is a valid AWS KMS ARN
    validate_aws_kms_arn(&params.envelope_encryption_key)?;

    // Make HTTP request to create a data encryption key
    // For now, we'll call the bridge logic directly since we don't have a running server
    // In a production setup, you might want to make an HTTP request to the server
    info!("Creating data encryption key with envelope encryption key: {}", params.envelope_encryption_key);

    let configuration = get_api_config()?;
    let create_data_encryption_key_params = soma_api_client::models::CreateDataEncryptionKeyParams {
        envelope_encryption_key_id: Box::new(soma_api_client::models::EnvelopeEncryptionKeyId {
            r#type: soma_api_client::models::envelope_encryption_key_id::Type::AwsKms,
            arn: params.envelope_encryption_key.clone(),
        }),
        id: Some(Some(DEFAULT_DATA_ENCRYPTION_KEY_ID.to_string())),
        encrypted_data_envelope_key: encrypted_data_envelope_key
    };
    let data_encryption_key = soma_api_client::apis::default_api::create_data_encryption_key(&configuration, create_data_encryption_key_params).await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create data encryption key: {:?}", e)))?;

    info!("Data encryption key created with ID: {}", data_encryption_key.id);

    // Update the soma.yaml file with the bridge configuration
    let mut encryption_config = HashMap::new();
    encryption_config.insert(DEFAULT_DATA_ENCRYPTION_KEY_ID.to_string(), EncryptionConfiguration {
        encrypted_data_envelope_key: data_encryption_key.encrypted_data_envelope_key.clone(),
        envelope_encryption_key_id: EnvelopeEncryptionKeyId::AwsKms { arn: params.envelope_encryption_key.clone() },
    });
    let bridge_config = BridgeConfig {
        encryption: BridgeEncryptionConfig(encryption_config)
    };

    soma_definition.bridge = Some(bridge_config);

    // Write the updated soma.yaml file
    let updated_yaml = soma_definition.to_yaml()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to serialize soma.yaml: {}", e)))?;

    std::fs::write(&soma_yaml_path, updated_yaml)
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to write soma.yaml: {}", e)))?;

    info!("Bridge initialized successfully! Data encryption key ID: {}", data_encryption_key.id);
    info!("Updated soma.yaml at: {}", soma_yaml_path.display());

    Ok(())
}

fn validate_aws_kms_arn(arn: &str) -> Result<(), CommonError> {
    // AWS KMS ARN format: arn:aws:kms:region:account-id:key/key-id
    // or: arn:aws:kms:region:account-id:alias/alias-name

    if !arn.starts_with("arn:aws:kms:") {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid AWS KMS ARN: must start with 'arn:aws:kms:'. Got: {}",
            arn
        )));
    }

    let parts: Vec<&str> = arn.split(':').collect();
    if parts.len() != 6 {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid AWS KMS ARN format: expected 6 parts separated by ':', got {}. ARN: {}",
            parts.len(),
            arn
        )));
    }

    // Check that the last part starts with "key/" or "alias/"
    let resource_part = parts[5];
    if !resource_part.starts_with("key/") && !resource_part.starts_with("alias/") {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid AWS KMS ARN: resource part must start with 'key/' or 'alias/'. Got: {}",
            resource_part
        )));
    }

    info!("Valid AWS KMS ARN: {}", arn);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_aws_kms_arn_valid() {
        // Valid key ARN
        assert!(validate_aws_kms_arn("arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012").is_ok());

        // Valid alias ARN
        assert!(validate_aws_kms_arn("arn:aws:kms:us-west-2:987654321098:alias/my-key-alias").is_ok());
    }

    #[test]
    fn test_validate_aws_kms_arn_invalid() {
        // Not starting with arn:aws:kms:
        assert!(validate_aws_kms_arn("arn:aws:s3:us-east-1:123456789012:bucket/my-bucket").is_err());

        // Wrong number of parts
        assert!(validate_aws_kms_arn("arn:aws:kms:us-east-1:key/12345678-1234-1234-1234-123456789012").is_err());

        // Invalid resource type
        assert!(validate_aws_kms_arn("arn:aws:kms:us-east-1:123456789012:invalid/12345678-1234-1234-1234-123456789012").is_err());

        // Empty string
        assert!(validate_aws_kms_arn("").is_err());
    }
}
