use std::{sync::Arc, time::Duration};

use ::bridge::router::bridge::BridgeService;
use bridge::logic::OnConfigChangeTx;
use encryption::{CryptoCache, EncryptionKeyEventSender, EncryptionService};
use shared::{
    error::CommonError,
    restate::{admin_client::AdminClient, invoke::RestateIngressClient},
    soma_agent_definition::SomaAgentDefinitionLike,
};
use url::Url;

use crate::{
    logic::task::ConnectionManager,
    repository::Repository,
    router::{
        a2a::{Agent2AgentService, Agent2AgentServiceParams},
        internal,
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

#[derive(Clone)]
pub struct ApiService {
    pub agent_service: Arc<Agent2AgentService>,
    pub task_service: Arc<TaskService>,
    pub bridge_service: BridgeService,
    pub internal_service: Arc<internal::InternalService>,
    pub encryption_service: encryption::router::EncryptionService,
}

pub struct InitApiServiceParams {
    pub host: String,
    pub port: u16,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub mcp_transport_tx:
        tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport>,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
    pub on_bridge_config_change_tx: OnConfigChangeTx,
    pub on_encryption_change_tx: EncryptionKeyEventSender,
    pub encryption_repository: encryption::repository::Repository,
    pub crypto_cache: CryptoCache,
    pub bridge_repository: ::bridge::repository::Repository,
    pub mcp_sse_ping_interval: Duration,
    pub sdk_client: Arc<
        tokio::sync::Mutex<
            Option<
                sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<tonic::transport::Channel>,
            >,
        >,
    >,
}

impl ApiService {
    pub async fn new(init_params: InitApiServiceParams) -> Result<Self, CommonError> {
        let encryption_service = encryption::router::EncryptionService::new(
            init_params.encryption_repository.clone(),
            init_params.on_encryption_change_tx.clone(),
            init_params.crypto_cache.clone(),
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
        ));

        Ok(Self {
            agent_service,
            task_service,
            bridge_service,
            internal_service,
            encryption_service,
        })
    }
}
