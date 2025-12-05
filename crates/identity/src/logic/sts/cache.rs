use std::sync::Arc;

use dashmap::DashMap;
use shared::error::CommonError;
use shared::primitives::PaginationRequest;

use crate::logic::sts::config::{StsConfigId, StsTokenConfig};
use crate::repository::{Repository, UserRepositoryLike};

/// Cache for STS configurations with repository fallback
///
/// This cache stores STS configurations keyed by their ID for fast O(1) lookups
/// during token exchange. If a config is not found in the cache, it falls back
/// to the repository.
#[derive(Clone)]
pub struct StsConfigCache {
    /// Cache of STS configs keyed by ID
    cache: Arc<DashMap<StsConfigId, StsTokenConfig>>,
    /// Repository for fallback lookups
    repository: Arc<Repository>,
}

impl StsConfigCache {
    /// Create a new STS config cache with the given repository
    pub fn new(repository: Arc<Repository>) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            repository,
        }
    }

    /// Look up an STS config by its ID
    ///
    /// First checks the cache, then falls back to the repository if not found.
    /// If found in the repository, the config is added to the cache.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<StsTokenConfig>, CommonError> {
        // First check the cache
        if let Some(cached) = self.cache.get(id) {
            return Ok(Some(cached.value().clone()));
        }

        // Fall back to repository
        let config_db = self.repository.get_sts_configuration_by_id(id).await?;

        if let Some(config_db) = config_db {
            // Add to cache for future lookups
            self.cache.insert(id.to_string(), config_db.config.clone());
            Ok(Some(config_db.config))
        } else {
            Ok(None)
        }
    }

    /// Add an STS config to the cache
    pub fn add(&self, config: StsTokenConfig) {
        let id = match &config {
            StsTokenConfig::JwtTemplate(c) => c.id.clone(),
            StsTokenConfig::DevMode(c) => c.id.clone(),
        };
        self.cache.insert(id, config);
    }

    /// Remove an STS config from the cache by its ID
    pub fn remove_by_id(&self, id: &str) {
        self.cache.remove(id);
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

    /// Refresh a specific STS config from the repository
    ///
    /// This is useful after a config is updated.
    pub async fn refresh(&self, id: &str) -> Result<Option<StsTokenConfig>, CommonError> {
        // Remove from cache first
        self.cache.remove(id);

        // Fetch fresh from repository
        let config_db = self.repository.get_sts_configuration_by_id(id).await?;

        if let Some(config_db) = config_db {
            self.cache.insert(id.to_string(), config_db.config.clone());
            Ok(Some(config_db.config))
        } else {
            Ok(None)
        }
    }

    /// Refresh the entire cache from the repository
    ///
    /// This clears the cache and reloads all STS configurations.
    pub async fn refresh_all(&self) -> Result<(), CommonError> {
        self.cache.clear();

        // Load all configs in pages
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };

            let result = self
                .repository
                .list_sts_configurations(&pagination, None)
                .await?;

            for config_db in result.items {
                let id = match &config_db.config {
                    StsTokenConfig::JwtTemplate(c) => c.id.clone(),
                    StsTokenConfig::DevMode(c) => c.id.clone(),
                };
                self.cache.insert(id, config_db.config);
            }

            match result.next_page_token {
                Some(token) => next_page_token = Some(token),
                None => break,
            }
        }

        Ok(())
    }

    /// Get all cached configs
    pub fn get_all_cached(&self) -> Vec<StsTokenConfig> {
        self.cache
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// Initialize the STS config cache with all configurations from the repository
pub async fn init_sts_config_cache(cache: &StsConfigCache) -> Result<(), CommonError> {
    cache.refresh_all().await
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::sts::config::DevModeConfig;
    use crate::repository::Repository;
    use shared::primitives::SqlMigrationLoader;
    use shared::test_utils::repository::setup_in_memory_database;

    async fn setup_test_cache() -> StsConfigCache {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        StsConfigCache::new(Arc::new(repo))
    }

    fn create_test_config(id: &str) -> StsTokenConfig {
        StsTokenConfig::DevMode(DevModeConfig { id: id.to_string() })
    }

    #[tokio::test]
    async fn test_sts_config_cache_new() {
        let cache = setup_test_cache().await;
        // Cache should be empty initially
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_sts_config_cache_add() {
        let cache = setup_test_cache().await;

        let config = create_test_config("test-config-1");
        cache.add(config);

        assert_eq!(cache.len(), 1);
    }

    #[tokio::test]
    async fn test_sts_config_cache_add_multiple() {
        let cache = setup_test_cache().await;

        cache.add(create_test_config("config-1"));
        cache.add(create_test_config("config-2"));
        cache.add(create_test_config("config-3"));

        assert_eq!(cache.len(), 3);
    }

    #[tokio::test]
    async fn test_sts_config_cache_remove_by_id() {
        let cache = setup_test_cache().await;

        cache.add(create_test_config("config-1"));
        cache.add(create_test_config("config-2"));
        assert_eq!(cache.len(), 2);

        cache.remove_by_id("config-1");

        assert_eq!(cache.len(), 1);
    }

    #[tokio::test]
    async fn test_sts_config_cache_remove_nonexistent() {
        let cache = setup_test_cache().await;

        cache.add(create_test_config("config-1"));
        assert_eq!(cache.len(), 1);

        // Removing a non-existent key should not fail
        cache.remove_by_id("nonexistent-config");

        assert_eq!(cache.len(), 1);
    }

    #[tokio::test]
    async fn test_sts_config_cache_clear() {
        let cache = setup_test_cache().await;

        cache.add(create_test_config("config-1"));
        cache.add(create_test_config("config-2"));
        cache.add(create_test_config("config-3"));
        assert_eq!(cache.len(), 3);

        cache.clear();

        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_sts_config_cache_get_all_cached() {
        let cache = setup_test_cache().await;

        cache.add(create_test_config("config-1"));
        cache.add(create_test_config("config-2"));

        let configs = cache.get_all_cached();
        assert_eq!(configs.len(), 2);
    }

    #[tokio::test]
    async fn test_sts_config_cache_get_by_id_not_in_cache() {
        let cache = setup_test_cache().await;

        // Should return None when not in cache or repo
        let result = cache.get_by_id("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_sts_config_cache_refresh_all_empty_repo() {
        let cache = setup_test_cache().await;

        // Add a config to cache
        cache.add(create_test_config("stale-config"));
        assert_eq!(cache.len(), 1);

        // Refresh from empty repo should clear cache
        cache.refresh_all().await.unwrap();

        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_init_sts_config_cache() {
        let cache = setup_test_cache().await;

        // Add a config to cache (simulating stale data)
        cache.add(create_test_config("stale-config"));
        assert_eq!(cache.len(), 1);

        // Initialize should refresh from repo (which is empty)
        init_sts_config_cache(&cache).await.unwrap();

        assert!(cache.is_empty());
    }
}
