use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use bridge::logic::register_all_bridge_providers;
use url::Url;
use utoipa::openapi::OpenApi;

use crate::ApiService;
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
pub(crate) mod internal;
pub(crate) mod task;


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

    // internal router
    let (internal_router, _) = internal::create_router().split_for_parts();
    let internal_router = internal_router.with_state(api_service.internal_service);
    router = router.merge(internal_router);

    Ok(router)
}

pub fn generate_openapi_spec() -> OpenApi {
    let (_, mut spec) = a2a::create_router().split_for_parts();
    let (_, task_spec) = task::create_router().split_for_parts();
    let (_, bridge_spec) = create_bridge_router().split_for_parts();
    let (_, internal_spec) = internal::create_router().split_for_parts();
    spec.merge(task_spec);
    spec.merge(bridge_spec);
    spec.merge(internal_spec);

    spec
}
