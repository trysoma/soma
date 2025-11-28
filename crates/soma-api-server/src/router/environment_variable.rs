use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    logic::environment_variable::{
        CreateEnvironmentVariableRequest, CreateEnvironmentVariableResponse,
        DeleteEnvironmentVariableResponse, EnvironmentVariable, GetEnvironmentVariableResponse,
        ImportEnvironmentVariableRequest, ListEnvironmentVariablesResponse,
        UpdateEnvironmentVariableRequest, UpdateEnvironmentVariableResponse,
        create_environment_variable, delete_environment_variable, get_environment_variable_by_id,
        get_environment_variable_by_key, import_environment_variable, list_environment_variables,
        update_environment_variable,
    },
    logic::on_change_pubsub::EnvironmentVariableChangeTx,
    repository::Repository,
};
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
};
use tokio::sync::Mutex;
use tonic::transport::Channel;

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "environment-variable";

pub fn create_router() -> OpenApiRouter<Arc<EnvironmentVariableService>> {
    OpenApiRouter::new()
        .routes(routes!(route_create_environment_variable))
        .routes(routes!(route_import_environment_variable))
        .routes(routes!(route_list_environment_variables))
        .routes(routes!(route_get_environment_variable_by_id))
        .routes(routes!(route_get_environment_variable_by_key))
        .routes(routes!(route_update_environment_variable))
        .routes(routes!(route_delete_environment_variable))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateEnvironmentVariableRequest,
    responses(
        (status = 200, description = "Create an environment variable", body = CreateEnvironmentVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create environment variable",
    description = "Create a new environment variable with the specified key and value",
    operation_id = "create-environment-variable",
)]
async fn route_create_environment_variable(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Json(request): Json<CreateEnvironmentVariableRequest>,
) -> JsonResponse<CreateEnvironmentVariableResponse, CommonError> {
    let res = create_environment_variable(
        &ctx.on_change_tx,
        &ctx.repository,
        &ctx.sdk_client,
        request,
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ImportEnvironmentVariableRequest,
    responses(
        (status = 200, description = "Import an environment variable", body = EnvironmentVariable),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Import environment variable",
    description = "Import an existing environment variable into the system",
    operation_id = "import-environment-variable",
)]
async fn route_import_environment_variable(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Json(request): Json<ImportEnvironmentVariableRequest>,
) -> JsonResponse<EnvironmentVariable, CommonError> {
    let res = import_environment_variable(&ctx.repository, request).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List environment variables", body = ListEnvironmentVariablesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List environment variables",
    description = "List all environment variables with pagination",
    operation_id = "list-environment-variables",
)]
async fn route_list_environment_variables(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListEnvironmentVariablesResponse, CommonError> {
    let res = list_environment_variables(&ctx.repository, pagination).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/{{env_var_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("env_var_id" = WrappedUuidV4, Path, description = "Environment variable ID"),
    ),
    responses(
        (status = 200, description = "Get environment variable by id", body = GetEnvironmentVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get environment variable",
    description = "Retrieve an environment variable by its unique identifier",
    operation_id = "get-environment-variable-by-id",
)]
async fn route_get_environment_variable_by_id(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Path(env_var_id): Path<WrappedUuidV4>,
) -> JsonResponse<GetEnvironmentVariableResponse, CommonError> {
    let res = get_environment_variable_by_id(&ctx.repository, env_var_id).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/key/{{key}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("key" = String, Path, description = "Environment variable key"),
    ),
    responses(
        (status = 200, description = "Get environment variable by key", body = GetEnvironmentVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get environment variable by key",
    description = "Retrieve an environment variable by its key name",
    operation_id = "get-environment-variable-by-key",
)]
async fn route_get_environment_variable_by_key(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Path(key): Path<String>,
) -> JsonResponse<GetEnvironmentVariableResponse, CommonError> {
    let res = get_environment_variable_by_key(&ctx.repository, key).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/{{env_var_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("env_var_id" = WrappedUuidV4, Path, description = "Environment variable ID"),
    ),
    request_body = UpdateEnvironmentVariableRequest,
    responses(
        (status = 200, description = "Update environment variable", body = UpdateEnvironmentVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update environment variable",
    description = "Update an existing environment variable's value",
    operation_id = "update-environment-variable",
)]
async fn route_update_environment_variable(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Path(env_var_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateEnvironmentVariableRequest>,
) -> JsonResponse<UpdateEnvironmentVariableResponse, CommonError> {
    let res = update_environment_variable(
        &ctx.on_change_tx,
        &ctx.repository,
        &ctx.sdk_client,
        env_var_id,
        request,
        true,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/{{env_var_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("env_var_id" = WrappedUuidV4, Path, description = "Environment variable ID"),
    ),
    responses(
        (status = 200, description = "Delete environment variable", body = DeleteEnvironmentVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete environment variable",
    description = "Delete an environment variable by its unique identifier",
    operation_id = "delete-environment-variable",
)]
async fn route_delete_environment_variable(
    State(ctx): State<Arc<EnvironmentVariableService>>,
    Path(env_var_id): Path<WrappedUuidV4>,
) -> JsonResponse<DeleteEnvironmentVariableResponse, CommonError> {
    let res = delete_environment_variable(
        &ctx.on_change_tx,
        &ctx.repository,
        &ctx.sdk_client,
        env_var_id,
        true,
    )
    .await;
    JsonResponse::from(res)
}

pub struct EnvironmentVariableService {
    repository: Repository,
    on_change_tx: EnvironmentVariableChangeTx,
    sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
}

impl EnvironmentVariableService {
    pub fn new(
        repository: Repository,
        on_change_tx: EnvironmentVariableChangeTx,
        sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    ) -> Self {
        Self {
            repository,
            on_change_tx,
            sdk_client,
        }
    }
}
