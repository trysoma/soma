use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use shared::{
    adapters::openapi::{JsonResponse, API_VERSION_TAG},
    error::CommonError,
};
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::sts_config::{
    create_sts_config, delete_sts_config, get_sts_config, import_sts_config, list_sts_configs,
    CreateStsConfigParams, CreateStsConfigResponse, DeleteStsConfigParams, DeleteStsConfigResponse,
    GetStsConfigParams, ListStsConfigParams, ListStsConfigResponse,
};
use crate::repository::StsConfiguration;
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

pub fn create_sts_config_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_create_sts_config))
        .routes(routes!(route_get_sts_config))
        .routes(routes!(route_delete_sts_config))
        .routes(routes!(route_list_sts_configs))
        .routes(routes!(route_import_sts_config))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListStsConfigsQuery {
    #[param(example = "10")]
    page_size: Option<u32>,
    #[param(example = "")]
    next_page_token: Option<String>,
    /// Filter by configuration type (jwt_template, dev)
    #[param(example = "jwt_template")]
    #[serde(rename = "type")]
    config_type: Option<String>,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/sts-configuration", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateStsConfigParams,
    responses(
        (status = 201, description = "STS configuration created successfully", body = CreateStsConfigResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Create STS configuration",
    description = "Create a new STS configuration (e.g., JWT template or dev settings)",
)]
async fn route_create_sts_config(
    State(service): State<IdentityService>,
    Json(params): Json<CreateStsConfigParams>,
) -> JsonResponse<CreateStsConfigResponse, CommonError> {
    let result = create_sts_config(
        service.repository.as_ref(),
        service.on_config_change_tx(),
        params,
        true, // publish_on_change_evt
    )
    .await;
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
        (status = 200, description = "STS configuration found", body = StsConfiguration),
        (status = 404, description = "STS configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Get STS configuration",
    description = "Get an STS configuration by ID",
)]
async fn route_get_sts_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<StsConfiguration, CommonError> {
    let params = GetStsConfigParams { id };
    let result = get_sts_config(service.repository.as_ref(), params).await;
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
)]
async fn route_delete_sts_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<DeleteStsConfigResponse, CommonError> {
    let params = DeleteStsConfigParams { id };
    let result = delete_sts_config(
        service.repository.as_ref(),
        service.on_config_change_tx(),
        params,
        true, // publish_on_change_evt
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/sts-configuration", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListStsConfigsQuery
    ),
    responses(
        (status = 200, description = "List of STS configurations", body = ListStsConfigResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "List STS configurations",
    description = "List all STS configurations with optional filtering by type",
)]
async fn route_list_sts_configs(
    State(service): State<IdentityService>,
    Query(query): Query<ListStsConfigsQuery>,
) -> JsonResponse<ListStsConfigResponse, CommonError> {
    use shared::primitives::PaginationRequest;
    let params = ListStsConfigParams {
        pagination: PaginationRequest {
            page_size: query.page_size.unwrap_or(10) as i64,
            next_page_token: query.next_page_token,
        },
        config_type: query.config_type,
    };
    let result = list_sts_configs(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/sts-configuration/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateStsConfigParams,
    responses(
        (status = 201, description = "STS configuration imported successfully", body = CreateStsConfigResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Import STS configuration",
    description = "Import an STS configuration (idempotent, used for syncing from soma.yaml)",
)]
async fn route_import_sts_config(
    State(service): State<IdentityService>,
    Json(params): Json<CreateStsConfigParams>,
) -> JsonResponse<CreateStsConfigResponse, CommonError> {
    let result = import_sts_config(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}
