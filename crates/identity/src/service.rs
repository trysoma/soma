use std::sync::Arc;

use encryption::logic::crypto_services::CryptoCache;

use crate::logic::api_key::cache::ApiKeyCache;
use crate::logic::auth_client::AuthClient;
use crate::logic::jwk::cache::JwksCache;
use crate::logic::sts::cache::StsConfigCache;
use crate::logic::sts::external_jwk_cache::ExternalJwksCache;
use crate::logic::{OnConfigChangeEvt, OnConfigChangeTx};
use crate::repository::Repository;

/// Default broadcast channel capacity
const BROADCAST_CHANNEL_CAPACITY: usize = 100;

/// Parameters for constructing an IdentityService
pub struct IdentityServiceParams {
    pub base_redirect_uri: String,
    pub repository: Arc<Repository>,
    pub crypto_cache: CryptoCache,
    pub internal_jwks_cache: JwksCache,
    pub api_key_cache: ApiKeyCache,
    pub sts_config_cache: StsConfigCache,
    pub external_jwks_cache: ExternalJwksCache,
    pub auth_client: Arc<AuthClient>,
}

#[derive(Clone)]
pub struct IdentityService {
    pub base_redirect_uri: String,
    pub repository: Arc<Repository>,
    pub crypto_cache: CryptoCache,
    pub internal_jwks_cache: JwksCache,
    pub api_key_cache: ApiKeyCache,
    pub sts_config_cache: StsConfigCache,
    pub external_jwks_cache: ExternalJwksCache,
    pub auth_client: Arc<AuthClient>,
    pub on_config_change_tx: OnConfigChangeTx,
}

impl IdentityService {
    /// Create a new IdentityService with pre-constructed caches
    pub fn new(params: IdentityServiceParams) -> Self {
        let (on_config_change_tx, _) =
            tokio::sync::broadcast::channel::<OnConfigChangeEvt>(BROADCAST_CHANNEL_CAPACITY);

        Self {
            base_redirect_uri: params.base_redirect_uri,
            repository: params.repository,
            crypto_cache: params.crypto_cache,
            internal_jwks_cache: params.internal_jwks_cache,
            api_key_cache: params.api_key_cache,
            sts_config_cache: params.sts_config_cache,
            external_jwks_cache: params.external_jwks_cache,
            auth_client: params.auth_client,
            on_config_change_tx,
        }
    }

    /// Get a reference to the auth client
    pub fn auth_client(&self) -> &Arc<AuthClient> {
        &self.auth_client
    }
}
