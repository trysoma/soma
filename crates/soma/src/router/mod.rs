use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use bridge::logic::{CreateDataEncryptionKeyParams, EncryptedDataEncryptionKey, OnConfigChangeRx};
use tower_http::cors::CorsLayer;
use tracing::info;
use url::Url;
use utoipa::openapi::OpenApi;

use crate::router::mcp::McpService;
use crate::router::task::TaskService;
use crate::utils::construct_src_dir_absolute;
use crate::utils::restate::admin_client::AdminClient;
use crate::utils::restate::invoke::RestateIngressClient;
use crate::{
    commands::dev::DevParams,
    router::{a2a::Agent2AgentService, frontend::FrontendService},
};
use crate::{logic::ConnectionManager, repository::Repository};
use bridge::{
    logic::{
        EnvelopeEncryptionKeyContents, OnConfigChangeTx, create_data_encryption_key,
        register_all_bridge_providers,
    },
    router::bridge::{BridgeService, create_router as create_bridge_router},
};
use shared::error::CommonError;
use shared::soma_agent_definition::{SomaAgentDefinition, SomaAgentDefinitionLike};

pub(crate) mod a2a;
pub(crate) mod frontend;
pub(crate) mod mcp;
pub(crate) mod task;

#[derive(Clone)]
pub(crate) struct Routers {
    pub agent_service: Arc<Agent2AgentService>,
    pub task_service: Arc<TaskService>,
    pub frontend_service: Arc<FrontendService>,
    // pub mcp_service: McpService,
    pub bridge_service: BridgeService,
}

pub(crate) struct InitRouterParams {
    pub project_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub mcp_transport_tx:
        tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport>,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub runtime_port: u16,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
    pub db_connection: shared::libsql::Connection,
    pub on_bridge_config_change_tx: OnConfigChangeTx,
    pub envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    pub bridge_repository: bridge::repository::Repository,
    pub mcp_sse_ping_interval: Duration,
}

impl Routers {
    pub async fn new(
        init_params: InitRouterParams,
    ) -> Result<Self, CommonError> {

        let agent_service = Arc::new(Agent2AgentService::new(
            init_params.project_dir.clone(),
            init_params.soma_definition.clone(),
            Url::parse(format!("http://{}:{}", init_params.host, init_params.port).as_str())?,
            init_params.connection_manager.clone(),
            init_params.repository.clone(),
            init_params.runtime_port,
            init_params.restate_ingress_client.clone(),
            init_params.restate_admin_client.clone(),
        ));
        let task_service = Arc::new(TaskService::new(
            init_params.connection_manager.clone(),
            init_params.repository.clone(),
        ));
        let frontend_service = Arc::new(FrontendService::new());
        // internally it's an Arc<McpServiceInner>
        // let mcp_service = McpService::new(
        //     init_params.mcp_transport_tx,
        //     init_params.repository.clone(),
        //     init_params.connection_manager.clone(),
        // );

        let bridge_service = BridgeService::new(
            init_params.bridge_repository.clone(),
            init_params.on_bridge_config_change_tx.clone(),
            init_params.envelope_encryption_key_contents.clone(),
            init_params.mcp_transport_tx,
            init_params.mcp_sse_ping_interval,
        ).await?;



        // register_all_bridge_providers().await?;
        // info!("Bridge providers registered");

        // let defintion = init_params.soma_definition.get_definition().await?;
        // if let Some(bridge) = &defintion.bridge {
        //     futures::future::try_join_all(bridge.encryption.0.iter().map(async |(id, encryption)| {
        //         create_data_encryption_key(
        //             &init_params.envelope_encryption_key_contents,
        //             &init_params.on_bridge_config_change_tx.clone(),
        //             &bridge_repository.clone(),
        //             CreateDataEncryptionKeyParams {
        //                 id: Some(id.clone()),
        //                 encrypted_data_envelope_key: Some(EncryptedDataEncryptionKey(encryption.encrypted_data_encryption_key.clone())),
        //             },
        //             true,
        //         )
        //         .await
        //     })).await?;
        // }

        Ok(Self {
            agent_service,
            task_service,
            frontend_service,
            // mcp_service,
            bridge_service,
        })
    }
}

pub(crate) fn initiate_routers(routers: Routers) -> Result<Router, CommonError> {
    let mut router = Router::new();

    // let (live_connection_changes_tx, mut live_connection_changes_rx) = tokio::sync::mpsc::channel(10);

    // agent router

    let (agent_router, _) = a2a::create_router().split_for_parts();

    let agent_router = agent_router.with_state(routers.agent_service);
    router = router.merge(agent_router);

    // task router
    let (task_router, _) = task::create_router().split_for_parts();
    let task_router = task_router.with_state(routers.task_service);
    router = router.merge(task_router);

    // frontend router
    let (fe_router, _) = frontend::create_router().split_for_parts();
    let fe_router = fe_router.with_state(routers.frontend_service);
    router = router.merge(fe_router);

    // mcp router
    // let (mcp_router, _) = mcp::create_router().split_for_parts();
    // let mcp_router = mcp_router.with_state(router_params.mcp_service);
    // router = router.merge(mcp_router);

    // bridge router
    let (bridge_router, _) = create_bridge_router().split_for_parts();
    let bridge_router = bridge_router.with_state(routers.bridge_service);
    router = router.merge(bridge_router);

    let router = router.layer(CorsLayer::permissive());

    Ok(router)
}

pub(crate) fn generate_openapi_spec() -> OpenApi {
    let (_, mut spec) = frontend::create_router().split_for_parts();
    let (_, agent_spec) = a2a::create_router().split_for_parts();
    let (_, task_spec) = task::create_router().split_for_parts();
    let (_, mcp_spec) = mcp::create_router().split_for_parts();
    let (_, bridge_spec) = create_bridge_router().split_for_parts();
    spec.merge(agent_spec);
    spec.merge(task_spec);
    spec.merge(mcp_spec);
    spec.merge(bridge_spec);

    spec
}
