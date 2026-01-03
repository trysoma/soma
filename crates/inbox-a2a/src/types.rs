//! A2A protocol type definitions for inbox configuration
//!
//! Defines the configuration schema and types used by the A2A inbox provider.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Configuration for an A2A inbox
///
/// This configuration is validated when creating an A2A inbox instance.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct A2aConfiguration {
    /// The agent ID that this inbox is connected to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Optional webhook URL for push notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,

    /// Optional authentication token for webhook calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_auth_token: Option<String>,

    /// Whether to automatically acknowledge messages
    #[serde(default)]
    pub auto_acknowledge: bool,
}

impl Default for A2aConfiguration {
    fn default() -> Self {
        Self {
            agent_id: None,
            webhook_url: None,
            webhook_auth_token: None,
            auto_acknowledge: true,
        }
    }
}

/// A2A task ID stored in inbox_settings
#[allow(dead_code)]
pub const A2A_TASK_ID_KEY: &str = "a2a_task_id";

/// A2A context ID stored in inbox_settings
#[allow(dead_code)]
pub const A2A_CONTEXT_ID_KEY: &str = "a2a_context_id";

/// A2A message ID stored in inbox_settings
#[allow(dead_code)]
pub const A2A_MESSAGE_ID_KEY: &str = "a2a_message_id";

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_default_configuration() {
            let config = A2aConfiguration::default();
            assert!(config.agent_id.is_none());
            assert!(config.webhook_url.is_none());
            assert!(config.auto_acknowledge);
        }

        #[test]
        fn test_configuration_serialization() {
            let config = A2aConfiguration {
                agent_id: Some("agent-123".to_string()),
                webhook_url: Some("https://example.com/webhook".to_string()),
                webhook_auth_token: Some("secret".to_string()),
                auto_acknowledge: false,
            };

            let json = serde_json::to_string(&config).unwrap();
            assert!(json.contains("\"agent_id\":\"agent-123\""));
            assert!(json.contains("\"auto_acknowledge\":false"));
        }
    }
}
