use a2a_rs::agent_execution::SimpleRequestContextBuilder;
use a2a_rs::agent_execution::agent_executor::BoxedFuture;
use a2a_rs::errors::A2aServerError;
use a2a_rs::events::InMemoryQueueManager;
use a2a_rs::service::A2aServiceLike;
use a2a_rs::tasks::base_push_notification_sender::BasePushNotificationSenderBuilder;
use a2a_rs::tasks::in_memory_push_notification_config_store::InMemoryPushNotificationConfigStoreBuilder;
use a2a_rs::{
    adapters::jsonrpc::axum::create_router as create_a2a_router,
    agent_execution::{agent_executor::AgentExecutor, context::RequestContext},
    events::event_queue::{Event, EventQueue},
    request_handlers::{
        default_request_handler::DefaultRequestHandler, request_handler::RequestHandler,
    },
    types::{Task, TaskState, TaskStatus},
};
use async_trait::async_trait;
use axum::extract::State;
use reqwest::Client;
use serde_json::json;
use shared::adapters::openapi::{API_VERSION_TAG, JsonResponse};
use shared::error::CommonError;
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use shared::uds::{DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client};
use std::collections::HashMap;
use std::str::FromStr;
use std::{pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tracing::info;
use url::Url;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::logic::a2a::ConstructAgentCardParams;
use crate::logic::a2a::{RepositoryTaskStore, construct_agent_card};
use crate::logic::task::{
    self as task_logic, ConnectionManager, CreateMessageRequest, UpdateTaskStatusRequest,
    WithTaskId, update_task_status,
};
use crate::repository::{CreateTask, Repository, TaskRepositoryLike};
use shared::restate::admin_client::AdminClient;
use shared::restate::invoke::{RestateIngressClient, construct_initial_object_id};
use shared::soma_agent_definition::{SomaAgentDefinition, SomaAgentDefinitionLike};

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "a2a";
pub const API_VERSION_1: &str = "v1";

pub fn create_router() -> OpenApiRouter<Arc<Agent2AgentService>> {
    let openapi_router = OpenApiRouter::new().routes(routes!(route_definition));

    let a2a_router: OpenApiRouter<Arc<Agent2AgentService>> = create_a2a_router();

    openapi_router.nest(
        &format!("{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}"),
        a2a_router,
    )
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/definition", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    responses(
        (status = 200, description = "Agent definition", body = SomaAgentDefinition),
    ),
    summary = "Get agent definition",
    description = "Get the agent definition (capabilities and metadata)",
    operation_id = "get-agent-definition",
)]
async fn route_definition(
    State(ctx): State<Arc<Agent2AgentService>>,
) -> JsonResponse<SomaAgentDefinition, CommonError> {
    let soma_definition = ctx.soma_definition.get_definition().await;
    JsonResponse::from(soma_definition)
}

pub struct Agent2AgentService {
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    host: Url,
    request_handler: Arc<dyn RequestHandler + Send + Sync>,
}

pub struct Agent2AgentServiceParams {
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub host: Url,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
}

impl Agent2AgentService {
    pub fn new(params: Agent2AgentServiceParams) -> Self {
        let Agent2AgentServiceParams {
            soma_definition,
            host,
            connection_manager,
            repository,
            restate_ingress_client,
            restate_admin_client,
        } = params;
        // Create the agent executor
        let agent_executor = Arc::new(ProxiedAgent {
            connection_manager,
            soma_definition: soma_definition.clone(),
            repository: repository.clone(),
            restate_ingress_client,
            restate_admin_client,
        });

        // Create a task store
        let task_store = Arc::new(RepositoryTaskStore::new(repository.clone()));
        let config_store = Arc::new(
            InMemoryPushNotificationConfigStoreBuilder::default()
                .push_notification_infos(Arc::new(RwLock::new(HashMap::new())))
                .build()
                .unwrap(),
        );
        // Create the request handler
        let request_handler: Arc<dyn RequestHandler + Send + Sync> =
            Arc::new(DefaultRequestHandler::new(
                agent_executor.clone(),
                task_store.clone(),
                Some(Arc::new(InMemoryQueueManager::new())),
                Some(config_store.clone()),
                Some(Arc::new(
                    BasePushNotificationSenderBuilder::default()
                        .client(Arc::new(Client::new()))
                        .config_store(config_store)
                        .build()
                        .unwrap(),
                )),
                Some(Arc::new(SimpleRequestContextBuilder::new(
                    false,
                    Some(task_store),
                ))),
            ));

        Self {
            soma_definition: soma_definition.clone(),
            host,
            request_handler,
        }
    }
}

#[async_trait]
impl A2aServiceLike for Agent2AgentService {
    async fn agent_card(
        &self,
        _context: a2a_rs::service::RequestContext,
    ) -> Result<a2a_rs::types::AgentCard, A2aServerError> {
        let soma_definition = self.soma_definition.get_definition().await?;
        let mut full_url = self.host.clone();
        full_url.set_path(&format!(
            "{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}"
        ));

        let card = construct_agent_card(ConstructAgentCardParams {
            definition: soma_definition,
            url: full_url.to_string(),
        });
        Ok(card)
    }

    async fn extended_agent_card(
        &self,
        _context: a2a_rs::service::RequestContext,
    ) -> Result<Option<a2a_rs::types::AgentCard>, A2aServerError> {
        Ok(None)
    }

    fn request_handler(
        &self,
        _context: a2a_rs::service::RequestContext,
    ) -> Arc<dyn a2a_rs::request_handlers::request_handler::RequestHandler + Send + Sync> {
        self.request_handler.clone()
    }
}

struct ProxiedAgent {
    connection_manager: ConnectionManager,
    #[allow(dead_code)]
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    repository: Repository,
    restate_ingress_client: RestateIngressClient,
    restate_admin_client: AdminClient,
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
                None => a2a_rs::types::Task {
                    id: context
                        .task_id()
                        .expect("task_id must be present")
                        .to_string(),
                    context_id: context_id.to_string(),
                    status: a2a_rs::types::TaskStatus {
                        state: a2a_rs::types::TaskState::Submitted,
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
            // self.restate_ingress_client.resolve_awakeable(&task.id, &json!({ "task": task, "timelineItem": message.timeline_item })).await?;
            // let client = Client::new();

            // let service_url = format!("http://localhost:{}", self.runtime_port);
            // let service_url = "http://localhost:8080";
            // restate::invoke::invoke_virtual_object_handler(
            //     &client,
            //     &service_url,
            //     &self.soma_config.project,
            //     &task.id,
            //     "onNewMessage",
            //     body,
            // )
            // .await?;
            let connection_manager = self.connection_manager.clone();
            let task_id_clone = task_id.clone();
            let connection_id_clone = connection_id.clone();
            let event_queue_clone = event_queue.clone();
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    info!("Received event: {:?}", event);

                    // Send event back to a2a response stream
                    match event_queue_clone.enqueue_event(event.clone()).await {
                        Ok(_) => (),
                        Err(e) => {
                            info!(
                                "Failed to enqueue event: {:?}, channel most likely closed",
                                e
                            );
                            break;
                        }
                    }
                }
                info!("Removing connection");
                connection_manager
                    .remove_connection(task_id_clone, connection_id_clone)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)
                    .unwrap();
            });

            let db_task = self.repository.get_task_by_id(&task_id).await?;
            if db_task.is_none() {
                self.repository
                    .create_task(&CreateTask {
                        id: task_id.clone(),
                        context_id: WrappedUuidV4::from_str(&task.context_id).unwrap(),
                        // todo: convert to task_logic::TaskStatus
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

            info!("Invoking runtime agent with task: {:?}", task);

            // assume the latest timelineitem is the one for processing
            let message = task_logic::create_message(
                &self.repository,
                &self.connection_manager,
                WithTaskId {
                    task_id: task_id.clone(),
                    inner: CreateMessageRequest {
                        reference_task_ids: vec![],
                        role: match message.role {
                            a2a_rs::types::MessageRole::User => task_logic::MessageRole::User,
                            a2a_rs::types::MessageRole::Agent => task_logic::MessageRole::Agent,
                        },
                        metadata: task_logic::Metadata::new(),
                        // parts: vec![],
                        parts: message
                            .parts
                            .iter()
                            .map(|part| match part {
                                a2a_rs::types::Part::TextPart(text_part) => {
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

            // Get agent metadata
            let socket_path = std::env::var("SOMA_SERVER_SOCK")
                .unwrap_or_else(|_| DEFAULT_SOMA_SERVER_SOCK.to_string());
            let mut sdk_client =
                create_soma_unix_socket_client(&socket_path)
                    .await
                    .map_err(|e| {
                        Box::new(CommonError::Unknown(anyhow::anyhow!(
                            "Failed to connect to SDK: {e}"
                        )))
                            as Box<dyn std::error::Error + Send + Sync + 'static>
                    })?;
            let metadata_response =
                sdk_client
                    .metadata(tonic::Request::new(()))
                    .await
                    .map_err(|e| {
                        Box::new(CommonError::Unknown(anyhow::anyhow!(
                            "Failed to get SDK metadata: {e}"
                        )))
                            as Box<dyn std::error::Error + Send + Sync + 'static>
                    })?;
            let metadata = metadata_response.into_inner();
            let project_id = metadata
                .agents
                .first()
                .map(|agent| &agent.project_id)
                .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("No agents registered")))?;
            let agent_id = metadata
                .agents
                .first()
                .map(|agent| &agent.id)
                .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("No agents registered")))?;
            let service_name = format!("{project_id}.{agent_id}");
            let object_id = construct_initial_object_id(&task.id);

            // Use the task status from the database if available, otherwise convert from context task
            let task_status = if let Some(db_task) = &db_task {
                db_task.task.status.clone()
            } else {
                // Convert from a2a_rs TaskState to logic TaskStatus
                match task.status.state {
                    a2a_rs::types::TaskState::Submitted => task_logic::TaskStatus::Submitted,
                    a2a_rs::types::TaskState::Working => task_logic::TaskStatus::Working,
                    a2a_rs::types::TaskState::InputRequired => {
                        task_logic::TaskStatus::InputRequired
                    }
                    a2a_rs::types::TaskState::Completed => task_logic::TaskStatus::Completed,
                    a2a_rs::types::TaskState::Canceled => task_logic::TaskStatus::Canceled,
                    a2a_rs::types::TaskState::Failed => task_logic::TaskStatus::Failed,
                    a2a_rs::types::TaskState::Rejected => task_logic::TaskStatus::Rejected,
                    a2a_rs::types::TaskState::AuthRequired => task_logic::TaskStatus::AuthRequired,
                    a2a_rs::types::TaskState::Unknown => task_logic::TaskStatus::Unknown,
                }
            };

            match task_status {
                task_logic::TaskStatus::Submitted => {
                    info!("New task detected, invoking entrypoint handler");

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
                    info!("Existing task detected, resolving new_input_promise awakeable");

                    // TODO: we could have a race condition where promise is not created yet in restate sdk
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

                    info!("Restate state: {:?}", restate_state);

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
            dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            info!("HelloWorldAgent cancel called");

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
