use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::Json;
use shared::error::CommonError;

use crate::logic::{
    create_tool_alias, delete_tool, delete_tool_alias, get_tool_by_alias, get_tool_by_id,
    list_tool_aliases, list_tools, register_tool, update_tool_alias, CreateToolAliasRequest,
    CreateToolAliasResponse, ListToolAliasesParams, ListToolAliasesResponse, ListToolsParams,
    ListToolsResponse, RegisterToolRequest, RegisterToolResponse, UpdateToolAliasRequest,
};

use super::ToolService;

// ============================================================================
// Route Handlers
// ============================================================================

/// Register a new tool
///
/// Registers a new HTTP-based tool with encrypted invocation key
#[utoipa::path(
    post,
    path = "/api/tool/v1/tool-group-deployment",
    request_body = RegisterToolRequest,
    responses(
        (status = 200, description = "Tool registered successfully", body = RegisterToolResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Register tool",
    description = "Register a new HTTP-based tool with encrypted invocation key",
    operation_id = "register-tool",
    tag = "tool"
)]
pub async fn route_register_tool(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Json(request): Json<RegisterToolRequest>,
) -> Result<Json<RegisterToolResponse>, CommonError> {
    use tracing::trace;

    trace!(
        type_id = %request.type_id,
        deployment_id = %request.deployment_id,
        name = %request.name,
        "Registering tool"
    );

    let res = register_tool(service.auth_client.clone(), headers, &service.repository, &service.encryption_service, request).await;

    trace!(
        success = res.is_ok(),
        "Registering tool completed"
    );

    res.map(Json)
}

/// List registered tools
///
/// Returns a paginated list of registered tools, optionally filtered by endpoint type and category
#[utoipa::path(
    get,
    path = "/api/tool/v1/tool-group-deployment",
    params(ListToolsParams),
    responses(
        (status = 200, description = "List of tools"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    summary = "List tools",
    description = "Returns a paginated list of registered tools, optionally filtered by endpoint type and category",
    operation_id = "list-tools",
    tag = "tool"
)]
pub async fn route_list_tools(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Query(params): Query<ListToolsParams>,
) -> Result<Json<ListToolsResponse>, CommonError> {
    use tracing::trace;

    trace!("Listing tools");

    let res = list_tools(service.auth_client.clone(), headers, &service.repository, params).await;

    trace!(success = res.is_ok(), "Listing tools completed");

    res.map(Json)
}

/// Get a specific tool by ID
///
/// Returns tool details including encrypted endpoint configuration
#[utoipa::path(
    get,
    path = "/api/tool/v1/tool-group-deployment/type/{type_id}/deployment/{deployment_id}",
    params(
        ("type_id" = String, Path, description = "Tool type identifier"),
        ("deployment_id" = String, Path, description = "Tool deployment identifier")
    ),
    responses(
        (status = 200, description = "Tool details", body = crate::logic::ToolGroupDeploymentSerialized),
        (status = 404, description = "Tool not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get tool",
    description = "Returns tool details including encrypted endpoint configuration",
    operation_id = "get-tool",
    tag = "tool"
)]
pub async fn route_get_tool(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Path((type_id, deployment_id)): Path<(String, String)>,
) -> Result<Json<crate::logic::ToolSerialized>, CommonError> {
    use tracing::trace;

    trace!(
        type_id = %type_id,
        deployment_id = %deployment_id,
        "Getting tool"
    );

    let res = get_tool_by_id(service.auth_client.clone(), headers, &service.repository, type_id, deployment_id).await;

    trace!(success = res.is_ok(), "Getting tool completed");

    res.map(Json)
}

/// Deregister a tool
///
/// Removes a tool registration and all its aliases
#[utoipa::path(
    delete,
    path = "/api/tool/v1/tool-group-deployment/type/{type_id}/deployment/{deployment_id}",
    params(
        ("type_id" = String, Path, description = "Tool type identifier"),
        ("deployment_id" = String, Path, description = "Tool deployment identifier")
    ),
    responses(
        (status = 204, description = "Tool deregistered successfully"),
        (status = 404, description = "Tool not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Deregister tool",
    description = "Removes a tool registration and all its aliases",
    operation_id = "deregister-tool",
    tag = "tool"
)]
pub async fn route_deregister_tool(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Path((type_id, deployment_id)): Path<(String, String)>,
) -> Result<StatusCode, CommonError> {
    use tracing::trace;

    trace!(
        type_id = %type_id,
        deployment_id = %deployment_id,
        "Deregistering tool"
    );

    let res = delete_tool(service.auth_client.clone(), headers, &service.repository, type_id, deployment_id).await;

    trace!(success = res.is_ok(), "Deregistering tool completed");

    res.map(|_| StatusCode::NO_CONTENT)
}

/// Create a tool alias
///
/// Creates an alias that points to a specific tool deployment
#[utoipa::path(
    post,
    path = "/api/tool/v1/tool-group-deployment-alias",
    request_body = CreateToolAliasRequest,
    responses(
        (status = 200, description = "Alias created successfully", body = CreateToolAliasResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Alias already exists"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Create tool alias",
    description = "Creates an alias that points to a specific tool deployment",
    operation_id = "create-tool-alias",
    tag = "tool"
)]
pub async fn route_create_tool_alias(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Json(request): Json<CreateToolAliasRequest>,
) -> Result<Json<CreateToolAliasResponse>, CommonError> {
    use tracing::trace;

    trace!(
        tool_type_id = %request.tool_type_id,
        tool_deployment_id = %request.tool_deployment_id,
        alias = %request.alias,
        "Creating tool alias"
    );

    let res = create_tool_alias(service.auth_client.clone(), headers, &service.repository, request).await;

    trace!(success = res.is_ok(), "Creating tool alias completed");

    res.map(Json)
}

/// List tool aliases
///
/// Returns a paginated list of tool aliases, optionally filtered by tool
#[utoipa::path(
    get,
    path = "/api/tool/v1/tool-group-deployment-alias",
    params(ListToolAliasesParams),
    responses(
        (status = 200, description = "List of aliases"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    summary = "List tool aliases",
    description = "Returns a paginated list of tool aliases, optionally filtered by tool",
    operation_id = "list-tool-aliases",
    tag = "tool"
)]
pub async fn route_list_tool_aliases(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Query(params): Query<ListToolAliasesParams>,
) -> Result<Json<ListToolAliasesResponse>, CommonError> {
    use tracing::trace;

    trace!("Listing tool aliases");

    let res = list_tool_aliases(service.auth_client.clone(), headers, &service.repository, params).await;

    trace!(success = res.is_ok(), "Listing tool aliases completed");

    res.map(Json)
}

/// Get tool by alias
///
/// Resolves an alias and returns the tool it points to
#[utoipa::path(
    get,
    path = "/api/tool/v1/tool-group-deployment-alias/{alias}",
    params(
        ("alias" = String, Path, description = "Tool alias (e.g., 'latest', 'stable')")
    ),
    responses(
        (status = 200, description = "Tool details", body = crate::logic::ToolGroupDeploymentSerialized),
        (status = 404, description = "Alias not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get tool by alias",
    description = "Resolves an alias and returns the tool it points to",
    operation_id = "get-tool-by-alias",
    tag = "tool"
)]
pub async fn route_get_tool_by_alias(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Path(alias): Path<String>,
) -> Result<Json<crate::logic::ToolSerialized>, CommonError> {
    use tracing::trace;

    trace!(alias = %alias, "Getting tool by alias");

    let res = get_tool_by_alias(service.auth_client.clone(), headers, &service.repository, alias).await;

    trace!(success = res.is_ok(), "Getting tool by alias completed");

    res.map(Json)
}

/// Update tool alias
///
/// Updates an alias to point to a different deployment
#[utoipa::path(
    put,
    path = "/api/tool/v1/tool-group-deployment-alias/{tool_type_id}/{alias}",
    params(
        ("tool_type_id" = String, Path, description = "Tool type identifier"),
        ("alias" = String, Path, description = "Alias to update")
    ),
    request_body = UpdateToolAliasRequest,
    responses(
        (status = 204, description = "Alias updated successfully"),
        (status = 404, description = "Alias not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Update tool alias",
    description = "Updates an alias to point to a different deployment",
    operation_id = "update-tool-alias",
    tag = "tool"
)]
pub async fn route_update_tool_alias(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Path((tool_type_id, alias)): Path<(String, String)>,
    Json(request): Json<UpdateToolAliasRequest>,
) -> Result<StatusCode, CommonError> {
    use tracing::trace;

    trace!(
        tool_type_id = %tool_type_id,
        alias = %alias,
        new_deployment_id = %request.tool_deployment_id,
        "Updating tool alias"
    );

    let res = update_tool_alias(
        service.auth_client.clone(),
        headers,
        &service.repository,
        tool_type_id,
        alias,
        request.tool_deployment_id,
    )
    .await;

    trace!(success = res.is_ok(), "Updating tool alias completed");

    res.map(|_| StatusCode::NO_CONTENT)
}

/// Delete tool alias
///
/// Removes an alias (does not delete the tool itself)
#[utoipa::path(
    delete,
    path = "/api/tool/v1/tool-group-deployment-alias/{alias}",
    params(
        ("alias" = String, Path, description = "Alias to delete")
    ),
    responses(
        (status = 204, description = "Alias deleted successfully"),
        (status = 404, description = "Alias not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Delete tool alias",
    description = "Removes an alias (does not delete the tool itself)",
    operation_id = "delete-tool-alias",
    tag = "tool"
)]
pub async fn route_delete_tool_alias(
    Extension(service): Extension<ToolService>,
    headers: http::HeaderMap,
    Path(alias): Path<String>,
) -> Result<StatusCode, CommonError> {
    use tracing::trace;

    trace!(alias = %alias, "Deleting tool alias");

    let res = delete_tool_alias(service.auth_client.clone(), headers, &service.repository, alias).await;

    trace!(success = res.is_ok(), "Deleting tool alias completed");

    res.map(|_| StatusCode::NO_CONTENT)
}
