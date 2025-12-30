//! Secret HTTP endpoints

use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    logic::secret::{
        CreateSecretRequest, CreateSecretResponse, DeleteSecretResponse, GetSecretResponse,
        ImportSecretRequest, ListDecryptedSecretsResponse, ListSecretsResponse, Secret,
        UpdateSecretRequest, UpdateSecretResponse, create_secret, delete_secret, get_secret_by_id,
        get_secret_by_key, import_secret, list_decrypted_secrets, list_secrets, update_secret,
    },
    service::EnvironmentService,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "environment";

/// Create the secret router
pub fn create_router() -> OpenApiRouter<Arc<EnvironmentService>> {
    OpenApiRouter::new()
        .routes(routes!(route_create_secret))
        .routes(routes!(route_import_secret))
        .routes(routes!(route_list_secrets))
        .routes(routes!(route_list_decrypted_secrets))
        .routes(routes!(route_get_secret_by_id))
        .routes(routes!(route_get_secret_by_key))
        .routes(routes!(route_update_secret))
        .routes(routes!(route_delete_secret))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/secret", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateSecretRequest,
    responses(
        (status = 200, description = "Create a secret", body = CreateSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create secret",
    description = "Create a new encrypted secret with the specified key and value",
    operation_id = "create-secret",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_create_secret(
    State(ctx): State<Arc<EnvironmentService>>,
    Json(request): Json<CreateSecretRequest>,
) -> JsonResponse<CreateSecretResponse, CommonError> {
    trace!(key = %request.key, "Creating secret");
    let res = create_secret(
        &ctx.secret_change_tx,
        &ctx.repository,
        &ctx.crypto_cache,
        request,
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Creating secret completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/secret/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ImportSecretRequest,
    responses(
        (status = 200, description = "Import a pre-encrypted secret", body = Secret),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Import secret",
    description = "Import an existing pre-encrypted secret into the system",
    operation_id = "import-secret",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_import_secret(
    State(ctx): State<Arc<EnvironmentService>>,
    Json(request): Json<ImportSecretRequest>,
) -> JsonResponse<Secret, CommonError> {
    trace!(secret_key = %request.key, "Importing secret");
    let res = import_secret(&ctx.repository, request).await;
    trace!(success = res.is_ok(), "Importing secret completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/secret", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List secrets", body = ListSecretsResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List secrets",
    description = "List all secrets with pagination (values are encrypted)",
    operation_id = "list-secrets",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_secrets(
    State(ctx): State<Arc<EnvironmentService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListSecretsResponse, CommonError> {
    trace!(page_size = pagination.page_size, "Listing secrets");
    let res = list_secrets(&ctx.repository, pagination).await;
    trace!(success = res.is_ok(), "Listing secrets completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/secret/list-decrypted", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List secrets with decrypted values", body = ListDecryptedSecretsResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List decrypted secrets",
    description = "List all secrets with decrypted values (requires decryption access)",
    operation_id = "list-decrypted-secrets",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_decrypted_secrets(
    State(ctx): State<Arc<EnvironmentService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListDecryptedSecretsResponse, CommonError> {
    trace!(
        page_size = pagination.page_size,
        "Listing decrypted secrets"
    );
    let res = list_decrypted_secrets(&ctx.repository, &ctx.crypto_cache, pagination).await;
    trace!(success = res.is_ok(), "Listing decrypted secrets completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/secret/{{secret_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("secret_id" = WrappedUuidV4, Path, description = "Secret ID"),
    ),
    responses(
        (status = 200, description = "Get secret by id", body = GetSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get secret",
    description = "Retrieve a secret by its unique identifier",
    operation_id = "get-secret-by-id",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_secret_by_id(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(secret_id): Path<WrappedUuidV4>,
) -> JsonResponse<GetSecretResponse, CommonError> {
    trace!(secret_id = %secret_id, "Getting secret by ID");
    let res = get_secret_by_id(&ctx.repository, secret_id).await;
    trace!(success = res.is_ok(), "Getting secret by ID completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/secret/key/{{key}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("key" = String, Path, description = "Secret key"),
    ),
    responses(
        (status = 200, description = "Get secret by key", body = GetSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get secret by key",
    description = "Retrieve a secret by its key name",
    operation_id = "get-secret-by-key",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_secret_by_key(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(key): Path<String>,
) -> JsonResponse<GetSecretResponse, CommonError> {
    trace!(key = %key, "Getting secret by key");
    let res = get_secret_by_key(&ctx.repository, key).await;
    trace!(success = res.is_ok(), "Getting secret by key completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/secret/{{secret_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("secret_id" = WrappedUuidV4, Path, description = "Secret ID"),
    ),
    request_body = UpdateSecretRequest,
    responses(
        (status = 200, description = "Update secret", body = UpdateSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update secret",
    description = "Update an existing secret's value or metadata",
    operation_id = "update-secret",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_update_secret(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(secret_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateSecretRequest>,
) -> JsonResponse<UpdateSecretResponse, CommonError> {
    trace!(secret_id = %secret_id, "Updating secret");
    let res = update_secret(
        &ctx.secret_change_tx,
        &ctx.repository,
        &ctx.crypto_cache,
        secret_id,
        request,
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Updating secret completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/secret/{{secret_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("secret_id" = WrappedUuidV4, Path, description = "Secret ID"),
    ),
    responses(
        (status = 200, description = "Delete secret", body = DeleteSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete secret",
    description = "Delete a secret by its unique identifier",
    operation_id = "delete-secret",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_delete_secret(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(secret_id): Path<WrappedUuidV4>,
) -> JsonResponse<DeleteSecretResponse, CommonError> {
    trace!(secret_id = %secret_id, "Deleting secret");
    let res = delete_secret(&ctx.secret_change_tx, &ctx.repository, secret_id, true).await;
    trace!(success = res.is_ok(), "Deleting secret completed");
    JsonResponse::from(res)
}
