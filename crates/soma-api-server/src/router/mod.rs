use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use url::Url;
use utoipa::openapi::OpenApi;

use crate::router::a2a::Agent2AgentServiceParams;
use crate::router::task::TaskService;
use crate::router::{a2a::Agent2AgentService};
use shared::restate::admin_client::AdminClient;
use shared::restate::invoke::RestateIngressClient;
use crate::{logic::ConnectionManager, repository::Repository};
use bridge::{
    logic::{EnvelopeEncryptionKeyContents, OnConfigChangeTx},
    router::bridge::{BridgeService, create_router as create_bridge_router},
};
use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;

pub(crate) mod a2a;
pub(crate) mod task;

#[derive(Clone)]
pub struct ApiService {
    pub agent_service: Arc<Agent2AgentService>,
    pub task_service: Arc<TaskService>,
    pub bridge_service: BridgeService,
}

pub struct InitRouterParams {
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
    pub envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    pub bridge_repository: bridge::repository::Repository,
    pub mcp_sse_ping_interval: Duration,
}

impl ApiService {
    pub async fn new(init_params: InitRouterParams) -> Result<Self, CommonError> {
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
            init_params.envelope_encryption_key_contents.clone(),
            init_params.mcp_transport_tx,
            init_params.mcp_sse_ping_interval,
        )
        .await?;

        Ok(Self {
            agent_service,
            task_service,
            // frontend_service,
            // mcp_service,
            bridge_service,
        })
    }
}

pub fn initiaite_api_router(api_service: ApiService) -> Result<Router, CommonError> {
    let mut router = Router::new();

    // let (live_connection_changes_tx, mut live_connection_changes_rx) = tokio::sync::mpsc::channel(10);

    // agent router

    let (agent_router, _) = a2a::create_router().split_for_parts();

    let agent_router = agent_router.with_state(api_service.agent_service);
    router = router.merge(agent_router);

    // task router
    let (task_router, _) = task::create_router().split_for_parts();
    let task_router = task_router.with_state(api_service.task_service);
    router = router.merge(task_router);


    // bridge router
    let (bridge_router, _) = create_bridge_router().split_for_parts();
    let bridge_router = bridge_router.with_state(api_service.bridge_service);
    router = router.merge(bridge_router);

    Ok(router)
}

pub fn generate_openapi_spec() -> OpenApi {
    // let (_, mut spec) = frontend::create_router().split_for_parts();
    let (_, mut spec) = a2a::create_router().split_for_parts();
    let (_, task_spec) = task::create_router().split_for_parts();
    let (_, bridge_spec) = create_bridge_router().split_for_parts();
    spec.merge(task_spec);
    spec.merge(bridge_spec);

    spec
}
