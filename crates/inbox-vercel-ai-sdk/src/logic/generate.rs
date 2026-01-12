//! Message generation logic for Vercel AI SDK endpoints
//!
//! Handles the transformation and generation of messages in UIMessage and TextMessage formats.

use inbox::logic::message::{Message, MessageRole, TextMessage, UIMessage, UIMessagePart};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::primitives::{WrappedJsonValue, WrappedUuidV4};
use utoipa::ToSchema;

/// Parameters for generating a UI message response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct GenerateUiParams {
    /// The input message from the user
    pub message: UIMessage,
    /// Optional thread ID to associate with the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<WrappedUuidV4>,
    /// Optional metadata to include with the response
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
}

/// Response containing a generated UI message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct GenerateUiResponse {
    /// The generated assistant message
    pub message: UIMessage,
}

/// Parameters for generating a text message response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct GenerateTextParams {
    /// The input message from the user (can be UIMessage or TextMessage)
    pub message: Message,
    /// Optional thread ID to associate with the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<WrappedUuidV4>,
    /// Optional metadata to include with the response
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<WrappedJsonValue>,
}

/// Response containing a generated text message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct GenerateTextResponse {
    /// The generated assistant message text
    pub text: String,
    /// Message ID for tracking
    pub id: WrappedUuidV4,
}

/// Union type for generate parameters (UI or Text)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(untagged)]
pub enum GenerateParams {
    Ui(GenerateUiParams),
    Text(GenerateTextParams),
}

/// Union type for generate response (UI or Text)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(untagged)]
pub enum GenerateResponse {
    Ui(GenerateUiResponse),
    Text(GenerateTextResponse),
}

/// Stream item for UI message streaming
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UiStreamItem {
    /// The message ID being streamed
    pub id: WrappedUuidV4,
    /// The part being streamed (incremental update)
    pub part: UIMessagePart,
    /// Whether this is the final part
    #[serde(default)]
    pub done: bool,
}

/// Stream item for text message streaming
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextStreamItem {
    /// The message ID being streamed
    pub id: WrappedUuidV4,
    /// The text delta being streamed
    pub text: String,
    /// Whether this is the final chunk
    #[serde(default)]
    pub done: bool,
}

/// Union type for stream items
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(untagged)]
pub enum StreamItem {
    Ui(UiStreamItem),
    Text(TextStreamItem),
}

/// Extract text content from a message (works with both UIMessage and TextMessage)
#[allow(dead_code)]
pub fn extract_text_content(message: &Message) -> String {
    message.text_content()
}

/// Convert a UIMessage to text-only response (extracting only text parts)
#[allow(dead_code)]
pub fn ui_message_to_text(message: &UIMessage) -> String {
    message.text_content()
}

/// Create a simple text response UIMessage
#[allow(dead_code)]
pub fn create_text_ui_message(thread_id: WrappedUuidV4, text: impl Into<String>) -> UIMessage {
    UIMessage::assistant_text(thread_id, text)
}

/// Create a simple text response TextMessage
#[allow(dead_code)]
pub fn create_text_message(thread_id: WrappedUuidV4, text: impl Into<String>) -> TextMessage {
    TextMessage::new(thread_id, MessageRole::Assistant, text)
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_extract_text_from_text_message() {
            let thread_id = WrappedUuidV4::new();
            let msg = Message::text(thread_id, MessageRole::User, "Hello, world!");
            assert_eq!(extract_text_content(&msg), "Hello, world!");
        }

        #[test]
        fn test_extract_text_from_ui_message() {
            let thread_id = WrappedUuidV4::new();
            let msg = Message::ui(
                thread_id,
                MessageRole::User,
                vec![UIMessagePart::text("Hello, UI!")],
            );
            assert_eq!(extract_text_content(&msg), "Hello, UI!");
        }

        #[test]
        fn test_ui_message_to_text() {
            let thread_id = WrappedUuidV4::new();
            let msg = UIMessage::user_text(thread_id, "Test message");
            assert_eq!(ui_message_to_text(&msg), "Test message");
        }

        #[test]
        fn test_create_text_ui_message() {
            let thread_id = WrappedUuidV4::new();
            let msg = create_text_ui_message(thread_id.clone(), "Response text");
            assert_eq!(msg.role, MessageRole::Assistant);
            assert_eq!(msg.text_content(), "Response text");
            assert_eq!(msg.thread_id, thread_id);
        }

        #[test]
        fn test_create_text_message() {
            let thread_id = WrappedUuidV4::new();
            let msg = create_text_message(thread_id.clone(), "Response text");
            assert_eq!(msg.role, MessageRole::Assistant);
            assert_eq!(msg.text, "Response text");
            assert_eq!(msg.thread_id, thread_id);
        }

        #[test]
        fn test_stream_item_serialization() {
            let id = WrappedUuidV4::new();
            let item = TextStreamItem {
                id: id.clone(),
                text: "Hello".to_string(),
                done: false,
            };
            let json = serde_json::to_string(&item).unwrap();
            assert!(json.contains("\"text\":\"Hello\""));
            assert!(json.contains("\"done\":false"));
        }

        #[test]
        fn test_ui_stream_item_serialization() {
            let id = WrappedUuidV4::new();
            let item = UiStreamItem {
                id,
                part: UIMessagePart::text("Streaming..."),
                done: false,
            };
            let json = serde_json::to_string(&item).unwrap();
            assert!(json.contains("\"type\":\"text\""));
            assert!(json.contains("\"done\":false"));
        }
    }
}
