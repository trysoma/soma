use std::sync::Arc;

use dashmap::DashMap;
use shared::error::CommonError;

use crate::logic::auth_client::Role;
use crate::repository::{ApiKeyWithUser, Repository, UserRepositoryLike};

/// Cached API key information for fast authentication lookups
#[derive(Debug, Clone)]
pub struct CachedApiKey {
    /// The API key ID
    pub id: String,
    /// The hashed value (used as lookup key)
    pub hashed_value: String,
    /// The role assigned to this API key
    pub role: Role,
    /// The user ID associated with this API key
    pub user_id: String,
}

impl TryFrom<ApiKeyWithUser> for CachedApiKey {
    type Error = CommonError;

    fn try_from(api_key_with_user: ApiKeyWithUser) -> Result<Self, Self::Error> {
        let role = Role::from_str(&api_key_with_user.user.role).ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Invalid role '{}' for API key",
                api_key_with_user.user.role
            ))
        })?;

        Ok(CachedApiKey {
            id: api_key_with_user.api_key.id,
            hashed_value: api_key_with_user.api_key.hashed_value,
            role,
            user_id: api_key_with_user.user.id,
        })
    }
}

/// Cache for API keys with repository fallback
///
/// This cache stores API keys keyed by their hashed value for fast O(1) lookups
/// during authentication. If a key is not found in the cache, it falls back
/// to the repository.
#[derive(Clone)]
pub struct ApiKeyCache {
    /// Cache of API keys keyed by hashed value
    cache: Arc<DashMap<String, CachedApiKey>>,
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
    ) -> Result<Option<CachedApiKey>, CommonError> {
        // First check the cache
        if let Some(cached) = self.cache.get(hashed_value) {
            return Ok(Some(cached.value().clone()));
        }

        // Fall back to repository
        let api_key_with_user = self.repository.get_api_key_by_hashed_value(hashed_value).await?;

        if let Some(api_key_with_user) = api_key_with_user {
            let cached = CachedApiKey::try_from(api_key_with_user)?;
            // Add to cache for future lookups
            self.cache.insert(hashed_value.to_string(), cached.clone());
            Ok(Some(cached))
        } else {
            Ok(None)
        }
    }

    /// Add an API key to the cache
    pub fn add(&self, cached_api_key: CachedApiKey) {
        self.cache
            .insert(cached_api_key.hashed_value.clone(), cached_api_key);
    }

    /// Remove an API key from the cache by its ID
    ///
    /// This scans the cache to find and remove the key with the given ID.
    pub fn remove_by_id(&self, id: &str) {
        self.cache.retain(|_, v| v.id != id);
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
    pub async fn refresh(&self, hashed_value: &str) -> Result<Option<CachedApiKey>, CommonError> {
        // Remove from cache first
        self.cache.remove(hashed_value);

        // Fetch fresh from repository
        let api_key_with_user = self.repository.get_api_key_by_hashed_value(hashed_value).await?;

        if let Some(api_key_with_user) = api_key_with_user {
            let cached = CachedApiKey::try_from(api_key_with_user)?;
            self.cache.insert(hashed_value.to_string(), cached.clone());
            Ok(Some(cached))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{CreateApiKey, CreateUser, Repository};
    use shared::primitives::{SqlMigrationLoader, WrappedChronoDateTime};
    use shared::test_utils::repository::setup_in_memory_database;

    async fn setup_test_cache() -> ApiKeyCache {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Arc::new(Repository::new(conn));
        ApiKeyCache::new(repo)
    }

    async fn create_test_api_key(cache: &ApiKeyCache, id: &str, hashed_value: &str, role: &str) {
        let now = WrappedChronoDateTime::now();
        let user_id = format!("user-{}", id);

        // Create user first
        cache
            .repository
            .create_user(&CreateUser {
                id: user_id.clone(),
                user_type: "service_principal".to_string(),
                email: None,
                role: role.to_string(),
                description: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .unwrap();

        // Create API key
        cache
            .repository
            .create_api_key(&CreateApiKey {
                id: id.to_string(),
                hashed_value: hashed_value.to_string(),
                description: None,
                user_id,
                created_at: now,
                updated_at: now,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_cache_new_empty() {
        let cache = setup_test_cache().await;
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_cache_get_not_found() {
        let cache = setup_test_cache().await;
        let result = cache.get_by_hashed_value("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_get_from_repository() {
        let cache = setup_test_cache().await;

        // Create an API key in the repository (not in cache)
        create_test_api_key(&cache, "api-key-1", "hashed-value-1", "agent").await;

        // First lookup should hit repository and cache
        let result = cache.get_by_hashed_value("hashed-value-1").await.unwrap();
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.id, "api-key-1");
        assert_eq!(cached.hashed_value, "hashed-value-1");
        assert_eq!(cached.role, Role::Agent);

        // Should now be in cache
        assert_eq!(cache.len(), 1);

        // Second lookup should hit cache directly
        let result2 = cache.get_by_hashed_value("hashed-value-1").await.unwrap();
        assert!(result2.is_some());
    }

    #[tokio::test]
    async fn test_cache_add() {
        let cache = setup_test_cache().await;

        let cached_key = CachedApiKey {
            id: "api-key-1".to_string(),
            hashed_value: "hashed-value-1".to_string(),
            role: Role::Agent,
            user_id: "user-1".to_string(),
        };

        cache.add(cached_key);

        assert_eq!(cache.len(), 1);

        // Can retrieve it (but won't hit repo since it's in cache)
        let result = cache.get_by_hashed_value("hashed-value-1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "api-key-1");
    }

    #[tokio::test]
    async fn test_cache_remove_by_id() {
        let cache = setup_test_cache().await;

        cache.add(CachedApiKey {
            id: "api-key-1".to_string(),
            hashed_value: "hashed-value-1".to_string(),
            role: Role::Agent,
            user_id: "user-1".to_string(),
        });
        cache.add(CachedApiKey {
            id: "api-key-2".to_string(),
            hashed_value: "hashed-value-2".to_string(),
            role: Role::User,
            user_id: "user-2".to_string(),
        });

        assert_eq!(cache.len(), 2);

        cache.remove_by_id("api-key-1");

        assert_eq!(cache.len(), 1);

        // api-key-1 should be gone
        let result = cache.cache.get("hashed-value-1");
        assert!(result.is_none());

        // api-key-2 should still be there
        let result = cache.cache.get("hashed-value-2");
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_cache_remove_by_hashed_value() {
        let cache = setup_test_cache().await;

        cache.add(CachedApiKey {
            id: "api-key-1".to_string(),
            hashed_value: "hashed-value-1".to_string(),
            role: Role::Agent,
            user_id: "user-1".to_string(),
        });

        assert_eq!(cache.len(), 1);

        cache.remove_by_hashed_value("hashed-value-1");

        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = setup_test_cache().await;

        cache.add(CachedApiKey {
            id: "api-key-1".to_string(),
            hashed_value: "hashed-value-1".to_string(),
            role: Role::Agent,
            user_id: "user-1".to_string(),
        });
        cache.add(CachedApiKey {
            id: "api-key-2".to_string(),
            hashed_value: "hashed-value-2".to_string(),
            role: Role::User,
            user_id: "user-2".to_string(),
        });

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_cache_refresh() {
        let cache = setup_test_cache().await;

        // Create an API key in the repository
        create_test_api_key(&cache, "api-key-1", "hashed-value-1", "agent").await;

        // Add stale data to cache
        cache.add(CachedApiKey {
            id: "api-key-1".to_string(),
            hashed_value: "hashed-value-1".to_string(),
            role: Role::User, // Wrong role!
            user_id: "wrong-user".to_string(),
        });

        // Refresh should get correct data from repo
        let result = cache.refresh("hashed-value-1").await.unwrap();
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.role, Role::Agent); // Correct role from repo
    }
}
