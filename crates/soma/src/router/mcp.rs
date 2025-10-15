use std::{io, sync::Arc, time::Duration};

use axum::{
    Extension, Json,
    extract::{NestedPath, Path, Query, State},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
};
use http::{HeaderMap, StatusCode, request::Parts};
use rmcp::{
    RoleServer, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolRequestParam, CallToolResult, ClientJsonRpcMessage, ErrorData, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_router,
    transport::{
        common::server_side_http::session_id,
        sse_server::{PostEventQuery, SseServerTransport},
    },
};
use serde::{Deserialize, Serialize};
use shared::{
    adapters::mcp::StructuredResponse,
    primitives::WrappedUuidV4,
};
use utoipa::{PartialSchema, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    logic::{
        ConnectionManager, WithTaskId, create_message, update_task_status,
    },
    mcp,
    repository::Repository,
};

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "mcp";
pub const API_VERSION_1: &str = "v1";

pub fn create_router() -> OpenApiRouter<McpService> {
    OpenApiRouter::new()
        .routes(routes!(mcp_sse))
        .routes(routes!(mcp_message))
}

#[derive(Clone)]
pub struct McpServiceInstanceExt {
    pub task_id: WrappedUuidV4,
}

#[derive(Clone)]
pub struct McpServiceInner {
    mcp_tool_router: ToolRouter<McpService>,
    mcp_sessions: rmcp::transport::sse_server::TxStore,
    mcp_transport_tx:
        tokio::sync::mpsc::UnboundedSender<rmcp::transport::sse_server::SseServerTransport>,
    mcp_sse_ping_interval: Duration,
    repository: Repository,
    connection_manager: ConnectionManager,
}

#[derive(Clone)]
pub struct McpService(pub Arc<McpServiceInner>);

impl McpService {
    pub fn new(
        mcp_transport_tx: tokio::sync::mpsc::UnboundedSender<
            rmcp::transport::sse_server::SseServerTransport,
        >,
        repository: Repository,
        connection_manager: ConnectionManager,
    ) -> Self {
        Self(Arc::new(McpServiceInner {
            mcp_tool_router: Self::tool_router(),
            mcp_sessions: Default::default(),
            mcp_transport_tx,
            mcp_sse_ping_interval: Duration::from_secs(10),
            repository,
            connection_manager,
        }))
    }
}

#[tool_router(vis = "pub")]
impl McpService {
    #[tool(description = "send a message back to the user")]
    async fn send_message(
        &self,
        // TODO: shouldnt have to pass the task id here. It should get inferred via metadata.. perhaps custom HTTP header.
        params: Parameters<mcp::CreateMessageRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let ext_data = match context.extensions.get::<McpServiceInstanceExt>() {
            Some(ext_data) => (*ext_data).clone(),
            None => {
                return Err(ErrorData::internal_error(
                    "MCPServiceInstanceExt not found",
                    None,
                ));
            }
        };

        let res = create_message(
            &self.0.repository,
            &self.0.connection_manager,
            WithTaskId {
                task_id: ext_data.task_id,
                inner: params.0.into(),
            },
            false,
        )
        .await;
        let res = StructuredResponse::new(res);
        res.into()
    }

    #[tool(description = "update the status of the task")]
    async fn update_task_status(
        &self,
        params: Parameters<mcp::UpdateTaskStatusRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let ext_data = match context.extensions.get::<McpServiceInstanceExt>() {
            Some(ext_data) => (*ext_data).clone(),
            None => {
                return Err(ErrorData::internal_error(
                    "MCPServiceInstanceExt not found",
                    None,
                ));
            }
        };

        let res = update_task_status(
            &self.0.repository,
            &self.0.connection_manager,
            WithTaskId {
                task_id: ext_data.task_id,
                inner: params.0.into(),
            },
        )
        .await;
        let res = StructuredResponse::new(res);
        res.into()
    }
}

impl ServerHandler for McpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(concat!("This is the MCP server for soma.").into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tcc: rmcp::handler::server::tool::ToolCallContext<'_, McpService> =
            rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        self.0.mcp_tool_router.call(tcc).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(rmcp::model::ListToolsResult::with_all_items(
            self.0.mcp_tool_router.list_all(),
        ))
    }
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/task/{{task_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    params(
        ("task_id" = WrappedUuidV4, Path, description = "Task ID"),
    ),
    tag = SERVICE_ROUTE_KEY,
    responses(
        (status = 200, description = "MCP server running"),
    ),
    operation_id = "listen-to-mcp-sse",
)]
async fn mcp_sse(
    State(app): State<McpService>,
    nested_path: Option<Extension<NestedPath>>,
    parts: Parts,
    Path(task_id): Path<WrappedUuidV4>,
) -> impl IntoResponse {
    // taken from rmcp sse_handler source code.
    let session = session_id();
    tracing::info!(%session, ?parts, "sse connection");
    use tokio_stream::{StreamExt, wrappers::ReceiverStream};
    use tokio_util::sync::PollSender;
    let (from_client_tx, from_client_rx) = tokio::sync::mpsc::channel(64);
    let (to_client_tx, to_client_rx) = tokio::sync::mpsc::channel(64);
    let to_client_tx_clone = to_client_tx.clone();

    // app.txs
    app.0
        .mcp_sessions
        .write()
        .await
        .insert(session.clone(), from_client_tx);
    let session = session.clone();
    let stream = ReceiverStream::new(from_client_rx);
    let sink = PollSender::new(to_client_tx);
    let transport = SseServerTransport {
        stream,
        sink,
        session_id: session.clone(),
        // tx_store: app.txs.clone(),
        tx_store: app.0.mcp_sessions.clone(),
    };
    let transport_send_result = app.0.mcp_transport_tx.send(transport);
    if transport_send_result.is_err() {
        tracing::warn!("send transport out error");
        let mut response =
            Response::new("fail to send out transport, it seems server is closed".to_string());
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Err(response);
    }
    let nested_path = nested_path.as_deref().map(NestedPath::as_str).unwrap_or("");
    // let post_path = app.post_path.as_ref();
    // let post_path = app.mcp_post_path.clone();
    let post_path = parts.uri.path();
    // let ping_interval = app.sse_ping_interval;
    let ping_interval = app.0.mcp_sse_ping_interval;
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

    tokio::spawn(async move {
        // Wait for connection closure
        to_client_tx_clone.closed().await;

        // Clean up session
        let session_id = session.clone();
        // let tx_store = app.txs.clone();
        let tx_store = app.0.mcp_sessions.clone();
        let mut txs = tx_store.write().await;
        txs.remove(&session_id);
        tracing::debug!(%session_id, "Closed session and cleaned up resources");
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(ping_interval)))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct WrappedClientJsonRpcMessage(ClientJsonRpcMessage);

// TODO: implement ToSchema and PartialSchema
impl ToSchema for WrappedClientJsonRpcMessage {}

impl PartialSchema for WrappedClientJsonRpcMessage {
    // TODO: Implement schema generation for AgentCard
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/task/{{task_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tag = SERVICE_ROUTE_KEY,
    params(
        ("task_id" = WrappedUuidV4, Path, description = "Task ID"),
    ),
    responses(
        (status = 200, description = "MCP server running"),
    ),
    operation_id = "trigger-mcp-message",
)]
async fn mcp_message(
    State(app): State<McpService>,
    Query(PostEventQuery { session_id }): Query<PostEventQuery>,
    parts: Parts,
    Path(task_id): Path<WrappedUuidV4>,
    Json(message): Json<WrappedClientJsonRpcMessage>,
) -> impl IntoResponse {
    let mut message = message.0;
    tracing::debug!(session_id, ?parts, ?message, "new client message");
    let tx = {
        // let rg = app.txs.read().await;
        let rg = app.0.mcp_sessions.read().await;
        rg.get(session_id.as_str())
            .ok_or(StatusCode::NOT_FOUND)?
            .clone()
    };
    message.insert_extension(parts);
    let mcp_server_ext = McpServiceInstanceExt { task_id };
    message.insert_extension(mcp_server_ext);

    if tx.send(message).await.is_err() {
        tracing::error!("send message error");
        return Err(StatusCode::GONE);
    }
    Ok(StatusCode::ACCEPTED)
}
