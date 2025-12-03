use std::{collections::HashMap, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use encryption::logic::crypto_services::CryptoCache;
use encryption::repository::Repository as EncryptionRepository;

use crate::logic::api_key_cache::ApiKeyCache;
use crate::logic::auth_config::{AuthConfig, ExternalJwksCache};
use crate::logic::jwks_cache::JwksCache;
use crate::logic::{OnConfigChangeTx, OnConfigChangeEvt};
use crate::repository::Repository;

/// Default broadcast channel capacity
const BROADCAST_CHANNEL_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct IdentityService {
    pub repository: Arc<Repository>,
    pub crypto_cache: CryptoCache,
    pub jwks_cache: JwksCache,
    pub api_key_cache: ApiKeyCache,
    pub external_jwks_cache: ExternalJwksCache,
    pub auth_middleware_config: Arc<ArcSwap<AuthConfig>>,
    pub on_config_change_tx: OnConfigChangeTx,
}

impl IdentityService {
    pub fn new(
        repository: Repository,
        encryption_repository: EncryptionRepository,
        local_envelope_encryption_key_path: PathBuf,
    ) -> Self {
        let repository = Arc::new(repository);
        let crypto_cache =
            CryptoCache::new(encryption_repository, local_envelope_encryption_key_path);
        let jwks_cache = JwksCache::new(Repository::new(repository.connection().clone()));
        let api_key_cache = ApiKeyCache::new(repository.clone());
        let external_jwks_cache = ExternalJwksCache::new();
        let auth_middleware_config = Arc::new(ArcSwap::from_pointee(AuthConfig {
            api_keys: HashMap::new(),
            sts_token_config: HashMap::new(),
        }));
        let (on_config_change_tx, _) =
            tokio::sync::broadcast::channel::<OnConfigChangeEvt>(BROADCAST_CHANNEL_CAPACITY);

        Self {
            repository,
            crypto_cache,
            jwks_cache,
            api_key_cache,
            external_jwks_cache,
            auth_middleware_config,
            on_config_change_tx,
        }
    }

    /// Create a new service with an externally provided broadcaster
    pub fn with_broadcaster(
        repository: Repository,
        encryption_repository: EncryptionRepository,
        local_envelope_encryption_key_path: PathBuf,
        on_config_change_tx: OnConfigChangeTx,
    ) -> Self {
        let repository = Arc::new(repository);
        let crypto_cache =
            CryptoCache::new(encryption_repository, local_envelope_encryption_key_path);
        let jwks_cache = JwksCache::new(Repository::new(repository.connection().clone()));
        let api_key_cache = ApiKeyCache::new(repository.clone());
        let external_jwks_cache = ExternalJwksCache::new();
        let auth_middleware_config = Arc::new(ArcSwap::from_pointee(AuthConfig {
            api_keys: HashMap::new(),
            sts_token_config: HashMap::new(),
        }));

        Self {
            repository,
            crypto_cache,
            jwks_cache,
            api_key_cache,
            external_jwks_cache,
            auth_middleware_config,
            on_config_change_tx,
        }
    }

    /// Get a reference to the config change broadcaster
    pub fn on_config_change_tx(&self) -> &OnConfigChangeTx {
        &self.on_config_change_tx
    }

    /// Subscribe to config change events
    pub fn subscribe_to_config_changes(&self) -> crate::logic::OnConfigChangeRx {
        self.on_config_change_tx.subscribe()
    }
}
