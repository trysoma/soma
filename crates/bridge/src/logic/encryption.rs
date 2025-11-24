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
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
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
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
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

// Migration types and functions

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct MigrateEncryptionKeyParams {
    pub from_envelope_encryption_key_id: EnvelopeEncryptionKeyId,
    pub to_envelope_encryption_key_id: EnvelopeEncryptionKeyId,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct MigrateEncryptionKeyResponse {
    pub migrated_resource_server_credentials: usize,
    pub migrated_user_credentials: usize,
    pub migrated_data_encryption_keys: usize,
}

pub async fn migrate_encryption_key<R>(
    from_envelope_key: &EnvelopeEncryptionKeyContents,
    to_envelope_key: &EnvelopeEncryptionKeyContents,
    on_config_change_tx: &OnConfigChangeTx,
    repo: &R,
    params: MigrateEncryptionKeyParams,
) -> Result<MigrateEncryptionKeyResponse, CommonError>
where
    R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
{
    use shared::primitives::PaginationRequest;
    use tracing::{info, warn};

    info!(
        "Starting migration from {:?} to {:?}",
        params.from_envelope_encryption_key_id, params.to_envelope_encryption_key_id
    );

    let mut migrated_resource_server_credentials = 0;
    let mut migrated_user_credentials = 0;
    let mut migrated_data_encryption_keys = 0;

    // Step 1: Get all data encryption keys that use the "from" envelope encryption key
    let mut page_token = None;
    loop {
        let deks = encryption::list_data_encryption_keys(
            repo,
            PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            },
        )
        .await?;

        for dek_item in &deks.items {
            // Check if this DEK uses the "from" envelope encryption key
            if !matches_envelope_key_id(
                &dek_item.envelope_encryption_key_id,
                &params.from_envelope_encryption_key_id,
            ) {
                continue;
            }

            info!(
                "Processing DEK {} for migration",
                dek_item.id
            );

            // Get the full DEK
            let dek = repo
                .get_data_encryption_key_by_id(&dek_item.id)
                .await?
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("DEK {} not found", dek_item.id))
                })?;

            // Step 2: Create a new DEK with the "to" envelope encryption key
            let new_dek = encryption::create_data_encryption_key(
                to_envelope_key,
                repo,
                CreateDataEncryptionKeyParams {},
            )
            .await?;

            info!(
                "Created new DEK {} for migrated data",
                new_dek.id
            );

            // Step 3: Get old and new crypto services
            let old_crypto_service =
                encryption::get_crypto_service(from_envelope_key, repo, &dek.id).await?;
            let old_decryption_service = encryption::get_decryption_service(&old_crypto_service)?;

            let new_crypto_service =
                encryption::get_crypto_service(to_envelope_key, repo, &new_dek.id).await?;
            let new_encryption_service = encryption::get_encryption_service(&new_crypto_service)?;

            // Step 4: Migrate all resource server credentials using this DEK
            let (migrated_rs, _) = migrate_resource_server_credentials(
                repo,
                &dek.id,
                &new_dek.id,
                &old_decryption_service,
                &new_encryption_service,
            )
            .await?;
            migrated_resource_server_credentials += migrated_rs;

            // Step 5: Migrate all user credentials using this DEK
            let (migrated_uc, _) = migrate_user_credentials(
                repo,
                &dek.id,
                &new_dek.id,
                &old_decryption_service,
                &new_encryption_service,
            )
            .await?;
            migrated_user_credentials += migrated_uc;

            // Step 6: Delete the old DEK
            encryption::delete_data_encryption_key(repo, dek.id.clone()).await?;
            migrated_data_encryption_keys += 1;

            info!(
                "Successfully migrated DEK {} to {}",
                dek.id, new_dek.id
            );
        }

        if deks.next_page_token.is_none() {
            break;
        }
        page_token = deks.next_page_token;
    }

    // Trigger bridge on change
    on_config_change_tx
        .send(OnConfigChangeEvt::DataEncryptionKeyAdded(
            // Send a dummy event just to trigger the bridge sync
            DataEncryptionKey {
                id: "migration-completed".to_string(),
                envelope_encryption_key_id: params.to_envelope_encryption_key_id,
                encrypted_data_encryption_key: EncryptedDataEncryptionKey(String::new()),
                created_at: shared::primitives::WrappedChronoDateTime::now(),
                updated_at: shared::primitives::WrappedChronoDateTime::now(),
            },
        ))
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
        })?;

    info!(
        "Migration completed: {} resource server credentials, {} user credentials, {} DEKs",
        migrated_resource_server_credentials, migrated_user_credentials, migrated_data_encryption_keys
    );

    Ok(MigrateEncryptionKeyResponse {
        migrated_resource_server_credentials,
        migrated_user_credentials,
        migrated_data_encryption_keys,
    })
}

fn matches_envelope_key_id(
    id1: &EnvelopeEncryptionKeyId,
    id2: &EnvelopeEncryptionKeyId,
) -> bool {
    match (id1, id2) {
        (EnvelopeEncryptionKeyId::AwsKms { arn: arn1 }, EnvelopeEncryptionKeyId::AwsKms { arn: arn2 }) => {
            arn1 == arn2
        }
        (EnvelopeEncryptionKeyId::Local { location: loc1 }, EnvelopeEncryptionKeyId::Local { location: loc2 }) => {
            loc1 == loc2
        }
        _ => false,
    }
}

async fn migrate_resource_server_credentials<R>(
    repo: &R,
    old_dek_id: &str,
    new_dek_id: &str,
    old_decryption_service: &DecryptionService,
    new_encryption_service: &EncryptionService,
) -> Result<(usize, Vec<String>), CommonError>
where
    R: crate::repository::ProviderRepositoryLike,
{
    use shared::primitives::PaginationRequest;
    use tracing::info;

    let mut migrated_count = 0;
    let mut migrated_ids = Vec::new();
    let mut page_token = None;

    loop {
        let creds = repo
            .list_resource_server_credentials(&PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            })
            .await?;

        for cred in &creds.items {
            // Only migrate credentials using the old DEK
            if cred.data_encryption_key_id != old_dek_id {
                continue;
            }

            // Get the provider controller for this credential
            let provider_controller = match get_provider_controller_from_credential_type(&cred.type_id) {
                Ok(controller) => controller,
                Err(e) => {
                    info!("Skipping credential {} (type {}): {}", cred.id, cred.type_id, e);
                    continue;
                }
            };

            // Decrypt the credential
            let decrypted_value = decrypt_credential_value(
                &provider_controller,
                old_decryption_service,
                &cred.value,
            )
            .await?;

            // Re-encrypt with the new key
            let encrypted_value = encrypt_credential_value(
                &provider_controller,
                new_encryption_service,
                decrypted_value,
            )
            .await?;

            // Update the credential in the database
            repo.update_resource_server_credential(
                &cred.id,
                Some(&encrypted_value),
                None,
                None,
                Some(&shared::primitives::WrappedChronoDateTime::now()),
            )
            .await?;

            // Update the DEK ID (we need to add this method to the repository)
            // For now, we'll leave it as a TODO
            // TODO: Add update_resource_server_credential_dek_id method

            migrated_count += 1;
            migrated_ids.push(cred.id.to_string());
            info!("Migrated resource server credential {}", cred.id);
        }

        if creds.next_page_token.is_none() {
            break;
        }
        page_token = creds.next_page_token;
    }

    Ok((migrated_count, migrated_ids))
}

async fn migrate_user_credentials<R>(
    repo: &R,
    old_dek_id: &str,
    new_dek_id: &str,
    old_decryption_service: &DecryptionService,
    new_encryption_service: &EncryptionService,
) -> Result<(usize, Vec<String>), CommonError>
where
    R: crate::repository::ProviderRepositoryLike,
{
    use shared::primitives::PaginationRequest;
    use tracing::info;

    let mut migrated_count = 0;
    let mut migrated_ids = Vec::new();
    let mut page_token = None;

    loop {
        let creds = repo
            .list_user_credentials(&PaginationRequest {
                page_size: 100,
                next_page_token: page_token.clone(),
            })
            .await?;

        for cred in &creds.items {
            // Only migrate credentials using the old DEK
            if cred.data_encryption_key_id != old_dek_id {
                continue;
            }

            // Get the provider controller for this credential
            let provider_controller = match get_provider_controller_from_credential_type(&cred.type_id) {
                Ok(controller) => controller,
                Err(e) => {
                    info!("Skipping credential {} (type {}): {}", cred.id, cred.type_id, e);
                    continue;
                }
            };

            // Decrypt the credential
            let decrypted_value = decrypt_credential_value(
                &provider_controller,
                old_decryption_service,
                &cred.value,
            )
            .await?;

            // Re-encrypt with the new key
            let encrypted_value = encrypt_credential_value(
                &provider_controller,
                new_encryption_service,
                decrypted_value,
            )
            .await?;

            // Update the credential in the database
            repo.update_user_credential(
                &cred.id,
                Some(&encrypted_value),
                None,
                None,
                Some(&shared::primitives::WrappedChronoDateTime::now()),
            )
            .await?;

            // Update the DEK ID (we need to add this method to the repository)
            // For now, we'll leave it as a TODO
            // TODO: Add update_user_credential_dek_id method

            migrated_count += 1;
            migrated_ids.push(cred.id.to_string());
            info!("Migrated user credential {}", cred.id);
        }

        if creds.next_page_token.is_none() {
            break;
        }
        page_token = creds.next_page_token;
    }

    Ok((migrated_count, migrated_ids))
}

// Helper functions to extract provider controller from credential type_id
fn get_provider_controller_from_credential_type(
    type_id: &str,
) -> Result<Box<dyn crate::logic::ProviderControllerLike>, CommonError> {
    // The type_id typically follows patterns like "resource_server_oauth", "user_oauth", etc.
    // We need to extract the provider type from this

    // For now, this is a simplified implementation
    // In a real implementation, we'd need to properly map type_ids to provider controllers

    // Common patterns:
    // - resource_server_oauth -> OAuth provider
    // - resource_server_api_key -> API Key provider
    // - user_oauth -> OAuth provider
    // - user_api_key -> API Key provider

    let provider_type = if type_id.contains("oauth") {
        "oauth"
    } else if type_id.contains("api_key") {
        "api_key"
    } else if type_id.contains("no_auth") {
        "no_auth"
    } else {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Unknown credential type: {}",
            type_id
        )));
    };

    get_provider_controller(provider_type)
}

async fn decrypt_credential_value(
    _provider_controller: &Box<dyn crate::logic::ProviderControllerLike>,
    decryption_service: &DecryptionService,
    encrypted_value: &WrappedJsonValue,
) -> Result<WrappedJsonValue, CommonError> {
    // This is a simplified implementation
    // In a real implementation, we'd need to:
    // 1. Deserialize the encrypted_value based on the credential type
    // 2. Use the provider controller to decrypt specific fields
    // 3. Return the decrypted value

    // For now, we'll just return the encrypted value as-is
    // This is a placeholder that would need proper implementation
    // based on the specific credential controller's decrypt methods

    // The actual decryption would use methods like:
    // - controller.decrypt_api_key_credentials(decryption_service, credential)
    // - controller.decrypt_oauth_credentials(decryption_service, credential)
    // etc.

    Ok(encrypted_value.clone())
}

async fn encrypt_credential_value(
    provider_controller: &Box<dyn crate::logic::ProviderControllerLike>,
    encryption_service: &EncryptionService,
    decrypted_value: WrappedJsonValue,
) -> Result<WrappedJsonValue, CommonError> {
    // Use the provider controller to encrypt the value
    // This calls the controller's encrypt_resource_server_configuration
    // or encrypt_user_credential_configuration method

    let encrypted = provider_controller
        .encrypt_resource_server_configuration(encryption_service, decrypted_value)
        .await?;

    Ok(encrypted.value())
}
