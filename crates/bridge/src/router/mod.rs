use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use utoipa::openapi::OpenApi;

use crate::repository::Repository;
use crate::router::bridge::BridgeService;

pub(crate) mod bridge;

#[derive(Clone)]
pub(crate) struct RouterParams {
    pub bridge_service: Arc<BridgeService>,
}

pub(crate) struct InitRouterParams {
    pub repository: Repository,
}

impl RouterParams {
    pub fn new(init_params: InitRouterParams) -> Self {
        let bridge_service = Arc::new(BridgeService::new(init_params.repository));
        Self { bridge_service }
    }
}

pub(crate) fn initiate_routers(router_params: RouterParams) -> Router {
    let mut router = Router::new();

    // bridge router
    let (bridge_router, _) = bridge::create_router().split_for_parts();
    let bridge_router = bridge_router.with_state(router_params.bridge_service);
    router = router.merge(bridge_router);

    let router = router.layer(CorsLayer::permissive());

    router
}

pub(crate) fn generate_openapi_spec() -> OpenApi {
    let (_, spec) = bridge::create_router().split_for_parts();
    spec
}
