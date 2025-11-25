use encryption::logic::crypto_services::CryptoCache;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedUuidV4},
};
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

// CRUD functions
pub async fn create_secret<R: SecretRepositoryLike>(
    on_change_tx: &SecretChangeTx,
    repository: &R,
    crypto_cache: &CryptoCache,
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
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let create_params = CreateSecret {
        id,
        key: request.key,
        encrypted_secret: encrypted_secret.0,
        dek_alias: request.dek_alias,
        created_at: now.clone(),
        updated_at: now,
    };

    repository.create_secret(&create_params).await?;

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
    id: WrappedUuidV4,
    request: UpdateSecretRequest,
    publish_on_change_evt: bool,
) -> Result<UpdateSecretResponse, CommonError> {
    // First verify the secret exists and get its dek_alias
    let existing = repository.get_secret_by_id(&id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Secret with id {} not found", id),
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
        updated_at: now.clone(),
    };

    repository.update_secret(&update_params).await?;

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
    id: WrappedUuidV4,
    publish_on_change_evt: bool,
) -> Result<DeleteSecretResponse, CommonError> {
    // First verify the secret exists and get its key
    let existing = repository.get_secret_by_id(&id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Secret with id {} not found", id),
        lookup_id: id.to_string(),
        source: None,
    })?;

    repository.delete_secret(&id).await?;

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
        msg: format!("Secret with id {} not found", id),
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
        msg: format!("Secret with key {} not found", key),
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
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let create_params = CreateSecret {
        id,
        key: request.key,
        encrypted_secret: request.encrypted_value,
        dek_alias: request.dek_alias,
        created_at: now.clone(),
        updated_at: now,
    };

    repository.create_secret(&create_params).await?;

    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use crate::test::encryption_service::setup_test_encryption;
    use shared::primitives::SqlMigrationLoader;

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

        let request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let result = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
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
            create_request,
            false,
        )
        .await
        .unwrap();

        // Delete the secret
        let result = delete_secret(&on_change_tx, &repository, created.id.clone(), true).await;

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
        for i in 0..3 {
            let create_request = CreateSecretRequest {
                key: format!("secret-key-{}", i),
                raw_value: format!("secret-value-{}", i),
                dek_alias: encryption_setup.dek_alias.clone(),
            };

            create_secret(
                &on_change_tx,
                &repository,
                &encryption_setup.crypto_cache,
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
            e => panic!("Expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_import_secret() {
        let repository = setup_test_repository().await;

        let request = ImportSecretRequest {
            key: "my-imported-secret".to_string(),
            encrypted_value: "pre-encrypted-value-from-yaml".to_string(),
            dek_alias: "test-alias".to_string(),
        };

        let result = import_secret(&repository, request.clone()).await;

        assert!(result.is_ok());
        let secret = result.unwrap();
        assert_eq!(secret.key, "my-imported-secret");
        assert_eq!(secret.encrypted_secret, "pre-encrypted-value-from-yaml");
        assert_eq!(secret.dek_alias, "test-alias");

        // Verify the secret was persisted
        let fetched = get_secret_by_key(&repository, "my-imported-secret".to_string()).await;
        assert!(fetched.is_ok());
        let fetched_secret = fetched.unwrap();
        assert_eq!(fetched_secret.key, "my-imported-secret");
        assert_eq!(fetched_secret.encrypted_secret, "pre-encrypted-value-from-yaml");
        assert_eq!(fetched_secret.dek_alias, "test-alias");
    }

    #[tokio::test]
    async fn test_import_secret_duplicate_key() {
        let repository = setup_test_repository().await;

        let request = ImportSecretRequest {
            key: "my-imported-secret".to_string(),
            encrypted_value: "pre-encrypted-value-1".to_string(),
            dek_alias: "test-alias".to_string(),
        };

        // First import should succeed
        let result = import_secret(&repository, request.clone()).await;
        assert!(result.is_ok());

        // Second import with same key should fail
        let request2 = ImportSecretRequest {
            key: "my-imported-secret".to_string(),
            encrypted_value: "pre-encrypted-value-2".to_string(),
            dek_alias: "test-alias".to_string(),
        };
        let result2 = import_secret(&repository, request2).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_create_secret_no_publish() {
        let encryption_setup = setup_test_encryption("test-alias").await;
        let repository = setup_test_repository().await;
        let (on_change_tx, mut on_change_rx) = tokio::sync::broadcast::channel(10);

        let request = CreateSecretRequest {
            key: "my-secret-key".to_string(),
            raw_value: "my-secret-value".to_string(),
            dek_alias: encryption_setup.dek_alias.clone(),
        };

        let result = create_secret(
            &on_change_tx,
            &repository,
            &encryption_setup.crypto_cache,
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
