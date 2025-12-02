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
