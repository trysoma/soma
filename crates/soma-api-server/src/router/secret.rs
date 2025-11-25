use axum::extract::{Json, Path, Query, State};
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    logic::on_change_pubsub::SecretChangeTx,
    logic::secret::{
        create_secret, delete_secret, get_secret_by_id, get_secret_by_key, import_secret,
        list_secrets, update_secret, CreateSecretRequest, CreateSecretResponse,
        DeleteSecretResponse, GetSecretResponse, ImportSecretRequest, ListSecretsResponse,
        Secret, UpdateSecretRequest, UpdateSecretResponse,
    },
    repository::Repository,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "secret";

pub fn create_router() -> OpenApiRouter<Arc<SecretService>> {
    OpenApiRouter::new()
        .routes(routes!(route_create_secret))
        .routes(routes!(route_import_secret))
        .routes(routes!(route_list_secrets))
        .routes(routes!(route_get_secret_by_id))
        .routes(routes!(route_get_secret_by_key))
        .routes(routes!(route_update_secret))
        .routes(routes!(route_delete_secret))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = CreateSecretRequest,
    responses(
        (status = 200, description = "Create a secret", body = CreateSecretResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "create-secret",
)]
async fn route_create_secret(
    State(ctx): State<Arc<SecretService>>,
    Json(request): Json<CreateSecretRequest>,
) -> JsonResponse<CreateSecretResponse, CommonError> {
    let res = create_secret(
        &ctx.on_change_tx,
        &ctx.repository,
        ctx.encryption_service.cache(),
        request,
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    request_body = ImportSecretRequest,
    responses(
        (status = 200, description = "Import a pre-encrypted secret", body = Secret),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    operation_id = "import-secret",
)]
async fn route_import_secret(
    State(ctx): State<Arc<SecretService>>,
    Json(request): Json<ImportSecretRequest>,
) -> JsonResponse<Secret, CommonError> {
    let res = import_secret(&ctx.repository, request).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "list-secrets",
)]
async fn route_list_secrets(
    State(ctx): State<Arc<SecretService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListSecretsResponse, CommonError> {
    let res = list_secrets(&ctx.repository, pagination).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/{{secret_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "get-secret-by-id",
)]
async fn route_get_secret_by_id(
    State(ctx): State<Arc<SecretService>>,
    Path(secret_id): Path<WrappedUuidV4>,
) -> JsonResponse<GetSecretResponse, CommonError> {
    let res = get_secret_by_id(&ctx.repository, secret_id).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/key/{{key}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "get-secret-by-key",
)]
async fn route_get_secret_by_key(
    State(ctx): State<Arc<SecretService>>,
    Path(key): Path<String>,
) -> JsonResponse<GetSecretResponse, CommonError> {
    let res = get_secret_by_key(&ctx.repository, key).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/{{secret_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "update-secret",
)]
async fn route_update_secret(
    State(ctx): State<Arc<SecretService>>,
    Path(secret_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateSecretRequest>,
) -> JsonResponse<UpdateSecretResponse, CommonError> {
    let res = update_secret(
        &ctx.on_change_tx,
        &ctx.repository,
        ctx.encryption_service.cache(),
        secret_id,
        request,
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/{{secret_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    operation_id = "delete-secret",
)]
async fn route_delete_secret(
    State(ctx): State<Arc<SecretService>>,
    Path(secret_id): Path<WrappedUuidV4>,
) -> JsonResponse<DeleteSecretResponse, CommonError> {
    let res = delete_secret(&ctx.on_change_tx, &ctx.repository, secret_id, true).await;
    JsonResponse::from(res)
}

pub struct SecretService {
    repository: Repository,
    encryption_service: encryption::router::EncryptionService,
    on_change_tx: SecretChangeTx,
}

impl SecretService {
    pub fn new(
        repository: Repository,
        encryption_service: encryption::router::EncryptionService,
        on_change_tx: SecretChangeTx,
    ) -> Self {
        Self {
            repository,
            encryption_service,
            on_change_tx,
        }
    }
}
