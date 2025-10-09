use std::{env, sync::Arc};

use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use url::Url;
use utoipa::openapi::OpenApi;

use crate::{commands::StartParams, router::{agent::AgentService, frontend::FrontendService}};
use shared::{error::CommonError};
use crate::utils::construct_src_dir_absolute;

pub(crate) mod frontend;
pub(crate) mod agent;
pub(crate) mod task;

pub(crate) fn initiate_routers(params: &StartParams) -> Result<Router, CommonError> {
    let mut router = Router::new();

  

    // agent router
    let src_dir = construct_src_dir_absolute(params.src_dir.clone())?;


    let (agent_router, _) = agent::create_router()
        .split_for_parts();
    let agent_router = agent_router.with_state(Arc::new(AgentService::new(src_dir, Url::parse(format!("http://{}:{}", params.host, params.port).as_str())?)));
    router = router.merge(agent_router);

    // frontend router
    let (fe_router, _) = frontend::create_router()
        .split_for_parts();
    let fe_router = fe_router.with_state(Arc::new(FrontendService {}));
    router = router
        .merge(fe_router);

    let router = router.layer(CorsLayer::permissive());

    Ok(router)
}

pub(crate) fn generate_openapi_spec() -> OpenApi {
    let (_, spec) = frontend::create_router()
        .split_for_parts();

    let (_, spec) = agent::create_router()
        .split_for_parts();
    
    spec
}