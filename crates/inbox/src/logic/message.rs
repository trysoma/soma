//! Message domain models and logic for inbox
//!
//! This module defines core message types. Messages can be either:
//! - TextMessage: Simple text content (body: {"text": "..."})
//! - UIMessage: Rich UI content following Vercel AI SDK spec (body: {"parts": [...]})

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::error::CommonError;
use shared::primitives::{
    PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4,
};
use utoipa::ToSchema;

use super::event::InboxEvent;
use crate::repository::{CreateMessage, MessageRepositoryLike, ThreadRepositoryLike, UpdateMessage};

/// Type of message stored in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Ui,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::Ui => write!(f, "ui"),
        }
    }
}

impl std::str::FromStr for MessageType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(MessageType::Text),
            "ui" => Ok(MessageType::Ui),
            _ => Err(format!("Unknown message type: {s}")),
        }
    }
}

impl libsql::FromValue for MessageType {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => s.parse().map_err(|_| libsql::Error::InvalidColumnType),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<MessageType> for libsql::Value {
    fn from(val: MessageType) -> Self {
        libsql::Value::Text(val.to_string())
    }
}

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
    #[schema(value_type = Option<Object>)]
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
    #[schema(value_type = Option<Object>)]
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
    #[schema(value_type = Option<Object>)]
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
    #[schema(value_type = Option<Object>)]
    pub input: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub output: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<Approval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
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
    #[schema(value_type = Option<Object>)]
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
    #[schema(value_type = Option<Object>)]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Step start UI part - marks the beginning of an agent step
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct StepStartUIPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub provider_metadata: Option<WrappedJsonValue>,
}

/// Data UI part - represents custom data parts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DataUIPart {
    pub data_type: String,
    #[schemars(with = "serde_json::Value")]
    #[schema(value_type = Object)]
    pub data: WrappedJsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
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

// --- Body types for database serialization ---

/// Body for TextMessage stored in DB as {"text": "..."}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessageBody {
    pub text: String,
}

/// Body for UIMessage stored in DB as {"parts": [...]}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIMessageBody {
    pub parts: Vec<UIMessagePart>,
}

// --- Message types ---

/// A simple text message with raw text content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextMessage {
    pub id: WrappedUuidV4,
    pub thread_id: WrappedUuidV4,
    pub role: MessageRole,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    /// Provider-specific metadata (e.g., from LLM providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub provider_metadata: Option<WrappedJsonValue>,
    /// Inbox-specific settings that can be added by inbox providers
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub inbox_settings: Map<String, Value>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl TextMessage {
    /// Create a new text message
    pub fn new(
        thread_id: WrappedUuidV4,
        role: MessageRole,
        text: impl Into<String>,
    ) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: WrappedUuidV4::new(),
            thread_id,
            role,
            text: text.into(),
            metadata: None,
            provider_metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
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
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    /// Provider-specific metadata (e.g., from LLM providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub provider_metadata: Option<WrappedJsonValue>,
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
            provider_metadata: None,
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
            provider_metadata: None,
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
            provider_metadata: None,
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

/// A message in an inbox - can be either a simple text message or a rich UI message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    Text(TextMessage),
    Ui(UIMessage),
}

impl Message {
    /// Get the message ID
    pub fn id(&self) -> &WrappedUuidV4 {
        match self {
            Message::Text(msg) => &msg.id,
            Message::Ui(msg) => &msg.id,
        }
    }

    /// Get the thread ID
    pub fn thread_id(&self) -> &WrappedUuidV4 {
        match self {
            Message::Text(msg) => &msg.thread_id,
            Message::Ui(msg) => &msg.thread_id,
        }
    }

    /// Get the message role
    pub fn role(&self) -> &MessageRole {
        match self {
            Message::Text(msg) => &msg.role,
            Message::Ui(msg) => &msg.role,
        }
    }

    /// Get the message type
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Text(_) => MessageType::Text,
            Message::Ui(_) => MessageType::Ui,
        }
    }

    /// Get the metadata
    pub fn metadata(&self) -> &Option<WrappedJsonValue> {
        match self {
            Message::Text(msg) => &msg.metadata,
            Message::Ui(msg) => &msg.metadata,
        }
    }

    /// Get the provider metadata
    pub fn provider_metadata(&self) -> &Option<WrappedJsonValue> {
        match self {
            Message::Text(msg) => &msg.provider_metadata,
            Message::Ui(msg) => &msg.provider_metadata,
        }
    }

    /// Get the inbox settings
    pub fn inbox_settings(&self) -> &Map<String, Value> {
        match self {
            Message::Text(msg) => &msg.inbox_settings,
            Message::Ui(msg) => &msg.inbox_settings,
        }
    }

    /// Get the created_at timestamp
    pub fn created_at(&self) -> &WrappedChronoDateTime {
        match self {
            Message::Text(msg) => &msg.created_at,
            Message::Ui(msg) => &msg.created_at,
        }
    }

    /// Get the updated_at timestamp
    pub fn updated_at(&self) -> &WrappedChronoDateTime {
        match self {
            Message::Text(msg) => &msg.updated_at,
            Message::Ui(msg) => &msg.updated_at,
        }
    }

    /// Get the text content of the message
    pub fn text_content(&self) -> String {
        match self {
            Message::Text(msg) => msg.text.clone(),
            Message::Ui(msg) => msg.text_content(),
        }
    }

    /// Create a new text message
    pub fn text(thread_id: WrappedUuidV4, role: MessageRole, text: impl Into<String>) -> Self {
        Message::Text(TextMessage::new(thread_id, role, text))
    }

    /// Create a new UI message with parts
    pub fn ui(thread_id: WrappedUuidV4, role: MessageRole, parts: Vec<UIMessagePart>) -> Self {
        let now = WrappedChronoDateTime::now();
        Message::Ui(UIMessage {
            id: WrappedUuidV4::new(),
            thread_id,
            role,
            parts,
            metadata: None,
            provider_metadata: None,
            inbox_settings: Map::new(),
            created_at: now,
            updated_at: now,
        })
    }
}

/// Request to create a new text message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateTextMessageRequest {
    pub thread_id: WrappedUuidV4,
    pub role: MessageRole,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(default)]
    pub inbox_settings: Map<String, Value>,
}

/// Request to create a new UI message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateUIMessageRequest {
    pub thread_id: WrappedUuidV4,
    pub role: MessageRole,
    pub parts: Vec<UIMessagePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(default)]
    pub inbox_settings: Map<String, Value>,
}

/// Request to create a new message (either text or UI)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CreateMessageRequest {
    Text(CreateTextMessageRequest),
    Ui(CreateUIMessageRequest),
}

/// Request to update a text message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateTextMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_settings: Option<Map<String, Value>>,
}

/// Request to update a UI message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateUIMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<UIMessagePart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_settings: Option<Map<String, Value>>,
}

/// Request to update an existing message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum UpdateMessageRequest {
    Text(UpdateTextMessageRequest),
    Ui(UpdateUIMessageRequest),
}

pub type CreateMessageResponse = Message;
pub type UpdateMessageResponse = Message;
pub type GetMessageResponse = Message;
pub type ListMessagesResponse = PaginatedResponse<Message>;

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
        fn test_message_type_serialization() {
            let msg_type = MessageType::Text;
            let json = serde_json::to_string(&msg_type).unwrap();
            assert_eq!(json, "\"text\"");

            let msg_type = MessageType::Ui;
            let json = serde_json::to_string(&msg_type).unwrap();
            assert_eq!(json, "\"ui\"");
        }

        #[test]
        fn test_message_type_deserialization() {
            let msg_type: MessageType = serde_json::from_str("\"text\"").unwrap();
            assert_eq!(msg_type, MessageType::Text);

            let msg_type: MessageType = serde_json::from_str("\"ui\"").unwrap();
            assert_eq!(msg_type, MessageType::Ui);
        }

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
        fn test_text_message_creation() {
            let thread_id = WrappedUuidV4::new();
            let msg = TextMessage::new(thread_id.clone(), MessageRole::User, "Hello!");

            assert_eq!(msg.thread_id, thread_id);
            assert_eq!(msg.role, MessageRole::User);
            assert_eq!(msg.text, "Hello!");
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
                provider_metadata: None,
                inbox_settings: Map::new(),
                created_at: now,
                updated_at: now,
            };

            assert!(msg.is_streaming());
        }

        #[test]
        fn test_message_enum_text() {
            let thread_id = WrappedUuidV4::new();
            let msg = Message::text(thread_id.clone(), MessageRole::User, "Hello!");

            assert_eq!(msg.thread_id(), &thread_id);
            assert_eq!(msg.role(), &MessageRole::User);
            assert_eq!(msg.message_type(), MessageType::Text);
            assert_eq!(msg.text_content(), "Hello!");
        }

        #[test]
        fn test_message_enum_ui() {
            let thread_id = WrappedUuidV4::new();
            let msg = Message::ui(
                thread_id.clone(),
                MessageRole::Assistant,
                vec![UIMessagePart::text("Hello!")],
            );

            assert_eq!(msg.thread_id(), &thread_id);
            assert_eq!(msg.role(), &MessageRole::Assistant);
            assert_eq!(msg.message_type(), MessageType::Ui);
            assert_eq!(msg.text_content(), "Hello!");
        }

        #[test]
        fn test_message_enum_serialization() {
            let thread_id = WrappedUuidV4::new();
            let text_msg = Message::text(thread_id.clone(), MessageRole::User, "Hello!");
            let json = serde_json::to_string(&text_msg).unwrap();
            assert!(json.contains("\"type\":\"text\""));

            let ui_msg = Message::ui(
                thread_id,
                MessageRole::Assistant,
                vec![UIMessagePart::text("Hi!")],
            );
            let json = serde_json::to_string(&ui_msg).unwrap();
            assert!(json.contains("\"type\":\"ui\""));
        }

        #[test]
        fn test_text_message_body_serialization() {
            let body = TextMessageBody {
                text: "Hello, world!".to_string(),
            };
            let json = serde_json::to_string(&body).unwrap();
            assert_eq!(json, r#"{"text":"Hello, world!"}"#);
        }

        #[test]
        fn test_ui_message_body_serialization() {
            let body = UIMessageBody {
                parts: vec![UIMessagePart::text("Hello")],
            };
            let json = serde_json::to_string(&body).unwrap();
            assert!(json.contains("\"parts\":["));
            assert!(json.contains("\"type\":\"text\""));
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
                provider_metadata: None,
                inbox_settings: settings,
                created_at: now,
                updated_at: now,
            };

            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("\"a2a_task_id\":\"task-123\""));
        }
    }
}

// --- Logic Functions ---

/// List messages with pagination
pub async fn list_messages<R: MessageRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListMessagesResponse, CommonError> {
    let paginated = repository.get_messages(&pagination).await?;
    Ok(ListMessagesResponse {
        items: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// Create a new message
pub async fn create_message<R: MessageRepositoryLike + ThreadRepositoryLike>(
    repository: &R,
    event_bus: &super::event::EventBus,
    request: CreateMessageRequest,
) -> Result<CreateMessageResponse, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let message = match request {
        CreateMessageRequest::Text(req) => {
            // Verify thread exists
            let thread = repository.get_thread_by_id(&req.thread_id).await?;
            let _ = thread.ok_or_else(|| CommonError::NotFound {
                msg: format!("Thread with id {} not found", req.thread_id),
                lookup_id: req.thread_id.to_string(),
                source: None,
            })?;

            let body = TextMessageBody {
                text: req.text.clone(),
            };
            let body_json = WrappedJsonValue::new(serde_json::to_value(&body).map_err(|e| {
                CommonError::InvalidRequest {
                    msg: format!("Failed to serialize body: {e}"),
                    source: Some(e.into()),
                }
            })?);

            let inbox_settings_json =
                WrappedJsonValue::new(serde_json::to_value(&req.inbox_settings).map_err(|e| {
                    CommonError::InvalidRequest {
                        msg: format!("Failed to serialize inbox_settings: {e}"),
                        source: Some(e.into()),
                    }
                })?);

            let create_params = CreateMessage {
                id: id.clone(),
                thread_id: req.thread_id.clone(),
                message_type: MessageType::Text,
                role: req.role.clone(),
                body: body_json,
                metadata: req.metadata.clone(),
                inbox_settings: inbox_settings_json,
                created_at: now,
                updated_at: now,
            };

            repository.create_message(&create_params).await?;

            Message::Text(TextMessage {
                id,
                thread_id: req.thread_id,
                role: req.role,
                text: req.text,
                metadata: req.metadata,
                provider_metadata: None,
                inbox_settings: req.inbox_settings,
                created_at: now,
                updated_at: now,
            })
        }
        CreateMessageRequest::Ui(req) => {
            // Verify thread exists
            let thread = repository.get_thread_by_id(&req.thread_id).await?;
            let _ = thread.ok_or_else(|| CommonError::NotFound {
                msg: format!("Thread with id {} not found", req.thread_id),
                lookup_id: req.thread_id.to_string(),
                source: None,
            })?;

            let body = UIMessageBody {
                parts: req.parts.clone(),
            };
            let body_json = WrappedJsonValue::new(serde_json::to_value(&body).map_err(|e| {
                CommonError::InvalidRequest {
                    msg: format!("Failed to serialize body: {e}"),
                    source: Some(e.into()),
                }
            })?);

            let inbox_settings_json =
                WrappedJsonValue::new(serde_json::to_value(&req.inbox_settings).map_err(|e| {
                    CommonError::InvalidRequest {
                        msg: format!("Failed to serialize inbox_settings: {e}"),
                        source: Some(e.into()),
                    }
                })?);

            let create_params = CreateMessage {
                id: id.clone(),
                thread_id: req.thread_id.clone(),
                message_type: MessageType::Ui,
                role: req.role.clone(),
                body: body_json,
                metadata: req.metadata.clone(),
                inbox_settings: inbox_settings_json,
                created_at: now,
                updated_at: now,
            };

            repository.create_message(&create_params).await?;

            Message::Ui(UIMessage {
                id,
                thread_id: req.thread_id,
                role: req.role,
                parts: req.parts,
                metadata: req.metadata,
                provider_metadata: None,
                inbox_settings: req.inbox_settings,
                created_at: now,
                updated_at: now,
            })
        }
    };

    // Publish event
    let _ = event_bus.publish(InboxEvent::message_created(message.clone()));

    Ok(message)
}

/// Get a message by ID
pub async fn get_message<R: MessageRepositoryLike>(
    repository: &R,
    message_id: WrappedUuidV4,
) -> Result<GetMessageResponse, CommonError> {
    let message = repository.get_message_by_id(&message_id).await?;
    message.ok_or_else(|| CommonError::NotFound {
        msg: format!("Message with id {message_id} not found"),
        lookup_id: message_id.to_string(),
        source: None,
    })
}

/// Update an existing message
pub async fn update_message<R: MessageRepositoryLike>(
    repository: &R,
    event_bus: &super::event::EventBus,
    message_id: WrappedUuidV4,
    request: UpdateMessageRequest,
) -> Result<UpdateMessageResponse, CommonError> {
    let existing = repository.get_message_by_id(&message_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Message with id {message_id} not found"),
        lookup_id: message_id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();

    // Ensure update type matches existing message type
    let updated_message = match (existing, request) {
        (Message::Text(existing_text), UpdateMessageRequest::Text(req)) => {
            let new_text = req.text.unwrap_or(existing_text.text.clone());
            let new_metadata = req.metadata.or(existing_text.metadata.clone());
            let new_inbox_settings = req
                .inbox_settings
                .unwrap_or(existing_text.inbox_settings.clone());

            let body = TextMessageBody {
                text: new_text.clone(),
            };
            let body_json = WrappedJsonValue::new(serde_json::to_value(&body).map_err(|e| {
                CommonError::InvalidRequest {
                    msg: format!("Failed to serialize body: {e}"),
                    source: Some(e.into()),
                }
            })?);

            let inbox_settings_json =
                WrappedJsonValue::new(serde_json::to_value(&new_inbox_settings).map_err(|e| {
                    CommonError::InvalidRequest {
                        msg: format!("Failed to serialize inbox_settings: {e}"),
                        source: Some(e.into()),
                    }
                })?);

            let update_params = UpdateMessage {
                id: message_id.clone(),
                body: body_json,
                metadata: new_metadata.clone(),
                inbox_settings: inbox_settings_json,
                updated_at: now,
            };

            repository.update_message(&update_params).await?;

            Message::Text(TextMessage {
                id: message_id,
                thread_id: existing_text.thread_id,
                role: existing_text.role,
                text: new_text,
                metadata: new_metadata,
                provider_metadata: existing_text.provider_metadata,
                inbox_settings: new_inbox_settings,
                created_at: existing_text.created_at,
                updated_at: now,
            })
        }
        (Message::Ui(existing_ui), UpdateMessageRequest::Ui(req)) => {
            let new_parts = req.parts.unwrap_or(existing_ui.parts.clone());
            let new_metadata = req.metadata.or(existing_ui.metadata.clone());
            let new_inbox_settings = req
                .inbox_settings
                .unwrap_or(existing_ui.inbox_settings.clone());

            let body = UIMessageBody {
                parts: new_parts.clone(),
            };
            let body_json = WrappedJsonValue::new(serde_json::to_value(&body).map_err(|e| {
                CommonError::InvalidRequest {
                    msg: format!("Failed to serialize body: {e}"),
                    source: Some(e.into()),
                }
            })?);

            let inbox_settings_json =
                WrappedJsonValue::new(serde_json::to_value(&new_inbox_settings).map_err(|e| {
                    CommonError::InvalidRequest {
                        msg: format!("Failed to serialize inbox_settings: {e}"),
                        source: Some(e.into()),
                    }
                })?);

            let update_params = UpdateMessage {
                id: message_id.clone(),
                body: body_json,
                metadata: new_metadata.clone(),
                inbox_settings: inbox_settings_json,
                updated_at: now,
            };

            repository.update_message(&update_params).await?;

            Message::Ui(UIMessage {
                id: message_id,
                thread_id: existing_ui.thread_id,
                role: existing_ui.role,
                parts: new_parts,
                metadata: new_metadata,
                provider_metadata: existing_ui.provider_metadata,
                inbox_settings: new_inbox_settings,
                created_at: existing_ui.created_at,
                updated_at: now,
            })
        }
        (Message::Text(_), UpdateMessageRequest::Ui(_)) => {
            return Err(CommonError::InvalidRequest {
                msg: "Cannot update a text message with UI message request".to_string(),
                source: None,
            });
        }
        (Message::Ui(_), UpdateMessageRequest::Text(_)) => {
            return Err(CommonError::InvalidRequest {
                msg: "Cannot update a UI message with text message request".to_string(),
                source: None,
            });
        }
    };

    // Publish event
    let _ = event_bus.publish(InboxEvent::message_updated(updated_message.clone()));

    Ok(updated_message)
}

/// Delete a message
pub async fn delete_message<R: MessageRepositoryLike>(
    repository: &R,
    event_bus: &super::event::EventBus,
    message_id: WrappedUuidV4,
) -> Result<DeleteMessageResponse, CommonError> {
    // Verify message exists
    let existing = repository.get_message_by_id(&message_id).await?;
    let _ = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Message with id {message_id} not found"),
        lookup_id: message_id.to_string(),
        source: None,
    })?;

    repository.delete_message(&message_id).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::message_deleted(message_id));

    Ok(DeleteMessageResponse { success: true })
}
