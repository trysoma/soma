// Re-export encryption crate types
pub use encryption::*;

use serde::{Deserialize, Serialize};
use shared::{error::CommonError, primitives::WrappedJsonValue};
use utoipa::ToSchema;

use crate::logic::{
    OnConfigChangeEvt, OnConfigChangeTx,
    controller::{
        WithCredentialControllerTypeId, WithProviderControllerTypeId, get_credential_controller,
        get_provider_controller,
    },
};

// Bridge-specific data encryption key management functions

pub async fn create_data_encryption_key<R>(
    key_encryption_key: &EnvelopeEncryptionKeyContents,
    on_config_change_tx: &OnConfigChangeTx,
    repo: &R,
    params: CreateDataEncryptionKeyParams,
    publish_on_change_evt: bool,
) -> Result<CreateDataEncryptionKeyResponse, CommonError>
where
    R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    let dek = encryption::create_data_encryption_key(key_encryption_key, repo, params).await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::DataEncryptionKeyAdded(dek.clone()))
            .await?;
    }

    Ok(dek)
}

pub async fn delete_data_encryption_key<R>(
    on_config_change_tx: &OnConfigChangeTx,
    repo: &R,
    id: DeleteDataEncryptionKeyParams,
    publish_on_change_evt: bool,
) -> Result<DeleteDataEncryptionKeyResponse, CommonError>
where
    R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    encryption::delete_data_encryption_key(repo, id.clone()).await?;

    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::DataEncryptionKeyRemoved(id))
            .await?;
    }

    Ok(())
}

// Bridge-specific credential encryption functions

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EncryptCredentialConfigurationParamsInner {
    pub value: WrappedJsonValue,
    pub data_encryption_key_id: String,
}

pub type EncryptedCredentialConfigurationResponse = WrappedJsonValue;

pub type EncryptConfigurationParams = WithProviderControllerTypeId<
    WithCredentialControllerTypeId<EncryptCredentialConfigurationParamsInner>,
>;

pub async fn encrypt_resource_server_configuration<R>(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &R,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError>
where
    R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    let crypto_service = encryption::get_crypto_service(
        envelope_encryption_key_contents,
        repo,
        &params.inner.inner.data_encryption_key_id,
    )
    .await?;
    let encryption_service = encryption::get_encryption_service(&crypto_service)?;
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

pub async fn encrypt_user_credential_configuration<R>(
    envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
    repo: &R,
    params: EncryptConfigurationParams,
) -> Result<EncryptedCredentialConfigurationResponse, CommonError>
where
    R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    let crypto_service = encryption::get_crypto_service(
        envelope_encryption_key_contents,
        repo,
        &params.inner.inner.data_encryption_key_id,
    )
    .await?;
    let encryption_service = encryption::get_encryption_service(&crypto_service)?;
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
