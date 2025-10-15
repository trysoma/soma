use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
    routing::any,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use vite_rs_axum_0_8::ViteServe;

use shared::{adapters::openapi::JsonResponse, error::CommonError};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "frontend";

fn create_api_router() -> OpenApiRouter<Arc<FrontendService>> {
    OpenApiRouter::new().routes(routes!(route_runtime_config))
}

#[cfg(debug_assertions)]
pub fn create_router() -> OpenApiRouter<Arc<FrontendService>> {
    use crate::vite::Assets;

    let vite = ViteServe::new(Assets::boxed());

    let fe_router = create_api_router();

    let vite_router =
        OpenApiRouter::new()
            // "/" handled explicitly
            .route(
                "/",
                any(
                    |State(vite): State<ViteServe>, req: Request<Body>| async move {
                        vite.serve(req).await
                    },
                ),
            )
            // all other paths handled by SPA fallback
            .route("/{*path}", any(tanstack_spa_handler))
            .with_state(vite);

    fe_router.merge(vite_router)
}

#[cfg(not(debug_assertions))]
const ROUTES_JSON: &[u8] = include_bytes!(concat!(
    env!("FRONTEND_APP_DIR"),
    "/dist/.vite-rs/routes.json"
));

#[cfg(not(debug_assertions))]
pub fn create_router() -> OpenApiRouter<Arc<FrontendService>> {
    let vite = ViteServe::new(Assets::boxed());

    #[derive(Debug, Deserialize, Serialize, ToSchema)]
    struct RouteFile {
        paths: Vec<String>,
        assets: Vec<String>,
    }

    let routes = serde_json::from_slice::<RouteFile>(ROUTES_JSON).unwrap();
    let fe_router = create_api_router();

    let mut vite_router = OpenApiRouter::new()
        .route(
            "/",
            any(|State(vite): State<ViteServe>, req: Request<Body>| async move {
                vite.serve(req).await
            }),
        );

    let mut all_paths = vec![];

    all_paths.extend(routes.paths);
    all_paths.extend(routes.assets.iter().map(|asset| format!("/{}", asset)));

    for path in all_paths {
        if path == "/" {
            continue;
        }
        vite_router = vite_router.route(path.as_str(), any(tanstack_spa_handler));
    }

    let vite_router = vite_router.with_state(vite);

    fe_router.merge(vite_router)
}

async fn tanstack_spa_handler(State(vite): State<ViteServe>, req: Request<Body>) -> Response {
    let resp = vite.serve(req).await;

    if resp.status() == StatusCode::NOT_FOUND {
        if let Some(index_file) = vite.assets.get("index.html") {
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", index_file.content_type)
                .body(Body::from(index_file.bytes))
                .unwrap();
        }
    }

    resp
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/runtime_config", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    responses(
        (status = 200, description = "Runtime config", body = RuntimeConfig),
    ),
    operation_id = "get-frontend-env",
)]
async fn route_runtime_config(
    State(ctx): State<Arc<FrontendService>>,
) -> JsonResponse<RuntimeConfig, CommonError> {
    let runtime_config = runtime_config().await;
    JsonResponse::from(runtime_config)
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RuntimeConfig {}

async fn runtime_config() -> Result<RuntimeConfig, CommonError> {
    Ok(RuntimeConfig {})
}

pub struct FrontendService {}

impl FrontendService {
    pub fn new() -> Self {
        Self {}
    }
}
