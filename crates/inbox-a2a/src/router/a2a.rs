//! Agent routes for the A2A protocol
//!
//! Provides endpoints for:
//! - Agent card discovery (/.well-known/agent.json)
//! - A2A JSON-RPC handling (tasks/get, tasks/pushNotificationConfig/set, tasks/pushNotificationConfig/get)

use crate::a2a_core::errors::{A2aError, A2aServerError};
use crate::a2a_core::events::EventConsumer;
use crate::a2a_core::types::{
    CustomJsonRpcPayload, CustomJsonrpcResponse, JsonrpcRequest, JsonrpcRequestId,
    PushNotificationConfig, SendStreamingMessageSuccessResponseResult, TaskIdParams,
    TaskPushNotificationConfig, TaskQueryParams,
};
use crate::logic::push_notification;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::response::sse::{Event as SseEvent, Sse};
use serde::Deserialize;
use shared::adapters::openapi::API_VERSION_TAG;
use shared::error::CommonError;
use shared::primitives::WrappedUuidV4;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::trace;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::logic::agent::{construct_agent_card, ConstructAgentCardParams};
use crate::logic::task as task_logic;
use crate::A2aService;

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "agent";

/// Path parameters for multi-agent routes
#[derive(Debug, Clone, Deserialize)]
pub struct AgentPathParams {
    pub project_id: String,
    pub agent_id: String,
}

/// Creates the agent router with agent card and JSON-RPC endpoints
pub fn create_agent_router() -> OpenApiRouter<Arc<A2aService>> {
    OpenApiRouter::new()
        .routes(routes!(route_agent_card))
        .routes(routes!(route_a2a_jsonrpc))
}

/// GET /api/agent/{project_id}/{agent_id}/a2a/.well-known/agent.json - Get A2A agent card
#[utoipa::path(
    get,
    path = format!("{}/{}/{{project_id}}/{{agent_id}}/a2a/.well-known/agent.json", PATH_PREFIX, SERVICE_ROUTE_KEY),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("project_id" = String, Path, description = "Project ID"),
        ("agent_id" = String, Path, description = "Agent ID"),
    ),
    responses(
        (status = 200, description = "Agent card", body = crate::a2a_core::types::AgentCard),
    ),
    summary = "Get agent card for specific agent",
    description = "Get the agent card describing agent capabilities and metadata for a specific agent",
    operation_id = "get-agent-card",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_agent_card(
    State(ctx): State<Arc<A2aService>>,
    Path(path_params): Path<AgentPathParams>,
) -> impl IntoResponse {
    trace!(
        project_id = %path_params.project_id,
        agent_id = %path_params.agent_id,
        "Getting agent card"
    );

    let result = get_agent_card(&ctx, &path_params).await;
    trace!(success = result.is_ok(), "Getting agent card completed");
    match result {
        Ok(card) => (http::StatusCode::OK, Json(card)).into_response(),
        Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// POST /api/agent/{project_id}/{agent_id}/a2a - Handle A2A JSON-RPC requests
#[utoipa::path(
    post,
    path = format!("{}/{}/{{project_id}}/{{agent_id}}/a2a", PATH_PREFIX, SERVICE_ROUTE_KEY),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("project_id" = String, Path, description = "Project ID"),
        ("agent_id" = String, Path, description = "Agent ID"),
    ),
    responses(
        (status = 200, description = "Successful response"),
        (status = 500, description = "Internal Server Error"),
    ),
    summary = "Handle A2A JSON-RPC for specific agent",
    description = "Handle JSON-RPC requests for agent-to-agent communication for a specific agent",
    operation_id = "handle-a2a-jsonrpc-request",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_a2a_jsonrpc(
    State(ctx): State<Arc<A2aService>>,
    Path(path_params): Path<AgentPathParams>,
    Json(body): Json<JsonrpcRequest>,
) -> impl IntoResponse {
    trace!(
        project_id = %path_params.project_id,
        agent_id = %path_params.agent_id,
        method = %body.method,
        "Handling A2A JSON-RPC request"
    );

    let id = body.id.clone();

    match body.method.as_str() {
        "tasks/get" => handle_tasks_get(&ctx, body, &id).await,
        "tasks/resubscribe" => handle_tasks_resubscribe(&ctx, body, &id).await,
        "tasks/pushNotificationConfig/set" => {
            handle_push_notification_config_set(&ctx, body, &id).await
        }
        "tasks/pushNotificationConfig/get" => {
            handle_push_notification_config_get(&ctx, body, &id).await
        }
        _ => (http::StatusCode::NOT_FOUND, "Unknown method").into_response(),
    }
}

/// Handle tasks/get JSON-RPC method
async fn handle_tasks_get(
    ctx: &A2aService,
    body: JsonrpcRequest,
    id: &Option<JsonrpcRequestId>,
) -> axum::response::Response {
    let params: TaskQueryParams =
        match serde_json::from_value(serde_json::Value::Object(body.params)) {
            Ok(p) => p,
            Err(e) => return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        };

    // Validate and parse task ID
    let task_id = match WrappedUuidV4::try_from(params.id.clone()) {
        Ok(tid) => tid,
        Err(_) => return (http::StatusCode::BAD_REQUEST, "Invalid task ID format").into_response(),
    };

    // Use the logic function to get the task and convert to A2A format
    match task_logic::get_task(ctx.repository(), task_id).await {
        Ok(task) => {
            let a2a_task: crate::a2a_core::types::Task = task.into();
            let response = CustomJsonrpcResponse::new(
                id.clone(),
                CustomJsonRpcPayload::Ok(serde_json::to_value(a2a_task).unwrap()),
            );
            (http::StatusCode::OK, Json(response)).into_response()
        }
        Err(shared::error::CommonError::NotFound { .. }) => {
            let error = A2aServerError::TaskNotFoundError(A2aError::new("Task not found"));
            (http::StatusCode::NOT_FOUND, error.to_string()).into_response()
        }
        Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Handle tasks/resubscribe JSON-RPC method (SSE streaming for existing task)
async fn handle_tasks_resubscribe(
    ctx: &A2aService,
    body: JsonrpcRequest,
    id: &Option<JsonrpcRequestId>,
) -> axum::response::Response {
    trace!("Processing tasks/resubscribe request");

    let params: TaskIdParams = match serde_json::from_value(serde_json::Value::Object(body.params))
    {
        Ok(p) => p,
        Err(e) => return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    // Try to tap into existing queue
    let event_queue = match ctx.queue_manager().tap(&params.id).await {
        Some(queue) => queue,
        None => {
            let error =
                A2aServerError::TaskNotFoundError(A2aError::new("No active stream for task"));
            return (http::StatusCode::NOT_FOUND, error.to_string()).into_response();
        }
    };

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let id_for_task = id.clone();

    tokio::spawn(async move {
        let consumer = EventConsumer::new(event_queue);
        let mut stream = std::pin::pin!(consumer.consume_all().await);

        while let Some(item) = futures::StreamExt::next(&mut stream).await {
            if tx.send(item).is_err() {
                break;
            }
        }
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    let stream = TokioStreamExt::map(stream, move |item| {
        let data: CustomJsonRpcPayload<SendStreamingMessageSuccessResponseResult> = item.into();
        let res = CustomJsonrpcResponse::new(id_for_task.clone(), data);
        SseEvent::default().json_data(res)
    });

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive"),
        )
        .into_response()
}

/// Handle tasks/pushNotificationConfig/set JSON-RPC method
async fn handle_push_notification_config_set(
    ctx: &A2aService,
    body: JsonrpcRequest,
    id: &Option<JsonrpcRequestId>,
) -> axum::response::Response {
    let params: TaskPushNotificationConfig =
        match serde_json::from_value(serde_json::Value::Object(body.params)) {
            Ok(p) => p,
            Err(e) => return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        };

    let task_id = match WrappedUuidV4::try_from(params.task_id.clone()) {
        Ok(id) => id,
        Err(e) => return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    match push_notification::set_push_notification_config(
        ctx.repository(),
        &task_id,
        &params.push_notification_config,
    )
    .await
    {
        Ok(config) => {
            let response = TaskPushNotificationConfig {
                task_id: params.task_id,
                push_notification_config: config,
            };
            let response = CustomJsonrpcResponse::new(
                id.clone(),
                CustomJsonRpcPayload::Ok(serde_json::to_value(response).unwrap()),
            );
            (http::StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Handle tasks/pushNotificationConfig/get JSON-RPC method
async fn handle_push_notification_config_get(
    ctx: &A2aService,
    body: JsonrpcRequest,
    id: &Option<JsonrpcRequestId>,
) -> axum::response::Response {
    let params: TaskIdParams = match serde_json::from_value(serde_json::Value::Object(body.params))
    {
        Ok(p) => p,
        Err(e) => return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    let task_id = match WrappedUuidV4::try_from(params.id.clone()) {
        Ok(tid) => tid,
        Err(e) => return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    match push_notification::get_push_notification_configs(ctx.repository(), &task_id).await {
        Ok(configs) => {
            // Return the first config if exists, or a default
            let config = configs.into_iter().next().unwrap_or(PushNotificationConfig {
                id: None,
                url: String::new(),
                token: None,
                authentication: None,
            });
            let response = TaskPushNotificationConfig {
                task_id: params.id,
                push_notification_config: config,
            };
            let response = CustomJsonrpcResponse::new(
                id.clone(),
                CustomJsonRpcPayload::Ok(serde_json::to_value(response).unwrap()),
            );
            (http::StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Parameters for creating the A2aService with agent capabilities
pub struct A2aRouterServiceParams {
    pub soma_definition: Arc<dyn shared::soma_agent_definition::SomaAgentDefinitionLike>,
    pub host: url::Url,
    pub connection_manager: crate::ConnectionManager,
    pub repository: crate::Repository,
    /// Optional event bus for inbox integration
    pub event_bus: Option<inbox::logic::event::EventBus>,
}

/// Get agent card for a specific project/agent
async fn get_agent_card(
    ctx: &A2aService,
    path_params: &AgentPathParams,
) -> Result<crate::a2a_core::types::AgentCard, CommonError> {
    let soma_definition = ctx
        .soma_definition()
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "A2aService not configured with agent capabilities"
            ))
        })?
        .get_definition()
        .await?;

    let host = ctx.host().ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!(
            "A2aService not configured with agent capabilities"
        ))
    })?;

    let mut full_url = host.clone();
    full_url.set_path(&format!(
        "{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{}/{}/a2a",
        path_params.project_id, path_params.agent_id
    ));

    let card = construct_agent_card(ConstructAgentCardParams {
        definition: soma_definition,
        url: full_url.to_string(),
    });
    Ok(card)
}
