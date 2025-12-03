use base64::Engine;
use encryption::logic::crypto_services::CryptoCache;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::logic::api_key_cache::{ApiKeyCache, CachedApiKey};
use crate::logic::{ApiKeyCreatedInfo, OnConfigChangeEvt, OnConfigChangeTx};
use crate::repository::{ApiKey, CreateApiKey, CreateUser, UserRepositoryLike};

use super::auth_client::Role;

/// Default DEK alias for API key encryption
pub const DEFAULT_DEK_ALIAS: &str = "default";

/// Parameters for creating an API key
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyParams {
    /// Description of the API key
    pub description: Option<String>,
    /// Role to assign to the API key's user
    pub role: String,
}

/// Response from creating an API key
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    /// The API key ID
    pub id: String,
    /// The raw API key value (only returned once, not stored)
    pub api_key: String,
}

/// Parameters for deleting an API key
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteApiKeyParams {
    /// The ID of the API key to delete
    pub id: String,
}

/// Response from deleting an API key
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteApiKeyResponse {
    /// Whether the deletion was successful
    pub success: bool,
}

/// Parameters for listing API keys
#[derive(Debug)]
pub struct ListApiKeysParams {
    pub pagination: PaginationRequest,
    pub user_id: Option<String>,
}

/// Parameters for importing an API key
#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportApiKeyParams {
    /// The ID of the API key
    pub id: String,
    /// The encrypted hashed value of the API key
    pub encrypted_hashed_value: String,
    /// The DEK alias used for encryption
    pub dek_alias: String,
    /// Description of the API key
    pub description: Option<String>,
    /// Role to assign to the API key's user
    pub role: String,
    /// The user ID for this API key
    pub user_id: String,
}

/// Response from importing an API key
#[derive(Debug, Serialize, ToSchema)]
pub struct ImportApiKeyResponse {
    /// The API key ID
    pub id: String,
    /// Whether the import was successful
    pub success: bool,
}

/// Response from listing API keys
#[derive(Debug, Serialize, ToSchema)]
pub struct ListApiKeysResponse {
    pub items: Vec<ApiKey>,
    pub next_page_token: Option<String>,
}

/// Generate a random API key string in the format sk--{random}
fn generate_api_key() -> String {
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    let random_part = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    format!("sk--{}", random_part)
}

/// Hash an API key using SHA-256
///
/// This function is deterministic - the same input always produces the same output.
/// No salt is used because API keys are:
/// 1. Randomly generated with high entropy (192 bits)
/// 2. Not reused across systems
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let result = hasher.finalize();
    // Use base64 encoding instead of hex
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result)
}

/// Create a new API key
///
/// This function:
/// 1. Generates a random API key in format sk--{random}
/// 2. Hashes the key for storage
/// 3. Creates an associated user for the API key
/// 4. Stores the API key in the repository
/// 5. Optionally updates the API key cache
/// 6. Optionally broadcasts a config change event with encrypted hashed value
pub async fn create_api_key<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    on_config_change_tx: &OnConfigChangeTx,
    api_key_cache: Option<&ApiKeyCache>,
    params: CreateApiKeyParams,
    publish_on_change_evt: bool,
) -> Result<CreateApiKeyResponse, CommonError> {
    // Validate role
    let role = Role::from_str(&params.role).ok_or_else(|| CommonError::InvalidRequest {
        msg: format!(
            "Invalid role '{}'. Valid roles are: admin, maintainer, read-only-maintainer, agent, user",
            params.role
        ),
        source: None,
    })?;

    // Generate API key and hash
    let raw_api_key = generate_api_key();
    let hashed_value = hash_api_key(&raw_api_key);

    // Generate unique ID for the API key
    let api_key_id = Uuid::new_v4().to_string();

    // Create user ID for this API key (machine_$generated_id format)
    let user_id = format!("machine_{}", Uuid::new_v4());
    let now = WrappedChronoDateTime::now();

    // Check if user already exists (shouldn't happen with unique UUID)
    if repository.get_user_by_id(&user_id).await?.is_some() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "API key collision detected, please try again"
        )));
    }

    // Create the user for this API key (machine type)
    // The user description is the same as the API key description
    let create_user = CreateUser {
        id: user_id.clone(),
        user_type: "machine".to_string(),
        email: None,
        role: role.as_str().to_string(),
        description: params.description.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_user(&create_user).await?;

    // Create the API key
    let create_api_key = CreateApiKey {
        id: api_key_id.clone(),
        hashed_value: hashed_value.clone(),
        description: params.description,
        user_id: user_id.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_api_key(&create_api_key).await?;

    // Update the API key cache if provided
    if let Some(cache) = api_key_cache {
        cache.add(CachedApiKey {
            id: api_key_id.clone(),
            hashed_value: hashed_value.clone(),
            role: role.clone(),
            user_id: user_id.clone(),
        });
    }

    // Broadcast config change event with encrypted hashed value
    if publish_on_change_evt {
        // Get encryption service for the default DEK
        let encryption_service = crypto_cache.get_encryption_service(DEFAULT_DEK_ALIAS).await?;

        // Encrypt the hashed value
        let encrypted_hashed_value = encryption_service.encrypt_data(hashed_value).await?;

        on_config_change_tx
            .send(OnConfigChangeEvt::ApiKeyCreated(ApiKeyCreatedInfo {
                id: api_key_id.clone(),
                encrypted_hashed_value: encrypted_hashed_value.0,
                dek_alias: DEFAULT_DEK_ALIAS.to_string(),
                role: role.clone(),
                user_id: user_id.clone(),
            }))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(CreateApiKeyResponse {
        id: api_key_id,
        api_key: raw_api_key,
    })
}

/// Delete an API key
///
/// This function:
/// 1. Verifies the API key exists
/// 2. Deletes the API key from the repository
/// 3. Optionally deletes the associated user
/// 4. Optionally removes from the API key cache
/// 5. Optionally broadcasts a config change event
pub async fn delete_api_key<R: UserRepositoryLike>(
    repository: &R,
    on_config_change_tx: &OnConfigChangeTx,
    api_key_cache: Option<&ApiKeyCache>,
    params: DeleteApiKeyParams,
    publish_on_change_evt: bool,
) -> Result<DeleteApiKeyResponse, CommonError> {
    // Verify the API key exists
    let api_key_with_user = repository
        .get_api_key_by_id(&params.id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "API key not found".to_string(),
            lookup_id: params.id.clone(),
            source: None,
        })?;

    // Delete the API key
    repository.delete_api_key(&params.id).await?;

    // Delete the associated user if it was created for this API key
    // machine users are created specifically for API keys
    if api_key_with_user.user.user_type == "machine" {
        repository.delete_user(&api_key_with_user.user.id).await?;
    }

    // Remove from API key cache if provided
    if let Some(cache) = api_key_cache {
        cache.remove_by_hashed_value(&api_key_with_user.api_key.hashed_value);
    }

    // Broadcast config change event
    if publish_on_change_evt {
        on_config_change_tx
            .send(OnConfigChangeEvt::ApiKeyDeleted(params.id))
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
            })?;
    }

    Ok(DeleteApiKeyResponse { success: true })
}

/// List API keys
///
/// This function lists all API keys with optional filtering by user_id.
/// Note: The raw API key values are never returned, only the hashed values.
pub async fn list_api_keys<R: UserRepositoryLike>(
    repository: &R,
    params: ListApiKeysParams,
) -> Result<ListApiKeysResponse, CommonError> {
    let result = repository
        .list_api_keys(&params.pagination, params.user_id.as_deref())
        .await?;

    Ok(ListApiKeysResponse {
        items: result.items,
        next_page_token: result.next_page_token,
    })
}

/// Import an API key
///
/// This function imports an API key from an encrypted hashed value.
/// It decrypts the hashed value using the specified DEK and stores it.
/// This is used to sync API keys from soma.yaml to the database.
///
/// This function:
/// 1. Decrypts the encrypted hashed value using the specified DEK
/// 2. Creates an associated user for the API key (if it doesn't exist)
/// 3. Stores the API key in the repository (if it doesn't exist)
/// 4. Optionally updates the API key cache
pub async fn import_api_key<R: UserRepositoryLike>(
    repository: &R,
    crypto_cache: &CryptoCache,
    api_key_cache: Option<&ApiKeyCache>,
    params: ImportApiKeyParams,
) -> Result<ImportApiKeyResponse, CommonError> {
    // Validate role
    let role = Role::from_str(&params.role).ok_or_else(|| CommonError::InvalidRequest {
        msg: format!(
            "Invalid role '{}'. Valid roles are: admin, maintainer, read-only-maintainer, agent, user",
            params.role
        ),
        source: None,
    })?;

    // Get decryption service for the specified DEK alias
    let decryption_service = crypto_cache
        .get_decryption_service(&params.dek_alias)
        .await?;

    // Decrypt the hashed value
    let hashed_value = decryption_service
        .decrypt_data(encryption::logic::EncryptedString(
            params.encrypted_hashed_value.clone(),
        ))
        .await?;

    let now = WrappedChronoDateTime::now();

    // Check if API key already exists
    if repository.get_api_key_by_id(&params.id).await?.is_some() {
        // API key already exists, skip import
        return Ok(ImportApiKeyResponse {
            id: params.id,
            success: true,
        });
    }

    // Check if user already exists
    if repository.get_user_by_id(&params.user_id).await?.is_none() {
        // Create the user for this API key (machine type)
        let create_user = CreateUser {
            id: params.user_id.clone(),
            user_type: "machine".to_string(),
            email: None,
            role: role.as_str().to_string(),
            description: params.description.clone(),
            created_at: now,
            updated_at: now,
        };
        repository.create_user(&create_user).await?;
    }

    // Create the API key
    let create_api_key = CreateApiKey {
        id: params.id.clone(),
        hashed_value: hashed_value.clone(),
        description: params.description,
        user_id: params.user_id.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_api_key(&create_api_key).await?;

    // Update the API key cache if provided
    if let Some(cache) = api_key_cache {
        cache.add(CachedApiKey {
            id: params.id.clone(),
            hashed_value: hashed_value.clone(),
            role: role.clone(),
            user_id: params.user_id.clone(),
        });
    }

    Ok(ImportApiKeyResponse {
        id: params.id,
        success: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use encryption::logic::crypto_services::init_crypto_cache;
    use encryption::logic::dek::{CreateDekInnerParams, CreateDekParams};
    use encryption::logic::dek_alias::{CreateAliasInnerParams, CreateAliasParams};
    use encryption::logic::envelope::get_or_create_local_envelope_encryption_key;
    use encryption::repository::{EncryptionKeyRepositoryLike, Repository as EncryptionRepository};
    use shared::primitives::{SqlMigrationLoader, WrappedChronoDateTime};
    use shared::test_utils::repository::setup_in_memory_database;
    use tokio::sync::broadcast;

    struct TestContext {
        identity_repo: Repository,
        crypto_cache: CryptoCache,
        on_config_change_tx: OnConfigChangeTx,
        #[allow(dead_code)]
        temp_dir: tempfile::TempDir,
    }

    async fn setup_test_context() -> TestContext {
        shared::setup_test!();

        // Setup identity database
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let identity_repo = Repository::new(conn);

        // Setup encryption database
        let (_encryption_db, encryption_conn) =
            setup_in_memory_database(vec![EncryptionRepository::load_sql_migrations()])
                .await
                .unwrap();
        let encryption_repo = EncryptionRepository::new(encryption_conn);

        // Create temp dir for local keys
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let key_path = temp_dir.path().join("test-key");

        // Create envelope key
        let envelope_key_contents = get_or_create_local_envelope_encryption_key(&key_path).unwrap();
        let envelope_key =
            encryption::logic::envelope::EnvelopeEncryptionKey::from(envelope_key_contents);
        let create_params = encryption::repository::CreateEnvelopeEncryptionKey::from((
            envelope_key.clone(),
            WrappedChronoDateTime::now(),
        ));
        EncryptionKeyRepositoryLike::create_envelope_encryption_key(
            &encryption_repo,
            &create_params,
        )
        .await
        .unwrap();

        // Create DEK
        let (tx, _rx) = broadcast::channel(100);
        let dek = encryption::logic::dek::create_data_encryption_key(
            &tx,
            &encryption_repo,
            CreateDekParams {
                envelope_encryption_key_id: envelope_key.id(),
                inner: CreateDekInnerParams {
                    id: Some("test-dek".to_string()),
                    encrypted_dek: None,
                },
            },
            temp_dir.path(),
            false,
        )
        .await
        .unwrap();

        // Create CryptoCache
        let crypto_cache = CryptoCache::new(encryption_repo.clone(), temp_dir.path().to_path_buf());
        init_crypto_cache(&crypto_cache).await.unwrap();

        // Create alias for the DEK (using "default" which is what create_api_key expects)
        encryption::logic::dek_alias::create_alias(
            &tx,
            &encryption_repo,
            &crypto_cache,
            CreateAliasParams {
                dek_id: dek.id.clone(),
                inner: CreateAliasInnerParams {
                    alias: DEFAULT_DEK_ALIAS.to_string(),
                },
            },
        )
        .await
        .unwrap();

        let (on_config_change_tx, _rx) = tokio::sync::broadcast::channel(100);

        TestContext {
            identity_repo,
            crypto_cache,
            on_config_change_tx,
            temp_dir,
        }
    }

    #[test]
    fn test_generate_api_key() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();

        // Keys should start with sk--
        assert!(key1.starts_with("sk--"));
        assert!(key2.starts_with("sk--"));

        // Keys should be unique
        assert_ne!(key1, key2);

        // Keys should be the right length (sk-- + 32 base64 chars for 24 bytes)
        assert_eq!(key1.len(), 4 + 32); // "sk--" + 32 chars
    }

    #[test]
    fn test_hash_api_key() {
        let key = "sk--test123456789012345678901234";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);

        // Same key should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be 43 base64 chars (SHA-256 = 32 bytes = 43 base64 chars without padding)
        assert_eq!(hash1.len(), 43);

        // Different keys should produce different hashes
        let different_hash = hash_api_key("sk--different");
        assert_ne!(hash1, different_hash);
    }

    #[tokio::test]
    async fn test_create_api_key() {
        let ctx = setup_test_context().await;

        let params = CreateApiKeyParams {
            description: Some("Test API key".to_string()),
            role: "agent".to_string(),
        };

        let result = create_api_key(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.on_config_change_tx,
            None,
            params,
            false,
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.api_key.starts_with("sk--"));
        assert!(!response.id.is_empty());

        // Verify user was created
        let api_key_with_user = ctx.identity_repo.get_api_key_by_id(&response.id).await.unwrap();
        assert!(api_key_with_user.is_some());
        let api_key_with_user = api_key_with_user.unwrap();
        assert_eq!(api_key_with_user.user.user_type, "machine");
        assert!(api_key_with_user.user.id.starts_with("machine_"));
        assert_eq!(api_key_with_user.user.role, "agent");
    }

    #[tokio::test]
    async fn test_create_api_key_invalid_role() {
        let ctx = setup_test_context().await;

        let params = CreateApiKeyParams {
            description: None,
            role: "invalid-role".to_string(),
        };

        let result = create_api_key(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.on_config_change_tx,
            None,
            params,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_api_key_broadcasts_event() {
        let ctx = setup_test_context().await;
        let mut rx = ctx.on_config_change_tx.subscribe();

        let params = CreateApiKeyParams {
            description: Some("Test key".to_string()),
            role: "user".to_string(),
        };

        let result = create_api_key(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.on_config_change_tx,
            None,
            params,
            true,
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();

        // Check that event was broadcast
        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            OnConfigChangeEvt::ApiKeyCreated(info) => {
                assert_eq!(info.id, response.id);
                assert!(!info.encrypted_hashed_value.is_empty());
                assert_eq!(info.dek_alias, DEFAULT_DEK_ALIAS);
                assert_eq!(info.role, Role::User);
                assert!(!info.user_id.is_empty());
            }
            _ => panic!("Expected ApiKeyCreated event"),
        }
    }

    #[tokio::test]
    async fn test_delete_api_key() {
        let ctx = setup_test_context().await;

        // First create an API key
        let create_params = CreateApiKeyParams {
            description: None,
            role: "agent".to_string(),
        };
        let created = create_api_key(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.on_config_change_tx,
            None,
            create_params,
            false,
        )
        .await
        .unwrap();

        // Now delete it
        let delete_params = DeleteApiKeyParams {
            id: created.id.clone(),
        };
        let result = delete_api_key(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            None,
            delete_params,
            false,
        )
        .await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);

        // Verify API key is gone
        let api_key = ctx.identity_repo.get_api_key_by_id(&created.id).await.unwrap();
        assert!(api_key.is_none());
    }

    #[tokio::test]
    async fn test_delete_api_key_not_found() {
        let ctx = setup_test_context().await;

        let params = DeleteApiKeyParams {
            id: "nonexistent".to_string(),
        };

        let result = delete_api_key(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            None,
            params,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_api_key_broadcasts_event() {
        let ctx = setup_test_context().await;

        // First create an API key
        let create_params = CreateApiKeyParams {
            description: None,
            role: "agent".to_string(),
        };
        let created = create_api_key(
            &ctx.identity_repo,
            &ctx.crypto_cache,
            &ctx.on_config_change_tx,
            None,
            create_params,
            false,
        )
        .await
        .unwrap();

        let mut rx = ctx.on_config_change_tx.subscribe();

        // Now delete it with broadcast
        let delete_params = DeleteApiKeyParams {
            id: created.id.clone(),
        };
        let result = delete_api_key(
            &ctx.identity_repo,
            &ctx.on_config_change_tx,
            None,
            delete_params,
            true,
        )
        .await;
        assert!(result.is_ok());

        // Check that event was broadcast
        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            OnConfigChangeEvt::ApiKeyDeleted(id) => {
                assert_eq!(id, created.id);
            }
            _ => panic!("Expected ApiKeyDeleted event"),
        }
    }

    #[tokio::test]
    async fn test_list_api_keys() {
        let ctx = setup_test_context().await;

        // Create a few API keys
        for _ in 0..3 {
            let params = CreateApiKeyParams {
                description: None,
                role: "user".to_string(),
            };
            create_api_key(
                &ctx.identity_repo,
                &ctx.crypto_cache,
                &ctx.on_config_change_tx,
                None,
                params,
                false,
            )
            .await
            .unwrap();
        }

        // List all
        let params = ListApiKeysParams {
            pagination: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
            user_id: None,
        };

        let result = list_api_keys(&ctx.identity_repo, params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.items.len(), 3);
    }

    #[tokio::test]
    async fn test_list_api_keys_pagination() {
        let ctx = setup_test_context().await;

        // Create 5 API keys
        for _ in 0..5 {
            let params = CreateApiKeyParams {
                description: None,
                role: "user".to_string(),
            };
            create_api_key(
                &ctx.identity_repo,
                &ctx.crypto_cache,
                &ctx.on_config_change_tx,
                None,
                params,
                false,
            )
            .await
            .unwrap();
        }

        // List with page size of 2
        let params = ListApiKeysParams {
            pagination: PaginationRequest {
                page_size: 2,
                next_page_token: None,
            },
            user_id: None,
        };

        let result = list_api_keys(&ctx.identity_repo, params).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get next page
        let params = ListApiKeysParams {
            pagination: PaginationRequest {
                page_size: 2,
                next_page_token: result.next_page_token,
            },
            user_id: None,
        };

        let result = list_api_keys(&ctx.identity_repo, params).await.unwrap();
        assert_eq!(result.items.len(), 2);
    }
}
