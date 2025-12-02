use std::{collections::HashMap, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use encryption::logic::crypto_services::CryptoCache;
use encryption::repository::Repository as EncryptionRepository;

use crate::logic::auth_config::AuthMiddlewareConfig;
use crate::logic::jwks_cache::JwksCache;
use crate::repository::Repository;

#[derive(Clone)]
pub struct IdentityService {
    pub repository: Arc<Repository>,
    pub crypto_cache: CryptoCache,
    pub jwks_cache: JwksCache,
    pub auth_middleware_config: Arc<ArcSwap<AuthMiddlewareConfig>>,
}

impl IdentityService {
    pub fn new(
        repository: Repository,
        encryption_repository: EncryptionRepository,
        local_envelope_encryption_key_path: PathBuf,
    ) -> Self {
        let crypto_cache =
            CryptoCache::new(encryption_repository, local_envelope_encryption_key_path);
        let jwks_cache = JwksCache::new(repository.clone());
        let auth_middleware_config = Arc::new(ArcSwap::from_pointee(AuthMiddlewareConfig {
            api_keys: HashMap::new(),
            sts_token_config: HashMap::new(),
        }));

        Self {
            repository: Arc::new(repository),
            crypto_cache,
            jwks_cache,
            auth_middleware_config,
        }
    }
}
