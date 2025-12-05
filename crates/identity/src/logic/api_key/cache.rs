use std::sync::Arc;

use dashmap::DashMap;
use shared::error::CommonError;

use crate::logic::api_key::HashedApiKeyWithUser;
use crate::repository::{Repository, UserRepositoryLike};

/// Cache for API keys with repository fallback
///
/// This cache stores API keys keyed by their hashed value for fast O(1) lookups
/// during authentication. If a key is not found in the cache, it falls back
/// to the repository.
#[derive(Clone)]
pub struct ApiKeyCache {
    /// Cache of API keys keyed by hashed value
    cache: Arc<DashMap<String, HashedApiKeyWithUser>>,
    /// Repository for fallback lookups
    repository: Arc<Repository>,
}

impl ApiKeyCache {
    /// Create a new API key cache with the given repository
    pub fn new(repository: Arc<Repository>) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            repository,
        }
    }

    /// Look up an API key by its hashed value
    ///
    /// First checks the cache, then falls back to the repository if not found.
    /// If found in the repository, the key is added to the cache.
    pub async fn get_by_hashed_value(
        &self,
        hashed_value: &str,
    ) -> Result<Option<HashedApiKeyWithUser>, CommonError> {
        // First check the cache
        if let Some(cached) = self.cache.get(hashed_value) {
            return Ok(Some(cached.value().clone()));
        }

        // Fall back to repository
        let api_key_with_user = self
            .repository
            .get_api_key_by_hashed_value(hashed_value)
            .await?;

        if let Some(api_key_with_user) = api_key_with_user {
            // Add to cache for future lookups
            self.cache
                .insert(hashed_value.to_string(), api_key_with_user.clone());
            Ok(Some(api_key_with_user))
        } else {
            Ok(None)
        }
    }

    /// Add an API key to the cache
    pub fn add(&self, cached_api_key: HashedApiKeyWithUser) {
        self.cache
            .insert(cached_api_key.api_key.hashed_value.clone(), cached_api_key);
    }

    /// Remove an API key from the cache by its ID
    ///
    /// This scans the cache to find and remove the key with the given ID.
    pub fn remove_by_id(&self, id: &str) {
        self.cache.retain(|_, v| v.api_key.id != id);
    }

    /// Remove an API key from the cache by its hashed value
    pub fn remove_by_hashed_value(&self, hashed_value: &str) {
        self.cache.remove(hashed_value);
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Refresh a specific API key from the repository
    ///
    /// This is useful after an API key is updated.
    pub async fn refresh(
        &self,
        hashed_value: &str,
    ) -> Result<Option<HashedApiKeyWithUser>, CommonError> {
        // Remove from cache first
        self.cache.remove(hashed_value);

        // Fetch fresh from repository
        let api_key_with_user = self
            .repository
            .get_api_key_by_hashed_value(hashed_value)
            .await?;

        if let Some(api_key_with_user) = api_key_with_user {
            self.cache
                .insert(hashed_value.to_string(), api_key_with_user.clone());
            Ok(Some(api_key_with_user))
        } else {
            Ok(None)
        }
    }
}
