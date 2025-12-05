use axum::Json;
use axum::extract::{Path, Query, State};
use shared::primitives::PaginationRequest;
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::logic::api_key::{
    CreateApiKeyParams, CreateApiKeyResponse, DeleteApiKeyParams, DeleteApiKeyResponse,
    EncryptedApiKeyConfig, ImportApiKeyResponse, ListApiKeysResponse, create_api_key,
    delete_api_key, import_api_key, list_api_keys,
};
use crate::service::IdentityService;

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

pub fn create_api_key_routes() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        .routes(routes!(route_create_api_key))
        .routes(routes!(route_delete_api_key))
        .routes(routes!(route_list_api_keys))
        .routes(routes!(route_import_api_key))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/api-key", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateApiKeyParams,
    responses(
        (status = 201, description = "API key created successfully", body = CreateApiKeyResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_create_api_key(
    State(service): State<IdentityService>,
    Json(params): Json<CreateApiKeyParams>,
) -> JsonResponse<CreateApiKeyResponse, CommonError> {
    let result = create_api_key(
        service.repository.as_ref(),
        &service.crypto_cache,
        &service.on_config_change_tx,
        Some(&service.api_key_cache),
        params,
        true, // publish_on_change_evt
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/api-key/{{id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("id" = String, Path, description = "ID of the API key to delete")
    ),
    responses(
        (status = 200, description = "API key deleted successfully", body = DeleteApiKeyResponse),
        (status = 404, description = "API key not found", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_delete_api_key(
    State(service): State<IdentityService>,
    Path(id): Path<String>,
) -> JsonResponse<DeleteApiKeyResponse, CommonError> {
    let params = DeleteApiKeyParams { id };
    let result = delete_api_key(
        service.repository.as_ref(),
        &service.on_config_change_tx,
        Some(&service.api_key_cache),
        params,
        true, // publish_on_change_evt
    )
    .await;
    JsonResponse::from(result)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/api-key", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List of API keys", body = ListApiKeysResponse),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_list_api_keys(
    State(service): State<IdentityService>,
    Query(query): Query<PaginationRequest>,
) -> JsonResponse<ListApiKeysResponse, CommonError> {
    let result = list_api_keys(service.repository.as_ref(), query).await;
    JsonResponse::from(result)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/api-key/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = EncryptedApiKeyConfig,
    responses(
        (status = 201, description = "API key imported successfully", body = ImportApiKeyResponse),
        (status = 400, description = "Invalid request", body = CommonError),
        (status = 500, description = "Internal server error", body = CommonError),
    ),
)]
async fn route_import_api_key(
    State(service): State<IdentityService>,
    Json(params): Json<EncryptedApiKeyConfig>,
) -> JsonResponse<ImportApiKeyResponse, CommonError> {
    let result = import_api_key(
        service.repository.as_ref(),
        &service.crypto_cache,
        Some(&service.api_key_cache),
        params,
    )
    .await;
    JsonResponse::from(result)
}
