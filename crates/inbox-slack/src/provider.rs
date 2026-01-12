//! Slack Inbox Provider implementation
//!
//! Implements the InboxProvider trait for Slack integration,
//! allowing agents to receive and send messages through Slack.

use dashmap::DashMap;
use inbox::logic::inbox::{InboxHandle, InboxProvider, InboxProviderState};
use serde_json::Value;
use shared::{error::CommonError, primitives::WrappedSchema};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{trace, warn};
use utoipa_axum::router::OpenApiRouter;

use crate::logic::SlackClient;
use crate::router::create_router;
use crate::types::SlackConfiguration;

/// Slack Inbox Provider
///
/// Provides Slack integration for inbox message handling.
/// This allows receiving messages from Slack channels/DMs and sending responses
/// back through the Slack API.
pub struct SlackInboxProvider {
    /// Active inbox handlers - maps inbox_id to their background task handles
    active_handlers: DashMap<String, JoinHandle<()>>,
}

impl SlackInboxProvider {
    /// Create a new Slack inbox provider
    pub fn new() -> Self {
        Self {
            active_handlers: DashMap::new(),
        }
    }
}

impl Default for SlackInboxProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl InboxProvider for SlackInboxProvider {
    fn id(&self) -> &str {
        "slack"
    }

    fn title(&self) -> &str {
        "Slack"
    }

    fn description(&self) -> &str {
        "Slack inbox provider. Enables receiving and sending messages through Slack \
         channels and direct messages. Supports threaded conversations and rich \
         message formatting via Block Kit."
    }

    fn configuration_schema(&self) -> WrappedSchema {
        WrappedSchema::new(schemars::schema_for!(SlackConfiguration))
    }

    fn router(&self) -> OpenApiRouter<InboxProviderState> {
        // Return the Slack webhook router that uses InboxProviderState
        create_router()
    }

    fn validate_configuration(&self, config: &Value) -> Result<(), CommonError> {
        // Validate configuration by attempting to deserialize it
        if config.is_null() {
            return Err(CommonError::InvalidRequest {
                msg: "Slack configuration requires at least a bot_token".to_string(),
                source: None,
            });
        }

        let slack_config: SlackConfiguration =
            serde_json::from_value(config.clone()).map_err(|e| CommonError::InvalidRequest {
                msg: format!("Invalid Slack configuration: {e}"),
                source: Some(e.into()),
            })?;

        // Validate bot token format
        if !slack_config.bot_token.starts_with("xoxb-") {
            return Err(CommonError::InvalidRequest {
                msg: "bot_token must start with 'xoxb-'".to_string(),
                source: None,
            });
        }

        // Validate app token format if provided
        if let Some(ref app_token) = slack_config.app_token {
            if !app_token.starts_with("xapp-") {
                return Err(CommonError::InvalidRequest {
                    msg: "app_token must start with 'xapp-'".to_string(),
                    source: None,
                });
            }
        }

        Ok(())
    }

    async fn on_inbox_activated(&self, handle: InboxHandle) {
        let inbox_id = handle.inbox.id.clone();
        trace!(inbox_id = %inbox_id, "Activating Slack inbox");

        // Parse configuration
        let config: SlackConfiguration =
            match serde_json::from_value(handle.inbox.configuration.get_inner().clone()) {
                Ok(c) => c,
                Err(e) => {
                    warn!(inbox_id = %inbox_id, error = %e, "Failed to parse Slack configuration");
                    return;
                }
            };

        // Create Slack client
        let client = Arc::new(SlackClient::new(config.bot_token.clone()));

        // Spawn background task to listen for events and send responses to Slack
        let handle_clone = handle.clone();
        let task = tokio::spawn(async move {
            crate::logic::run_event_handler(handle_clone, client).await;
        });

        self.active_handlers.insert(inbox_id, task);
    }

    async fn on_inbox_deactivated(&self, inbox_id: &str) {
        trace!(inbox_id = %inbox_id, "Deactivating Slack inbox");

        if let Some((_, task)) = self.active_handlers.remove(inbox_id) {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_provider_id() {
            let provider = SlackInboxProvider::new();
            assert_eq!(provider.id(), "slack");
        }

        #[test]
        fn test_provider_title() {
            let provider = SlackInboxProvider::new();
            assert_eq!(provider.title(), "Slack");
        }

        #[test]
        fn test_validate_configuration_null() {
            let provider = SlackInboxProvider::new();
            let result = provider.validate_configuration(&Value::Null);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_configuration_valid() {
            let provider = SlackInboxProvider::new();
            let config = serde_json::json!({
                "bot_token": "xoxb-123456789-123456789-abcdefghijk"
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_configuration_invalid_bot_token() {
            let provider = SlackInboxProvider::new();
            let config = serde_json::json!({
                "bot_token": "invalid-token"
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_err(), "Expected error for invalid bot token");
        }

        #[test]
        fn test_validate_configuration_invalid_app_token() {
            let provider = SlackInboxProvider::new();
            let config = serde_json::json!({
                "bot_token": "xoxb-valid-token",
                "app_token": "invalid-app-token"
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_err(), "Expected error for invalid app token");
        }

        #[test]
        fn test_validate_configuration_with_all_fields() {
            let provider = SlackInboxProvider::new();
            let config = serde_json::json!({
                "bot_token": "xoxb-123456789-123456789-abcdefghijk",
                "signing_secret": "secret123",
                "app_token": "xapp-1-A123-456-abc",
                "default_channel_id": "C12345",
                "auto_acknowledge": true,
                "send_typing_indicator": true
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_ok());
        }
    }
}
