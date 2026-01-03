use a2a_core::types::TaskStatusUpdateEvent;
use libsql::FromValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr};
use tracing::info;
use utoipa::ToSchema;

use crate::logic::ConnectionManager;
use crate::repository::{Repository, TaskRepositoryLike, UpdateTaskStatus};
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedUuidV4},
};

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

/// Domain model for Task
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

/// Minimal context information for grouping tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ContextInfo {
    pub context_id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
}

/// Task with additional details like messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TaskWithDetails {
    pub task: Task,
    pub status_message: Option<Message>,
    pub messages: Vec<Message>,
    pub messages_next_page_token: Option<String>,
}

impl From<TaskWithDetails> for a2a_core::types::Task {
    fn from(value: TaskWithDetails) -> Self {
        a2a_core::types::Task {
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
            status: a2a_core::types::TaskStatus {
                message: value.status_message.map(|message| message.into()),
                state: value.task.status.into(),
                timestamp: Some(value.task.status_timestamp.to_string()),
            },
        }
    }
}

/// Task status enum matching A2A protocol states
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

impl From<TaskStatus> for a2a_core::types::TaskState {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Submitted => a2a_core::types::TaskState::Submitted,
            TaskStatus::Working => a2a_core::types::TaskState::Working,
            TaskStatus::InputRequired => a2a_core::types::TaskState::InputRequired,
            TaskStatus::Completed => a2a_core::types::TaskState::Completed,
            TaskStatus::Canceled => a2a_core::types::TaskState::Canceled,
            TaskStatus::Failed => a2a_core::types::TaskState::Failed,
            TaskStatus::Rejected => a2a_core::types::TaskState::Rejected,
            TaskStatus::AuthRequired => a2a_core::types::TaskState::AuthRequired,
            TaskStatus::Unknown => a2a_core::types::TaskState::Unknown,
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

/// Task status update timeline item payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TaskStatusUpdateTaskTimelineItem {
    pub status: TaskStatus,
    pub status_message_id: Option<WrappedUuidV4>,
}

/// Message timeline item payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct MessageTaskTimelineItem {
    pub message: Message,
}

/// Task timeline item payload discriminated union
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum TaskTimelineItemPayload {
    TaskStatusUpdate(TaskStatusUpdateTaskTimelineItem),
    Message(MessageTaskTimelineItem),
}

/// Task timeline item for tracking task history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TaskTimelineItem {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub event_payload: TaskTimelineItemPayload,
    pub created_at: WrappedChronoDateTime,
}

/// Text part of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextPart {
    pub text: String,
    pub metadata: Metadata,
}

impl From<TextPart> for a2a_core::types::TextPart {
    fn from(value: TextPart) -> Self {
        a2a_core::types::TextPart {
            text: value.text,
            metadata: value.metadata.0.clone(),
            kind: "text".to_string(),
        }
    }
}

/// Message part discriminated union
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum MessagePart {
    TextPart(TextPart),
    // TODO: Add FilePart and DataPart
}

impl From<MessagePart> for a2a_core::types::Part {
    fn from(value: MessagePart) -> Self {
        match value {
            MessagePart::TextPart(text_part) => a2a_core::types::Part::TextPart(text_part.into()),
        }
    }
}

/// Message role (user or agent)
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

impl From<MessageRole> for a2a_core::types::MessageRole {
    fn from(value: MessageRole) -> Self {
        match value {
            MessageRole::User => a2a_core::types::MessageRole::User,
            MessageRole::Agent => a2a_core::types::MessageRole::Agent,
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

/// Message domain model
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

impl From<Message> for a2a_core::types::Message {
    fn from(value: Message) -> Self {
        a2a_core::types::Message {
            message_id: value.id.to_string(),
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

/// Task event update type for timeline items
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

impl TryInto<libsql::Value> for TaskEventUpdateType {
    type Error = libsql::Error;
    fn try_into(self) -> Result<libsql::Value, libsql::Error> {
        Ok(libsql::Value::Text(self.as_str().to_string()))
    }
}

impl FromValue for TaskEventUpdateType {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => Ok(TaskEventUpdateType::from(s)),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

pub type ListTasksResponse = PaginatedResponse<Task>;

/// List all tasks with pagination
pub async fn list_tasks(
    repository: &Repository,
    pagination: PaginationRequest,
) -> Result<ListTasksResponse, CommonError> {
    let tasks = repository.get_tasks(&pagination).await?;
    Ok(tasks)
}

pub type ListUniqueContextsResponse = PaginatedResponse<ContextInfo>;

/// List unique task contexts with pagination
pub async fn list_unique_contexts(
    repository: &Repository,
    pagination: PaginationRequest,
) -> Result<ListUniqueContextsResponse, CommonError> {
    let contexts = repository.get_unique_contexts(&pagination).await?;
    Ok(contexts)
}

/// Wrapper for request with context ID
pub struct WithContextId<T> {
    pub context_id: WrappedUuidV4,
    pub inner: T,
}

pub type ListTasksByContextIdResponse = PaginatedResponse<Task>;

/// List tasks by context ID with pagination
pub async fn list_tasks_by_context_id(
    repository: &Repository,
    request: WithContextId<PaginationRequest>,
) -> Result<ListTasksByContextIdResponse, CommonError> {
    let tasks = repository
        .get_tasks_by_context_id(&request.context_id, &request.inner)
        .await?;
    Ok(tasks)
}

/// Request to update task status
#[derive(Debug, Deserialize, Serialize, ToSchema, JsonSchema)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
    pub message: Option<CreateMessageRequest>,
}

/// Wrapper for request with task ID
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct WithTaskId<T> {
    pub task_id: WrappedUuidV4,
    pub inner: T,
}

pub type UpdateTaskStatusResponse = ();

/// Update task status with optional message
pub async fn update_task_status(
    repository: &Repository,
    connection_manager: &ConnectionManager,
    event_queue: Option<a2a_core::events::EventQueue>,
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

    info!(
        task_id = %request.task_id,
        status = request.inner.status.as_str(),
        "Task status updated"
    );

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
            a2a_core::events::Event::TaskStatusUpdate(a2a_core::types::TaskStatusUpdateEvent {
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
                status: a2a_core::types::TaskStatus {
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
            .enqueue_event(a2a_core::events::Event::TaskStatusUpdate(
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
                    status: a2a_core::types::TaskStatus {
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

/// Request to create a message
#[derive(Debug, Deserialize, Serialize, ToSchema, JsonSchema)]
pub struct CreateMessageRequest {
    pub reference_task_ids: Vec<WrappedUuidV4>,
    pub role: MessageRole,
    pub metadata: Metadata,
    pub parts: Vec<MessagePart>,
}

/// Response from creating a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateMessageResponse {
    pub message: Message,
    pub timeline_item: TaskTimelineItem,
}

/// Create a message for a task
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

    info!(
        task_id = %request.task_id,
        message_id = %message.id,
        role = %message.role,
        "Message created"
    );

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
                a2a_core::events::Event::Message(message.clone().into()),
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

/// Get task timeline items with pagination
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

/// Get a task by ID with details
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
