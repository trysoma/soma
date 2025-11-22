use std::sync::Arc;

use axum::Router;
use shared::error::CommonError;
use soma_api_server::router::{ApiService, generate_openapi_spec as generate_soma_api_server_openapi_spec, initiaite_api_router as initiaite_soma_api_server_router};
use tower_http::cors::CorsLayer;
use utoipa::openapi::OpenApi;

use crate::router::frontend::FrontendService;

mod frontend;

pub fn generate_combined_openapi_spec() -> OpenApi {
    let mut api_spec = generate_soma_api_server_openapi_spec();
    let (_, spec) = frontend::create_router().split_for_parts();

    api_spec.merge(spec);

    api_spec
}

fn initiate_fe_router(frontend_service: Arc<FrontendService>) -> Result<Router, CommonError> {
    let (fe_router, _) = frontend::create_router().split_for_parts();
    let router = fe_router.with_state(frontend_service);
    Ok(router)
}

fn initiaite_combined_router(
    api_service: ApiService,
    frontend_service: Arc<FrontendService>,
) -> Result<Router, CommonError> {
    let router = Router::new()
        .merge(initiate_fe_router(frontend_service)?)
        .merge(initiaite_soma_api_server_router(api_service)?)
        .layer(CorsLayer::permissive());

    Ok(router)
}
