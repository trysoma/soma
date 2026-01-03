//! Thread domain model
//!
//! A thread represents a conversation that contains multiple messages.
//! Threads provide grouping and context for related messages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use utoipa::ToSchema;

/// A thread represents a conversation containing related messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Thread {
    pub id: WrappedUuidV4,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
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
    pub metadata: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_settings: Option<Map<String, Value>>,
}

pub type CreateThreadResponse = Thread;
pub type UpdateThreadResponse = Thread;
pub type GetThreadResponse = Thread;

/// Response for listing threads
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListThreadsResponse {
    pub threads: Vec<Thread>,
    pub next_page_token: Option<String>,
}

/// Response for deleting a thread
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DeleteThreadResponse {
    pub success: bool,
}

/// Response for getting a thread with its messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct GetThreadWithMessagesResponse {
    pub thread: Thread,
    pub messages: Vec<super::message::UIMessage>,
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
