mod sqlite;

use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};

pub use sqlite::Repository;
use tracing::info;

use crate::logic::environment_variable::EnvironmentVariable;
use crate::logic::secret::Secret;
use crate::logic::task::{
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
                message.parts.into_iter().collect::<Vec<MessagePart>>(),
            )?),
            created_at: message.created_at,
        })
    }
}

// Secret repository parameter structs
#[derive(Debug)]
pub struct CreateSecret {
    pub id: WrappedUuidV4,
    pub key: String,
    pub encrypted_secret: String,
    pub dek_alias: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct UpdateSecret {
    pub id: WrappedUuidV4,
    pub encrypted_secret: String,
    pub dek_alias: String,
    pub updated_at: WrappedChronoDateTime,
}

// Repository trait
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
    ) -> Result<PaginatedResponse<crate::logic::task::ContextInfo>, CommonError>;
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

// Secret repository trait
#[allow(async_fn_in_trait)]
pub trait SecretRepositoryLike: Send + Sync {
    async fn create_secret(&self, params: &CreateSecret) -> Result<(), CommonError>;
    async fn update_secret(&self, params: &UpdateSecret) -> Result<(), CommonError>;
    async fn delete_secret(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;
    async fn get_secret_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Secret>, CommonError>;
    async fn get_secret_by_key(&self, key: &str) -> Result<Option<Secret>, CommonError>;
    async fn get_secrets(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Secret>, CommonError>;
}

// Environment variable repository parameter structs
#[derive(Debug)]
pub struct CreateEnvironmentVariable {
    pub id: WrappedUuidV4,
    pub key: String,
    pub value: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct UpdateEnvironmentVariable {
    pub id: WrappedUuidV4,
    pub value: String,
    pub updated_at: WrappedChronoDateTime,
}

// Environment variable repository trait
#[allow(async_fn_in_trait)]
pub trait EnvironmentVariableRepositoryLike {
    async fn create_environment_variable(
        &self,
        params: &CreateEnvironmentVariable,
    ) -> Result<(), CommonError>;
    async fn update_environment_variable(
        &self,
        params: &UpdateEnvironmentVariable,
    ) -> Result<(), CommonError>;
    async fn delete_environment_variable(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;
    async fn get_environment_variable_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<EnvironmentVariable>, CommonError>;
    async fn get_environment_variable_by_key(
        &self,
        key: &str,
    ) -> Result<Option<EnvironmentVariable>, CommonError>;
    async fn get_environment_variables(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<EnvironmentVariable>, CommonError>;
}

// Repository setup utilities
use shared::libsql::{
    establish_db_connection, inject_auth_token_to_db_url, merge_nested_migrations,
};
use shared::primitives::SqlMigrationLoader;
use url::Url;

/// Sets up the database repository and runs migrations
pub async fn setup_repository(
    conn_string: &Url,
    auth_token: &Option<String>,
) -> Result<
    (
        libsql::Database,
        shared::libsql::Connection,
        Repository,
        bridge::repository::Repository,
        encryption::repository::Repository,
    ),
    CommonError,
> {
    info!("Setting up database repository...");
    info!("conn_string: {}", conn_string);
    let migrations = merge_nested_migrations(vec![
        Repository::load_sql_migrations(),
        bridge::repository::Repository::load_sql_migrations(),
        <encryption::repository::Repository as SqlMigrationLoader>::load_sql_migrations(),
        identity::repository::Repository::load_sql_migrations(),
    ]);
    let auth_conn_string = inject_auth_token_to_db_url(conn_string, auth_token)?;
    let (db, conn) = establish_db_connection(&auth_conn_string, Some(migrations)).await?;

    let repo = Repository::new(conn.clone());
    let bridge_repo = bridge::repository::Repository::new(conn.clone());
    let encryption_repo = encryption::repository::Repository::new(conn.clone());
    Ok((db, conn, repo, bridge_repo, encryption_repo))
}
