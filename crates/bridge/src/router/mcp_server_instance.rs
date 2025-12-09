//! MCP server instance management and protocol routes

use super::{API_VERSION_1, BridgeService, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::{
    AddMcpServerInstanceFunctionRequest, AddMcpServerInstanceFunctionResponse,
    CreateMcpServerInstanceRequest, CreateMcpServerInstanceResponse, GetMcpServerInstanceResponse,
    ListMcpServerInstancesParams, ListMcpServerInstancesResponse, McpServiceInstanceExt,
    RemoveMcpServerInstanceFunctionResponse, UpdateMcpServerInstanceFunctionRequest,
    UpdateMcpServerInstanceFunctionResponse, UpdateMcpServerInstanceRequest,
    UpdateMcpServerInstanceResponse, add_mcp_server_instance_function, create_mcp_server_instance,
    delete_mcp_server_instance, get_mcp_server_instance, list_mcp_server_instances,
    remove_mcp_server_instance_function, update_mcp_server_instance,
    update_mcp_server_instance_function,
};
use axum::Extension;
use axum::extract::{Json, NestedPath, Path, Query, State};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Response, Sse};
use http::StatusCode;
use http::request::Parts;
use rmcp::model::ClientJsonRpcMessage;
use rmcp::transport::common::server_side_http::session_id;
use rmcp::transport::sse_server::PostEventQuery;
use serde::{Deserialize, Serialize};
use shared::adapters::openapi::{API_VERSION_TAG, JsonResponse};
use shared::error::CommonError;
use std::io;
use utoipa::{PartialSchema, ToSchema};

// ============================================================================
// MCP Server Instance endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/mcp-instance", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Json(request): Json<CreateMcpServerInstanceRequest>,
) -> JsonResponse<CreateMcpServerInstanceResponse, CommonError> {
    let res = create_mcp_server_instance(ctx.repository(), request).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Path(mcp_server_instance_id): Path<String>,
) -> JsonResponse<GetMcpServerInstanceResponse, CommonError> {
    let res = get_mcp_server_instance(ctx.repository(), &mcp_server_instance_id).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Path(mcp_server_instance_id): Path<String>,
    Json(request): Json<UpdateMcpServerInstanceRequest>,
) -> JsonResponse<UpdateMcpServerInstanceResponse, CommonError> {
    let res = update_mcp_server_instance(ctx.repository(), &mcp_server_instance_id, request).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Path(mcp_server_instance_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    let res = delete_mcp_server_instance(ctx.repository(), &mcp_server_instance_id).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/mcp-instance", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Query(params): Query<ListMcpServerInstancesParams>,
) -> JsonResponse<ListMcpServerInstancesResponse, CommonError> {
    let res = list_mcp_server_instances(ctx.repository(), params).await;
    JsonResponse::from(res)
}

// ============================================================================
// MCP Server Instance Function endpoints
// ============================================================================

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}/function", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Path(mcp_server_instance_id): Path<String>,
    Json(request): Json<AddMcpServerInstanceFunctionRequest>,
) -> JsonResponse<AddMcpServerInstanceFunctionResponse, CommonError> {
    let res =
        add_mcp_server_instance_function(ctx.repository(), &mcp_server_instance_id, request).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}/function/{{function_controller_type_id}}/{{provider_controller_type_id}}/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Path((
        mcp_server_instance_id,
        function_controller_type_id,
        provider_controller_type_id,
        provider_instance_id,
    )): Path<(String, String, String, String)>,
    Json(request): Json<UpdateMcpServerInstanceFunctionRequest>,
) -> JsonResponse<UpdateMcpServerInstanceFunctionResponse, CommonError> {
    let res = update_mcp_server_instance_function(
        ctx.repository(),
        &mcp_server_instance_id,
        &function_controller_type_id,
        &provider_controller_type_id,
        &provider_instance_id,
        request,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}/function/{{function_controller_type_id}}/{{provider_controller_type_id}}/{{provider_instance_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
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
    Path((
        mcp_server_instance_id,
        function_controller_type_id,
        provider_controller_type_id,
        provider_instance_id,
    )): Path<(String, String, String, String)>,
) -> JsonResponse<RemoveMcpServerInstanceFunctionResponse, CommonError> {
    let res = remove_mcp_server_instance_function(
        ctx.repository(),
        &mcp_server_instance_id,
        &function_controller_type_id,
        &provider_controller_type_id,
        &provider_instance_id,
    )
    .await;
    JsonResponse::from(res)
}

// ============================================================================
// MCP Protocol (SSE/Message) endpoints
// ============================================================================

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct WrappedClientJsonRpcMessage(ClientJsonRpcMessage);

impl ToSchema for WrappedClientJsonRpcMessage {}

impl PartialSchema for WrappedClientJsonRpcMessage {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}/mcp", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
    ),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    responses(
        (status = 200, description = "MCP server running"),
    ),
    summary = "MCP SSE connection",
    description = "Establish Server-Sent Events (SSE) connection for MCP protocol communication",
    operation_id = "listen-to-mcp-sse",
)]
pub async fn mcp_sse(
    State(ctx): State<BridgeService>,
    Path(_mcp_server_instance_id): Path<String>,
    nested_path: Option<Extension<NestedPath>>,
    parts: Parts,
) -> impl IntoResponse {
    // Note: mcp_server_instance_id is not used in the SSE handler directly.
    // The mcp_server_instance_id is injected into extensions via the mcp_message handler.
    // taken from rmcp sse_handler source code.
    let session = session_id();
    tracing::info!(%session, ?parts, "sse connection");
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::sync::PollSender;
    let (from_client_tx, from_client_rx) = tokio::sync::mpsc::channel(64);
    let (to_client_tx, to_client_rx) = tokio::sync::mpsc::channel(64);
    let to_client_tx_clone = to_client_tx.clone();

    ctx.mcp_sessions()
        .write()
        .await
        .insert(session.clone(), from_client_tx);
    let session = session.clone();
    let stream = ReceiverStream::new(from_client_rx);
    let sink = PollSender::new(to_client_tx);
    let transport = rmcp::transport::sse_server::SseServerTransport {
        stream,
        sink,
        session_id: session.clone(),
        tx_store: ctx.mcp_sessions().clone(),
    };
    let transport_send_result = ctx.mcp_transport_tx().send(transport);
    if transport_send_result.is_err() {
        tracing::warn!("send transport out error");
        let mut response =
            Response::new("fail to send out transport, it seems server is closed".to_string());
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Err(response);
    }
    let nested_path = nested_path.as_deref().map(NestedPath::as_str).unwrap_or("");
    let post_path = parts.uri.path();
    let ping_interval = ctx.mcp_sse_ping_interval();
    let stream = futures::stream::once(futures::future::ok(
        Event::default()
            .event("endpoint")
            .data(format!("{nested_path}{post_path}?sessionId={session}")),
    ))
    .chain(ReceiverStream::new(to_client_rx).map(|message| {
        match serde_json::to_string(&message) {
            Ok(bytes) => Ok(Event::default().event("message").data(&bytes)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }));
    let tx_store = ctx.mcp_sessions().clone();
    tokio::spawn(async move {
        // Wait for connection closure
        to_client_tx_clone.closed().await;

        // Clean up session
        let session_id = session.clone();
        let mut txs = tx_store.write().await;
        txs.remove(&session_id);
        tracing::debug!(%session_id, "Closed session and cleaned up resources");
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(*ping_interval)))
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/mcp-instance/{{mcp_server_instance_id}}/mcp", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("mcp_server_instance_id" = String, Path, description = "MCP server instance ID"),
    ),
    responses(
        (status = 200, description = "MCP server running"),
    ),
    summary = "Send MCP message",
    description = "Send a JSON-RPC message to the MCP server",
    operation_id = "trigger-mcp-message",
)]
pub async fn mcp_message(
    State(ctx): State<BridgeService>,
    Path(mcp_server_instance_id): Path<String>,
    Query(PostEventQuery { session_id }): Query<PostEventQuery>,
    parts: Parts,
    Json(message): Json<WrappedClientJsonRpcMessage>,
) -> impl IntoResponse {
    let mut message = message.0;
    tracing::debug!(session_id, ?parts, ?message, "new client message");
    let tx = {
        let rg = ctx.mcp_sessions().read().await;
        rg.get(session_id.as_str())
            .ok_or(StatusCode::NOT_FOUND)?
            .clone()
    };

    // Inject the MCP server instance ID into extensions so ServerHandler can access it
    message.insert_extension(McpServiceInstanceExt {
        mcp_server_instance_id,
    });
    message.insert_extension(parts);

    if tx.send(message).await.is_err() {
        tracing::error!("send message error");
        return Err(StatusCode::GONE);
    }
    Ok(StatusCode::ACCEPTED)
}
