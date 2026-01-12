//! Thread domain model and logic
//!
//! A thread represents a conversation that contains multiple messages.
//! Threads provide grouping and context for related messages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::error::CommonError;
use shared::primitives::{
    PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4,
};
use utoipa::ToSchema;

use super::event::InboxEvent;
use crate::repository::{CreateThread, MessageRepositoryLike, ThreadRepositoryLike, UpdateThread};

/// A thread represents a conversation containing related messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Thread {
    pub id: WrappedUuidV4,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    /// Inbox-specific settings that can be added by inbox providers
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub inbox_settings: Map<String, Value>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl Thread {
    /// Create a new thread with an optional title
    pub fn new(title: Option<String>) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: WrappedUuidV4::new(),
            title,
            metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new thread with a specific ID
    pub fn with_id(id: WrappedUuidV4, title: Option<String>) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id,
            title,
            metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Request to create a new thread
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateThreadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<WrappedUuidV4>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(default)]
    pub inbox_settings: Map<String, Value>,
}

/// Request to update an existing thread
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateThreadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_settings: Option<Map<String, Value>>,
}

pub type CreateThreadResponse = Thread;
pub type UpdateThreadResponse = Thread;
pub type GetThreadResponse = Thread;
pub type ListThreadsResponse = PaginatedResponse<Thread>;

/// Response for deleting a thread
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DeleteThreadResponse {
    pub success: bool,
}

/// Response for getting a thread with its messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct GetThreadWithMessagesResponse {
    pub thread: Thread,
    pub messages: Vec<super::message::Message>,
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_thread_new() {
            let thread = Thread::new(Some("Test Thread".to_string()));
            assert_eq!(thread.title, Some("Test Thread".to_string()));
            assert!(thread.metadata.is_none());
            assert!(thread.inbox_settings.is_empty());
        }

        #[test]
        fn test_thread_with_id() {
            let id = WrappedUuidV4::new();
            let thread = Thread::with_id(id.clone(), None);
            assert_eq!(thread.id, id);
            assert!(thread.title.is_none());
        }

        #[test]
        fn test_thread_serialization() {
            let thread = Thread::new(Some("My Thread".to_string()));
            let json = serde_json::to_string(&thread).unwrap();
            assert!(json.contains("\"title\":\"My Thread\""));
        }

        #[test]
        fn test_create_thread_request() {
            let request = CreateThreadRequest {
                id: None,
                title: Some("New Thread".to_string()),
                metadata: None,
                inbox_settings: Map::new(),
            };
            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("\"title\":\"New Thread\""));
        }

        #[test]
        fn test_inbox_settings_in_thread() {
            let mut settings = Map::new();
            settings.insert("slack_channel_id".to_string(), Value::String("C123".to_string()));

            let now = WrappedChronoDateTime::now();
            let thread = Thread {
                id: WrappedUuidV4::new(),
                title: Some("Slack Thread".to_string()),
                metadata: None,
                inbox_settings: settings,
                created_at: now,
                updated_at: now,
            };

            let json = serde_json::to_string(&thread).unwrap();
            assert!(json.contains("\"slack_channel_id\":\"C123\""));
        }
    }
}

// --- Logic Functions ---

/// List threads with pagination
pub async fn list_threads<R: ThreadRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListThreadsResponse, CommonError> {
    let paginated = repository.get_threads(&pagination).await?;
    Ok(ListThreadsResponse {
        items: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// Create a new thread
pub async fn create_thread<R: ThreadRepositoryLike>(
    repository: &R,
    event_bus: &super::event::EventBus,
    request: CreateThreadRequest,
) -> Result<CreateThreadResponse, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = request.id.unwrap_or_default();

    let inbox_settings_json =
        WrappedJsonValue::new(serde_json::to_value(&request.inbox_settings).map_err(|e| {
            CommonError::InvalidRequest {
                msg: format!("Failed to serialize inbox_settings: {e}"),
                source: Some(e.into()),
            }
        })?);

    let thread = Thread {
        id: id.clone(),
        title: request.title.clone(),
        metadata: request.metadata.clone(),
        inbox_settings: request.inbox_settings.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateThread {
        id,
        title: request.title,
        metadata: request.metadata,
        inbox_settings: inbox_settings_json,
        created_at: now,
        updated_at: now,
    };

    repository.create_thread(&create_params).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::thread_created(thread.clone()));

    Ok(thread)
}

/// Get a thread with its messages
pub async fn get_thread_with_messages<R: ThreadRepositoryLike + MessageRepositoryLike>(
    repository: &R,
    thread_id: WrappedUuidV4,
    pagination: PaginationRequest,
) -> Result<GetThreadWithMessagesResponse, CommonError> {
    let thread = repository.get_thread_by_id(&thread_id).await?;
    let thread = thread.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {thread_id} not found"),
        lookup_id: thread_id.to_string(),
        source: None,
    })?;

    let messages = repository
        .get_messages_by_thread(&thread_id, &pagination)
        .await?;

    Ok(GetThreadWithMessagesResponse {
        thread,
        messages: messages.items,
        next_page_token: messages.next_page_token,
    })
}

/// Update an existing thread
pub async fn update_thread<R: ThreadRepositoryLike>(
    repository: &R,
    event_bus: &super::event::EventBus,
    thread_id: WrappedUuidV4,
    request: UpdateThreadRequest,
) -> Result<UpdateThreadResponse, CommonError> {
    let existing = repository.get_thread_by_id(&thread_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {thread_id} not found"),
        lookup_id: thread_id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();
    let new_title = request.title.or(existing.title.clone());
    let new_metadata = request.metadata.or(existing.metadata.clone());
    let new_inbox_settings = request
        .inbox_settings
        .unwrap_or(existing.inbox_settings.clone());

    let inbox_settings_json =
        WrappedJsonValue::new(serde_json::to_value(&new_inbox_settings).map_err(|e| {
            CommonError::InvalidRequest {
                msg: format!("Failed to serialize inbox_settings: {e}"),
                source: Some(e.into()),
            }
        })?);

    let update_params = UpdateThread {
        id: thread_id.clone(),
        title: new_title.clone(),
        metadata: new_metadata.clone(),
        inbox_settings: inbox_settings_json,
        updated_at: now,
    };

    repository.update_thread(&update_params).await?;

    let updated_thread = Thread {
        id: thread_id,
        title: new_title,
        metadata: new_metadata,
        inbox_settings: new_inbox_settings,
        created_at: existing.created_at,
        updated_at: now,
    };

    // Publish event
    let _ = event_bus.publish(InboxEvent::thread_updated(updated_thread.clone()));

    Ok(updated_thread)
}

/// Delete a thread
pub async fn delete_thread<R: ThreadRepositoryLike + MessageRepositoryLike>(
    repository: &R,
    event_bus: &super::event::EventBus,
    thread_id: WrappedUuidV4,
) -> Result<DeleteThreadResponse, CommonError> {
    // Verify thread exists
    let existing = repository.get_thread_by_id(&thread_id).await?;
    let _ = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {thread_id} not found"),
        lookup_id: thread_id.to_string(),
        source: None,
    })?;

    // Delete all messages in the thread first (cascade should handle this, but be explicit)
    repository.delete_messages_by_thread(&thread_id).await?;

    // Delete the thread
    repository.delete_thread(&thread_id).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::thread_deleted(thread_id));

    Ok(DeleteThreadResponse { success: true })
}
