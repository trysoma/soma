use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use shared::{
    adapters::openapi::{JsonResponse, API_VERSION_TAG},
    error::CommonError,
};
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::idp_config::{
    create_idp_config, delete_idp_config, get_idp_config, import_idp_config, list_idp_configs,
    update_idp_config, CreateIdpConfigParams, CreateIdpConfigResponse, DeleteIdpConfigParams,
    DeleteIdpConfigResponse, GetIdpConfigParams, GetIdpConfigResponse, ListIdpConfigParams,
    ListIdpConfigResponse, UpdateIdpConfigParams, UpdateIdpConfigResponse,
};
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

pub fn create_idp_config_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_create_idp_config))
        .routes(routes!(route_get_idp_config))
        .routes(routes!(route_update_idp_config))
        .routes(routes!(route_delete_idp_config))
        .routes(routes!(route_list_idp_configs))
        .routes(routes!(route_import_idp_config))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListIdpConfigsQuery {
    #[param(example = "10")]
    page_size: Option<u32>,
    #[param(example = "")]
    next_page_token: Option<String>,
    /// Filter by configuration type (oidc_authorization_flow, oauth_authorization_flow)
    #[param(example = "oidc_authorization_flow")]
    #[serde(rename = "type")]
    config_type: Option<String>,
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/idp-configuration", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateIdpConfigParams,
    responses(
        (status = 201, description = "IdP configuration created successfully", body = CreateIdpConfigResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Create IdP configuration",
    description = "Create a new IdP configuration for OAuth/OIDC authorization flows",
)]
async fn route_create_idp_config(
    State(service): State<IdentityService>,
    Json(params): Json<CreateIdpConfigParams>,
) -> JsonResponse<CreateIdpConfigResponse, CommonError> {
    let result = create_idp_config(
        service.repository.as_ref(),
        &service.crypto_cache,
        service.on_config_change_tx(),
        params,
        true, // publish_on_change_evt
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/idp-configuration/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the IdP configuration to retrieve")
    ),
    responses(
        (status = 200, description = "IdP configuration found", body = GetIdpConfigResponse),
        (status = 404, description = "IdP configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Get IdP configuration",
    description = "Get an IdP configuration by ID",
)]
async fn route_get_idp_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<GetIdpConfigResponse, CommonError> {
    let params = GetIdpConfigParams { id };
    let result = get_idp_config(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/idp-configuration/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the IdP configuration to update")
    ),
    request_body = UpdateIdpConfigParams,
    responses(
        (status = 200, description = "IdP configuration updated successfully", body = UpdateIdpConfigResponse),
        (status = 404, description = "IdP configuration not found", body = CommonError),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Update IdP configuration",
    description = "Update an existing IdP configuration",
)]
async fn route_update_idp_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
    Json(params): Json<UpdateIdpConfigParams>,
) -> JsonResponse<UpdateIdpConfigResponse, CommonError> {
    let result = update_idp_config(
        service.repository.as_ref(),
        &service.crypto_cache,
        &id,
        params,
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/idp-configuration/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the IdP configuration to delete")
    ),
    responses(
        (status = 200, description = "IdP configuration deleted successfully", body = DeleteIdpConfigResponse),
        (status = 404, description = "IdP configuration not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Delete IdP configuration",
    description = "Delete an IdP configuration by ID",
)]
async fn route_delete_idp_config(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<DeleteIdpConfigResponse, CommonError> {
    let params = DeleteIdpConfigParams { id };
    let result = delete_idp_config(
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
    path = format!("{}/{}/{}/idp-configuration", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListIdpConfigsQuery
    ),
    responses(
        (status = 200, description = "List of IdP configurations", body = ListIdpConfigResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "List IdP configurations",
    description = "List all IdP configurations with optional filtering by type",
)]
async fn route_list_idp_configs(
    State(service): State<IdentityService>,
    Query(query): Query<ListIdpConfigsQuery>,
) -> JsonResponse<ListIdpConfigResponse, CommonError> {
    use shared::primitives::PaginationRequest;
    let params = ListIdpConfigParams {
        pagination: PaginationRequest {
            page_size: query.page_size.unwrap_or(10) as i64,
            next_page_token: query.next_page_token,
        },
        config_type: query.config_type,
    };
    let result = list_idp_configs(service.repository.as_ref(), params).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/idp-configuration/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateIdpConfigParams,
    responses(
        (status = 201, description = "IdP configuration imported successfully", body = CreateIdpConfigResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
    summary = "Import IdP configuration",
    description = "Import an IdP configuration (idempotent, used for syncing from soma.yaml)",
)]
async fn route_import_idp_config(
    State(service): State<IdentityService>,
    Json(params): Json<CreateIdpConfigParams>,
) -> JsonResponse<CreateIdpConfigResponse, CommonError> {
    let result = import_idp_config(
        service.repository.as_ref(),
        &service.crypto_cache,
        params,
    )
    .await;
    JsonResponse::from(result)
}
