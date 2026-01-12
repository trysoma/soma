//! Task repository module for A2A task operations
//!
//! Provides database abstractions for task storage and retrieval.

mod sqlite;

use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};

pub use sqlite::Repository;

use crate::logic::push_notification::{
    CreatePushNotificationConfig, PushNotificationConfigModel, UpdatePushNotificationConfig,
};
use crate::logic::task::{
    ContextInfo, Task, TaskEventUpdateType, TaskStatus, TaskTimelineItem, TaskTimelineItemPayload,
};

/// Parameters for creating a new task in the repository
#[derive(Debug)]
pub struct CreateTask {
    pub id: WrappedUuidV4,
    pub context_id: WrappedUuidV4,
    pub status: TaskStatus,
    pub status_timestamp: WrappedChronoDateTime,
    pub metadata: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl TryFrom<Task> for CreateTask {
    type Error = CommonError;
    fn try_from(task: Task) -> Result<Self, Self::Error> {
        let metadata: WrappedJsonValue =
            WrappedJsonValue::new(serde_json::to_value(task.metadata)?);
        Ok(CreateTask {
            id: task.id,
            context_id: task.context_id,
            status: task.status,
            status_timestamp: task.status_timestamp,
            metadata,
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
    }
}

/// Parameters for updating a task's status
#[derive(Debug)]
pub struct UpdateTaskStatus {
    pub id: WrappedUuidV4,
    pub status: TaskStatus,
    pub status_timestamp: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for creating a task timeline item
#[derive(Debug)]
pub struct CreateTaskTimelineItem {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub event_update_type: TaskEventUpdateType,
    pub event_payload: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
}

impl TryFrom<TaskTimelineItem> for CreateTaskTimelineItem {
    type Error = CommonError;
    fn try_from(task_timeline_item: TaskTimelineItem) -> Result<Self, Self::Error> {
        let event_update_type = match &task_timeline_item.event_payload {
            TaskTimelineItemPayload::TaskStatusUpdate(_) => TaskEventUpdateType::TaskStatusUpdate,
            TaskTimelineItemPayload::Message(_) => TaskEventUpdateType::Message,
        };
        let event_payload =
            WrappedJsonValue::new(serde_json::to_value(&task_timeline_item.event_payload)?);
        Ok(CreateTaskTimelineItem {
            id: task_timeline_item.id,
            task_id: task_timeline_item.task_id,
            event_update_type,
            event_payload,
            created_at: task_timeline_item.created_at,
        })
    }
}

/// Repository trait for task operations
#[allow(async_fn_in_trait)]
pub trait TaskRepositoryLike {
    async fn create_task(&self, params: &CreateTask) -> Result<(), CommonError>;
    async fn update_task_status(&self, params: &UpdateTaskStatus) -> Result<(), CommonError>;
    async fn insert_task_timeline_item(
        &self,
        params: &CreateTaskTimelineItem,
    ) -> Result<(), CommonError>;
    async fn get_tasks(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Task>, CommonError>;
    async fn get_unique_contexts(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<ContextInfo>, CommonError>;
    async fn get_tasks_by_context_id(
        &self,
        context_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Task>, CommonError>;
    async fn get_task_timeline_items(
        &self,
        task_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<TaskTimelineItem>, CommonError>;
    async fn get_task_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Task>, CommonError>;

    // Push notification config methods
    async fn create_push_notification_config(
        &self,
        params: &CreatePushNotificationConfig,
    ) -> Result<(), CommonError>;
    async fn update_push_notification_config(
        &self,
        params: &UpdatePushNotificationConfig,
    ) -> Result<(), CommonError>;
    async fn get_push_notification_configs_by_task_id(
        &self,
        task_id: &WrappedUuidV4,
    ) -> Result<Vec<PushNotificationConfigModel>, CommonError>;
    async fn get_push_notification_config_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<PushNotificationConfigModel>, CommonError>;
    async fn delete_push_notification_config(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;
    async fn delete_push_notification_configs_by_task_id(
        &self,
        task_id: &WrappedUuidV4,
    ) -> Result<(), CommonError>;
}
