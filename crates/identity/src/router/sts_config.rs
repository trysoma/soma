use axum::Json;
use axum::extract::{Path, Query, State};
use http::HeaderMap;
use shared::primitives::PaginationRequest;
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::sts::config::{
    DeleteStsConfigParams, DeleteStsConfigResponse, GetStsConfigParams, ListStsConfigResponse,
    StsTokenConfig, create_sts_config, delete_sts_config, get_sts_config, list_sts_configs,
};
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

pub fn create_sts_config_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_create_sts_config))
        .routes(routes!(route_get_sts_config))
        .routes(routes!(route_delete_sts_config))
        .routes(routes!(route_list_sts_configs))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/sts-configuration", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = StsTokenConfig,
    responses(
        (status = 201, description = "STS configuration created successfully", body = StsTokenConfig),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Create STS configuration",
    description = "Create a new STS configuration (e.g., JWT template or dev settings)",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_create_sts_config(
    State(service): State<IdentityService>,
    headers: HeaderMap,
    Json(params): Json<StsTokenConfig>,
) -> JsonResponse<StsTokenConfig, CommonError> {
    trace!("Creating STS configuration");
    let result = create_sts_config(
        service.auth_client.clone(),
        headers,
        service.repository.as_ref(),
        &service.on_config_change_tx,
        params,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = result.is_ok(),
        "Creating STS configuration completed"
    );
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/sts-configuration/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the STS configuration to retrieve")
    ),
    responses(
        (status = 200, description = "STS configuration found", body = StsTokenConfig),
        (status = 404, description = "STS configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Get STS configuration",
    description = "Get an STS configuration by ID",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_sts_config(
    State(service): State<IdentityService>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> JsonResponse<StsTokenConfig, CommonError> {
    trace!(config_id = %id, "Getting STS configuration");
    let params = GetStsConfigParams { id };
    let result = get_sts_config(
        service.auth_client.clone(),
        headers,
        service.repository.as_ref(),
        params,
    )
    .await;
    trace!(
        success = result.is_ok(),
        "Getting STS configuration completed"
    );
    JsonResponse::from(result)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/sts-configuration/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the STS configuration to delete")
    ),
    responses(
        (status = 200, description = "STS configuration deleted successfully", body = DeleteStsConfigResponse),
        (status = 404, description = "STS configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Delete STS configuration",
    description = "Delete an STS configuration by ID",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_delete_sts_config(
    State(service): State<IdentityService>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> JsonResponse<DeleteStsConfigResponse, CommonError> {
    trace!(config_id = %id, "Deleting STS configuration");
    let params = DeleteStsConfigParams { id };
    let result = delete_sts_config(
        service.auth_client.clone(),
        headers,
        service.repository.as_ref(),
        &service.on_config_change_tx,
        params,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = result.is_ok(),
        "Deleting STS configuration completed"
    );
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/sts-configuration", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List of STS configurations", body = ListStsConfigResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "List STS configurations",
    description = "List all STS configurations with optional filtering by type",
    security(
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_sts_configs(
    State(service): State<IdentityService>,
    headers: HeaderMap,
    Query(query): Query<PaginationRequest>,
) -> JsonResponse<ListStsConfigResponse, CommonError> {
    trace!(page_size = query.page_size, "Listing STS configurations");
    let result = list_sts_configs(
        service.auth_client.clone(),
        headers,
        service.repository.as_ref(),
        &query,
    )
    .await;
    trace!(
        success = result.is_ok(),
        "Listing STS configurations completed"
    );
    JsonResponse::from(result)
}
