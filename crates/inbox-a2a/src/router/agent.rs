//! Agent routes for the A2A protocol
//!
//! Provides endpoints for:
//! - Agent card discovery (/.well-known/agent.json)
//! - A2A JSON-RPC handling (message/send, message/stream, tasks/get, tasks/cancel, tasks/resubscribe)

use a2a_core::agent_execution::SimpleRequestContextBuilder;
use a2a_core::agent_execution::agent_executor::BoxedFuture;
use a2a_core::tasks::base_push_notification_sender::BasePushNotificationSenderBuilder;
use a2a_core::{
    agent_execution::{agent_executor::AgentExecutor, context::RequestContext},
    events::event_queue::{Event, EventQueue},
    request_handlers::{
        default_request_handler::DefaultRequestHandler, request_handler::RequestHandler,
    },
    types::{Task, TaskState, TaskStatus},
};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::response::sse::{Event as SseEvent, Sse};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use shared::adapters::openapi::API_VERSION_TAG;
use shared::error::CommonError;
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::trace;
use url::Url;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use a2a::logic::agent::{ConstructAgentCardParams, construct_agent_card};
use a2a::logic::task::{
    self as task_logic, CreateMessageRequest, UpdateTaskStatusRequest, WithTaskId,
    update_task_status,
};
use a2a::{A2aService, ConnectionManager, Repository, TaskRepositoryLike};
use shared::restate::admin_client::AdminClient;
use shared::restate::invoke::{RestateIngressClient, construct_initial_object_id};
use shared::soma_agent_definition::SomaAgentDefinitionLike;

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "agent";

/// Path parameters for multi-agent routes
#[derive(Debug, Clone, Deserialize)]
pub struct AgentPathParams {
    pub project_id: String,
    pub agent_id: String,
}

/// Creates the agent router with agent card and JSON-RPC endpoints
/// Note: list_agents route is in soma-api-server, not here
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
        (status = 200, description = "Agent card", body = a2a_core::types::AgentCard),
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

/// POST /api/agent/{project_id}/{agent_id}/a2a - Handle A2A JSON-RPC requests (SSE chat endpoint)
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
    Json(body): Json<a2a_core::types::JsonrpcRequest>,
) -> impl IntoResponse {
    trace!(
        project_id = %path_params.project_id,
        agent_id = %path_params.agent_id,
        method = %body.method,
        "Handling A2A JSON-RPC request"
    );

    // Get the request handler with path params context
    let handler = get_request_handler_with_params(&ctx, &path_params);

    // Handle the JSON-RPC method
    let id = body.id.clone();
    let result = match body.method.as_str() {
        "message/send" => {
            let params: a2a_core::types::MessageSendParams =
                match serde_json::from_value(serde_json::Value::Object(body.params)) {
                    Ok(p) => p,
                    Err(e) => {
                        return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
                    }
                };
            match handler.on_message_send(params).await {
                Ok(result) => serde_json::to_value(result).unwrap(),
                Err(e) => {
                    return (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                        .into_response();
                }
            }
        }
        "tasks/get" => {
            let params: a2a_core::types::TaskQueryParams =
                match serde_json::from_value(serde_json::Value::Object(body.params)) {
                    Ok(p) => p,
                    Err(e) => {
                        return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
                    }
                };
            match handler.on_get_task(params).await {
                Ok(result) => serde_json::to_value(result).unwrap(),
                Err(e) => {
                    return (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                        .into_response();
                }
            }
        }
        "tasks/cancel" => {
            let params: a2a_core::types::TaskIdParams =
                match serde_json::from_value(serde_json::Value::Object(body.params)) {
                    Ok(p) => p,
                    Err(e) => {
                        return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
                    }
                };
            match handler.on_cancel_task(params).await {
                Ok(result) => serde_json::to_value(result).unwrap(),
                Err(e) => {
                    return (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                        .into_response();
                }
            }
        }
        "message/stream" => {
            trace!("Processing message/stream request");
            let params: a2a_core::types::MessageSendParams =
                match serde_json::from_value(serde_json::Value::Object(body.params)) {
                    Ok(p) => p,
                    Err(e) => {
                        return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
                    }
                };
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let id_for_task = id.clone();

            tokio::spawn(async move {
                let stream_res = handler.on_message_send_stream(params).await;

                match stream_res {
                    Ok(mut stream) => {
                        while let Some(item) = stream.next().await {
                            trace!("Sending message stream item");
                            if tx.send(item).is_err() {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err));
                    }
                }
            });

            let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            let stream = TokioStreamExt::map(stream, move |item| {
                let data: a2a_core::types::CustomJsonRpcPayload<
                    a2a_core::types::SendStreamingMessageSuccessResponseResult,
                > = item.into();
                let res = a2a_core::types::CustomJsonrpcResponse::new(id_for_task.clone(), data);
                trace!("Emitting SSE event");
                SseEvent::default().json_data(res)
            });

            return Sse::new(stream)
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(1))
                        .text("keep-alive"),
                )
                .into_response();
        }
        "tasks/resubscribe" => {
            trace!("Processing tasks/resubscribe request");
            let params: a2a_core::types::TaskIdParams =
                match serde_json::from_value(serde_json::Value::Object(body.params)) {
                    Ok(p) => p,
                    Err(e) => {
                        return (http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
                    }
                };
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let id_for_task = id.clone();

            tokio::spawn(async move {
                let stream_res = handler.on_resubscribe_to_task(params);

                match stream_res {
                    Ok(mut stream) => {
                        while let Some(item) = stream.next().await {
                            if tx.send(item).is_err() {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err));
                    }
                }
            });

            let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            let stream = TokioStreamExt::map(stream, move |item| {
                let data: a2a_core::types::CustomJsonRpcPayload<
                    a2a_core::types::SendStreamingMessageSuccessResponseResult,
                > = item.into();
                let res = a2a_core::types::CustomJsonrpcResponse::new(id_for_task.clone(), data);
                SseEvent::default().json_data(res)
            });

            return Sse::new(stream)
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(1))
                        .text("keep-alive"),
                )
                .into_response();
        }
        _ => {
            return (http::StatusCode::NOT_FOUND, "Unknown method").into_response();
        }
    };

    let response = a2a_core::types::CustomJsonrpcResponse::new(
        id,
        a2a_core::types::CustomJsonRpcPayload::Ok(result),
    );
    (http::StatusCode::OK, Json(response)).into_response()
}

/// Parameters for creating the A2aService with agent capabilities
pub struct A2aServiceParams {
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub host: Url,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
    pub agent_cache: a2a::AgentCache,
}

/// Proxied agent that forwards requests to Restate
pub struct ProxiedAgent {
    connection_manager: ConnectionManager,
    #[allow(dead_code)]
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    repository: Repository,
    restate_ingress_client: RestateIngressClient,
    restate_admin_client: AdminClient,
    /// Path params for multi-agent routing
    project_id: String,
    agent_id: String,
}

impl AgentExecutor for ProxiedAgent {
    fn execute(&self, context: RequestContext, event_queue: EventQueue) -> BoxedFuture<'_> {
        Box::pin(async move {
            let context_id = match context.context_id() {
                Some(context_id) => context_id.to_string(),
                None => uuid::Uuid::new_v4().to_string(),
            };

            let task = match context.current_task() {
                Some(task) => task.clone(),
                None => a2a_core::types::Task {
                    id: context
                        .task_id()
                        .expect("task_id must be present")
                        .to_string(),
                    context_id: context_id.to_string(),
                    status: a2a_core::types::TaskStatus {
                        state: a2a_core::types::TaskState::Submitted,
                        message: None,
                        timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    },
                    artifacts: vec![],
                    history: vec![],
                    kind: "task".to_string(),
                    metadata: Default::default(),
                },
            };

            let task_id = match context.task_id() {
                Some(task_id) => task_id,
                None => {
                    let err = CommonError::Unknown(anyhow::anyhow!("Task ID is required"));
                    return Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync + 'static>);
                }
            };
            let task_id = match WrappedUuidV4::from_str(task_id) {
                Ok(task_id) => task_id,
                Err(e) => {
                    let err =
                        CommonError::Unknown(anyhow::anyhow!("Failed to parse task ID: {e:?}"));
                    return Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync + 'static>);
                }
            };

            // Register the connection BEFORE invoking the handler
            // so that any messages sent during handler execution can be received
            let (connection_id, mut receiver) = match self
                .connection_manager
                .add_connection(task_id.clone())
            {
                Ok((connection_id, receiver)) => (connection_id, receiver),
                Err(e) => {
                    let err =
                        CommonError::Unknown(anyhow::anyhow!("Failed to add connection: {e:?}"));
                    return Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync + 'static>);
                }
            };

            let connection_manager = self.connection_manager.clone();
            let task_id_clone = task_id.clone();
            let connection_id_clone = connection_id.clone();
            let event_queue_clone = event_queue.clone();
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    trace!("Received A2A event from connection");

                    // Send event back to a2a response stream
                    match event_queue_clone.enqueue_event(event.clone()).await {
                        Ok(_) => (),
                        Err(e) => {
                            trace!(
                                error = %e,
                                "Failed to enqueue event, channel closed"
                            );
                            break;
                        }
                    }
                }
                trace!("Removing connection");
                connection_manager
                    .remove_connection(task_id_clone, connection_id_clone)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)
                    .unwrap();
            });

            let db_task = self.repository.get_task_by_id(&task_id).await?;
            if db_task.is_none() {
                self.repository
                    .create_task(&a2a::repository::CreateTask {
                        id: task_id.clone(),
                        context_id: WrappedUuidV4::from_str(&task.context_id).unwrap(),
                        status: task_logic::TaskStatus::from(task.status.state.to_string()),
                        status_timestamp: WrappedChronoDateTime::now(),
                        metadata: WrappedJsonValue::new(serde_json::to_value(
                            task.metadata.clone(),
                        )?),
                        created_at: WrappedChronoDateTime::now(),
                        updated_at: WrappedChronoDateTime::now(),
                    })
                    .await?;
                event_queue.enqueue_event(Event::Task(task.clone())).await?;
            }

            let message = match context.message() {
                Some(message) => message,
                None => unreachable!("message must be present"),
            };

            trace!(task_id = %task.id, "Invoking runtime agent");

            // Create message in the database
            let message = task_logic::create_message(
                &self.repository,
                &self.connection_manager,
                WithTaskId {
                    task_id: task_id.clone(),
                    inner: CreateMessageRequest {
                        reference_task_ids: vec![],
                        role: match message.role {
                            a2a_core::types::MessageRole::User => task_logic::MessageRole::User,
                            a2a_core::types::MessageRole::Agent => task_logic::MessageRole::Agent,
                        },
                        metadata: task_logic::Metadata::new(),
                        parts: message
                            .parts
                            .iter()
                            .map(|part| match part {
                                a2a_core::types::Part::TextPart(text_part) => {
                                    task_logic::MessagePart::TextPart(task_logic::TextPart {
                                        text: text_part.text.clone(),
                                        metadata: task_logic::Metadata::new(),
                                    })
                                }
                                _ => unreachable!("unsupported part type"),
                            })
                            .collect(),
                    },
                },
                true,
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)?;

            event_queue
                .enqueue_event(Event::Message(message.message.into()))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)?;

            // Use path params stored in the agent to construct service name
            let service_name = format!("{}.{}", self.project_id, self.agent_id);
            trace!(service = %service_name, "Resolved agent service name");
            let object_id = construct_initial_object_id(&task.id);

            // Use the task status from the database if available, otherwise convert from context task
            let task_status = if let Some(db_task) = &db_task {
                db_task.task.status.clone()
            } else {
                // Convert from a2a_core TaskState to logic TaskStatus
                match task.status.state {
                    a2a_core::types::TaskState::Submitted => task_logic::TaskStatus::Submitted,
                    a2a_core::types::TaskState::Working => task_logic::TaskStatus::Working,
                    a2a_core::types::TaskState::InputRequired => {
                        task_logic::TaskStatus::InputRequired
                    }
                    a2a_core::types::TaskState::Completed => task_logic::TaskStatus::Completed,
                    a2a_core::types::TaskState::Canceled => task_logic::TaskStatus::Canceled,
                    a2a_core::types::TaskState::Failed => task_logic::TaskStatus::Failed,
                    a2a_core::types::TaskState::Rejected => task_logic::TaskStatus::Rejected,
                    a2a_core::types::TaskState::AuthRequired => task_logic::TaskStatus::AuthRequired,
                    a2a_core::types::TaskState::Unknown => task_logic::TaskStatus::Unknown,
                }
            };

            match task_status {
                task_logic::TaskStatus::Submitted => {
                    trace!(task_id = %task.id, "New task, invoking entrypoint handler");

                    let body: serde_json::Value = json!({
                        "taskId": task.id,
                        "contextId": task.context_id,
                    });
                    update_task_status(
                        &self.repository,
                        &self.connection_manager,
                        Some(event_queue.clone()),
                        WithTaskId {
                            task_id: task_id.clone(),
                            inner: UpdateTaskStatusRequest {
                                status: task_logic::TaskStatus::Working,
                                message: None,
                            },
                        },
                    )
                    .await?;
                    self.restate_ingress_client
                        .invoke_virtual_object_handler(
                            &service_name,
                            &object_id,
                            "entrypoint",
                            body,
                        )
                        .await
                        .map_err(|e| {
                            Box::new(CommonError::Unknown(anyhow::anyhow!(
                                "Failed to invoke entrypoint: {e}"
                            )))
                                as Box<dyn std::error::Error + Send + Sync + 'static>
                        })?;
                }
                _ => {
                    // Existing task - resolve the new_input_promise awakeable
                    trace!(task_id = %task.id, "Existing task, resolving new_input_promise awakeable");

                    // Get the awakeable ID from Restate state using SQL API
                    let restate_state = self
                        .restate_admin_client
                        .get_state(&service_name, &object_id)
                        .await
                        .map_err(|e| {
                            Box::new(CommonError::Unknown(anyhow::anyhow!(
                                "Failed to get state: {e}"
                            )))
                                as Box<dyn std::error::Error + Send + Sync + 'static>
                        })?;

                    trace!("Retrieved Restate state for awakeable lookup");

                    let new_input_promise = restate_state.get("new_input_promise").cloned();
                    match new_input_promise {
                        Some(awakeable_id) => {
                            self.restate_ingress_client
                                .resolve_awakeable_generic(&awakeable_id, serde_json::Value::Null)
                                .await
                                .map_err(|e| {
                                    Box::new(CommonError::Unknown(anyhow::anyhow!(
                                        "Failed to resolve awakeable: {e}"
                                    )))
                                        as Box<dyn std::error::Error + Send + Sync + 'static>
                                })?;
                        }
                        None => {
                            return Err(Box::new(CommonError::Unknown(anyhow::anyhow!(
                                "Awakeable ID not found in state. Task may not be initialized."
                            )))
                                as Box<dyn std::error::Error + Send + Sync + 'static>);
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn cancel<'a>(
        &'a self,
        _context: RequestContext,
        event_queue: EventQueue,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            trace!("Executing task cancel");

            // Create a cancelled task
            let task = Task {
                id: _context
                    .task_id()
                    .expect("task_id must be present")
                    .to_string(),
                context_id: _context
                    .context_id()
                    .expect("context_id must be present")
                    .to_string(),
                status: TaskStatus {
                    state: TaskState::Canceled,
                    message: None,
                    timestamp: Some(chrono::Utc::now().to_rfc3339()),
                },
                artifacts: vec![],
                history: vec![],
                metadata: Default::default(),
                kind: "task".to_string(),
            };

            event_queue
                .enqueue_event(Event::Task(task))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)?;
            Ok(())
        })
    }
}

/// Get agent card for a specific project/agent
async fn get_agent_card(
    ctx: &A2aService,
    path_params: &AgentPathParams,
) -> Result<a2a_core::types::AgentCard, CommonError> {
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
    // URL for the A2A endpoint: /api/agent/{project_id}/{agent_id}/a2a
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

/// Create a request handler for a specific project/agent
fn get_request_handler_with_params(
    ctx: &A2aService,
    path_params: &AgentPathParams,
) -> Arc<dyn RequestHandler + Send + Sync> {
    // Create the agent executor with path params
    let agent_executor: Arc<dyn AgentExecutor + Send + Sync> = Arc::new(ProxiedAgent {
        connection_manager: ctx.connection_manager().clone(),
        soma_definition: ctx
            .soma_definition()
            .expect("soma_definition must be set for agent routes"),
        repository: ctx.repository().clone(),
        restate_ingress_client: ctx
            .restate_ingress_client()
            .expect("restate_ingress_client must be set for agent routes"),
        restate_admin_client: ctx
            .restate_admin_client()
            .expect("restate_admin_client must be set for agent routes"),
        project_id: path_params.project_id.clone(),
        agent_id: path_params.agent_id.clone(),
    });

    Arc::new(DefaultRequestHandler::new(
        agent_executor,
        ctx.task_store(),
        Some(ctx.queue_manager()),
        Some(ctx.config_store()),
        Some(Arc::new(
            BasePushNotificationSenderBuilder::default()
                .client(Arc::new(Client::new()))
                .config_store(ctx.config_store())
                .build()
                .unwrap(),
        )),
        Some(Arc::new(SimpleRequestContextBuilder::new(
            false,
            Some(ctx.task_store()),
        ))),
    ))
}
