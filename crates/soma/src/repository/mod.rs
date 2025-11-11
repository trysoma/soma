mod sqlite;

use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};

pub use sqlite::Repository;

use crate::logic::{
    Message, MessagePart, MessageRole, Task, TaskEventUpdateType, TaskStatus, TaskTimelineItem,
    TaskTimelineItemPayload, TaskWithDetails,
};

// Repository parameter structs
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

#[derive(Debug)]
pub struct UpdateTaskStatus {
    pub id: WrappedUuidV4,
    pub status: TaskStatus,
    pub status_message_id: Option<WrappedUuidV4>,
    pub status_timestamp: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

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
        // Serialize the entire event_payload enum to preserve the type discriminator
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

#[derive(Debug)]
pub struct CreateMessage {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub reference_task_ids: WrappedJsonValue,
    pub role: MessageRole,
    pub metadata: WrappedJsonValue,
    pub parts: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
}

impl TryFrom<Message> for CreateMessage {
    type Error = CommonError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        Ok(CreateMessage {
            id: message.id,
            task_id: message.task_id,
            reference_task_ids: WrappedJsonValue::new(serde_json::to_value(
                message
                    .reference_task_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>(),
            )?),
            role: message.role,
            metadata: WrappedJsonValue::new(serde_json::to_value(message.metadata)?),
            parts: WrappedJsonValue::new(serde_json::to_value(
                message
                    .parts
                    .into_iter()
                    .collect::<Vec<MessagePart>>(),
            )?),
            created_at: message.created_at,
        })
    }
}

// Repository trait
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
    ) -> Result<PaginatedResponse<crate::logic::ContextInfo>, CommonError>;
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
    async fn get_task_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<TaskWithDetails>, CommonError>;
    async fn insert_message(&self, params: &CreateMessage) -> Result<(), CommonError>;
    #[allow(dead_code)]
    async fn get_messages_by_task_id(
        &self,
        task_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Message>, CommonError>;
}
