use encryption::logic::crypto_services::CryptoCache;
use serde::{Deserialize, Serialize};
use shared::{error::CommonError, primitives::WrappedJsonValue};
use shared_macros::{authn, authz_role};
use utoipa::ToSchema;

use crate::logic::deployment::{
    WithCredentialDeploymentTypeId, WithToolGroupDeploymentTypeId,
};

/// Parameters for encrypting credential configuration.
/// Uses dek_alias to look up the DEK to use for encryption.
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EncryptCredentialConfigurationParamsInner {
    /// The raw credential configuration value to encrypt
    pub value: WrappedJsonValue,
    /// The DEK alias to use for encryption (resolved to actual DEK id internally)
    pub dek_alias: String,
}

pub type EncryptedCredentialConfigurationResponse = WrappedJsonValue;

pub type EncryptConfigurationParams = WithToolGroupDeploymentTypeId<
    WithCredentialDeploymentTypeId<EncryptCredentialConfigurationParamsInner>,
>;

/// Encrypts a resource server credential configuration using the specified DEK alias.
///
/// This function:
/// 1. Gets the encryption service for the specified DEK alias
/// 2. Serializes the JSON configuration to a string
/// 3. Encrypts the string and returns it as JSON
///
/// The tool_group_deployment_type_id and credential_deployment_type_id are metadata
/// for routing/organization but don't affect the encryption logic.
#[authz_role(Admin, permission = "credential:write")]
#[authn]
pub async fn encrypt_resource_server_configuration(
    crypto_cache: &CryptoCache,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError> {
    tracing::trace!(
        dek_alias = %params.inner.inner.dek_alias,
        "Encrypting resource server configuration"
    );

    // Get encryption service for the DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&params.inner.inner.dek_alias)
        .await?;

    // Serialize the configuration value to a JSON string
    let config_json = serde_json::to_string(params.inner.inner.value.get_inner())?;

    // Encrypt the serialized configuration
    let encrypted = encryption_service.encrypt_data(config_json).await?;

    tracing::trace!("Resource server configuration encrypted successfully");

    // Return the encrypted string wrapped in JSON
    Ok(WrappedJsonValue::new(serde_json::Value::String(encrypted.0)))
}

/// Encrypts a user credential configuration using the specified DEK alias.
///
/// This function:
/// 1. Gets the encryption service for the specified DEK alias
/// 2. Serializes the JSON configuration to a string
/// 3. Encrypts the string and returns it as JSON
///
/// The tool_group_deployment_type_id and credential_deployment_type_id are metadata
/// for routing/organization but don't affect the encryption logic.
#[authz_role(Admin, permission = "credential:write")]
#[authn]
pub async fn encrypt_user_credential_configuration(
    crypto_cache: &CryptoCache,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError> {
    tracing::trace!(
        dek_alias = %params.inner.inner.dek_alias,
        "Encrypting user credential configuration"
    );

    // Get encryption service for the DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&params.inner.inner.dek_alias)
        .await?;

    // Serialize the configuration value to a JSON string
    let config_json = serde_json::to_string(params.inner.inner.value.get_inner())?;

    // Encrypt the serialized configuration
    let encrypted = encryption_service.encrypt_data(config_json).await?;

    tracing::trace!("User credential configuration encrypted successfully");

    // Return the encrypted string wrapped in JSON
    Ok(WrappedJsonValue::new(serde_json::Value::String(encrypted.0)))
}
