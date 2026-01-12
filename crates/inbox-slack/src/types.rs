//! Slack inbox provider type definitions
//!
//! Defines the configuration schema and types used by the Slack inbox provider,
//! including Slack Events API payloads and message formats.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

/// Configuration for a Slack inbox
///
/// This configuration is validated when creating a Slack inbox instance.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct SlackConfiguration {
    /// Bot token for authenticating with Slack API (starts with xoxb-)
    pub bot_token: String,

    /// Signing secret for verifying Slack webhook requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_secret: Option<String>,

    /// App-level token for Socket Mode (starts with xapp-)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_token: Option<String>,

    /// Slack channel ID to send responses to (if different from incoming)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_channel_id: Option<String>,

    /// Whether to automatically acknowledge incoming messages
    #[serde(default = "default_true")]
    pub auto_acknowledge: bool,

    /// Whether to send typing indicators while processing
    #[serde(default)]
    pub send_typing_indicator: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SlackConfiguration {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            signing_secret: None,
            app_token: None,
            default_channel_id: None,
            auto_acknowledge: true,
            send_typing_indicator: false,
        }
    }
}

/// Slack Events API outer envelope
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SlackEventEnvelope {
    /// URL verification challenge from Slack
    UrlVerification {
        challenge: String,
        token: String,
    },
    /// Event callback containing actual event data
    EventCallback {
        token: String,
        team_id: String,
        api_app_id: String,
        event: SlackEvent,
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        event_time: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        authorizations: Option<Vec<SlackAuthorization>>,
    },
    /// App rate limited notification
    AppRateLimited {
        token: String,
        team_id: String,
        minute_rate_limited: i64,
        api_app_id: String,
    },
}

/// Slack authorization info
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SlackAuthorization {
    pub enterprise_id: Option<String>,
    pub team_id: Option<String>,
    pub user_id: String,
    pub is_bot: bool,
    pub is_enterprise_install: bool,
}

/// Slack event types we care about
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SlackEvent {
    /// Message sent in a channel/DM
    Message(SlackMessageEvent),
    /// App mention (@bot)
    AppMention(SlackAppMentionEvent),
    /// Catch-all for unknown events
    #[serde(other)]
    Unknown,
}

/// Slack message event payload
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SlackMessageEvent {
    /// Channel ID where the message was sent
    pub channel: String,
    /// User ID of the sender (optional for bot messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Message text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Timestamp (used as message ID)
    pub ts: String,
    /// Thread timestamp (if in a thread)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
    /// Subtype of message (e.g., "bot_message", "channel_join")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    /// Bot ID if message is from a bot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_id: Option<String>,
    /// File attachments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<SlackFile>,
    /// Block elements in the message
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<Value>,
    /// Additional properties
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Slack app mention event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SlackAppMentionEvent {
    /// Channel ID where the mention occurred
    pub channel: String,
    /// User ID of the mentioner
    pub user: String,
    /// Message text including the mention
    pub text: String,
    /// Timestamp
    pub ts: String,
    /// Thread timestamp (if in a thread)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
    /// Additional properties
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Slack file attachment
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SlackFile {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_private: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_private_download: Option<String>,
    /// Additional properties
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Request to send a message to Slack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackPostMessageRequest {
    /// Channel ID to post to
    pub channel: String,
    /// Message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Block Kit blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<Value>>,
    /// Thread timestamp to reply in thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
    /// Whether to also post to channel when replying in thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_broadcast: Option<bool>,
    /// Metadata for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Response from Slack's chat.postMessage API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackPostMessageResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_metadata: Option<Value>,
}

/// Request to update a message in Slack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackUpdateMessageRequest {
    /// Channel ID containing the message
    pub channel: String,
    /// Timestamp of the message to update
    pub ts: String,
    /// New message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// New blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<Value>>,
}

/// Slack inbox settings keys stored in message metadata
pub const SLACK_CHANNEL_KEY: &str = "slack_channel";
pub const SLACK_TS_KEY: &str = "slack_ts";
pub const SLACK_THREAD_TS_KEY: &str = "slack_thread_ts";
pub const SLACK_USER_KEY: &str = "slack_user";

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_default_configuration() {
            let config = SlackConfiguration::default();
            assert!(config.bot_token.is_empty());
            assert!(config.signing_secret.is_none());
            assert!(config.auto_acknowledge);
            assert!(!config.send_typing_indicator);
        }

        #[test]
        fn test_configuration_serialization() {
            let config = SlackConfiguration {
                bot_token: "xoxb-test-token".to_string(),
                signing_secret: Some("secret123".to_string()),
                app_token: None,
                default_channel_id: Some("C12345".to_string()),
                auto_acknowledge: true,
                send_typing_indicator: true,
            };

            let json = serde_json::to_string(&config).unwrap();
            assert!(json.contains("\"bot_token\":\"xoxb-test-token\""));
            assert!(json.contains("\"signing_secret\":\"secret123\""));
        }

        #[test]
        fn test_url_verification_envelope_deserialization() {
            let json = r#"{
                "type": "url_verification",
                "challenge": "test_challenge_123",
                "token": "verification_token"
            }"#;

            let envelope: SlackEventEnvelope = serde_json::from_str(json).unwrap();
            match envelope {
                SlackEventEnvelope::UrlVerification { challenge, token } => {
                    assert_eq!(challenge, "test_challenge_123");
                    assert_eq!(token, "verification_token");
                }
                _ => panic!("Expected UrlVerification"),
            }
        }

        #[test]
        fn test_message_event_deserialization() {
            let json = r#"{
                "type": "event_callback",
                "token": "token123",
                "team_id": "T12345",
                "api_app_id": "A12345",
                "event": {
                    "type": "message",
                    "channel": "C12345",
                    "user": "U12345",
                    "text": "Hello, bot!",
                    "ts": "1234567890.123456"
                }
            }"#;

            let envelope: SlackEventEnvelope = serde_json::from_str(json).unwrap();
            match envelope {
                SlackEventEnvelope::EventCallback { event, .. } => {
                    match event {
                        SlackEvent::Message(msg) => {
                            assert_eq!(msg.channel, "C12345");
                            assert_eq!(msg.user, Some("U12345".to_string()));
                            assert_eq!(msg.text, Some("Hello, bot!".to_string()));
                        }
                        _ => panic!("Expected Message event"),
                    }
                }
                _ => panic!("Expected EventCallback"),
            }
        }

        #[test]
        fn test_app_mention_event_deserialization() {
            let json = r#"{
                "type": "event_callback",
                "token": "token123",
                "team_id": "T12345",
                "api_app_id": "A12345",
                "event": {
                    "type": "app_mention",
                    "channel": "C12345",
                    "user": "U12345",
                    "text": "<@U_BOT> help me",
                    "ts": "1234567890.123456"
                }
            }"#;

            let envelope: SlackEventEnvelope = serde_json::from_str(json).unwrap();
            match envelope {
                SlackEventEnvelope::EventCallback { event, .. } => {
                    match event {
                        SlackEvent::AppMention(mention) => {
                            assert_eq!(mention.channel, "C12345");
                            assert!(mention.text.contains("help me"));
                        }
                        _ => panic!("Expected AppMention event"),
                    }
                }
                _ => panic!("Expected EventCallback"),
            }
        }

        #[test]
        fn test_post_message_request_serialization() {
            let request = SlackPostMessageRequest {
                channel: "C12345".to_string(),
                text: Some("Hello from bot!".to_string()),
                blocks: None,
                thread_ts: Some("1234567890.123456".to_string()),
                reply_broadcast: None,
                metadata: None,
            };

            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("\"channel\":\"C12345\""));
            assert!(json.contains("\"text\":\"Hello from bot!\""));
            assert!(json.contains("\"thread_ts\":\"1234567890.123456\""));
        }
    }
}
