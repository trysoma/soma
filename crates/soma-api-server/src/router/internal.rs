use axum::extract::State;
use encryption::logic::crypto_services::CryptoCache;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};

use crate::logic::internal::HealthCheckResponse;

pub const PATH_PREFIX: &str = "/_internal";
pub const API_VERSION_1: &str = "v1";
#[allow(dead_code)]
pub const SERVICE_ROUTE_KEY: &str = "";

pub fn create_router() -> OpenApiRouter<Arc<InternalService>> {
    OpenApiRouter::new().routes(routes!(route_health_check))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/health", PATH_PREFIX, API_VERSION_1),
    tags = ["_internal", API_VERSION_TAG],
    responses(
        (status = 200, description = "Health check", body = HealthCheckResponse),
    ),
    summary = "Health check",
    description = "Check the health status of the service and SDK server connectivity",
    operation_id = "health-check",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_health_check(
    State(_ctx): State<Arc<InternalService>>,
) -> JsonResponse<HealthCheckResponse, CommonError> {
    trace!("Health check");
    let health_check = crate::logic::internal::health_check().await;
    trace!(
        success = health_check.is_ok(),
        "Health check completed"
    );
    JsonResponse::from(health_check)
}

pub struct InternalService {
    _mcp_service: tool::router::McpService,
    _environment_repository: std::sync::Arc<environment::repository::Repository>,
    _crypto_cache: CryptoCache,
}

impl InternalService {
    pub fn new(
        mcp_service: tool::router::McpService,
        environment_repository: std::sync::Arc<environment::repository::Repository>,
        crypto_cache: CryptoCache,
    ) -> Self {
        Self {
            _mcp_service: mcp_service,
            _environment_repository: environment_repository,
            _crypto_cache: crypto_cache,
        }
    }
}
