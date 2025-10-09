mod sqlite;

use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
};
use libsql::FromValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use sqlite::Repository;

// Domain models for Task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Task {
    pub id: WrappedUuidV4,
    pub context_id: WrappedUuidV4,
    pub status: TaskStatus,
    pub metadata: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TaskTimelineItem {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub event_update_type: TaskEventUpdateType,
    pub event_payload: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

// Repository parameter structs
#[derive(Debug)]
pub struct CreateTask<'a> {
    pub id: &'a WrappedUuidV4,
    pub context_id: &'a WrappedUuidV4,
    pub status: &'a TaskStatus,
    pub metadata: &'a WrappedJsonValue,
    pub created_at: &'a WrappedChronoDateTime,
    pub updated_at: &'a WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct UpdateTaskStatus<'a> {
    pub id: &'a WrappedUuidV4,
    pub status: &'a TaskStatus,
    pub updated_at: &'a WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct CreateTaskTimelineItem<'a> {
    pub id: &'a WrappedUuidV4,
    pub task_id: &'a WrappedUuidV4,
    pub event_update_type: &'a TaskEventUpdateType,
    pub event_payload: &'a WrappedJsonValue,
    pub created_at: &'a WrappedChronoDateTime,
}

// Repository trait
pub trait TaskRepositoryLike {
    async fn create_task(&self, params: &CreateTask<'_>) -> Result<(), CommonError>;
    async fn update_task_status(&self, params: &UpdateTaskStatus<'_>) -> Result<(), CommonError>;
    async fn insert_task_timeline_item(&self, params: &CreateTaskTimelineItem<'_>) -> Result<(), CommonError>;
    async fn get_tasks(&self, pagination: &PaginationRequest) -> Result<PaginatedResponse<Task>, CommonError>;
    async fn get_task_timeline_items(
        &self,
        task_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<TaskTimelineItem>, CommonError>;
    async fn get_task_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Task>, CommonError>;
}
