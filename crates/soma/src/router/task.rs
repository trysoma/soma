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

use crate::{repository::Repository};
use shared::{error::CommonError, adapters::openapi::JsonResponse};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "task";

fn create_router() -> OpenApiRouter<Arc<TaskService>> {
    OpenApiRouter::new()
        .routes(routes!(route_runtime_config))
}


#[utoipa::path(
    get,
    path = format!("{}/{}/{}/runtime_config", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    responses(
        (status = 200, description = "Runtime config", body = RuntimeConfig),
    )
)]
async fn route_runtime_config(
    State(ctx): State<Arc<TaskService>>,
) -> JsonResponse<RuntimeConfig, CommonError> {
    let runtime_config = runtime_config().await;
    JsonResponse::from(runtime_config)
}


#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RuntimeConfig {
}


async fn runtime_config() -> Result<RuntimeConfig, CommonError> {
    Ok(RuntimeConfig {
    })
}

pub struct TaskService {
    repository: Arc<Repository>,
}