//! Task domain models and logic for the A2A protocol
//!
//! This module provides the core task types and operations used by the A2A protocol.

use crate::a2a_core::types::TaskStatusUpdateEvent;
use libsql::FromValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr};
use tracing::info;
use utoipa::ToSchema;

use crate::logic::connection_manager::ConnectionManager;
use crate::task_repository::{Repository as TaskRepository, TaskRepositoryLike, UpdateTaskStatus};
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
    pub metadata: Metadata,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<Task> for crate::a2a_core::types::Task {
    fn from(value: Task) -> Self {
        crate::a2a_core::types::Task {
            artifacts: vec![],
            context_id: value.context_id.to_string(),
            history: vec![], // Messages are now in inbox, not here
            id: value.id.to_string(),
            kind: "task".to_string(),
            metadata: value.metadata.0.clone(),
            status: crate::a2a_core::types::TaskStatus {
                message: None, // Status messages are now in inbox
                state: value.status.into(),
                timestamp: Some(value.status_timestamp.to_string()),
            },
        }
    }
}

/// Minimal context information for grouping tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ContextInfo {
    pub context_id: WrappedUuidV4,
    pub created_at: WrappedChronoDateTime,
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

impl From<TaskStatus> for crate::a2a_core::types::TaskState {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Submitted => crate::a2a_core::types::TaskState::Submitted,
            TaskStatus::Working => crate::a2a_core::types::TaskState::Working,
            TaskStatus::InputRequired => crate::a2a_core::types::TaskState::InputRequired,
            TaskStatus::Completed => crate::a2a_core::types::TaskState::Completed,
            TaskStatus::Canceled => crate::a2a_core::types::TaskState::Canceled,
            TaskStatus::Failed => crate::a2a_core::types::TaskState::Failed,
            TaskStatus::Rejected => crate::a2a_core::types::TaskState::Rejected,
            TaskStatus::AuthRequired => crate::a2a_core::types::TaskState::AuthRequired,
            TaskStatus::Unknown => crate::a2a_core::types::TaskState::Unknown,
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

    /// Check if this is a terminal/final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Canceled | TaskStatus::Rejected
        )
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
}

/// Message timeline item payload (messages are stored as timeline events)
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

impl From<TextPart> for crate::a2a_core::types::TextPart {
    fn from(value: TextPart) -> Self {
        crate::a2a_core::types::TextPart {
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

impl From<MessagePart> for crate::a2a_core::types::Part {
    fn from(value: MessagePart) -> Self {
        match value {
            MessagePart::TextPart(text_part) => crate::a2a_core::types::Part::TextPart(text_part.into()),
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

impl From<MessageRole> for crate::a2a_core::types::MessageRole {
    fn from(value: MessageRole) -> Self {
        match value {
            MessageRole::User => crate::a2a_core::types::MessageRole::User,
            MessageRole::Agent => crate::a2a_core::types::MessageRole::Agent,
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

/// Message domain model (used in timeline payloads)
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

impl From<Message> for crate::a2a_core::types::Message {
    fn from(value: Message) -> Self {
        crate::a2a_core::types::Message {
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
    repository: &TaskRepository,
    pagination: PaginationRequest,
) -> Result<ListTasksResponse, CommonError> {
    let tasks = repository.get_tasks(&pagination).await?;
    Ok(tasks)
}

pub type ListUniqueContextsResponse = PaginatedResponse<ContextInfo>;

/// List unique task contexts with pagination
pub async fn list_unique_contexts(
    repository: &TaskRepository,
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
    repository: &TaskRepository,
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
}

/// Wrapper for request with task ID
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct WithTaskId<T> {
    pub task_id: WrappedUuidV4,
    pub inner: T,
}

pub type UpdateTaskStatusResponse = ();

/// Update task status
pub async fn update_task_status(
    repository: &TaskRepository,
    connection_manager: &ConnectionManager,
    event_queue: Option<crate::a2a_core::events::EventQueue>,
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

    let now = WrappedChronoDateTime::now();

    repository
        .update_task_status(&UpdateTaskStatus {
            id: request.task_id.clone(),
            status: request.inner.status.clone(),
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
            crate::a2a_core::events::Event::TaskStatusUpdate(crate::a2a_core::types::TaskStatusUpdateEvent {
                context_id: task.context_id.to_string(),
                final_: request.inner.status.is_final(),
                kind: "status-update".to_string(),
                metadata: task.metadata.0.clone(),
                status: crate::a2a_core::types::TaskStatus {
                    message: None,
                    state: request.inner.status.clone().into(),
                    timestamp: Some(now.to_string()),
                },
                task_id: task.id.to_string(),
            }),
        )
        .await?;
    if let Some(event_queue) = event_queue {
        event_queue
            .enqueue_event(crate::a2a_core::events::Event::TaskStatusUpdate(
                TaskStatusUpdateEvent {
                    context_id: task.context_id.to_string(),
                    final_: request.inner.status.is_final(),
                    kind: "status-update".to_string(),
                    metadata: task.metadata.0.clone(),
                    status: crate::a2a_core::types::TaskStatus {
                        message: None,
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

pub type GetTaskTimelineItemsRequest = WithTaskId<PaginationRequest>;
pub type GetTaskTimelineItemsResponse = PaginatedResponse<TaskTimelineItem>;

/// Get task timeline items with pagination
pub async fn get_task_timeline_items(
    repository: &TaskRepository,
    request: GetTaskTimelineItemsRequest,
) -> Result<GetTaskTimelineItemsResponse, CommonError> {
    let timeline_items = repository
        .get_task_timeline_items(&request.task_id, &request.inner)
        .await?;
    Ok(timeline_items)
}

pub type GetTaskResponse = Task;

/// Get a task by ID
pub async fn get_task(
    repository: &TaskRepository,
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
