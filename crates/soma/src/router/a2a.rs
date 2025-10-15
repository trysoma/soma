use a2a_rs::agent_execution::SimpleRequestContextBuilder;
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
    types::{
        Task, TaskState,
        TaskStatus,
    },
};
use axum::extract::State;
use reqwest::Client;
use serde_json::json;
use shared::adapters::openapi::JsonResponse;
use shared::error::CommonError;
use shared::primitives::WrappedUuidV4;
use std::collections::HashMap;
use std::str::FromStr;
use std::{path::PathBuf, pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tracing::info;
use url::Url;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::a2a::RepositoryTaskStore;
use crate::logic::{
    self, ConnectionManager, CreateMessageRequest,
    WithTaskId,
};
use crate::repository::Repository;
use crate::utils::restate::invoke::{
    RestateIngressClient, construct_initial_object_id,
};
use crate::utils::soma_agent_config::SomaConfig;

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "a2a";
pub const API_VERSION_1: &str = "v1";

pub fn create_router() -> OpenApiRouter<Arc<Agent2AgentService>> {
    let openapi_router = OpenApiRouter::new().routes(routes!(route_config));

    let a2a_router: OpenApiRouter<Arc<Agent2AgentService>> = create_a2a_router();
    

    openapi_router.nest(
        &format!("{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}"),
        a2a_router,
    )
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/config", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    responses(
        (status = 200, description = "Agent config", body = SomaConfig),
    ),
    operation_id = "get-agent-config",
)]
async fn route_config(
    State(ctx): State<Arc<Agent2AgentService>>,
) -> JsonResponse<SomaConfig, CommonError> {
    let config = ctx.config.clone();
    JsonResponse::from(Ok(config.clone()))
}

pub(crate) struct Agent2AgentService {
    src_dir: PathBuf,
    config: SomaConfig,
    host: Url,
    request_handler: Arc<dyn RequestHandler + Send + Sync>,
    runtime_port: u16,
    repository: Repository,
}

impl Agent2AgentService {
    pub fn new(
        src_dir: PathBuf,
        config: SomaConfig,
        host: Url,
        connection_manager: ConnectionManager,
        repository: Repository,
        runtime_port: u16,
        restate_ingress_client: RestateIngressClient,
    ) -> Self {
        // Create the agent executor
        let agent_executor = Arc::new(ProxiedAgent {
            connection_manager,
            soma_config: config.clone(),
            repository: repository.clone(),
            restate_ingress_client,
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
            src_dir,
            config,
            host,
            request_handler,
            runtime_port,
            repository,
        }
    }
}

impl A2aServiceLike for Agent2AgentService {
    fn agent_card(&self, _context: a2a_rs::service::RequestContext) -> a2a_rs::types::AgentCard {
        let soma_str = std::fs::read_to_string(self.src_dir.join("soma.yaml")).unwrap();
        let soma_config =
            crate::utils::soma_agent_config::SomaConfig::from_yaml(&soma_str).unwrap();
        let mut full_url = self.host.clone();
        full_url.set_path(&format!(
            "{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}"
        ));
        
        crate::utils::soma_agent_config::construct_agent_card(&soma_config, &full_url)
    }

    fn extended_agent_card(
        &self,
        _context: a2a_rs::service::RequestContext,
    ) -> Option<a2a_rs::types::AgentCard> {
        None
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
    soma_config: SomaConfig,
    repository: Repository,
    restate_ingress_client: RestateIngressClient,
}

impl AgentExecutor for ProxiedAgent {
    fn execute<'a>(
        &'a self,
        context: RequestContext,
        event_queue: EventQueue,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send + 'a>>
    {
        Box::pin(async move {
            let context_id = match context.context_id() {
                Some(context_id) => context_id.to_string(),
                None => uuid::Uuid::new_v4().to_string(),
            };

            let task = match context.current_task() {
                Some(task) => task.clone(),
                None => {
                    
                    a2a_rs::types::Task {
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
                    }
                }
            };

            let task_id = match context.task_id() {
                Some(task_id) => task_id,
                None => {
                    let err = CommonError::Unknown(anyhow::anyhow!("Task ID is required"));
                    return Err(Box::new(err) as Box<dyn std::error::Error + Send>);
                }
            };
            let task_id = match WrappedUuidV4::from_str(task_id) {
                Ok(task_id) => task_id,
                Err(e) => {
                    let err =
                        CommonError::Unknown(anyhow::anyhow!("Failed to parse task ID: {e:?}"));
                    return Err(Box::new(err) as Box<dyn std::error::Error + Send>);
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
                    return Err(Box::new(err) as Box<dyn std::error::Error + Send>);
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

            let message = match context.message() {
                Some(message) => message,
                None => unreachable!("message must be present"),
            };

            event_queue
                .enqueue_event(Event::Task(task.clone()))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            info!("Invoking runtime agent with task: {:?}", task);

            // assume the latest timelineitem is the one for processing
            let message = logic::create_message(
                &self.repository,
                &self.connection_manager,
                WithTaskId {
                    task_id: task_id.clone(),
                    inner: CreateMessageRequest {
                        reference_task_ids: vec![],
                        role: match message.role {
                            a2a_rs::types::MessageRole::User => logic::MessageRole::User,
                            a2a_rs::types::MessageRole::Agent => logic::MessageRole::Agent,
                        },
                        metadata: logic::Metadata::new(),
                        // parts: vec![],
                        parts: message
                            .parts
                            .iter()
                            .map(|part| match part {
                                a2a_rs::types::Part::TextPart(text_part) => {
                                    logic::MessagePart::TextPart(logic::TextPart {
                                        text: text_part.text.clone(),
                                        metadata: logic::Metadata::new(),
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
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            event_queue
                .enqueue_event(Event::Message(message.message.into()))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            // suspend previous execution

            // self.restate_ingress_client.resolve_awakeable(&construct_cancel_awakeable_id(&task.id), &json!({ })).await?;
            // let body: serde_json::Value = json!({ "task": task, "timelineItem": message.timeline_item });
            let body: serde_json::Value = json!({ "task": task});
            info!("Invoking virtual object handler with body: {:?}", body);
            self.restate_ingress_client
                .invoke_virtual_object_handler(
                    &self.soma_config.project,
                    &construct_initial_object_id(&task.id),
                    "onNewMessage",
                    body,
                )
                .await?;

            Ok(())
        })
    }

    fn cancel<'a>(
        &'a self,
        _context: RequestContext,
        event_queue: EventQueue,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send + 'a>>
    {
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
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            Ok(())
        })
    }
}
