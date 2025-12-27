use axum::extract::State;
use encryption::logic::crypto_services::CryptoCache;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};

use crate::logic::internal::{
    CheckSdkHealthResponse, ResyncSdkResponse, RuntimeConfigResponse, TriggerCodegenResponse,
};
use crate::sdk::sdk_agent_sync::AgentCache;

pub const PATH_PREFIX: &str = "/_internal";
pub const API_VERSION_1: &str = "v1";
#[allow(dead_code)]
pub const SERVICE_ROUTE_KEY: &str = "";

pub fn create_router() -> OpenApiRouter<Arc<InternalService>> {
    OpenApiRouter::new()
        .routes(routes!(route_health))
        .routes(routes!(route_runtime_config))
        .routes(routes!(route_trigger_codegen))
        .routes(routes!(route_resync_sdk))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/health", PATH_PREFIX, API_VERSION_1),
    tags = ["_internal", API_VERSION_TAG],
    responses(
        (status = 200, description = "Service is healthy", body = CheckSdkHealthResponse),
        (status = 503, description = "Service unavailable - SDK server not ready"),
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
async fn route_health(
    State(ctx): State<Arc<InternalService>>,
) -> JsonResponse<CheckSdkHealthResponse, CommonError> {
    trace!("Checking SDK health");
    let response = crate::logic::internal::check_sdk_health(&ctx.sdk_client).await;
    trace!(success = response.is_ok(), "Checking SDK health completed");
    JsonResponse::from(response)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/runtime_config", PATH_PREFIX, API_VERSION_1),
    tags = ["_internal", API_VERSION_TAG],
    responses(
        (status = 200, description = "Runtime config", body = RuntimeConfigResponse),
    ),
    summary = "Get runtime config",
    description = "Get the current runtime configuration",
    operation_id = "get-internal-runtime-config",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_runtime_config(
    State(_ctx): State<Arc<InternalService>>,
) -> JsonResponse<RuntimeConfigResponse, CommonError> {
    trace!("Getting runtime config");
    let runtime_config = crate::logic::internal::runtime_config().await;
    trace!(
        success = runtime_config.is_ok(),
        "Getting runtime config completed"
    );
    JsonResponse::from(runtime_config)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/trigger_codegen", PATH_PREFIX, API_VERSION_1),
    tags = ["_internal", API_VERSION_TAG],
    responses(
        (status = 200, description = "Codegen triggered successfully", body = TriggerCodegenResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Trigger codegen",
    description = "Trigger code generation for the SDK",
    operation_id = "trigger-codegen",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_trigger_codegen(
    State(ctx): State<Arc<InternalService>>,
) -> JsonResponse<TriggerCodegenResponse, CommonError> {
    trace!("Triggering codegen");
    let response = crate::logic::internal::trigger_codegen(
        &ctx.sdk_client,
        ctx.mcp_service.repository(),
        &ctx.agent_cache,
    )
    .await;
    trace!(success = response.is_ok(), "Triggering codegen completed");

    JsonResponse::from(response)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/resync_sdk", PATH_PREFIX, API_VERSION_1),
    tags = ["_internal", API_VERSION_TAG],
    responses(
        (status = 200, description = "SDK resynced successfully", body = ResyncSdkResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Resync SDK",
    description = "Resync providers, agents, secrets, and environment variables between API server and SDK",
    operation_id = "resync-sdk",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_resync_sdk(
    State(ctx): State<Arc<InternalService>>,
) -> JsonResponse<ResyncSdkResponse, CommonError> {
    trace!("Resyncing SDK");
    let response = crate::logic::internal::resync_sdk(
        &ctx.repository,
        &ctx.crypto_cache,
        &ctx.restate_params,
        &ctx.sdk_client,
        &ctx.agent_cache,
        ctx.mcp_service.repository(),
    )
    .await;
    trace!(success = response.is_ok(), "Resyncing SDK completed");
    JsonResponse::from(response)
}

pub struct InternalService {
    mcp_service: mcp::router::McpService,
    sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    repository: std::sync::Arc<crate::repository::Repository>,
    crypto_cache: CryptoCache,
    restate_params: crate::restate::RestateServerParams,
    agent_cache: AgentCache,
}

impl InternalService {
    pub fn new(
        mcp_service: mcp::router::McpService,
        sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
        repository: std::sync::Arc<crate::repository::Repository>,
        crypto_cache: CryptoCache,
        restate_params: crate::restate::RestateServerParams,
        agent_cache: AgentCache,
    ) -> Self {
        Self {
            mcp_service,
            sdk_client,
            repository,
            crypto_cache,
            restate_params,
            agent_cache,
        }
    }
}
