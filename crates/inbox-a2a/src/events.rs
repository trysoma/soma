//! A2A-specific event types
//!
//! These types represent A2A protocol events that are published to the inbox
//! event bus as Custom events.

use inbox::logic::event::InboxEvent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::primitives::WrappedJsonValue;
use utoipa::ToSchema;

// --- Event Type Constants ---

/// Event type for task created
pub const EVENT_TYPE_TASK_CREATED: &str = "a2a.task_created";
/// Event type for task status updated
pub const EVENT_TYPE_TASK_STATUS_UPDATED: &str = "a2a.task_status_updated";
/// Event type for artifact created
pub const EVENT_TYPE_ARTIFACT_CREATED: &str = "a2a.artifact_created";
/// Event type for artifact updated
pub const EVENT_TYPE_ARTIFACT_UPDATED: &str = "a2a.artifact_updated";

// --- A2A Task Types ---

/// A2A Task status states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum A2aTaskState {
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

impl std::fmt::Display for A2aTaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            A2aTaskState::Submitted => "submitted",
            A2aTaskState::Working => "working",
            A2aTaskState::InputRequired => "input-required",
            A2aTaskState::Completed => "completed",
            A2aTaskState::Canceled => "canceled",
            A2aTaskState::Failed => "failed",
            A2aTaskState::Rejected => "rejected",
            A2aTaskState::AuthRequired => "auth-required",
            A2aTaskState::Unknown => "unknown",
        };
        write!(f, "{s}")
    }
}

impl A2aTaskState {
    /// Check if this is a terminal/final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            A2aTaskState::Completed
                | A2aTaskState::Canceled
                | A2aTaskState::Failed
                | A2aTaskState::Rejected
        )
    }
}

/// A2A Task status information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct A2aTaskStatus {
    pub state: A2aTaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// A2A Task representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct A2aTask {
    pub id: String,
    pub context_id: String,
    pub status: A2aTaskStatus,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
}

// --- A2A Artifact Types ---

/// A2A Artifact representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct A2aArtifact {
    pub id: String,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Parts of the artifact (text, file, data, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<A2aArtifactPart>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
}

/// A2A Artifact part types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum A2aArtifactPart {
    Text { text: String },
    File {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    Data { data: Value },
}

// --- Event Data Types ---

/// Data payload for task created event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TaskCreatedData {
    pub task: A2aTask,
}

/// Data payload for task status updated event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TaskStatusUpdatedData {
    pub task: A2aTask,
    /// Whether this is a final state (completed, failed, canceled, rejected)
    #[serde(rename = "final")]
    pub is_final: bool,
}

/// Data payload for artifact created event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ArtifactCreatedData {
    pub artifact: A2aArtifact,
}

/// Data payload for artifact updated event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ArtifactUpdatedData {
    pub artifact: A2aArtifact,
    /// Whether this is the last update for this artifact
    pub is_last: bool,
}

// --- Helper Functions ---

/// Create an inbox event for task created
pub fn task_created_event(task: A2aTask) -> InboxEvent {
    let data = TaskCreatedData { task };
    InboxEvent::custom(
        EVENT_TYPE_TASK_CREATED,
        WrappedJsonValue::new(serde_json::to_value(data).unwrap()),
    )
}

/// Create an inbox event for task status updated
pub fn task_status_updated_event(task: A2aTask, is_final: bool) -> InboxEvent {
    let data = TaskStatusUpdatedData { task, is_final };
    InboxEvent::custom(
        EVENT_TYPE_TASK_STATUS_UPDATED,
        WrappedJsonValue::new(serde_json::to_value(data).unwrap()),
    )
}

/// Create an inbox event for artifact created
pub fn artifact_created_event(artifact: A2aArtifact) -> InboxEvent {
    let data = ArtifactCreatedData { artifact };
    InboxEvent::custom(
        EVENT_TYPE_ARTIFACT_CREATED,
        WrappedJsonValue::new(serde_json::to_value(data).unwrap()),
    )
}

/// Create an inbox event for artifact updated
pub fn artifact_updated_event(artifact: A2aArtifact, is_last: bool) -> InboxEvent {
    let data = ArtifactUpdatedData { artifact, is_last };
    InboxEvent::custom(
        EVENT_TYPE_ARTIFACT_UPDATED,
        WrappedJsonValue::new(serde_json::to_value(data).unwrap()),
    )
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_task_state_is_final() {
            assert!(!A2aTaskState::Submitted.is_final());
            assert!(!A2aTaskState::Working.is_final());
            assert!(!A2aTaskState::InputRequired.is_final());
            assert!(A2aTaskState::Completed.is_final());
            assert!(A2aTaskState::Canceled.is_final());
            assert!(A2aTaskState::Failed.is_final());
            assert!(A2aTaskState::Rejected.is_final());
            assert!(!A2aTaskState::AuthRequired.is_final());
            assert!(!A2aTaskState::Unknown.is_final());
        }

        #[test]
        fn test_task_created_event() {
            let task = A2aTask {
                id: "task-123".to_string(),
                context_id: "ctx-456".to_string(),
                status: A2aTaskStatus {
                    state: A2aTaskState::Submitted,
                    message: None,
                    timestamp: None,
                },
                metadata: Map::new(),
            };

            let event = task_created_event(task);

            match event.kind {
                inbox::logic::event::InboxEventKind::Custom { event_type, data } => {
                    assert_eq!(event_type, EVENT_TYPE_TASK_CREATED);
                    let parsed: TaskCreatedData =
                        serde_json::from_value(data.get_inner().clone()).unwrap();
                    assert_eq!(parsed.task.id, "task-123");
                }
                _ => panic!("Expected Custom event"),
            }
        }

        #[test]
        fn test_task_status_updated_event() {
            let task = A2aTask {
                id: "task-123".to_string(),
                context_id: "ctx-456".to_string(),
                status: A2aTaskStatus {
                    state: A2aTaskState::Completed,
                    message: Some("Done!".to_string()),
                    timestamp: None,
                },
                metadata: Map::new(),
            };

            let event = task_status_updated_event(task, true);

            match event.kind {
                inbox::logic::event::InboxEventKind::Custom { event_type, data } => {
                    assert_eq!(event_type, EVENT_TYPE_TASK_STATUS_UPDATED);
                    let parsed: TaskStatusUpdatedData =
                        serde_json::from_value(data.get_inner().clone()).unwrap();
                    assert_eq!(parsed.task.status.state, A2aTaskState::Completed);
                    assert!(parsed.is_final);
                }
                _ => panic!("Expected Custom event"),
            }
        }

        #[test]
        fn test_artifact_serialization() {
            let artifact = A2aArtifact {
                id: "art-123".to_string(),
                task_id: "task-456".to_string(),
                name: Some("result.txt".to_string()),
                description: Some("The output file".to_string()),
                mime_type: Some("text/plain".to_string()),
                parts: vec![A2aArtifactPart::Text {
                    text: "Hello, world!".to_string(),
                }],
                metadata: Map::new(),
            };

            let json = serde_json::to_string(&artifact).unwrap();
            assert!(json.contains("\"id\":\"art-123\""));
            assert!(json.contains("\"kind\":\"text\""));
        }
    }
}
