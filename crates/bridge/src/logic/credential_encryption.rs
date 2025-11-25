use encryption::logic::crypto_services::CryptoCache;
use serde::{Deserialize, Serialize};
use shared::{error::CommonError, primitives::WrappedJsonValue};
use utoipa::ToSchema;

use crate::logic::controller::{
    WithCredentialControllerTypeId, WithProviderControllerTypeId, get_credential_controller,
    get_provider_controller,
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

pub type EncryptConfigurationParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<EncryptCredentialConfigurationParamsInner>,
>;

/// Encrypts a resource server credential configuration using the specified DEK alias.
///
/// This function:
/// 1. Resolves the dek_alias to an actual DEK
/// 2. Gets the encryption service for that DEK
/// 3. Encrypts the resource server configuration using the credential controller
pub async fn encrypt_resource_server_configuration(
    crypto_cache: &CryptoCache,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError> {
    // Resolve the DEK alias to get the encryption service (supports both alias and ID)
    let encryption_service = crypto_cache
        .get_encryption_service(&params.inner.inner.dek_alias)
        .await?;

    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;
    let resource_server_configuration = params.inner.inner.value;

    let encrypted_resource_server_configuration = credential_controller
        .encrypt_resource_server_configuration(&encryption_service, resource_server_configuration)
        .await?;

    Ok(encrypted_resource_server_configuration.value())
}

/// Encrypts a user credential configuration using the specified DEK alias.
///
/// This function:
/// 1. Resolves the dek_alias to an actual DEK
/// 2. Gets the encryption service for that DEK
/// 3. Encrypts the user credential configuration using the credential controller
pub async fn encrypt_user_credential_configuration(
    crypto_cache: &CryptoCache,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError> {
    // Resolve the DEK alias to get the encryption service (supports both alias and ID)
    let encryption_service = crypto_cache
        .get_encryption_service(&params.inner.inner.dek_alias)
        .await?;

    let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
    let credential_controller = get_credential_controller(
        &provider_controller,
        &params.inner.credential_controller_type_id,
    )?;
    let user_credential_configuration = params.inner.inner.value;

    let encrypted_user_credential_configuration = credential_controller
        .encrypt_user_credential_configuration(&encryption_service, user_credential_configuration)
        .await?;

    Ok(encrypted_user_credential_configuration.value())
}
