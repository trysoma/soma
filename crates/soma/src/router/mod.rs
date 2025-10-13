use std::{env, sync::Arc};

use axum::Router;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use url::Url;
use utoipa::openapi::OpenApi;

use crate::router::mcp::McpService;
use crate::utils::restate::invoke::RestateIngressClient;
use crate::utils::soma_agent_config::SomaConfig;
use crate::{repository::Repository, logic::ConnectionManager};
use crate::router::task::TaskService;
use crate::utils::construct_src_dir_absolute;
use crate::{
    commands::StartParams,
    router::{agent::AgentService, frontend::FrontendService},
};
use shared::error::CommonError;

pub(crate) mod agent;
pub(crate) mod frontend;
pub(crate) mod task;
pub(crate) mod mcp;


#[derive(Clone)]
pub(crate) struct RouterParams {
    pub params: StartParams,
    pub agent_service: Arc<AgentService>,
    pub task_service: Arc<TaskService>,
    pub frontend_service: Arc<FrontendService>,
    pub mcp_service: McpService,
}

pub(crate) struct InitRouterParams {
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport>,
    pub soma_config: SomaConfig,
    pub runtime_port: u16,
    pub restate_ingress_client: RestateIngressClient,
}

impl RouterParams {
    pub fn new(params: StartParams, init_params: InitRouterParams) -> Result<Self, CommonError> {
        let src_dir = construct_src_dir_absolute(params.src_dir.clone())?;

        let agent_service = Arc::new(AgentService::new(
            src_dir,
            init_params.soma_config,
            Url::parse(format!("http://{}:{}", params.host, params.port).as_str())?,
            init_params.connection_manager.clone(),
            init_params.repository.clone(),
            init_params.runtime_port,
            init_params.restate_ingress_client.clone(),
        ));
        let task_service = Arc::new(TaskService::new(init_params.connection_manager.clone(), init_params.repository.clone()));
        let frontend_service = Arc::new(FrontendService::new());
        // internally it's an Arc<McpServiceInner>
        let mcp_service = McpService::new(init_params.mcp_transport_tx, init_params.repository.clone(), init_params.connection_manager.clone());
        Ok(Self {
            params,
            agent_service,
            task_service,
            frontend_service,
            mcp_service,
        })
    }
}

pub(crate) fn initiate_routers(
    router_params: RouterParams,
) -> Result<Router, CommonError> {
    let mut router = Router::new();

    // let (live_connection_changes_tx, mut live_connection_changes_rx) = tokio::sync::mpsc::channel(10);

    // agent router

    let (agent_router, _) = agent::create_router().split_for_parts();
    
    let agent_router = agent_router.with_state(router_params.agent_service);
    router = router.merge(agent_router);

    // task router
    let (task_router, _) = task::create_router().split_for_parts();
    let task_router = task_router.with_state(router_params.task_service);
    router = router.merge(task_router);

    // frontend router
    let (fe_router, _) = frontend::create_router().split_for_parts();
    let fe_router = fe_router.with_state(router_params.frontend_service);
    router = router.merge(fe_router);

    // mcp router
    let (mcp_router, _) = mcp::create_router().split_for_parts();
    let mcp_router = mcp_router.with_state(router_params.mcp_service);
    router = router.merge(mcp_router);

    let router = router.layer(CorsLayer::permissive());

    Ok(router)
}

pub(crate) fn generate_openapi_spec() -> OpenApi {
    let (_, mut spec) = frontend::create_router().split_for_parts();
    let (_, agent_spec) = agent::create_router().split_for_parts();
    let (_, task_spec) = task::create_router().split_for_parts();
    let (_, mcp_spec) = mcp::create_router().split_for_parts();
    spec.merge(agent_spec);
    spec.merge(task_spec);
    spec.merge(mcp_spec);

    spec
}
