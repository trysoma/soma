use std::sync::Arc;

use dashmap::DashMap;
use shared::error::CommonError;

use crate::logic::jwk::Jwk;
use crate::repository::Repository;

/// JWKS cache structure for managing cached JWKs
/// Similar to CryptoCache, this provides thread-safe caching of JWKs
#[derive(Clone)]
pub struct JwksCache {
    jwks: Arc<DashMap<String, Jwk>>,
    repository: Arc<Repository>,
}

impl JwksCache {
    /// Create a new empty JWKS cache with the given repository
    pub fn new(repo: Repository) -> Self {
        Self {
            jwks: Arc::new(DashMap::new()),
            repository: Arc::new(repo),
        }
    }

    /// Invalidate a specific JWK by kid
    /// This removes the JWK from the cache
    pub fn invalidate_jwk(&self, kid: &str) {
        self.jwks.remove(kid);
    }

    /// Clear the entire cache
    pub fn clear_cache(&self) {
        self.jwks.clear();
    }

    /// Get JWKS from repository (used internally, without cache)
    pub(crate) async fn get_jwks_from_repo(&self) -> Result<Vec<Jwk>, CommonError> {
        use crate::logic::jwk::get_jwks_direct;
        let jwks_response = get_jwks_direct(self.repository.as_ref()).await?;
        Ok(jwks_response.keys)
    }

    /// Refresh the cache from the repository
    pub async fn refresh_cache(&self) -> Result<(), CommonError> {
        let jwks = self.get_jwks_from_repo().await?;
        self.jwks.clear();
        for jwk in jwks {
            self.jwks.insert(jwk.kid.clone(), jwk);
        }
        Ok(())
    }

    /// Get cached JWKS
    pub fn get_cached_jwks(&self) -> Vec<Jwk> {
        self.jwks
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Add a JWK to the cache
    pub fn add_jwk(&self, jwk: Jwk) {
        self.jwks.insert(jwk.kid.clone(), jwk);
    }

    /// Get a JWK by kid
    pub fn get_jwk(&self, kid: &str) -> Option<Jwk> {
        self.jwks.get(kid).map(|entry| entry.value().clone())
    }

    /// Remove expired or invalidated JWKs from cache
    pub async fn remove_expired(&self) -> Result<(), CommonError> {
        let jwks = self.get_jwks_from_repo().await?;
        let valid_kids: std::collections::HashSet<String> =
            jwks.iter().map(|jwk| jwk.kid.clone()).collect();

        // Remove any cached JWKs that are no longer valid
        self.jwks.retain(|kid, _| valid_kids.contains(kid));

        Ok(())
    }
}

/// Initialize the JWKS cache with all valid JWKs
pub async fn init_jwks_cache(cache: &JwksCache) -> Result<(), CommonError> {
    cache.refresh_cache().await
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::jwk::Jwk;
    use crate::repository::Repository;
    use shared::primitives::SqlMigrationLoader;
    use shared::test_utils::repository::setup_in_memory_database;

    async fn setup_test_cache() -> JwksCache {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        JwksCache::new(repo)
    }

    fn create_test_jwk(kid: &str) -> Jwk {
        Jwk {
            kty: "RSA".to_string(),
            kid: kid.to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            n: "test-modulus".to_string(),
            e: "AQAB".to_string(),
        }
    }

    #[tokio::test]
    async fn test_jwks_cache_new() {
        let cache = setup_test_cache().await;
        // Cache should be empty initially
        let jwks = cache.get_cached_jwks();
        assert!(jwks.is_empty());
    }

    #[tokio::test]
    async fn test_jwks_cache_add_jwk() {
        let cache = setup_test_cache().await;

        let jwk = create_test_jwk("test-kid-1");
        cache.add_jwk(jwk);

        let jwks = cache.get_cached_jwks();
        assert_eq!(jwks.len(), 1);
        assert_eq!(jwks[0].kid, "test-kid-1");
    }

    #[tokio::test]
    async fn test_jwks_cache_add_multiple_jwks() {
        let cache = setup_test_cache().await;

        cache.add_jwk(create_test_jwk("kid-1"));
        cache.add_jwk(create_test_jwk("kid-2"));
        cache.add_jwk(create_test_jwk("kid-3"));

        let jwks = cache.get_cached_jwks();
        assert_eq!(jwks.len(), 3);
    }

    #[tokio::test]
    async fn test_jwks_cache_invalidate_jwk() {
        let cache = setup_test_cache().await;

        cache.add_jwk(create_test_jwk("kid-1"));
        cache.add_jwk(create_test_jwk("kid-2"));
        assert_eq!(cache.get_cached_jwks().len(), 2);

        cache.invalidate_jwk("kid-1");

        let jwks = cache.get_cached_jwks();
        assert_eq!(jwks.len(), 1);
        assert_eq!(jwks[0].kid, "kid-2");
    }

    #[tokio::test]
    async fn test_jwks_cache_invalidate_nonexistent_jwk() {
        let cache = setup_test_cache().await;

        cache.add_jwk(create_test_jwk("kid-1"));
        assert_eq!(cache.get_cached_jwks().len(), 1);

        // Invalidating a non-existent key should not fail
        cache.invalidate_jwk("nonexistent-kid");

        assert_eq!(cache.get_cached_jwks().len(), 1);
    }

    #[tokio::test]
    async fn test_jwks_cache_clear_cache() {
        let cache = setup_test_cache().await;

        cache.add_jwk(create_test_jwk("kid-1"));
        cache.add_jwk(create_test_jwk("kid-2"));
        cache.add_jwk(create_test_jwk("kid-3"));
        assert_eq!(cache.get_cached_jwks().len(), 3);

        cache.clear_cache();

        assert!(cache.get_cached_jwks().is_empty());
    }

    #[tokio::test]
    async fn test_jwks_cache_get_cached_jwks_returns_clones() {
        let cache = setup_test_cache().await;

        cache.add_jwk(create_test_jwk("kid-1"));

        let jwks1 = cache.get_cached_jwks();
        let jwks2 = cache.get_cached_jwks();

        // Both should have the same data
        assert_eq!(jwks1.len(), jwks2.len());
        assert_eq!(jwks1[0].kid, jwks2[0].kid);
    }

    #[tokio::test]
    async fn test_jwks_cache_add_replaces_existing() {
        let cache = setup_test_cache().await;

        let jwk1 = Jwk {
            kty: "RSA".to_string(),
            kid: "kid-1".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            n: "old-modulus".to_string(),
            e: "AQAB".to_string(),
        };
        cache.add_jwk(jwk1);

        let jwk2 = Jwk {
            kty: "RSA".to_string(),
            kid: "kid-1".to_string(), // Same kid
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            n: "new-modulus".to_string(), // Different modulus
            e: "AQAB".to_string(),
        };
        cache.add_jwk(jwk2);

        let jwks = cache.get_cached_jwks();
        assert_eq!(jwks.len(), 1);
        assert_eq!(jwks[0].n, "new-modulus");
    }

    #[tokio::test]
    async fn test_jwks_cache_refresh_cache_empty_repo() {
        let cache = setup_test_cache().await;

        // Add a JWK to cache
        cache.add_jwk(create_test_jwk("kid-1"));
        assert_eq!(cache.get_cached_jwks().len(), 1);

        // Refresh from empty repo should clear cache
        cache.refresh_cache().await.unwrap();

        assert!(cache.get_cached_jwks().is_empty());
    }

    #[tokio::test]
    async fn test_jwks_cache_get_jwks_from_repo_empty() {
        let cache = setup_test_cache().await;

        let jwks = cache.get_jwks_from_repo().await.unwrap();
        assert!(jwks.is_empty());
    }

    #[tokio::test]
    async fn test_jwks_cache_remove_expired_empty_cache() {
        let cache = setup_test_cache().await;

        // Should not fail with empty cache
        cache.remove_expired().await.unwrap();

        assert!(cache.get_cached_jwks().is_empty());
    }

    #[tokio::test]
    async fn test_jwks_cache_remove_expired_removes_invalid_keys() {
        let cache = setup_test_cache().await;

        // Add some JWKs to cache that don't exist in repo
        cache.add_jwk(create_test_jwk("orphan-kid-1"));
        cache.add_jwk(create_test_jwk("orphan-kid-2"));
        assert_eq!(cache.get_cached_jwks().len(), 2);

        // Since repo is empty, these should be removed
        cache.remove_expired().await.unwrap();

        assert!(cache.get_cached_jwks().is_empty());
    }

    #[tokio::test]
    async fn test_init_jwks_cache() {
        let cache = setup_test_cache().await;

        // Add a JWK to cache (simulating stale data)
        cache.add_jwk(create_test_jwk("stale-kid"));
        assert_eq!(cache.get_cached_jwks().len(), 1);

        // Initialize should refresh from repo (which is empty)
        init_jwks_cache(&cache).await.unwrap();

        assert!(cache.get_cached_jwks().is_empty());
    }
}
