use std::{path::PathBuf, sync::Arc, time::Duration};

use ::bridge::router::BridgeService;
use bridge::logic::OnConfigChangeTx;
use encryption::logic::{EncryptionKeyEventSender, crypto_services::CryptoCache};
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
        a2a::{Agent2AgentService, Agent2AgentServiceParams},
        environment_variable::EnvironmentVariableService,
        internal,
        secret::SecretService,
        task::TaskService,
    },
};
pub mod factory;
pub mod logic;
pub mod repository;
pub mod restate;
pub mod router;
pub mod sdk;
pub mod subsystems;

#[cfg(all(test, feature = "unit_test"))]
pub mod test;

#[derive(Clone)]
pub struct ApiService {
    pub agent_service: Arc<Agent2AgentService>,
    pub task_service: Arc<TaskService>,
    pub bridge_service: BridgeService,
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
}

pub struct InitApiServiceParams {
    pub base_url: String,
    pub host: String,
    pub port: u16,
    pub soma_restate_service_port: u16,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub mcp_transport_tx:
        tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport>,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
    pub restate_params: crate::restate::RestateServerParams,
    pub on_bridge_config_change_tx: OnConfigChangeTx,
    pub on_encryption_change_tx: EncryptionKeyEventSender,
    pub on_secret_change_tx: SecretChangeTx,
    pub on_environment_variable_change_tx: EnvironmentVariableChangeTx,
    pub encryption_repository: encryption::repository::Repository,
    pub crypto_cache: CryptoCache,
    pub bridge_repository: ::bridge::repository::Repository,
    pub identity_repository: identity::repository::Repository,
    pub internal_jwks_cache: identity::logic::jwk::cache::JwksCache,
    pub mcp_sse_ping_interval: Duration,
    pub sdk_client: Arc<
        tokio::sync::Mutex<
            Option<
                sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<tonic::transport::Channel>,
            >,
        >,
    >,
    pub local_envelope_encryption_key_path: PathBuf,
}

impl ApiService {
    pub async fn new(init_params: InitApiServiceParams) -> Result<Self, CommonError> {
        let encryption_service = encryption::router::EncryptionService::new(
            init_params.encryption_repository.clone(),
            init_params.on_encryption_change_tx.clone(),
            init_params.crypto_cache.clone(),
            init_params.local_envelope_encryption_key_path.clone(),
        );
        let agent_service = Arc::new(Agent2AgentService::new(Agent2AgentServiceParams {
            soma_definition: init_params.soma_definition.clone(),
            host: Url::parse(format!("http://{}:{}", init_params.host, init_params.port).as_str())?,
            connection_manager: init_params.connection_manager.clone(),
            repository: init_params.repository.clone(),
            restate_ingress_client: init_params.restate_ingress_client.clone(),
            restate_admin_client: init_params.restate_admin_client.clone(),
        }));
        let task_service = Arc::new(TaskService::new(
            init_params.connection_manager.clone(),
            init_params.repository.clone(),
        ));
        let bridge_service = BridgeService::new(
            init_params.bridge_repository.clone(),
            init_params.on_bridge_config_change_tx.clone(),
            init_params.crypto_cache.clone(),
            init_params.mcp_transport_tx,
            init_params.mcp_sse_ping_interval,
        )
        .await?;

        let internal_service = Arc::new(internal::InternalService::new(
            bridge_service.clone(),
            init_params.sdk_client.clone(),
            std::sync::Arc::new(init_params.repository.clone()),
            init_params.crypto_cache.clone(),
            init_params.restate_params.clone(),
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

        // Construct identity service
        let identity_service = identity::service::IdentityService::new(
            init_params.base_url.clone(),
            init_params.identity_repository,
            init_params.encryption_repository.clone(),
            init_params.local_envelope_encryption_key_path,
            init_params.internal_jwks_cache.clone(),
        );

        Ok(Self {
            agent_service,
            task_service,
            bridge_service,
            internal_service,
            encryption_service,
            secret_service,
            environment_variable_service,
            identity_service,
            sdk_client: init_params.sdk_client,
        })
    }
}
