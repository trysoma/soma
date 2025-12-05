use a2a_rs::types::TaskStatusUpdateEvent;
use dashmap::DashMap;
use futures::stream::{self, StreamExt};
use libsql::FromValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;
use utoipa::ToSchema;

use crate::repository::{Repository, TaskRepositoryLike, UpdateTaskStatus};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedUuidV4},
};

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
    pub sender: Sender<a2a_rs::events::Event>,
}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    pub connections_by_task_id: Arc<DashMap<WrappedUuidV4, DashMap<WrappedUuidV4, Connection>>>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections_by_task_id: Arc::new(DashMap::new()),
        }
    }

    pub fn add_connection(
        &self,
        task_id: WrappedUuidV4,
    ) -> Result<(WrappedUuidV4, Receiver<a2a_rs::events::Event>), CommonError> {
        let connection_id = WrappedUuidV4::new();
        let (sender, receiver) = tokio::sync::mpsc::channel::<a2a_rs::events::Event>(100);
        let connections = self
            .connections_by_task_id
            .entry(task_id.clone())
            .or_default();
        connections.insert(
            connection_id.clone(),
            Connection {
                id: connection_id.clone(),
                created_at: WrappedChronoDateTime::now(),
                sender,
            },
        );
        Ok((connection_id, receiver))
    }

    pub fn remove_connection(
        &self,
        task_id: WrappedUuidV4,
        connection_id: WrappedUuidV4,
    ) -> Result<(), CommonError> {
        let connections = match self.connections_by_task_id.get_mut(&task_id) {
            Some(connections) => connections,
            None => {
                return Err(CommonError::NotFound {
                    msg: "Connections not found".to_string(),
                    lookup_id: task_id.to_string(),
                    source: None,
                });
            }
        };
        connections.remove(&connection_id);
        Ok(())
    }

    pub async fn message_to_connections(
        &self,
        task_id: WrappedUuidV4,
        message: a2a_rs::events::Event,
    ) -> Result<(), CommonError> {
        info!("Sending message to connections for task_id: {}", task_id);
        let connections = match self.connections_by_task_id.get(&task_id) {
            Some(connections) => connections,
            None => return Ok(()),
        };

        info!("Connections found for task_id: {}", task_id);

        // Collect all senders first (release DashMap guard)
        let senders: Vec<_> = connections
            .iter()
            .map(|entry| entry.sender.clone())
            .collect();
        drop(connections);

        // Run up to 32 sends in parallel (adjust concurrency level as needed)
        stream::iter(senders)
            .for_each_concurrent(32, |sender| {
                let message = message.clone();
                let task_id = task_id.clone();
                async move {
                    info!("Sending message to connection for task_id: {}", task_id);
                    if let Err(e) = sender.send(message).await {
                        tracing::warn!("Failed to send to connection: {e:?}");
                    }
                }
            })
            .await;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Metadata(pub serde_json::Map<String, Value>);

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    pub fn new() -> Self {
        Metadata(serde_json::Map::new())
    }
}

// Domain models for Task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Task {
    pub id: WrappedUuidV4,
    pub context_id: WrappedUuidV4,
    pub status: TaskStatus,
    pub status_timestamp: WrappedChronoDateTime,
    pub status_message_id: Option<WrappedUuidV4>,
    pub metadata: Metadata,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ContextInfo {
    pub context_id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TaskWithDetails {
    pub task: Task,
    pub status_message: Option<Message>,
    pub messages: Vec<Message>,
    pub messages_next_page_token: Option<String>,
}

impl From<TaskWithDetails> for a2a_rs::types::Task {
    fn from(value: TaskWithDetails) -> Self {
        a2a_rs::types::Task {
            artifacts: vec![],
            context_id: value.task.context_id.to_string(),
            history: value
                .messages
                .into_iter()
                .map(|message| message.into())
                .collect(),
            id: value.task.id.to_string(),
            kind: "task".to_string(),
            metadata: value.task.metadata.0.clone(),
            status: a2a_rs::types::TaskStatus {
                message: value.status_message.map(|message| message.into()),
                state: value.task.status.into(),
                timestamp: Some(value.task.status_timestamp.to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Canceled,
    Failed,
    Rejected,
    AuthRequired,
    Unknown,
}

impl From<TaskStatus> for a2a_rs::types::TaskState {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Submitted => a2a_rs::types::TaskState::Submitted,
            TaskStatus::Working => a2a_rs::types::TaskState::Working,
            TaskStatus::InputRequired => a2a_rs::types::TaskState::InputRequired,
            TaskStatus::Completed => a2a_rs::types::TaskState::Completed,
            TaskStatus::Canceled => a2a_rs::types::TaskState::Canceled,
            TaskStatus::Failed => a2a_rs::types::TaskState::Failed,
            TaskStatus::Rejected => a2a_rs::types::TaskState::Rejected,
            TaskStatus::AuthRequired => a2a_rs::types::TaskState::AuthRequired,
            TaskStatus::Unknown => a2a_rs::types::TaskState::Unknown,
        }
    }
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Submitted => "submitted",
            TaskStatus::Working => "working",
            TaskStatus::InputRequired => "input-required",
            TaskStatus::Completed => "completed",
            TaskStatus::Canceled => "canceled",
            TaskStatus::Failed => "failed",
            TaskStatus::Rejected => "rejected",
            TaskStatus::AuthRequired => "auth-required",
            TaskStatus::Unknown => "unknown",
        }
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "submitted" => TaskStatus::Submitted,
            "working" => TaskStatus::Working,
            "input-required" => TaskStatus::InputRequired,
            "completed" => TaskStatus::Completed,
            "canceled" => TaskStatus::Canceled,
            "failed" => TaskStatus::Failed,
            "rejected" => TaskStatus::Rejected,
            "auth-required" => TaskStatus::AuthRequired,
            _ => TaskStatus::Unknown,
        }
    }
}

impl From<&str> for TaskStatus {
    fn from(s: &str) -> Self {
        TaskStatus::from(s.to_string())
    }
}

impl TryInto<libsql::Value> for TaskStatus {
    type Error = libsql::Error;
    fn try_into(self) -> Result<libsql::Value, libsql::Error> {
        Ok(libsql::Value::Text(self.as_str().to_string()))
    }
}

impl FromValue for TaskStatus {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => Ok(TaskStatus::from(s)),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

// Domain models for TaskTimeline

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TaskStatusUpdateTaskTimelineItem {
    pub status: TaskStatus,
    pub status_message_id: Option<WrappedUuidV4>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct MessageTaskTimelineItem {
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum TaskTimelineItemPayload {
    TaskStatusUpdate(TaskStatusUpdateTaskTimelineItem),
    Message(MessageTaskTimelineItem),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TaskTimelineItem {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub event_payload: TaskTimelineItemPayload,
    pub created_at: WrappedChronoDateTime,
}

// Domain models for Message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextPart {
    pub text: String,
    pub metadata: Metadata,
}

impl From<TextPart> for a2a_rs::types::TextPart {
    fn from(value: TextPart) -> Self {
        a2a_rs::types::TextPart {
            text: value.text,
            metadata: value.metadata.0.clone(),
            kind: "text".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum MessagePart {
    TextPart(TextPart),
    // TODO: Add FilePart and DataPart
    // FilePart(FilePart),
    // DataPart(DataPart),
}

impl From<MessagePart> for a2a_rs::types::Part {
    fn from(value: MessagePart) -> Self {
        match value {
            MessagePart::TextPart(text_part) => a2a_rs::types::Part::TextPart(text_part.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MessageRole {
    User,
    Agent,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageRole::User => "user",
                MessageRole::Agent => "agent",
            }
        )
    }
}

impl From<MessageRole> for a2a_rs::types::MessageRole {
    fn from(value: MessageRole) -> Self {
        match value {
            MessageRole::User => a2a_rs::types::MessageRole::User,
            MessageRole::Agent => a2a_rs::types::MessageRole::Agent,
        }
    }
}

impl TryFrom<String> for MessageRole {
    type Error = CommonError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "user" => Ok(MessageRole::User),
            "agent" => Ok(MessageRole::Agent),
            _ => Err(CommonError::Unknown(anyhow::anyhow!(
                "Invalid message role: {s}"
            ))),
        }
    }
}

impl FromStr for MessageRole {
    type Err = CommonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MessageRole::try_from(s.to_string())
    }
}

impl FromValue for MessageRole {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => match s.as_str() {
                "user" => Ok(MessageRole::User),
                "agent" => Ok(MessageRole::Agent),
                _ => Err(libsql::Error::InvalidColumnType),
            },
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<libsql::Value> for MessageRole {
    fn from(val: libsql::Value) -> Self {
        match val {
            libsql::Value::Text(s) => MessageRole::try_from(s).unwrap(),
            _ => MessageRole::User,
        }
    }
}

impl From<MessageRole> for libsql::Value {
    fn from(val: MessageRole) -> Self {
        libsql::Value::Text(val.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Message {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub reference_task_ids: Vec<WrappedUuidV4>,
    pub role: MessageRole,
    pub metadata: Metadata,
    pub parts: Vec<MessagePart>,
    pub created_at: WrappedChronoDateTime,
}

impl From<Message> for a2a_rs::types::Message {
    fn from(value: Message) -> Self {
        a2a_rs::types::Message {
            message_id: value.id.to_string(),
            // TODO: add context_id to message
            context_id: None,
            extensions: vec![],
            kind: "message".to_string(),
            metadata: value.metadata.0.clone(),
            parts: value.parts.into_iter().map(|part| part.into()).collect(),
            reference_task_ids: value
                .reference_task_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            role: value.role.into(),
            task_id: Some(value.task_id.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TaskEventUpdateType {
    TaskStatusUpdate,
    Message,
}

impl TaskEventUpdateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskEventUpdateType::TaskStatusUpdate => "task-status-update",
            TaskEventUpdateType::Message => "message",
        }
    }
}

impl From<String> for TaskEventUpdateType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "task-status-update" => TaskEventUpdateType::TaskStatusUpdate,
            "message" => TaskEventUpdateType::Message,
            _ => TaskEventUpdateType::Message,
        }
    }
}

impl From<&str> for TaskEventUpdateType {
    fn from(s: &str) -> Self {
        TaskEventUpdateType::from(s.to_string())
    }
}

pub type ListTasksResponse = PaginatedResponse<Task>;

pub async fn list_tasks(
    repository: &Repository,
    pagination: PaginationRequest,
) -> Result<ListTasksResponse, CommonError> {
    let tasks = repository.get_tasks(&pagination).await?;
    Ok(tasks)
}

pub type ListUniqueContextsResponse = PaginatedResponse<ContextInfo>;

pub async fn list_unique_contexts(
    repository: &Repository,
    pagination: PaginationRequest,
) -> Result<ListUniqueContextsResponse, CommonError> {
    let contexts = repository.get_unique_contexts(&pagination).await?;
    Ok(contexts)
}

pub struct WithContextId<T> {
    pub context_id: WrappedUuidV4,
    pub inner: T,
}

pub type ListTasksByContextIdResponse = PaginatedResponse<Task>;

pub async fn list_tasks_by_context_id(
    repository: &Repository,
    request: WithContextId<PaginationRequest>,
) -> Result<ListTasksByContextIdResponse, CommonError> {
    let tasks = repository
        .get_tasks_by_context_id(&request.context_id, &request.inner)
        .await?;
    Ok(tasks)
}

#[derive(Debug, Deserialize, Serialize, ToSchema, JsonSchema)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
    pub message: Option<CreateMessageRequest>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct WithTaskId<T> {
    pub task_id: WrappedUuidV4,
    pub inner: T,
}

pub type UpdateTaskStatusResponse = ();

pub async fn update_task_status(
    repository: &Repository,
    connection_manager: &ConnectionManager,
    event_queue: Option<a2a_rs::events::EventQueue>,
    request: WithTaskId<UpdateTaskStatusRequest>,
) -> Result<UpdateTaskStatusResponse, CommonError> {
    let task = repository.get_task_by_id(&request.task_id).await?;
    let task = match task {
        Some(task) => task,
        None => {
            return Err(CommonError::NotFound {
                msg: "Task not found".to_string(),
                lookup_id: request.task_id.to_string(),
                source: None,
            });
        }
    };

    let message = match request.inner.message {
        Some(message) => {
            let message = create_message(
                repository,
                connection_manager,
                WithTaskId {
                    task_id: request.task_id.clone(),
                    inner: message,
                },
                true,
            )
            .await?;

            Some(message)
        }
        None => None,
    };

    let now = WrappedChronoDateTime::now();

    let message_id = message.clone().map(|message| message.message.id);
    repository
        .update_task_status(&UpdateTaskStatus {
            id: request.task_id.clone(),
            status: request.inner.status.clone(),
            status_message_id: message_id.clone(),
            status_timestamp: now,
            updated_at: now,
        })
        .await?;

    let timeline_item = TaskTimelineItem {
        id: WrappedUuidV4::new(),
        task_id: request.task_id.clone(),
        event_payload: TaskTimelineItemPayload::TaskStatusUpdate(
            TaskStatusUpdateTaskTimelineItem {
                status: request.inner.status.clone(),
                status_message_id: message_id.clone(),
            },
        ),
        created_at: now,
    };
    repository
        .insert_task_timeline_item(&timeline_item.try_into()?)
        .await?;
    connection_manager
        .message_to_connections(
            request.task_id.clone(),
            a2a_rs::events::Event::TaskStatusUpdate(a2a_rs::types::TaskStatusUpdateEvent {
                context_id: task.task.context_id.to_string(),
                final_: matches!(
                    request.inner.status,
                    TaskStatus::Completed
                        | TaskStatus::Failed
                        | TaskStatus::Canceled
                        | TaskStatus::Rejected
                ),
                kind: "status-update".to_string(),
                metadata: task.task.metadata.0.clone(),
                status: a2a_rs::types::TaskStatus {
                    message: message.clone().map(|message| message.message.into()),
                    state: request.inner.status.clone().into(),
                    timestamp: Some(now.to_string()),
                },
                task_id: task.task.id.to_string(),
            }),
        )
        .await?;
    if let Some(event_queue) = event_queue {
        event_queue
            .enqueue_event(a2a_rs::events::Event::TaskStatusUpdate(
                TaskStatusUpdateEvent {
                    context_id: task.task.context_id.to_string(),
                    final_: matches!(
                        request.inner.status,
                        TaskStatus::Completed
                            | TaskStatus::Failed
                            | TaskStatus::Canceled
                            | TaskStatus::Rejected
                    ),
                    kind: "status-update".to_string(),
                    metadata: task.task.metadata.0.clone(),
                    status: a2a_rs::types::TaskStatus {
                        message: message.map(|message| message.message.into()),
                        state: request.inner.status.clone().into(),
                        timestamp: Some(now.to_string()),
                    },
                    task_id: request.task_id.to_string(),
                },
            ))
            .await?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, ToSchema, JsonSchema)]
pub struct CreateMessageRequest {
    pub reference_task_ids: Vec<WrappedUuidV4>,
    pub role: MessageRole,
    pub metadata: Metadata,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateMessageResponse {
    pub message: Message,
    pub timeline_item: TaskTimelineItem,
}

pub async fn create_message(
    repository: &Repository,
    connection_manager: &ConnectionManager,
    request: WithTaskId<CreateMessageRequest>,
    is_associated_with_status_update: bool,
) -> Result<CreateMessageResponse, CommonError> {
    let message = Message {
        id: WrappedUuidV4::new(),
        task_id: request.task_id.clone(),
        reference_task_ids: request.inner.reference_task_ids,
        role: request.inner.role,
        metadata: request.inner.metadata,
        parts: request.inner.parts,
        created_at: WrappedChronoDateTime::now(),
    };
    repository
        .insert_message(&message.clone().try_into()?)
        .await?;

    let timeline_item = TaskTimelineItem {
        id: WrappedUuidV4::new(),
        task_id: request.task_id.clone(),
        event_payload: TaskTimelineItemPayload::Message(MessageTaskTimelineItem {
            message: message.clone(),
        }),
        created_at: WrappedChronoDateTime::now(),
    };
    repository
        .insert_task_timeline_item(&timeline_item.clone().try_into()?)
        .await?;

    if !is_associated_with_status_update {
        connection_manager
            .message_to_connections(
                request.task_id,
                a2a_rs::events::Event::Message(message.clone().into()),
            )
            .await?;
    }

    Ok(CreateMessageResponse {
        message,
        timeline_item,
    })
}

pub type GetTaskTimelineItemsRequest = WithTaskId<PaginationRequest>;
pub type GetTaskTimelineItemsResponse = PaginatedResponse<TaskTimelineItem>;

pub async fn get_task_timeline_items(
    repository: &Repository,
    request: GetTaskTimelineItemsRequest,
) -> Result<GetTaskTimelineItemsResponse, CommonError> {
    let timeline_items = repository
        .get_task_timeline_items(&request.task_id, &request.inner)
        .await?;
    Ok(timeline_items)
}

pub type GetTaskResponse = TaskWithDetails;

pub async fn get_task(
    repository: &Repository,
    task_id: WrappedUuidV4,
) -> Result<GetTaskResponse, CommonError> {
    let task = repository.get_task_by_id(&task_id).await?;

    match task {
        Some(task) => Ok(task),
        None => Err(CommonError::NotFound {
            msg: "Task not found".to_string(),
            lookup_id: task_id.to_string(),
            source: None,
        }),
    }
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::repository::{CreateTask, TaskRepositoryLike};
    use shared::primitives::{
        PaginationRequest, SqlMigrationLoader, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    };
    use shared::test_utils::repository::setup_in_memory_database;

    // Test helper to create a test repository
    async fn setup_test_repo() -> Repository {
        let (_db, conn) = setup_in_memory_database(vec![
            Repository::load_sql_migrations(),
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        Repository::new(conn)
    }

    // Test helper to create a test task
    async fn create_test_task(repo: &Repository) -> Task {
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        Task {
            id: task_id,
            context_id,
            status,
            status_timestamp: created_at,
            status_message_id: None,
            metadata,
            created_at,
            updated_at,
        }
    }

    #[tokio::test]
    async fn test_list_tasks_empty() {
        let repo = setup_test_repo().await;
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = list_tasks(&repo, pagination).await.unwrap();

        assert_eq!(result.items.len(), 0);
        assert!(result.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_list_tasks_with_data() {
        let repo = setup_test_repo().await;

        // Create multiple tasks
        let task1 = create_test_task(&repo).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let task2 = create_test_task(&repo).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let task3 = create_test_task(&repo).await;

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = list_tasks(&repo, pagination).await.unwrap();

        assert_eq!(result.items.len(), 3);
        assert!(result.next_page_token.is_none());

        // Verify task IDs are present
        let task_ids: Vec<_> = result.items.iter().map(|t| t.id.clone()).collect();
        assert!(task_ids.contains(&task1.id));
        assert!(task_ids.contains(&task2.id));
        assert!(task_ids.contains(&task3.id));
    }

    #[tokio::test]
    async fn test_list_tasks_pagination() {
        let repo = setup_test_repo().await;

        // Create 5 tasks
        for _ in 0..5 {
            create_test_task(&repo).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = list_tasks(&repo, pagination).await.unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get next page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = list_tasks(&repo, pagination).await.unwrap();

        assert!(result.items.len() >= 2);
    }

    #[tokio::test]
    async fn test_update_task_status_without_message() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        // Update status without message
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: UpdateTaskStatusRequest {
                status: TaskStatus::Working,
                message: None,
            },
        };

        let result = update_task_status(&repo, &connection_manager, None, request).await;
        assert!(result.is_ok());

        // Verify status was updated
        let updated_task = repo.get_task_by_id(&task.id).await.unwrap().unwrap();
        assert_eq!(updated_task.task.status, TaskStatus::Working);
        assert!(updated_task.status_message.is_none());
    }

    #[tokio::test]
    async fn test_update_task_status_with_message() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        // Update status with message
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: UpdateTaskStatusRequest {
                status: TaskStatus::Completed,
                message: Some(CreateMessageRequest {
                    reference_task_ids: vec![],
                    role: MessageRole::Agent,
                    metadata: Metadata::new(),
                    parts: vec![MessagePart::TextPart(TextPart {
                        text: "Task completed successfully".to_string(),
                        metadata: Metadata::new(),
                    })],
                }),
            },
        };

        let result = update_task_status(&repo, &connection_manager, None, request).await;
        assert!(result.is_ok());

        // Verify status was updated with message
        let updated_task = repo.get_task_by_id(&task.id).await.unwrap().unwrap();
        assert_eq!(updated_task.task.status, TaskStatus::Completed);
        assert!(updated_task.status_message.is_some());

        let status_message = updated_task.status_message.unwrap();
        assert_eq!(status_message.role, MessageRole::Agent);
        assert_eq!(status_message.parts.len(), 1);
    }

    #[tokio::test]
    async fn test_update_task_status_not_found() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let non_existent_id = WrappedUuidV4::new();

        let request = WithTaskId {
            task_id: non_existent_id.clone(),
            inner: UpdateTaskStatusRequest {
                status: TaskStatus::Working,
                message: None,
            },
        };

        let result = update_task_status(&repo, &connection_manager, None, request).await;
        assert!(result.is_err());

        match result {
            Err(CommonError::NotFound { lookup_id, .. }) => {
                assert_eq!(lookup_id, non_existent_id.to_string());
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_create_message() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: CreateMessageRequest {
                reference_task_ids: vec![],
                role: MessageRole::User,
                metadata: Metadata::new(),
                parts: vec![MessagePart::TextPart(TextPart {
                    text: "Hello, agent!".to_string(),
                    metadata: Metadata::new(),
                })],
            },
        };

        let result = create_message(&repo, &connection_manager, request, false).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        let message = message.message;
        assert_eq!(message.task_id, task.id);
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.parts.len(), 1);

        // Verify message was persisted
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let messages = repo
            .get_messages_by_task_id(&task.id, &pagination)
            .await
            .unwrap();
        assert_eq!(messages.items.len(), 1);
        assert_eq!(messages.items[0].id, message.id);
    }

    #[tokio::test]
    async fn test_create_message_with_reference_tasks() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;
        let ref_task1 = create_test_task(&repo).await;
        let ref_task2 = create_test_task(&repo).await;

        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: CreateMessageRequest {
                reference_task_ids: vec![ref_task1.id.clone(), ref_task2.id.clone()],
                role: MessageRole::Agent,
                metadata: Metadata::new(),
                parts: vec![MessagePart::TextPart(TextPart {
                    text: "Referencing other tasks".to_string(),
                    metadata: Metadata::new(),
                })],
            },
        };

        let result = create_message(&repo, &connection_manager, request, false).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        let message = message.message;
        assert_eq!(message.reference_task_ids.len(), 2);
        assert!(message.reference_task_ids.contains(&ref_task1.id));
        assert!(message.reference_task_ids.contains(&ref_task2.id));
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_empty() {
        let repo = setup_test_repo().await;
        let task = create_test_task(&repo).await;

        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        };

        let result = get_task_timeline_items(&repo, request).await.unwrap();

        assert_eq!(result.items.len(), 0);
        assert!(result.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_with_messages() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        // Create a message (which creates a timeline item)
        let message_request = WithTaskId {
            task_id: task.id.clone(),
            inner: CreateMessageRequest {
                reference_task_ids: vec![],
                role: MessageRole::User,
                metadata: Metadata::new(),
                parts: vec![MessagePart::TextPart(TextPart {
                    text: "Test message".to_string(),
                    metadata: Metadata::new(),
                })],
            },
        };
        create_message(&repo, &connection_manager, message_request, false)
            .await
            .unwrap();

        // Get timeline items
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        };

        let result = get_task_timeline_items(&repo, request).await.unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());

        // Verify it's a message timeline item
        match &result.items[0].event_payload {
            TaskTimelineItemPayload::Message(_) => {}
            _ => panic!("Expected Message timeline item"),
        }
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_with_status_updates() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        // Update status (which creates a timeline item)
        let status_request = WithTaskId {
            task_id: task.id.clone(),
            inner: UpdateTaskStatusRequest {
                status: TaskStatus::Working,
                message: None,
            },
        };
        update_task_status(&repo, &connection_manager, None, status_request)
            .await
            .unwrap();

        // Get timeline items
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        };

        let result = get_task_timeline_items(&repo, request).await.unwrap();

        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());

        // Verify it's a status update timeline item
        match &result.items[0].event_payload {
            TaskTimelineItemPayload::TaskStatusUpdate(update) => {
                assert_eq!(update.status, TaskStatus::Working);
            }
            _ => panic!("Expected TaskStatusUpdate timeline item"),
        }
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_mixed() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        // Create a message
        let message_request = WithTaskId {
            task_id: task.id.clone(),
            inner: CreateMessageRequest {
                reference_task_ids: vec![],
                role: MessageRole::User,
                metadata: Metadata::new(),
                parts: vec![MessagePart::TextPart(TextPart {
                    text: "Starting work".to_string(),
                    metadata: Metadata::new(),
                })],
            },
        };
        create_message(&repo, &connection_manager, message_request, false)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Update status
        let status_request = WithTaskId {
            task_id: task.id.clone(),
            inner: UpdateTaskStatusRequest {
                status: TaskStatus::Working,
                message: None,
            },
        };
        update_task_status(&repo, &connection_manager, None, status_request)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Create another message
        let message_request2 = WithTaskId {
            task_id: task.id.clone(),
            inner: CreateMessageRequest {
                reference_task_ids: vec![],
                role: MessageRole::Agent,
                metadata: Metadata::new(),
                parts: vec![MessagePart::TextPart(TextPart {
                    text: "Working on it".to_string(),
                    metadata: Metadata::new(),
                })],
            },
        };
        create_message(&repo, &connection_manager, message_request2, false)
            .await
            .unwrap();

        // Get timeline items
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        };

        let result = get_task_timeline_items(&repo, request).await.unwrap();

        assert_eq!(result.items.len(), 3);
        assert!(result.next_page_token.is_none());

        // Verify we have mixed types
        let mut has_message = false;
        let mut has_status_update = false;

        for item in &result.items {
            match &item.event_payload {
                TaskTimelineItemPayload::Message(_) => has_message = true,
                TaskTimelineItemPayload::TaskStatusUpdate(_) => has_status_update = true,
            }
        }

        assert!(has_message);
        assert!(has_status_update);
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_pagination() {
        let repo = setup_test_repo().await;
        let connection_manager = ConnectionManager::new();
        let task = create_test_task(&repo).await;

        // Create multiple timeline items
        for i in 0..5 {
            let message_request = WithTaskId {
                task_id: task.id.clone(),
                inner: CreateMessageRequest {
                    reference_task_ids: vec![],
                    role: MessageRole::User,
                    metadata: Metadata::new(),
                    parts: vec![MessagePart::TextPart(TextPart {
                        text: format!("Message {i}"),
                        metadata: Metadata::new(),
                    })],
                },
            };
            create_message(&repo, &connection_manager, message_request, false)
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: PaginationRequest {
                page_size: 2,
                next_page_token: None,
            },
        };

        let result = get_task_timeline_items(&repo, request).await.unwrap();

        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get next page
        let request = WithTaskId {
            task_id: task.id.clone(),
            inner: PaginationRequest {
                page_size: 2,
                next_page_token: result.next_page_token,
            },
        };

        let result = get_task_timeline_items(&repo, request).await.unwrap();

        assert!(result.items.len() >= 2);
    }

    #[tokio::test]
    async fn test_list_unique_contexts() {
        let repo = setup_test_repo().await;

        // Create tasks with 2 different context_ids
        let context_id_1 = WrappedUuidV4::new();
        let context_id_2 = WrappedUuidV4::new();

        // Create 2 tasks with context_id_1
        for _ in 0..2 {
            let task_id = WrappedUuidV4::new();
            let status = TaskStatus::Working;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id_1.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Create 1 task with context_id_2
        let task_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id_2.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get unique contexts
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = list_unique_contexts(&repo, pagination).await.unwrap();

        // Should have 3 entries (2 for context_id_1 with different created_at, 1 for context_id_2)
        // This is because the query does DISTINCT on (context_id, created_at)
        assert_eq!(result.items.len(), 3);

        // Verify both context_ids are present
        let context_ids: Vec<_> = result.items.iter().map(|c| c.context_id.clone()).collect();
        assert!(context_ids.contains(&context_id_1));
        assert!(context_ids.contains(&context_id_2));

        // Verify all items have created_at
        for item in &result.items {
            assert!(item.created_at.get_inner().timestamp() > 0);
        }
    }

    #[tokio::test]
    async fn test_list_tasks_by_context_id() {
        let repo = setup_test_repo().await;

        let context_id_1 = WrappedUuidV4::new();
        let context_id_2 = WrappedUuidV4::new();

        // Create 3 tasks with context_id_1
        let mut task_ids_1 = vec![];
        for _ in 0..3 {
            let task_id = WrappedUuidV4::new();
            task_ids_1.push(task_id.clone());
            let status = TaskStatus::Working;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id_1.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Create 2 tasks with context_id_2
        for _ in 0..2 {
            let task_id = WrappedUuidV4::new();
            let status = TaskStatus::Submitted;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id_2.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get tasks for context_id_1
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = list_tasks_by_context_id(
            &repo,
            WithContextId {
                context_id: context_id_1.clone(),
                inner: pagination,
            },
        )
        .await
        .unwrap();

        // Should have 3 tasks
        assert_eq!(result.items.len(), 3);

        // All tasks should belong to context_id_1
        for task in &result.items {
            assert_eq!(task.context_id, context_id_1);
        }

        // Verify all task IDs are present
        let retrieved_ids: Vec<_> = result.items.iter().map(|t| t.id.clone()).collect();
        for task_id in &task_ids_1 {
            assert!(retrieved_ids.contains(task_id));
        }
    }

    #[tokio::test]
    async fn test_list_tasks_by_context_id_pagination() {
        let repo = setup_test_repo().await;

        let context_id = WrappedUuidV4::new();

        // Create 5 tasks with the same context_id
        for _ in 0..5 {
            let task_id = WrappedUuidV4::new();
            let status = TaskStatus::Working;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Test pagination - get first page with smaller page size
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: None,
        };
        let result = list_tasks_by_context_id(
            &repo,
            WithContextId {
                context_id: context_id.clone(),
                inner: pagination,
            },
        )
        .await
        .unwrap();

        assert_eq!(result.items.len(), 3);
        assert!(result.next_page_token.is_some());

        // Get next page
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: result.next_page_token,
        };
        let result = list_tasks_by_context_id(
            &repo,
            WithContextId {
                context_id: context_id.clone(),
                inner: pagination,
            },
        )
        .await
        .unwrap();
        assert!(result.items.len() >= 2 && result.items.len() <= 3);
    }

    #[tokio::test]
    async fn test_connection_manager_add_and_remove() {
        let connection_manager = ConnectionManager::new();
        let task_id = WrappedUuidV4::new();

        // Add connection
        let result = connection_manager.add_connection(task_id.clone());
        assert!(result.is_ok());

        let (connection_id, _receiver) = result.unwrap();

        // Remove connection
        let result = connection_manager.remove_connection(task_id.clone(), connection_id.clone());
        assert!(result.is_ok());

        // Try to remove non-existent connection
        let result = connection_manager.remove_connection(task_id.clone(), WrappedUuidV4::new());
        assert!(result.is_ok()); // remove is idempotent

        // Try to remove from non-existent task
        let result =
            connection_manager.remove_connection(WrappedUuidV4::new(), connection_id.clone());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_manager_message_to_connections() {
        let connection_manager = ConnectionManager::new();
        let task_id = WrappedUuidV4::new();

        // Add two connections
        let (_conn_id1, mut receiver1) =
            connection_manager.add_connection(task_id.clone()).unwrap();
        let (_conn_id2, mut receiver2) =
            connection_manager.add_connection(task_id.clone()).unwrap();

        // Send a message
        let event = a2a_rs::events::Event::Message(a2a_rs::types::Message {
            message_id: "test-msg-id".to_string(),
            context_id: None,
            extensions: vec![],
            kind: "message".to_string(),
            metadata: serde_json::Map::new(),
            parts: vec![],
            reference_task_ids: vec![],
            role: a2a_rs::types::MessageRole::User,
            task_id: Some(task_id.to_string()),
        });

        connection_manager
            .message_to_connections(task_id.clone(), event.clone())
            .await
            .unwrap();

        // Both receivers should get the message
        tokio::select! {
            msg = receiver1.recv() => {
                assert!(msg.is_some());
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                panic!("Receiver 1 did not receive message");
            }
        }

        tokio::select! {
            msg = receiver2.recv() => {
                assert!(msg.is_some());
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                panic!("Receiver 2 did not receive message");
            }
        }
    }
}
