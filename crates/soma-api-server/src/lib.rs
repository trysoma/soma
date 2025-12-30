use std::{path::PathBuf, sync::Arc};

use ::mcp::logic::mcp::McpServerService;
use ::mcp::router::McpService;
use encryption::logic::{EncryptionKeyEventSender, crypto_services::CryptoCache};
use identity::logic::api_key::cache::ApiKeyCache;
use identity::logic::auth_client::AuthClient;
use identity::logic::sts::cache::StsConfigCache;
use identity::logic::sts::external_jwk_cache::ExternalJwksCache;
use mcp::logic::OnConfigChangeTx;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use shared::{
    error::CommonError,
    restate::{admin_client::AdminClient, invoke::RestateIngressClient},
    soma_agent_definition::SomaAgentDefinitionLike,
};
use url::Url;

use crate::{
    logic::on_change_pubsub::{EnvironmentVariableChangeTx, SecretChangeTx},
    logic::task::ConnectionManager,
    repository::Repository,
    router::{
        agent::{AgentService, AgentServiceParams},
        environment_variable::EnvironmentVariableService,
        internal,
        secret::SecretService,
        task::TaskService,
    },
    sdk::sdk_agent_sync::AgentCache,
};
pub mod factory;
pub mod logic;
pub mod repository;
pub mod restate;
pub mod router;
pub mod sdk;

#[cfg(test)]
pub mod test;

#[derive(Clone)]
pub struct ApiService {
    pub agent_service: Arc<AgentService>,
    pub task_service: Arc<TaskService>,
    pub mcp_service: McpService,
    pub internal_service: Arc<internal::InternalService>,
    pub encryption_service: encryption::router::EncryptionService,
    pub secret_service: Arc<SecretService>,
    pub environment_variable_service: Arc<EnvironmentVariableService>,
    pub identity_service: identity::service::IdentityService,
    pub sdk_client: Arc<
        tokio::sync::Mutex<
            Option<
                sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<tonic::transport::Channel>,
            >,
        >,
    >,
    /// Cache for storing agent metadata from SDK
    pub agent_cache: AgentCache,
}

pub struct InitApiServiceParams {
    pub base_url: String,
    pub host: String,
    pub port: u16,
    pub soma_restate_service_port: u16,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub mcp_service: StreamableHttpService<McpServerService, LocalSessionManager>,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
    pub restate_params: crate::restate::RestateServerParams,
    pub on_mcp_config_change_tx: OnConfigChangeTx,
    pub on_encryption_change_tx: EncryptionKeyEventSender,
    pub on_secret_change_tx: SecretChangeTx,
    pub on_environment_variable_change_tx: EnvironmentVariableChangeTx,
    pub encryption_repository: encryption::repository::Repository,
    pub crypto_cache: CryptoCache,
    pub mcp_repository: ::mcp::repository::Repository,
    pub identity_repository: identity::repository::Repository,
    pub internal_jwks_cache: identity::logic::jwk::cache::JwksCache,
    pub sdk_client: Arc<
        tokio::sync::Mutex<
            Option<
                sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<tonic::transport::Channel>,
            >,
        >,
    >,
    pub local_envelope_encryption_key_path: PathBuf,
    pub agent_cache: AgentCache,
}

impl ApiService {
    pub async fn new(init_params: InitApiServiceParams) -> Result<Self, CommonError> {
        let agent_cache = init_params.agent_cache.clone();

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
        let agent_service = Arc::new(AgentService::new(AgentServiceParams {
            soma_definition: init_params.soma_definition.clone(),
            host: Url::parse(format!("http://{}:{}", init_params.host, init_params.port).as_str())?,
            connection_manager: init_params.connection_manager.clone(),
            repository: init_params.repository.clone(),
            restate_ingress_client: init_params.restate_ingress_client.clone(),
            restate_admin_client: init_params.restate_admin_client.clone(),
            agent_cache: agent_cache.clone(),
        }));
        let task_service = Arc::new(TaskService::new(
            init_params.connection_manager.clone(),
            init_params.repository.clone(),
        ));
        let mcp_service = McpService::new(
            init_params.mcp_repository.clone(),
            init_params.on_mcp_config_change_tx.clone(),
            init_params.crypto_cache.clone(),
            init_params.mcp_service,
            auth_client.clone(),
        )
        .await?;

        let internal_service = Arc::new(internal::InternalService::new(
            mcp_service.clone(),
            init_params.sdk_client.clone(),
            std::sync::Arc::new(init_params.repository.clone()),
            init_params.crypto_cache.clone(),
            init_params.restate_params.clone(),
            agent_cache.clone(),
        ));

        let secret_service = Arc::new(SecretService::new(
            init_params.repository.clone(),
            encryption_service.clone(),
            init_params.on_secret_change_tx.clone(),
            init_params.sdk_client.clone(),
            init_params.crypto_cache.clone(),
        ));

        let environment_variable_service = Arc::new(EnvironmentVariableService::new(
            init_params.repository.clone(),
            init_params.on_environment_variable_change_tx.clone(),
            init_params.sdk_client.clone(),
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
            agent_service,
            task_service,
            mcp_service,
            internal_service,
            encryption_service,
            secret_service,
            environment_variable_service,
            identity_service,
            sdk_client: init_params.sdk_client,
            agent_cache,
        })
    }
}
