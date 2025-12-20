//! MCP server instance management and protocol routes

use http::HeaderMap;
use shared::identity::Identity;
use tracing::trace;

use super::{API_VERSION_1, BridgeService, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::{
    AddMcpServerInstanceFunctionRequest, AddMcpServerInstanceFunctionResponse,
    CreateMcpServerInstanceRequest, CreateMcpServerInstanceResponse, GetMcpServerInstanceResponse,
    ListMcpServerInstancesParams, ListMcpServerInstancesResponse,
    RemoveMcpServerInstanceFunctionResponse, UpdateMcpServerInstanceFunctionRequest,
    UpdateMcpServerInstanceFunctionResponse, UpdateMcpServerInstanceRequest,
    UpdateMcpServerInstanceResponse, add_mcp_server_instance_function, create_mcp_server_instance,
    delete_mcp_server_instance, get_mcp_server_instance, list_mcp_server_instances,
    remove_mcp_server_instance_function, update_mcp_server_instance,
    update_mcp_server_instance_function,
};
use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::{API_VERSION_TAG, JsonResponse};
use shared::error::CommonError;

// ============================================================================
// MCP Server Instance endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/mcp-server", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateMcpServerInstanceRequest,
    responses(
        (status = 200, description = "Create MCP server instance", body = CreateMcpServerInstanceResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create MCP server instance",
    description = "Create a new MCP server instance with a user-provided ID",
    operation_id = "create-mcp-server-instance",
)]
pub async fn route_create_mcp_server_instance(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Json(request): Json<CreateMcpServerInstanceRequest>,
) -> JsonResponse<CreateMcpServerInstanceResponse, CommonError> {
    trace!(
        instance_id = %request.id,
        name = %request.name,
        "Creating MCP server instance"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let res = create_mcp_server_instance(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.on_config_change_tx(),
        ctx.repository(),
        request,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Creating MCP server instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/mcp-server/{{mcp_server_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
    ),
    responses(
        (status = 200, description = "Get MCP server instance", body = GetMcpServerInstanceResponse),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get MCP server instance",
    description = "Retrieve an MCP server instance by its ID",
    operation_id = "get-mcp-server-instance",
)]
pub async fn route_get_mcp_server_instance(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Path(mcp_server_instance_id): Path<String>,
) -> JsonResponse<GetMcpServerInstanceResponse, CommonError> {
    trace!(instance_id = %mcp_server_instance_id, "Getting MCP server instance");
    let identity_placeholder = Identity::Unauthenticated;
    let res = get_mcp_server_instance(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &mcp_server_instance_id,
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Getting MCP server instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/mcp-server/{{mcp_server_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
    ),
    request_body = UpdateMcpServerInstanceRequest,
    responses(
        (status = 200, description = "Update MCP server instance", body = UpdateMcpServerInstanceResponse),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update MCP server instance",
    description = "Update an MCP server instance name",
    operation_id = "update-mcp-server-instance",
)]
pub async fn route_update_mcp_server_instance(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Path(mcp_server_instance_id): Path<String>,
    Json(request): Json<UpdateMcpServerInstanceRequest>,
) -> JsonResponse<UpdateMcpServerInstanceResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        name = ?request.name,
        "Updating MCP server instance"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let res = update_mcp_server_instance(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.on_config_change_tx(),
        ctx.repository(),
        &mcp_server_instance_id,
        request,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Updating MCP server instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/mcp-server/{{mcp_server_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
    ),
    responses(
        (status = 200, description = "Delete MCP server instance"),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete MCP server instance",
    description = "Delete an MCP server instance and all its function mappings",
    operation_id = "delete-mcp-server-instance",
)]
pub async fn route_delete_mcp_server_instance(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Path(mcp_server_instance_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    trace!(instance_id = %mcp_server_instance_id, "Deleting MCP server instance");
    let identity_placeholder = Identity::Unauthenticated;
    let res = delete_mcp_server_instance(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.on_config_change_tx(),
        ctx.repository(),
        &mcp_server_instance_id,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Deleting MCP server instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/mcp-server", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ListMcpServerInstancesParams
    ),
    responses(
        (status = 200, description = "List MCP server instances", body = ListMcpServerInstancesResponse),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List MCP server instances",
    description = "List all MCP server instances with pagination",
    operation_id = "list-mcp-server-instances",
)]
pub async fn route_list_mcp_server_instances(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Query(params): Query<ListMcpServerInstancesParams>,
) -> JsonResponse<ListMcpServerInstancesResponse, CommonError> {
    trace!(page_size = params.page_size, "Listing MCP server instances");
    let identity_placeholder = Identity::Unauthenticated;
    let res = list_mcp_server_instances(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.repository(),
        params,
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Listing MCP server instances completed"
    );
    JsonResponse::from(res)
}

// ============================================================================
// MCP Server Instance Function endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/mcp-server/{{mcp_server_instance_id}}/function", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
    ),
    request_body = AddMcpServerInstanceFunctionRequest,
    responses(
        (status = 200, description = "Add function to MCP server instance", body = AddMcpServerInstanceFunctionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 409, description = "Conflict (function name already exists)", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Add function to MCP server instance",
    description = "Add a function mapping to an MCP server instance with a custom name",
    operation_id = "add-mcp-server-instance-function",
)]
pub async fn route_add_mcp_server_instance_function(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Path(mcp_server_instance_id): Path<String>,
    Json(request): Json<AddMcpServerInstanceFunctionRequest>,
) -> JsonResponse<AddMcpServerInstanceFunctionResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        function_name = %request.function_name,
        "Adding function to MCP server instance"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let res = add_mcp_server_instance_function(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.on_config_change_tx(),
        ctx.repository(),
        &mcp_server_instance_id,
        request,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Adding function to MCP server instance completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/mcp-server/{{mcp_server_instance_id}}/function/{{function_controller_type_id}}/{{provider_controller_type_id}}/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
        ("function_controller_type_id" = String, Path, description = "Function controller type ID"),
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    request_body = UpdateMcpServerInstanceFunctionRequest,
    responses(
        (status = 200, description = "Update function in MCP server instance", body = UpdateMcpServerInstanceFunctionResponse),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 409, description = "Conflict (function name already exists)", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update function in MCP server instance",
    description = "Update the function name and description for a function mapping",
    operation_id = "update-mcp-server-instance-function",
)]
pub async fn route_update_mcp_server_instance_function(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Path((
        mcp_server_instance_id,
        function_controller_type_id,
        provider_controller_type_id,
        provider_instance_id,
    )): Path<(String, String, String, String)>,
    Json(request): Json<UpdateMcpServerInstanceFunctionRequest>,
) -> JsonResponse<UpdateMcpServerInstanceFunctionResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        function_type = %function_controller_type_id,
        provider_type = %provider_controller_type_id,
        provider_instance_id = %provider_instance_id,
        "Updating MCP server instance function"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let res = update_mcp_server_instance_function(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.on_config_change_tx(),
        ctx.repository(),
        &mcp_server_instance_id,
        &function_controller_type_id,
        &provider_controller_type_id,
        &provider_instance_id,
        request,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Updating MCP server instance function completed"
    );
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/mcp-server/{{mcp_server_instance_id}}/function/{{function_controller_type_id}}/{{provider_controller_type_id}}/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
        ("function_controller_type_id" = String, Path, description = "Function controller type ID"),
        ("provider_controller_type_id" = String, Path, description = "Provider controller type ID"),
        ("provider_instance_id" = String, Path, description = "Provider instance ID"),
    ),
    responses(
        (status = 200, description = "Remove function from MCP server instance", body = RemoveMcpServerInstanceFunctionResponse),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Remove function from MCP server instance",
    description = "Remove a function mapping from an MCP server instance",
    operation_id = "remove-mcp-server-instance-function",
)]
pub async fn route_remove_mcp_server_instance_function(
    State(ctx): State<BridgeService>,
    headers: HeaderMap,
    Path((
        mcp_server_instance_id,
        function_controller_type_id,
        provider_controller_type_id,
        provider_instance_id,
    )): Path<(String, String, String, String)>,
) -> JsonResponse<RemoveMcpServerInstanceFunctionResponse, CommonError> {
    trace!(
        instance_id = %mcp_server_instance_id,
        function_type = %function_controller_type_id,
        provider_type = %provider_controller_type_id,
        provider_instance_id = %provider_instance_id,
        "Removing function from MCP server instance"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let res = remove_mcp_server_instance_function(
        ctx.auth_client().clone(),
        headers,
        identity_placeholder,
        ctx.on_config_change_tx(),
        ctx.repository(),
        &mcp_server_instance_id,
        &function_controller_type_id,
        &provider_controller_type_id,
        &provider_instance_id,
        true, // publish_on_change_evt
    )
    .await;
    trace!(
        success = res.is_ok(),
        "Removing function from MCP server instance completed"
    );
    JsonResponse::from(res)
}
