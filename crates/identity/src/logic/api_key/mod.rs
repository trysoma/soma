pub mod cache;

use base64::Engine;
use encryption::logic::crypto_services::CryptoCache;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedChronoDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::logic::api_key::cache::ApiKeyCache;
use crate::logic::{DEFAULT_DEK_ALIAS, OnConfigChangeEvt, OnConfigChangeTx, validate_id};
use crate::repository::UserRepositoryLike;

use crate::logic::user::{Role, User, UserType};

/// Parameters for creating an API key
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyParams {
    /// The ID for this API key
    pub id: String,
    /// Description of the API key
    pub description: Option<String>,
    /// Role to assign to the API key's user
    pub role: Role,
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
pub type DeleteApiKeyResponse = ();

/// Parameters for listing API keys
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListApiKeysParams {
    pub pagination: PaginationRequest,
    pub user_id: Option<String>,
}

/// Parameters for importing an API key
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EncryptedApiKeyConfig {
    /// The ID of the API key
    pub id: String,
    /// The encrypted hashed value of the API key
    pub encrypted_hashed_value: String,
    /// The DEK alias used for encryption
    pub dek_alias: String,
    /// Description of the API key
    pub description: Option<String>,
    /// Role to assign to the API key's user
    pub role: Role,
    /// The user ID for this API key
    pub user_id: String,
}

// API key types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct HashedApiKey {
    pub id: String,
    pub hashed_value: String,
    pub description: Option<String>,
    pub user_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct HashedApiKeyWithUser {
    pub api_key: HashedApiKey,
    pub user: User,
}

/// Response from importing an API key
pub type ImportApiKeyResponse = ();

/// Response from listing API keys
#[derive(Debug, Serialize, ToSchema)]
pub struct ListApiKeysResponse {
    pub items: Vec<HashedApiKey>,
    pub next_page_token: Option<String>,
}

/// Generate a random API key string in the format sk--{random}
fn generate_api_key() -> String {
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    let random_part = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    format!("sk--{random_part}")
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
    // Validate the ID
    validate_id(&params.id, "API key")?;

    // Check if API key with this ID already exists
    if repository.get_api_key_by_id(&params.id).await?.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!("API key with ID '{}' already exists", params.id),
            source: None,
        });
    }

    // Generate API key and hash
    let raw_api_key = generate_api_key();
    let hashed_value = hash_api_key(&raw_api_key);

    // Generate unique ID for the API key
    let api_key_id = params.id.clone();

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
    let user = User {
        id: user_id.clone(),
        user_type: UserType::Machine,
        email: None,
        role: params.role.clone(),
        description: params.description.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_user(&user).await?;

    // Create the API key
    let api_key = crate::repository::HashedApiKey {
        id: api_key_id.clone(),
        hashed_value: hashed_value.clone(),
        description: params.description.clone(),
        user_id: user_id.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_api_key(&api_key).await?;

    // Update the API key cache if provided
    if let Some(cache) = api_key_cache {
        cache.add(HashedApiKeyWithUser { api_key, user });
    }

    // Broadcast config change event with encrypted hashed value
    if publish_on_change_evt {
        // Get encryption service for the default DEK
        let encryption_service = crypto_cache
            .get_encryption_service(DEFAULT_DEK_ALIAS)
            .await?;

        // Encrypt the hashed value
        let encrypted_hashed_value = encryption_service.encrypt_data(hashed_value).await?;

        on_config_change_tx
            .send(OnConfigChangeEvt::ApiKeyCreated(EncryptedApiKeyConfig {
                id: api_key_id.clone(),
                encrypted_hashed_value: encrypted_hashed_value.0,
                dek_alias: DEFAULT_DEK_ALIAS.to_string(),
                role: params.role.clone(),
                user_id: user_id.clone(),
                description: params.description.clone(),
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
    if matches!(api_key_with_user.user.user_type, UserType::Machine) {
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

    Ok(())
}

/// List API keys
///
/// This function lists all API keys.
/// Note: The raw API key values are never returned, only the hashed values.
pub async fn list_api_keys<R: UserRepositoryLike>(
    repository: &R,
    params: PaginationRequest,
) -> Result<ListApiKeysResponse, CommonError> {
    let result = repository.list_api_keys(&params, None).await?;

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
    params: EncryptedApiKeyConfig,
) -> Result<ImportApiKeyResponse, CommonError> {
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
        return Ok(());
    }

    let user = match repository.get_user_by_id(&params.user_id).await? {
        Some(user) => user,
        None => {
            // Create the user for this API key (machine type)
            let user = User {
                id: params.user_id.clone(),
                user_type: UserType::Machine,
                email: None,
                role: params.role.clone(),
                description: params.description.clone(),
                created_at: now,
                updated_at: now,
            };
            repository.create_user(&user).await?;
            user
        }
    };

    // Create the API key
    let api_key = crate::repository::HashedApiKey {
        id: params.id.clone(),
        hashed_value: hashed_value.clone(),
        description: params.description.clone(),
        user_id: params.user_id.clone(),
        created_at: now,
        updated_at: now,
    };
    repository.create_api_key(&api_key).await?;

    // Update the API key cache if provided
    if let Some(cache) = api_key_cache {
        cache.add(HashedApiKeyWithUser {
            api_key: HashedApiKey {
                id: params.id.clone(),
                hashed_value: hashed_value.clone(),
                description: params.description.clone(),
                user_id: params.user_id.clone(),
                created_at: now,
                updated_at: now,
            },
            user,
        });
    }

    Ok(())
}
