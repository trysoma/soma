use std::{path::PathBuf, sync::Arc};

use encryption::logic::crypto_services::CryptoCache;
use encryption::repository::Repository as EncryptionRepository;

use crate::logic::api_key::cache::ApiKeyCache;
use crate::logic::auth_client::AuthClient;
use crate::logic::jwk::cache::JwksCache;
use crate::logic::sts::cache::StsConfigCache;
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::{OnConfigChangeEvt, OnConfigChangeTx};
use crate::repository::Repository;

/// Default broadcast channel capacity
const BROADCAST_CHANNEL_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct IdentityService {
    pub base_redirect_uri: String,
    pub repository: Arc<Repository>,
    pub crypto_cache: CryptoCache,
    pub internal_jwks_cache: JwksCache,
    pub api_key_cache: ApiKeyCache,
    pub sts_config_cache: StsConfigCache,
    pub external_jwks_cache: ExternalJwksCache,
    pub on_config_change_tx: OnConfigChangeTx,
}

impl IdentityService {
    pub fn new(
        base_redirect_uri: String,
        repository: Repository,
        encryption_repository: EncryptionRepository,
        local_envelope_encryption_key_path: PathBuf,
        internal_jwks_cache: JwksCache,
    ) -> Self {
        let repository = Arc::new(repository);
        let crypto_cache =
            CryptoCache::new(encryption_repository, local_envelope_encryption_key_path);
        let api_key_cache = ApiKeyCache::new(repository.clone());
        let sts_config_cache = StsConfigCache::new(repository.clone());
        let external_jwks_cache = ExternalJwksCache::new();
        let (on_config_change_tx, _) =
            tokio::sync::broadcast::channel::<OnConfigChangeEvt>(BROADCAST_CHANNEL_CAPACITY);

        Self {
            base_redirect_uri,
            repository,
            crypto_cache,
            internal_jwks_cache,
            api_key_cache,
            sts_config_cache,
            external_jwks_cache,
            on_config_change_tx,
        }
    }

    /// Create an AuthClient from this service's caches
    pub fn auth_client(&self) -> AuthClient {
        AuthClient::new(self.internal_jwks_cache.clone(), self.api_key_cache.clone())
    }
}
