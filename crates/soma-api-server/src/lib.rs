use std::{path::PathBuf, sync::Arc};

use ::tool::logic::mcp::McpServerService;
use ::tool::router::McpService;
use encryption::logic::{EncryptionKeyEventSender, crypto_services::CryptoCache};
use identity::logic::api_key::cache::ApiKeyCache;
use identity::logic::auth_client::AuthClient;
use identity::logic::sts::cache::StsConfigCache;
use identity::logic::sts::external_jwk_cache::ExternalJwksCache;
use tool::logic::OnConfigChangeTx;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use shared::error::CommonError;

use crate::{
    logic::on_change_pubsub::{SecretChangeTx, VariableChangeTx},
    router::internal,
};
pub mod factory;
pub mod logic;
pub mod repository;
pub mod router;

#[cfg(test)]
pub mod test;

#[derive(Clone)]
pub struct ApiService {
    pub mcp_service: McpService,
    pub internal_service: Arc<internal::InternalService>,
    pub encryption_service: encryption::router::EncryptionService,
    pub environment_service: Arc<environment::service::EnvironmentService>,
    pub identity_service: identity::service::IdentityService,
}

pub struct InitApiServiceParams {
    pub base_url: String,
    pub environment_repository: environment::repository::Repository,
    pub mcp_service: StreamableHttpService<McpServerService, LocalSessionManager>,
    pub on_mcp_config_change_tx: OnConfigChangeTx,
    pub on_encryption_change_tx: EncryptionKeyEventSender,
    pub on_secret_change_tx: SecretChangeTx,
    pub on_variable_change_tx: VariableChangeTx,
    pub encryption_repository: encryption::repository::Repository,
    pub crypto_cache: CryptoCache,
    pub tool_repository: ::tool::repository::Repository,
    pub identity_repository: identity::repository::Repository,
    pub internal_jwks_cache: identity::logic::jwk::cache::JwksCache,
    pub local_envelope_encryption_key_path: PathBuf,
}

impl ApiService {
    pub async fn new(init_params: InitApiServiceParams) -> Result<Self, CommonError> {
        // Create identity caches that will be shared
        let identity_repository = Arc::new(init_params.identity_repository);
        let api_key_cache = ApiKeyCache::new(identity_repository.clone());
        let sts_config_cache = StsConfigCache::new(identity_repository.clone());
        let external_jwks_cache = ExternalJwksCache::new();

        // Create the AuthClient - this will be shared across services for authentication
        let auth_client = Arc::new(AuthClient::new(
            init_params.internal_jwks_cache.clone(),
            api_key_cache.clone(),
        ));

        let encryption_service = encryption::router::EncryptionService::new(
            init_params.encryption_repository.clone(),
            init_params.on_encryption_change_tx.clone(),
            init_params.crypto_cache.clone(),
            init_params.local_envelope_encryption_key_path.clone(),
        );

        let mcp_service = McpService::new(
            init_params.tool_repository.clone(),
            init_params.on_mcp_config_change_tx.clone(),
            init_params.crypto_cache.clone(),
            init_params.mcp_service,
            auth_client.clone(),
        )
        .await?;

        let internal_service = Arc::new(internal::InternalService::new(
            mcp_service.clone(),
            std::sync::Arc::new(init_params.environment_repository.clone()),
            init_params.crypto_cache.clone(),
        ));

        // Create environment service
        let environment_service = Arc::new(environment::service::EnvironmentService::new(
            environment::service::EnvironmentServiceParams {
                repository: init_params.environment_repository.clone(),
                crypto_cache: init_params.crypto_cache.clone(),
                secret_change_tx: init_params.on_secret_change_tx.clone(),
                variable_change_tx: init_params.on_variable_change_tx.clone(),
            },
        ));

        // Construct identity service with pre-built caches
        let identity_service =
            identity::service::IdentityService::new(identity::service::IdentityServiceParams {
                base_redirect_uri: init_params.base_url.clone(),
                repository: identity_repository,
                crypto_cache: init_params.crypto_cache.clone(),
                internal_jwks_cache: init_params.internal_jwks_cache.clone(),
                api_key_cache,
                sts_config_cache,
                external_jwks_cache,
                auth_client,
            });

        Ok(Self {
            mcp_service,
            internal_service,
            encryption_service,
            environment_service,
            identity_service,
        })
    }
}
