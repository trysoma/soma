//! Variable HTTP endpoints

use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    logic::variable::{
        CreateVariableRequest, CreateVariableResponse, DeleteVariableResponse, GetVariableResponse,
        ImportVariableRequest, ListVariablesResponse, UpdateVariableRequest,
        UpdateVariableResponse, Variable, create_variable, delete_variable, get_variable_by_id,
        get_variable_by_key, import_variable, list_variables, update_variable,
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

/// Create the variable router
pub fn create_router() -> OpenApiRouter<Arc<EnvironmentService>> {
    OpenApiRouter::new()
        .routes(routes!(route_create_variable))
        .routes(routes!(route_import_variable))
        .routes(routes!(route_list_variables))
        .routes(routes!(route_get_variable_by_id))
        .routes(routes!(route_get_variable_by_key))
        .routes(routes!(route_update_variable))
        .routes(routes!(route_delete_variable))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/variable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateVariableRequest,
    responses(
        (status = 200, description = "Create a variable", body = CreateVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create variable",
    description = "Create a new environment variable with the specified key and value",
    operation_id = "create-variable",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_create_variable(
    State(ctx): State<Arc<EnvironmentService>>,
    Json(request): Json<CreateVariableRequest>,
) -> JsonResponse<CreateVariableResponse, CommonError> {
    trace!(key = %request.key, "Creating variable");
    let res = create_variable(&ctx.variable_change_tx, &ctx.repository, request, true).await;
    trace!(success = res.is_ok(), "Creating variable completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/variable/import", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ImportVariableRequest,
    responses(
        (status = 200, description = "Import a variable", body = Variable),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Import variable",
    description = "Import an existing environment variable into the system",
    operation_id = "import-variable",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_import_variable(
    State(ctx): State<Arc<EnvironmentService>>,
    Json(request): Json<ImportVariableRequest>,
) -> JsonResponse<Variable, CommonError> {
    trace!(env_var_key = %request.key, "Importing variable");
    let res = import_variable(&ctx.repository, request).await;
    trace!(success = res.is_ok(), "Importing variable completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/variable", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List variables", body = ListVariablesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List variables",
    description = "List all environment variables with pagination",
    operation_id = "list-variables",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_variables(
    State(ctx): State<Arc<EnvironmentService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListVariablesResponse, CommonError> {
    trace!(page_size = pagination.page_size, "Listing variables");
    let res = list_variables(&ctx.repository, pagination).await;
    trace!(success = res.is_ok(), "Listing variables completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/variable/{{variable_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("variable_id" = WrappedUuidV4, Path, description = "Variable ID"),
    ),
    responses(
        (status = 200, description = "Get variable by id", body = GetVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get variable",
    description = "Retrieve an environment variable by its unique identifier",
    operation_id = "get-variable-by-id",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_variable_by_id(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(variable_id): Path<WrappedUuidV4>,
) -> JsonResponse<GetVariableResponse, CommonError> {
    trace!(variable_id = %variable_id, "Getting variable by ID");
    let res = get_variable_by_id(&ctx.repository, variable_id).await;
    trace!(success = res.is_ok(), "Getting variable by ID completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/variable/key/{{key}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("key" = String, Path, description = "Variable key"),
    ),
    responses(
        (status = 200, description = "Get variable by key", body = GetVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get variable by key",
    description = "Retrieve an environment variable by its key name",
    operation_id = "get-variable-by-key",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_variable_by_key(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(key): Path<String>,
) -> JsonResponse<GetVariableResponse, CommonError> {
    trace!(key = %key, "Getting variable by key");
    let res = get_variable_by_key(&ctx.repository, key).await;
    trace!(success = res.is_ok(), "Getting variable by key completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/variable/{{variable_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("variable_id" = WrappedUuidV4, Path, description = "Variable ID"),
    ),
    request_body = UpdateVariableRequest,
    responses(
        (status = 200, description = "Update variable", body = UpdateVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update variable",
    description = "Update an existing environment variable's value",
    operation_id = "update-variable",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_update_variable(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(variable_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateVariableRequest>,
) -> JsonResponse<UpdateVariableResponse, CommonError> {
    trace!(variable_id = %variable_id, "Updating variable");
    let res = update_variable(
        &ctx.variable_change_tx,
        &ctx.repository,
        variable_id,
        request,
        true,
    )
    .await;
    trace!(success = res.is_ok(), "Updating variable completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/variable/{{variable_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("variable_id" = WrappedUuidV4, Path, description = "Variable ID"),
    ),
    responses(
        (status = 200, description = "Delete variable", body = DeleteVariableResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete variable",
    description = "Delete an environment variable by its unique identifier",
    operation_id = "delete-variable",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_delete_variable(
    State(ctx): State<Arc<EnvironmentService>>,
    Path(variable_id): Path<WrappedUuidV4>,
) -> JsonResponse<DeleteVariableResponse, CommonError> {
    trace!(variable_id = %variable_id, "Deleting variable");
    let res = delete_variable(&ctx.variable_change_tx, &ctx.repository, variable_id, true).await;
    trace!(success = res.is_ok(), "Deleting variable completed");
    JsonResponse::from(res)
}
