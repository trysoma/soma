//! UIMessage domain model based on Vercel AI SDK specification
//!
//! This module defines the core message types that are compatible with the Vercel AI SDK
//! UIMessage format. Messages consist of parts that can be text, files, tool invocations,
//! reasoning, sources, or custom data parts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use utoipa::ToSchema;

/// Role of the message sender
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "system" => Ok(MessageRole::System),
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            _ => Err(format!("Unknown message role: {s}")),
        }
    }
}

impl libsql::FromValue for MessageRole {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => s.parse().map_err(|_| libsql::Error::InvalidColumnType),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<MessageRole> for libsql::Value {
    fn from(val: MessageRole) -> Self {
        libsql::Value::Text(val.to_string())
    }
}

/// State for streaming parts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PartState {
    Streaming,
    Done,
}

/// Tool invocation state variants
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolInvocationState {
    InputStreaming,
    InputAvailable,
    ApprovalRequested,
    ApprovalResponded,
    OutputAvailable,
    OutputError,
    OutputDenied,
}

/// Approval decision for tool invocations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalDecision {
    Approved,
    Rejected,
}

/// Approval information for tool invocations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Approval {
    pub decision: ApprovalDecision,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Text UI part - represents plain text content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextUIPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<PartState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Reasoning UI part - represents model reasoning/thinking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ReasoningUIPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<PartState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// File UI part - represents file attachments
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct FileUIPart {
    pub media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Tool UI part - represents tool/function invocations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ToolUIPart {
    pub tool_invocation_id: String,
    pub tool_name: String,
    pub state: ToolInvocationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub input: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub output: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<Approval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Source URL UI part - represents a source reference via URL
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SourceUrlUIPart {
    pub source_id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Source document UI part - represents a source reference via document
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SourceDocumentUIPart {
    pub source_id: String,
    pub media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Step start UI part - marks the beginning of an agent step
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct StepStartUIPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Data UI part - represents custom data parts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DataUIPart {
    pub data_type: String,
    #[schemars(with = "serde_json::Value")]
    pub data: WrappedJsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Union of all UI message part types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum UIMessagePart {
    Text(TextUIPart),
    Reasoning(ReasoningUIPart),
    File(FileUIPart),
    Tool(ToolUIPart),
    SourceUrl(SourceUrlUIPart),
    SourceDocument(SourceDocumentUIPart),
    StepStart(StepStartUIPart),
    Data(DataUIPart),
}

impl UIMessagePart {
    /// Create a new text part
    pub fn text(text: impl Into<String>) -> Self {
        UIMessagePart::Text(TextUIPart {
            text: Some(text.into()),
            state: Some(PartState::Done),
            provider_metadata: None,
        })
    }

    /// Create a new streaming text part
    pub fn text_streaming(text: impl Into<String>) -> Self {
        UIMessagePart::Text(TextUIPart {
            text: Some(text.into()),
            state: Some(PartState::Streaming),
            provider_metadata: None,
        })
    }

    /// Create a new file part
    pub fn file(media_type: impl Into<String>, url: impl Into<String>) -> Self {
        UIMessagePart::File(FileUIPart {
            media_type: media_type.into(),
            filename: None,
            url: url.into(),
            provider_metadata: None,
        })
    }
}

/// A UI message following the Vercel AI SDK specification
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UIMessage {
    pub id: WrappedUuidV4,
    pub thread_id: WrappedUuidV4,
    pub role: MessageRole,
    pub parts: Vec<UIMessagePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub metadata: Option<WrappedJsonValue>,
    /// Inbox-specific settings that can be added by inbox providers
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub inbox_settings: Map<String, Value>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl UIMessage {
    /// Create a new user message with text content
    pub fn user_text(thread_id: WrappedUuidV4, text: impl Into<String>) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: WrappedUuidV4::new(),
            thread_id,
            role: MessageRole::User,
            parts: vec![UIMessagePart::text(text)],
            metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new assistant message with text content
    pub fn assistant_text(thread_id: WrappedUuidV4, text: impl Into<String>) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: WrappedUuidV4::new(),
            thread_id,
            role: MessageRole::Assistant,
            parts: vec![UIMessagePart::text(text)],
            metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new system message with text content
    pub fn system_text(thread_id: WrappedUuidV4, text: impl Into<String>) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: WrappedUuidV4::new(),
            thread_id,
            role: MessageRole::System,
            parts: vec![UIMessagePart::text(text)],
            metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get all text parts concatenated
    pub fn text_content(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| match part {
                UIMessagePart::Text(text_part) => text_part.text.clone(),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if message is still streaming
    pub fn is_streaming(&self) -> bool {
        self.parts.iter().any(|part| match part {
            UIMessagePart::Text(text_part) => text_part.state == Some(PartState::Streaming),
            UIMessagePart::Reasoning(reasoning_part) => {
                reasoning_part.state == Some(PartState::Streaming)
            }
            UIMessagePart::Tool(tool_part) => {
                matches!(tool_part.state, ToolInvocationState::InputStreaming)
            }
            _ => false,
        })
    }
}

/// Request to create a new message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateMessageRequest {
    pub thread_id: WrappedUuidV4,
    pub role: MessageRole,
    pub parts: Vec<UIMessagePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(default)]
    pub inbox_settings: Map<String, Value>,
}

/// Request to update an existing message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<UIMessagePart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_settings: Option<Map<String, Value>>,
}

pub type CreateMessageResponse = UIMessage;
pub type UpdateMessageResponse = UIMessage;
pub type GetMessageResponse = UIMessage;

/// Response for listing messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListMessagesResponse {
    pub messages: Vec<UIMessage>,
    pub next_page_token: Option<String>,
}

/// Response for deleting a message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DeleteMessageResponse {
    pub success: bool,
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_message_role_serialization() {
            let role = MessageRole::User;
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, "\"user\"");

            let role = MessageRole::Assistant;
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, "\"assistant\"");
        }

        #[test]
        fn test_message_role_deserialization() {
            let role: MessageRole = serde_json::from_str("\"user\"").unwrap();
            assert_eq!(role, MessageRole::User);

            let role: MessageRole = serde_json::from_str("\"assistant\"").unwrap();
            assert_eq!(role, MessageRole::Assistant);
        }

        #[test]
        fn test_ui_message_part_text() {
            let part = UIMessagePart::text("Hello, world!");
            match part {
                UIMessagePart::Text(text_part) => {
                    assert_eq!(text_part.text, Some("Hello, world!".to_string()));
                    assert_eq!(text_part.state, Some(PartState::Done));
                }
                _ => panic!("Expected Text part"),
            }
        }

        #[test]
        fn test_ui_message_part_serialization() {
            let part = UIMessagePart::text("Hello");
            let json = serde_json::to_string(&part).unwrap();
            assert!(json.contains("\"type\":\"text\""));
            assert!(json.contains("\"text\":\"Hello\""));
        }

        #[test]
        fn test_tool_ui_part_serialization() {
            let part = UIMessagePart::Tool(ToolUIPart {
                tool_invocation_id: "inv-123".to_string(),
                tool_name: "get_weather".to_string(),
                state: ToolInvocationState::InputAvailable,
                input: Some(WrappedJsonValue::new(serde_json::json!({"city": "London"}))),
                output: None,
                error_text: None,
                approval: None,
                provider_metadata: None,
            });

            let json = serde_json::to_string(&part).unwrap();
            assert!(json.contains("\"type\":\"tool\""));
            assert!(json.contains("\"tool_name\":\"get_weather\""));
            assert!(json.contains("\"state\":\"input-available\""));
        }

        #[test]
        fn test_ui_message_user_text() {
            let thread_id = WrappedUuidV4::new();
            let msg = UIMessage::user_text(thread_id.clone(), "Hello!");

            assert_eq!(msg.thread_id, thread_id);
            assert_eq!(msg.role, MessageRole::User);
            assert_eq!(msg.text_content(), "Hello!");
            assert!(!msg.is_streaming());
        }

        #[test]
        fn test_ui_message_is_streaming() {
            let thread_id = WrappedUuidV4::new();
            let now = WrappedChronoDateTime::now();
            let msg = UIMessage {
                id: WrappedUuidV4::new(),
                thread_id,
                role: MessageRole::Assistant,
                parts: vec![UIMessagePart::text_streaming("Partial response...")],
                metadata: None,
                inbox_settings: Map::new(),
                created_at: now,
                updated_at: now,
            };

            assert!(msg.is_streaming());
        }

        #[test]
        fn test_inbox_settings_serialization() {
            let thread_id = WrappedUuidV4::new();
            let now = WrappedChronoDateTime::now();
            let mut settings = Map::new();
            settings.insert("a2a_task_id".to_string(), Value::String("task-123".to_string()));

            let msg = UIMessage {
                id: WrappedUuidV4::new(),
                thread_id,
                role: MessageRole::User,
                parts: vec![UIMessagePart::text("Hello")],
                metadata: None,
                inbox_settings: settings,
                created_at: now,
                updated_at: now,
            };

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("\"a2a_task_id\":\"task-123\""));
        }
    }
}
