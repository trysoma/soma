use std::time::Duration;

use shared::{
    error::CommonError,
    identity::{Role, User, UserType},
    primitives::WrappedChronoDateTime,
};
use tracing::{debug, trace};

use crate::{
    logic::api_key::{CreateApiKeyResponse, cache::ApiKeyCache, generate_api_key, hash_api_key},
    repository::HashedApiKeyWithUser,
};

pub async fn create_bootstrap_api_key(
    api_key_cache: Option<&ApiKeyCache>,
) -> Result<CreateApiKeyResponse, CommonError> {
    debug!(
        "Creating bootstrap API key. This should only be done on start up for initial sync. It will expire in 10 minutes."
    );

    // Generate API key and hash
    let raw_api_key = generate_api_key();
    let hashed_value = hash_api_key(&raw_api_key);

    // Generate unique ID for the API key
    let api_key_id = "bootstrap".to_string();

    // Create user ID for this API key (machine_$generated_id format)
    let user_id = "bootstrap".to_string();
    let now = WrappedChronoDateTime::now();

    // Create the user for this API key (machine type)
    // The user description is the same as the API key description
    let user = User {
        id: user_id.clone(),
        user_type: UserType::Machine,
        email: None,
        role: Role::Admin,
        description: Some("Bootstrap API key for initial sync".to_string()),
        created_at: now,
        updated_at: now,
    };

    // Create the API key
    let api_key = crate::repository::HashedApiKey {
        id: api_key_id.clone(),
        hashed_value: hashed_value.clone(),
        description: Some("Bootstrap API key for initial sync".to_string()),
        user_id: user_id.clone(),
        created_at: now,
        updated_at: now,
    };

    // Update the API key cache if provided
    if let Some(cache) = api_key_cache {
        cache.add(HashedApiKeyWithUser { api_key, user });

        let api_key_id_clone = api_key_id.clone();
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            trace!("Removing bootstrap API key after 10 minutes");
            tokio::time::sleep(Duration::from_secs(600)).await;
            cache_clone.remove_by_id(&api_key_id_clone);
            trace!("Bootstrap API key removed");
        });
    }

    Ok(CreateApiKeyResponse {
        id: api_key_id,
        api_key: raw_api_key,
    })
}
