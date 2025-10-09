use std::pin::Pin;
use std::sync::Arc;
use std::{collections::HashMap, future::Future};

use a2a_rs::agent_execution::SimpleRequestContextBuilder;
use a2a_rs::events::InMemoryQueueManager;
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
};
use axum::Router;
use reqwest::Client;
use serde_json::Map;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use utoipa::openapi::OpenApi;

/// A simple hello world agent that responds to any message
struct HelloWorldAgent;

impl AgentExecutor for HelloWorldAgent {
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
            let _message = context.message();

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

            // Create a task using the task ID from the context
            let task = Task {
                id: context.task_id().unwrap_or("test-task-id").to_string(),
                context_id: context
                    .context_id()
                    .unwrap_or("test-context-id")
                    .to_string(),
                status: TaskStatus {
                    state: TaskState::Completed,
                    message: None, // This expects a Message struct, not a string
                    timestamp: Some(chrono::Utc::now().to_rfc3339()),
                },
                artifacts: vec![],
                history: vec![],
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
                id: _context.task_id().unwrap_or("test-task-id").to_string(),
                context_id: _context
                    .context_id()
                    .unwrap_or("test-context-id")
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Hello World A2A Agent");

    // Create the agent executor
    let agent_executor = Arc::new(HelloWorldAgent);

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

    // Get the agent card from the executor
    let agent_card = AgentCard {
        protocol_version: "1.0".to_string(),
        signatures: vec![],
        name: "Hello World Agent".to_string(),
        description: "A simple agent that responds with 'Hello, World!'".to_string(),
        url: "http://localhost:8080".to_string(),
        preferred_transport: "GRPC".to_string(),
        additional_interfaces: vec![],
        provider: None,
        version: "1.0.0".to_string(),
        documentation_url: None,
        capabilities: AgentCapabilities {
            streaming: Some(true),
            push_notifications: Some(true),
            extensions: vec![],
            state_transition_history: Some(false),
        },
        security_schemes: Default::default(),
        security: vec![],
        default_input_modes: vec!["text/plain".to_string()],
        default_output_modes: vec!["text/plain".to_string()],
        skills: vec![AgentSkill {
            id: "hello".to_string(),
            name: "Hello World".to_string(),
            description: "Responds with a friendly greeting".to_string(),
            tags: vec!["greeting".to_string(), "hello".to_string()],
            examples: vec!["Say hello".to_string(), "Greet me".to_string()],
            input_modes: vec!["text/plain".to_string()],
            output_modes: vec!["text/plain".to_string()],
            security: vec![],
        }],
        supports_authenticated_extended_card: Some(false),
        icon_url: None,
    };

    // Create the A2A service
    let a2a_service = A2aServiceBuilder::default()
        .agent_card(Arc::new(agent_card))
        .extended_agent_card(Arc::new(None))
        .request_handler(request_handler)
        .build()
        .unwrap();

    // Create and start the JSON-RPC server

    let (router, spec): (Router, OpenApi) = a2a_rs::adapters::jsonrpc::axum::create_router()
        .with_state(Arc::new(a2a_service))
        .split_for_parts();

    let listener = TcpListener::bind("0.0.0.0:41241").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
