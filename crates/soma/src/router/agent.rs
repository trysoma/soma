use a2a_rs::agent_execution::SimpleRequestContextBuilder;
use a2a_rs::events::InMemoryQueueManager;
use a2a_rs::service::A2aServiceLike;
use a2a_rs::tasks::base_push_notification_sender::BasePushNotificationSenderBuilder;
use a2a_rs::tasks::in_memory_push_notification_config_store::InMemoryPushNotificationConfigStoreBuilder;
use a2a_rs::types::TextPart;
use a2a_rs::{
    agent_execution::{agent_executor::AgentExecutor, context::RequestContext},
    events::event_queue::{Event, EventQueue},
    request_handlers::{
        default_request_handler::DefaultRequestHandler, request_handler::RequestHandler,
    },
    service::A2aServiceBuilder,
    tasks::in_memory_task_store::InMemoryTaskStoreBuilder,
    types::{
        AgentCapabilities, AgentCard, AgentSkill, Message, MessageRole, Part, Task, TaskState,
        TaskStatus,
    },
    adapters::jsonrpc::axum::create_router as create_a2a_router
};
use axum::{
    Router,
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use serde_json::Map;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::{convert::Infallible, path::PathBuf, pin::Pin, sync::Arc};
use tracing::info;
use url::Url;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use reqwest::Client;

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "agent";
pub const API_VERSION_1: &str = "v1";

pub fn create_router() -> OpenApiRouter<Arc<AgentService>> {
    let openapi_router = OpenApiRouter::new();

    let a2a_router: OpenApiRouter<Arc<AgentService>> = create_a2a_router();
    let openapi_router = openapi_router.nest(
        &format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
        a2a_router,
    );

    return openapi_router;
}

pub(crate) struct AgentService {
    src_dir: PathBuf,
    host: Url,
    request_handler: Arc<dyn RequestHandler + Send + Sync>,
}

impl AgentService {
    pub fn new(src_dir: PathBuf, host: Url) -> Self {
        // Create the agent executor
        let agent_executor = Arc::new(ProxiedAgent);

        // Create a task store
        let task_store = Arc::new(
            InMemoryTaskStoreBuilder::default()
                .tasks(Arc::new(RwLock::new(HashMap::new())))
                .build()
                .unwrap(),
        );
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
            host,
            request_handler,
        }
    }
}

impl A2aServiceLike for AgentService {
    fn agent_card(&self, context: a2a_rs::service::RequestContext) -> a2a_rs::types::AgentCard {
        let soma_str = std::fs::read_to_string(self.src_dir.join("soma.yaml")).unwrap();
        let soma_config =
            crate::utils::soma_agent_config::SomaConfig::from_yaml(&soma_str).unwrap();
        let mut full_url = self.host.clone();
        full_url.set_path(&format!(
            "{}/{}/{}",
            PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1
        ));
        let agent_card =
            crate::utils::soma_agent_config::construct_agent_card(&soma_config, &full_url);
        agent_card
    }

    fn extended_agent_card(
        &self,
        context: a2a_rs::service::RequestContext,
    ) -> Option<a2a_rs::types::AgentCard> {
        None
    }

    fn request_handler(
        &self,
        context: a2a_rs::service::RequestContext,
    ) -> Arc<dyn a2a_rs::request_handlers::request_handler::RequestHandler + Send + Sync> {
        self.request_handler.clone()
    }
}

struct ProxiedAgent;

impl AgentExecutor for ProxiedAgent {
    fn execute<'a>(
        &'a self,
        context: RequestContext,
        event_queue: EventQueue,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + Sync
                + 'a,
        >,
    > {
        Box::pin(async move {
            info!("HelloWorldAgent executing with context: {:?}", context);

            // Extract the message from the context
            let user_message = context.message().cloned();

            // Create a response message
            let text_part = Part::TextPart(TextPart {
                kind: "text".into(),
                metadata: Map::new(),
                text: "Hello, World! I received your message.".to_string(),
            });
            let response_message = Message {
                message_id: uuid::Uuid::new_v4().to_string(),
                context_id: context.context_id().map(|s| s.to_string()),
                task_id: context.task_id().map(|s| s.to_string()),
                role: MessageRole::Agent,
                parts: vec![text_part],
                metadata: Default::default(),
                extensions: vec![],
                kind: "message".to_string(),
                reference_task_ids: vec![],
            };

            // Build history with user message and agent response
            let mut history = vec![];
            if let Some(user_msg) = user_message {
                history.push(user_msg);
            }
            history.push(response_message.clone());

            // Create a task using the task ID from the context
            let task = Task {
                id: context.task_id().expect("task_id must be present").to_string(),
                context_id: context
                    .context_id()
                    .expect("context_id must be present")
                    .to_string(),
                status: TaskStatus {
                    state: TaskState::InputRequired,
                    message: None,
                    timestamp: Some(chrono::Utc::now().to_rfc3339()),
                },
                artifacts: vec![],
                history,
                metadata: Default::default(),
                kind: "task".to_string(),
            };

            // Send the task event
            event_queue
                .enqueue_event(Event::Task(task))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            // Send the message event
            event_queue
                .enqueue_event(Event::Message(response_message))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            info!("HelloWorldAgent execution completed");
            Ok(())
        })
    }

    fn cancel<'a>(
        &'a self,
        _context: RequestContext,
        event_queue: EventQueue,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + Sync
                + 'a,
        >,
    > {
        Box::pin(async move {
            info!("HelloWorldAgent cancel called");

            // Create a cancelled task
            let task = Task {
                id: _context.task_id().expect("task_id must be present").to_string(),
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
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            Ok(())
        })
    }
}
