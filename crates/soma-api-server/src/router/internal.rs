use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tonic::Request;
use tracing::{info, warn};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use shared::{adapters::openapi::JsonResponse, error::CommonError};

pub const PATH_PREFIX: &str = "/_internal";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "";

pub fn create_router() -> OpenApiRouter<Arc<InternalService>> {
    OpenApiRouter::new()
        .routes(routes!(route_health))
        .routes(routes!(route_runtime_config))
        .routes(routes!(route_trigger_codegen))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/health", PATH_PREFIX, API_VERSION_1),
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service unavailable - SDK server not ready"),
    ),
    operation_id = "health-check",
)]
async fn route_health(State(ctx): State<Arc<InternalService>>) -> axum::http::StatusCode {
    match ctx.check_health().await {
        Ok(()) => axum::http::StatusCode::OK,
        Err(_) => axum::http::StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[utoipa::path(
    get,
    path = format!("{}/{}/runtime_config", PATH_PREFIX, API_VERSION_1),
    responses(
        (status = 200, description = "Runtime config", body = RuntimeConfig),
    ),
    operation_id = "get-internal-runtime-config",
)]
async fn route_runtime_config(
    State(_ctx): State<Arc<InternalService>>,
) -> JsonResponse<RuntimeConfig, CommonError> {
    let runtime_config = runtime_config().await;
    JsonResponse::from(runtime_config)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/trigger_codegen", PATH_PREFIX, API_VERSION_1),
    responses(
        (status = 200, description = "Codegen triggered successfully", body = TriggerCodegenResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "trigger-codegen",
)]
async fn route_trigger_codegen(
    State(ctx): State<Arc<InternalService>>,
) -> JsonResponse<TriggerCodegenResponse, CommonError> {
    let response = ctx.trigger_codegen().await;
    JsonResponse::from(response)
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RuntimeConfig {}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TriggerCodegenResponse {
    pub message: String,
}

async fn runtime_config() -> Result<RuntimeConfig, CommonError> {
    Ok(RuntimeConfig {})
}

pub struct InternalService {
    bridge_service: bridge::router::bridge::BridgeService,
}

impl InternalService {
    pub fn new(bridge_service: bridge::router::bridge::BridgeService) -> Self {
        Self { bridge_service }
    }

    /// Checks SDK server health
    async fn check_health(&self) -> Result<(), CommonError> {
        let mut sdk_client_guard = self.bridge_service.sdk_client().lock().await;

        if let Some(ref mut client) = *sdk_client_guard {
            // Call SDK health check
            let request = Request::new(());
            match client.health_check(request).await {
                Ok(_) => {
                    info!("SDK server health check passed");
                    Ok(())
                }
                Err(e) => {
                    warn!("SDK server health check failed: {:?}", e);
                    Err(CommonError::Unknown(anyhow::anyhow!(
                        "SDK server health check failed: {e}"
                    )))
                }
            }
        } else {
            warn!("SDK client not available");
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK client not available"
            )))
        }
    }

    pub async fn trigger_codegen(&self) -> Result<TriggerCodegenResponse, CommonError> {
        // Get SDK client from bridge service
        let mut sdk_client_guard = self.bridge_service.sdk_client().lock().await;

        if let Some(ref mut client) = *sdk_client_guard {
            // Trigger bridge client generation
            bridge::logic::codegen::trigger_bridge_client_generation(
                client,
                self.bridge_service.repository(),
            )
            .await?;

            Ok(TriggerCodegenResponse {
                message: "Bridge client generation completed successfully".to_string(),
            })
        } else {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK client not available. Please ensure the SDK server is running."
            )))
        }
    }
}
