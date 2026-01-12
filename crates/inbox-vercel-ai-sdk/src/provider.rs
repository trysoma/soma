//! Vercel AI SDK inbox provider implementation

use async_trait::async_trait;
use inbox::logic::inbox::{InboxHandle, InboxProvider, InboxProviderState};
use serde_json::Value;
use shared::error::CommonError;
use shared::primitives::WrappedSchema;
use utoipa_axum::router::OpenApiRouter;

use crate::router::create_router;
use crate::types::VercelAiSdkConfiguration;

/// Inbox provider for Vercel AI SDK compatible endpoints
pub struct VercelAiSdkInboxProvider;

impl VercelAiSdkInboxProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VercelAiSdkInboxProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InboxProvider for VercelAiSdkInboxProvider {
    fn id(&self) -> &str {
        "vercel-ai-sdk"
    }

    fn title(&self) -> &str {
        "Vercel AI SDK"
    }

    fn description(&self) -> &str {
        "Vercel AI SDK inbox provider for UIMessage and TextMessage compatible endpoints. \
         Supports streaming and non-streaming message generation in formats compatible with \
         the Vercel AI SDK useChat and useCompletion hooks."
    }

    fn configuration_schema(&self) -> WrappedSchema {
        WrappedSchema::new(schemars::schema_for!(VercelAiSdkConfiguration))
    }

    fn router(&self) -> OpenApiRouter<InboxProviderState> {
        create_router()
    }

    async fn on_inbox_activated(&self, _handle: InboxHandle) {
        // The Vercel AI SDK provider doesn't need to do anything special on activation
        // since it operates in request/response mode - events are handled per-request
    }

    async fn on_inbox_deactivated(&self, _inbox_id: &str) {
        // No cleanup needed for request/response mode
    }

    fn validate_configuration(&self, config: &Value) -> Result<(), CommonError> {
        if config.is_null() {
            return Ok(());
        }

        serde_json::from_value::<VercelAiSdkConfiguration>(config.clone()).map_err(|e| {
            CommonError::InvalidRequest {
                msg: format!("Invalid Vercel AI SDK configuration: {e}"),
                source: Some(e.into()),
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_provider_id() {
            let provider = VercelAiSdkInboxProvider::new();
            assert_eq!(provider.id(), "vercel-ai-sdk");
        }

        #[test]
        fn test_provider_title() {
            let provider = VercelAiSdkInboxProvider::new();
            assert_eq!(provider.title(), "Vercel AI SDK");
        }

        #[test]
        fn test_validate_null_config() {
            let provider = VercelAiSdkInboxProvider::new();
            let result = provider.validate_configuration(&Value::Null);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_valid_config() {
            let provider = VercelAiSdkInboxProvider::new();
            let config = serde_json::json!({
                "agent_id": "test-agent",
                "model": "gpt-4"
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_empty_config() {
            let provider = VercelAiSdkInboxProvider::new();
            let config = serde_json::json!({});
            let result = provider.validate_configuration(&config);
            assert!(result.is_ok());
        }
    }
}
