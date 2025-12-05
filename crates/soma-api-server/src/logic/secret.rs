use encryption::logic::crypto_services::CryptoCache;
use schemars::JsonSchema;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedUuidV4},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::warn;
use utoipa::ToSchema;

use crate::{
    logic::on_change_pubsub::{SecretChangeEvt, SecretChangeTx},
    repository::{CreateSecret, SecretRepositoryLike, UpdateSecret},
};

// Domain model for Secret
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Secret {
    pub id: WrappedUuidV4,
    pub key: String,
    pub encrypted_secret: String,
    pub dek_alias: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateSecretRequest {
    pub key: String,
    pub raw_value: String,
    pub dek_alias: String,
}

pub type CreateSecretResponse = Secret;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateSecretRequest {
    pub raw_value: String,
}

pub type UpdateSecretResponse = Secret;

pub type GetSecretResponse = Secret;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListSecretsResponse {
    pub secrets: Vec<Secret>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DeleteSecretResponse {
    pub success: bool,
}

// Decrypted secret type for list-decrypted endpoint
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DecryptedSecret {
    pub id: WrappedUuidV4,
    pub key: String,
    pub decrypted_value: String,
    pub dek_alias: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListDecryptedSecretsResponse {
    pub secrets: Vec<DecryptedSecret>,
    pub next_page_token: Option<String>,
}

// CRUD functions
/// Helper to incrementally sync a single secret to SDK
async fn sync_secret_to_sdk_incremental(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    crypto_cache: &CryptoCache,
    key: String,
    encrypted_secret: String,
    dek_alias: String,
) {
    let mut sdk_client_guard = sdk_client.lock().await;

    if let Some(ref mut client) = *sdk_client_guard {
        // Get decryption service for this secret's DEK alias
        match crypto_cache.get_decryption_service(&dek_alias).await {
            Ok(decryption_service) => {
                // Decrypt the secret value
                use encryption::logic::crypto_services::EncryptedString;
                match decryption_service
                    .decrypt_data(EncryptedString(encrypted_secret))
                    .await
                {
                    Ok(decrypted_value) => {
                        use crate::logic::secret_sync::sync_secret_to_sdk;
                        if let Err(e) =
                            sync_secret_to_sdk(client, key.clone(), decrypted_value).await
                        {
                            warn!("Failed to sync secret '{}' to SDK: {:?}", key, e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to decrypt secret '{}': {:?}", key, e);
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to get decryption service for DEK alias '{}': {:?}",
                    dek_alias, e
                );
            }
        }
    }
}

/// Helper to unset a secret in SDK
async fn unset_secret_in_sdk_incremental(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    key: String,
) {
    let mut sdk_client_guard = sdk_client.lock().await;

    if let Some(ref mut client) = *sdk_client_guard {
        use crate::logic::secret_sync::unset_secret_in_sdk;
        if let Err(e) = unset_secret_in_sdk(client, key.clone()).await {
            warn!("Failed to unset secret '{}' in SDK: {:?}", key, e);
        }
    }
}

pub async fn create_secret<R: SecretRepositoryLike>(
    on_change_tx: &SecretChangeTx,
    repository: &R,
    crypto_cache: &CryptoCache,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    request: CreateSecretRequest,
    publish_on_change_evt: bool,
) -> Result<CreateSecretResponse, CommonError> {
    // Get encryption service for the DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&request.dek_alias)
        .await?;

    // Encrypt the raw value
    let encrypted_secret = encryption_service.encrypt_data(request.raw_value).await?;

    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let secret = Secret {
        id: id.clone(),
        key: request.key.clone(),
        encrypted_secret: encrypted_secret.0.clone(),
        dek_alias: request.dek_alias.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateSecret {
        id,
        key: request.key,
        encrypted_secret: encrypted_secret.0,
        dek_alias: request.dek_alias,
        created_at: now,
        updated_at: now,
    };

    repository.create_secret(&create_params).await?;

    // Incrementally sync the new secret to SDK
    sync_secret_to_sdk_incremental(
        sdk_client,
        crypto_cache,
        secret.key.clone(),
        secret.encrypted_secret.clone(),
        secret.dek_alias.clone(),
    )
    .await;

    if publish_on_change_evt {
        on_change_tx
            .send(SecretChangeEvt::Created(secret.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send secret change event: {e}"))
            })?;
    }

    Ok(secret)
}

pub async fn update_secret<R: SecretRepositoryLike>(
    on_change_tx: &SecretChangeTx,
    repository: &R,
    crypto_cache: &CryptoCache,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    id: WrappedUuidV4,
    request: UpdateSecretRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateSecretResponse, CommonError> {
    // First verify the secret exists and get its dek_alias
    let existing = repository.get_secret_by_id(&id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Secret with id {id} not found"),
        lookup_id: id.to_string(),
        source: None,
    })?;

    // Get encryption service using the existing secret's DEK alias
    let encryption_service = crypto_cache
        .get_encryption_service(&existing.dek_alias)
        .await?;

    // Encrypt the new raw value
    let encrypted_secret = encryption_service.encrypt_data(request.raw_value).await?;

    let now = WrappedChronoDateTime::now();

    let update_params = UpdateSecret {
        id: id.clone(),
        encrypted_secret: encrypted_secret.0.clone(),
        dek_alias: existing.dek_alias.clone(),
        updated_at: now,
    };

    repository.update_secret(&update_params).await?;

    // Incrementally sync the updated secret to SDK
    sync_secret_to_sdk_incremental(
        sdk_client,
        crypto_cache,
        existing.key.clone(),
        encrypted_secret.0.clone(),
        existing.dek_alias.clone(),
    )
    .await;

    let updated_secret = Secret {
        id,
        key: existing.key,
        encrypted_secret: encrypted_secret.0,
        dek_alias: existing.dek_alias,
        created_at: existing.created_at,
        updated_at: now,
    };

    if publish_on_change_evt {
        on_change_tx
            .send(SecretChangeEvt::Updated(updated_secret.clone()))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send secret change event: {e}"))
            })?;
    }

    Ok(updated_secret)
}

pub async fn delete_secret<R: SecretRepositoryLike>(
    on_change_tx: &SecretChangeTx,
    repository: &R,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    _crypto_cache: &CryptoCache,
    id: WrappedUuidV4,
    publish_on_change_evt: bool,
) -> Result<DeleteSecretResponse, CommonError> {
    // First verify the secret exists and get its key
    let existing = repository.get_secret_by_id(&id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Secret with id {id} not found"),
        lookup_id: id.to_string(),
        source: None,
    })?;

    repository.delete_secret(&id).await?;

    // Unset the deleted secret in SDK
    unset_secret_in_sdk_incremental(sdk_client, existing.key.clone()).await;

    if publish_on_change_evt {
        on_change_tx
            .send(SecretChangeEvt::Deleted {
                id: id.to_string(),
                key: existing.key,
            })
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send secret change event: {e}"))
            })?;
    }

    Ok(DeleteSecretResponse { success: true })
}

pub async fn get_secret_by_id<R: SecretRepositoryLike>(
    repository: &R,
    id: WrappedUuidV4,
) -> Result<GetSecretResponse, CommonError> {
    let secret = repository.get_secret_by_id(&id).await?;
    let secret = secret.ok_or_else(|| CommonError::NotFound {
        msg: format!("Secret with id {id} not found"),
        lookup_id: id.to_string(),
        source: None,
    })?;

    Ok(secret)
}

pub async fn get_secret_by_key<R: SecretRepositoryLike>(
    repository: &R,
    key: String,
) -> Result<GetSecretResponse, CommonError> {
    let secret = repository.get_secret_by_key(&key).await?;
    let secret = secret.ok_or_else(|| CommonError::NotFound {
        msg: format!("Secret with key {key} not found"),
        lookup_id: key.clone(),
        source: None,
    })?;

    Ok(secret)
}

pub async fn list_secrets<R: SecretRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListSecretsResponse, CommonError> {
    let paginated: PaginatedResponse<Secret> = repository.get_secrets(&pagination).await?;

    Ok(ListSecretsResponse {
        secrets: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// List all secrets with their decrypted values
pub async fn list_decrypted_secrets<R: SecretRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    pagination: PaginationRequest,
) -> Result<ListDecryptedSecretsResponse, CommonError> {
    use encryption::logic::crypto_services::EncryptedString;

    let paginated: PaginatedResponse<Secret> = repository.get_secrets(&pagination).await?;

    let mut decrypted_secrets = Vec::with_capacity(paginated.items.len());
    for secret in paginated.items {
        // Get the decryption service for this secret's DEK alias
        let decryption_service = crypto_cache
            .get_decryption_service(&secret.dek_alias)
            .await?;

        // Decrypt the secret value
        let decrypted_value = decryption_service
            .decrypt_data(EncryptedString(secret.encrypted_secret.clone()))
            .await?;

        decrypted_secrets.push(DecryptedSecret {
            id: secret.id,
            key: secret.key,
            decrypted_value,
            dek_alias: secret.dek_alias,
            created_at: secret.created_at,
            updated_at: secret.updated_at,
        });
    }

    Ok(ListDecryptedSecretsResponse {
        secrets: decrypted_secrets,
        next_page_token: paginated.next_page_token,
    })
}

// Request type for importing pre-encrypted secrets (used by sync_yaml_to_api_on_start)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ImportSecretRequest {
    pub key: String,
    pub encrypted_value: String,
    pub dek_alias: String,
}

/// Import a pre-encrypted secret (used for syncing from soma.yaml)
/// This does NOT publish change events since it's used for initial sync
pub async fn import_secret<R: SecretRepositoryLike>(
    repository: &R,
    request: ImportSecretRequest,
) -> Result<Secret, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let secret = Secret {
        id: id.clone(),
        key: request.key.clone(),
        encrypted_secret: request.encrypted_value.clone(),
        dek_alias: request.dek_alias.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateSecret {
        id,
        key: request.key,
        encrypted_secret: request.encrypted_value,
        dek_alias: request.dek_alias,
        created_at: now,
        updated_at: now,
    };

    repository.create_secret(&create_params).await?;

    Ok(secret)
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::repository::Repository;
    use crate::test::encryption_service::setup_test_encryption;
    use shared::primitives::SqlMigrationLoader;

    fn create_test_sdk_client() -> Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>> {
        Arc::new(Mutex::new(None::<SomaSdkServiceClient<Channel>>))
    }

    async fn setup_test_repository() -> Repository {
        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            <Repository as SqlMigrationLoader>::load_sql_migrations(),
        ])
        .await
        .expect("Failed to setup test database");
        Repository::new(conn)
    }

    #[tokio::test]
    async fn test_create_secret() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        let request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let result = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            request.clone(),
            true,
        )
        .await;

        assert!(result.is_ok());
        let secret = result.unwrap();
        assert_eq!(secret.key, "my-secret-key");
        assert_eq!(secret.dek_alias, encryption_setup.dek_alias);
        assert!(!secret.encrypted_secret.is_empty());
        // Encrypted value should be different from raw value
        assert_ne!(secret.encrypted_secret, "my-secret-value");

        // Check event was published
        let event = on_change_rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            SecretChangeEvt::Created(s) => {
                assert_eq!(s.key, "my-secret-key");
            }
            _ => panic!("Expected Created event"),
        }
    }

    #[tokio::test]
    async fn test_update_secret() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        // Create a secret first
        let create_request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "original-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let created = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Update the secret
        let update_request = UpdateSecretRequest {
            raw_value: "updated-value".to_string(),
        };

        let result = update_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            created.id.clone(),
            update_request,
            true,
        )
        .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.key, "my-secret-key");
        assert_ne!(updated.encrypted_secret, created.encrypted_secret);

        // Check event was published
        let event = on_change_rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            SecretChangeEvt::Updated(s) => {
                assert_eq!(s.key, "my-secret-key");
            }
            _ => panic!("Expected Updated event"),
        }
    }

    #[tokio::test]
    async fn test_delete_secret() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        // Create a secret first
        let create_request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let created = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Delete the secret
        let result = delete_secret(
            &on_change_tx,
            &repository,
            &sdk_client,
            &encryption_setup.crypto_cache,
            created.id.clone(),
            true,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);

        // Check event was published
        let event = on_change_rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            SecretChangeEvt::Deleted { id, key } => {
                assert_eq!(id, created.id.to_string());
                assert_eq!(key, "my-secret-key");
            }
            _ => panic!("Expected Deleted event"),
        }

        // Verify it's deleted
        let get_result = get_secret_by_id(&repository, created.id).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_get_secret_by_id() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        // Create a secret first
        let create_request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let created = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Get by id
        let result = get_secret_by_id(&repository, created.id.clone()).await;

        assert!(result.is_ok());
        let secret = result.unwrap();
        assert_eq!(secret.key, "my-secret-key");
        assert_eq!(secret.id, created.id);
    }

    #[tokio::test]
    async fn test_get_secret_by_key() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        // Create a secret first
        let create_request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let created = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            create_request,
            false,
        )
        .await
        .unwrap();

        // Get by key
        let result = get_secret_by_key(&repository, "my-secret-key".to_string()).await;

        assert!(result.is_ok());
        let secret = result.unwrap();
        assert_eq!(secret.id, created.id);
        assert_eq!(secret.key, "my-secret-key");
    }

    #[tokio::test]
    async fn test_list_secrets() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, _on_change_rx) = tokio::sync::broadcast::channel(10);

        // Create multiple secrets
        let sdk_client = create_test_sdk_client();
        for i in 0..3 {
            let create_request = CreateSecretRequest {
                key: format!("secret-key-{i}"),
                raw_value: format!("secret-value-{i}"),
                dek_alias: encryption_setup.dek_alias.clone(),
            };

            create_secret(
                &on_change_tx,
                &repository,
                &encryption_setup.crypto_cache,
                &sdk_client,
                create_request,
                false,
            )
            .await
            .unwrap();
        }

        // List secrets
        let result = list_secrets(
            &repository,
            PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.secrets.len(), 3);
    }

    #[tokio::test]
    async fn test_get_secret_not_found() {
        let repository = setup_test_repository().await;

        let result = get_secret_by_id(&repository, WrappedUuidV4::new()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CommonError::NotFound { .. } => {}
            e => panic!("Expected NotFound error, got: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_create_secret_no_publish() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);
        let sdk_client = create_test_sdk_client();

        let request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let result = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
            &sdk_client,
            request,
            false, // Don't publish
        )
        .await;

        assert!(result.is_ok());

        // Should be no event
        let event = on_change_rx.try_recv();
        assert!(event.is_err());
    }
}
