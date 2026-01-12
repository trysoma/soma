//! Repository layer for inbox crate
//! Contains trait definitions and implementations for thread, message, event, and inbox storage

pub mod sqlite;

use async_trait::async_trait;
use serde_json::{Map, Value};
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    },
};

pub use sqlite::Repository;

use crate::logic::{
    inbox::{DestinationType, Inbox},
    message::{Message, MessageRole, MessageType},
    thread::Thread,
};

// --- Thread Repository Types ---

/// Parameters for creating a new thread
#[derive(Debug, Clone)]
pub struct CreateThread {
    pub id: WrappedUuidV4,
    pub title: Option<String>,
    pub metadata: Option<WrappedJsonValue>,
    pub inbox_settings: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for updating an existing thread
#[derive(Debug, Clone)]
pub struct UpdateThread {
    pub id: WrappedUuidV4,
    pub title: Option<String>,
    pub metadata: Option<WrappedJsonValue>,
    pub inbox_settings: WrappedJsonValue,
    pub updated_at: WrappedChronoDateTime,
}

// --- Message Repository Types ---

/// Parameters for creating a new message
#[derive(Debug, Clone)]
pub struct CreateMessage {
    pub id: WrappedUuidV4,
    pub thread_id: WrappedUuidV4,
    pub message_type: MessageType,
    pub role: MessageRole,
    pub body: WrappedJsonValue,
    pub metadata: Option<WrappedJsonValue>,
    pub inbox_settings: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for updating an existing message
#[derive(Debug, Clone)]
pub struct UpdateMessage {
    pub id: WrappedUuidV4,
    pub body: WrappedJsonValue,
    pub metadata: Option<WrappedJsonValue>,
    pub inbox_settings: WrappedJsonValue,
    pub updated_at: WrappedChronoDateTime,
}

// --- Event Repository Types ---

/// Parameters for creating a new event
#[derive(Debug, Clone)]
pub struct CreateEvent {
    pub id: WrappedUuidV4,
    pub kind: String,
    pub payload: WrappedJsonValue,
    pub inbox_id: Option<String>,
    pub inbox_settings: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
}

/// Stored event representation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, schemars::JsonSchema)]
pub struct StoredEvent {
    pub id: WrappedUuidV4,
    pub kind: String,
    #[schemars(with = "serde_json::Value")]
    pub payload: WrappedJsonValue,
    pub inbox_id: Option<String>,
    pub inbox_settings: Map<String, Value>,
    pub created_at: WrappedChronoDateTime,
}

// --- Inbox Repository Types ---

/// Parameters for creating a new inbox
#[derive(Debug, Clone)]
pub struct CreateInbox {
    pub id: String,
    pub provider_id: String,
    pub destination_type: DestinationType,
    pub destination_id: String,
    pub configuration: WrappedJsonValue,
    pub settings: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for updating an existing inbox
#[derive(Debug, Clone)]
pub struct UpdateInbox {
    pub id: String,
    pub configuration: WrappedJsonValue,
    pub settings: WrappedJsonValue,
    pub updated_at: WrappedChronoDateTime,
}

// --- Repository Traits ---

/// Repository trait for thread operations
#[async_trait]
pub trait ThreadRepositoryLike: Send + Sync {
    /// Create a new thread
    async fn create_thread(&self, params: &CreateThread) -> Result<(), CommonError>;

    /// Update an existing thread
    async fn update_thread(&self, params: &UpdateThread) -> Result<(), CommonError>;

    /// Delete a thread by ID
    async fn delete_thread(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    /// Get a thread by ID
    async fn get_thread_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Thread>, CommonError>;

    /// List threads with pagination
    async fn get_threads(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Thread>, CommonError>;
}

/// Repository trait for message operations
#[async_trait]
pub trait MessageRepositoryLike: Send + Sync {
    /// Create a new message
    async fn create_message(&self, params: &CreateMessage) -> Result<(), CommonError>;

    /// Update an existing message
    async fn update_message(&self, params: &UpdateMessage) -> Result<(), CommonError>;

    /// Delete a message by ID
    async fn delete_message(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    /// Get a message by ID
    async fn get_message_by_id(&self, id: &WrappedUuidV4)
        -> Result<Option<Message>, CommonError>;

    /// List messages with pagination
    async fn get_messages(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Message>, CommonError>;

    /// List messages by thread with pagination
    async fn get_messages_by_thread(
        &self,
        thread_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Message>, CommonError>;

    /// Delete all messages in a thread
    async fn delete_messages_by_thread(&self, thread_id: &WrappedUuidV4)
        -> Result<(), CommonError>;
}

/// Repository trait for event operations
#[async_trait]
pub trait EventRepositoryLike: Send + Sync {
    /// Create a new event
    async fn create_event(&self, params: &CreateEvent) -> Result<(), CommonError>;

    /// Get an event by ID
    async fn get_event_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<StoredEvent>, CommonError>;

    /// List events with pagination
    async fn get_events(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<StoredEvent>, CommonError>;

    /// List events by inbox with pagination
    async fn get_events_by_inbox(
        &self,
        inbox_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<StoredEvent>, CommonError>;

    /// List events by kind with pagination
    async fn get_events_by_kind(
        &self,
        kind: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<StoredEvent>, CommonError>;

    /// Delete events before a given date
    async fn delete_events_before(
        &self,
        before_date: &WrappedChronoDateTime,
    ) -> Result<(), CommonError>;
}

/// Repository trait for inbox operations
#[async_trait]
pub trait InboxRepositoryLike: Send + Sync {
    /// Create a new inbox
    async fn create_inbox(&self, params: &CreateInbox) -> Result<(), CommonError>;

    /// Update an existing inbox
    async fn update_inbox(&self, params: &UpdateInbox) -> Result<(), CommonError>;

    /// Delete an inbox by ID
    async fn delete_inbox(&self, id: &str) -> Result<(), CommonError>;

    /// Get an inbox by ID
    async fn get_inbox_by_id(&self, id: &str) -> Result<Option<Inbox>, CommonError>;

    /// List inboxes with pagination
    async fn get_inboxes(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError>;

    /// List inboxes by provider with pagination
    async fn get_inboxes_by_provider(
        &self,
        provider_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError>;

    /// List inboxes by destination with pagination
    async fn get_inboxes_by_destination(
        &self,
        destination_type: &DestinationType,
        destination_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError>;
}

/// Combined repository trait for all inbox operations
#[async_trait]
pub trait InboxFullRepositoryLike:
    ThreadRepositoryLike + MessageRepositoryLike + EventRepositoryLike + InboxRepositoryLike
{
}

// Blanket implementation for any type that implements all traits
impl<T> InboxFullRepositoryLike for T where
    T: ThreadRepositoryLike + MessageRepositoryLike + EventRepositoryLike + InboxRepositoryLike
{
}
