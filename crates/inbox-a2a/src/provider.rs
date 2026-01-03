//! A2A Inbox Provider implementation
//!
//! Implements the InboxProvider trait for the A2A protocol.
//!
//! Note: The A2A routes are now served from the main router module with A2aService state.
//! This provider is kept for registry/discovery purposes but does not provide routes
//! through the InboxProvider interface.

use inbox::logic::inbox::{InboxProvider, InboxProviderState};
use serde_json::Value;
use shared::{error::CommonError, primitives::WrappedSchema};
use utoipa_axum::router::OpenApiRouter;

use crate::types::A2aConfiguration;

/// A2A Protocol Inbox Provider
///
/// Provides A2A (Agent-to-Agent) protocol support for inbox message handling.
/// This allows receiving task requests, streaming updates, and sending responses
/// through the standard A2A protocol.
pub struct A2aInboxProvider;

impl A2aInboxProvider {
    /// Create a new A2A inbox provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for A2aInboxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl InboxProvider for A2aInboxProvider {
    fn id(&self) -> &str {
        "a2a"
    }

    fn title(&self) -> &str {
        "A2A Protocol"
    }

    fn description(&self) -> &str {
        "Agent-to-Agent (A2A) protocol inbox provider. Enables receiving and sending \
         messages through the A2A protocol for inter-agent communication."
    }

    fn configuration_schema(&self) -> WrappedSchema {
        WrappedSchema::new(schemars::schema_for!(A2aConfiguration))
    }

    fn router(&self) -> OpenApiRouter<InboxProviderState> {
        // A2A routes are served from inbox_a2a::router::create_router() with A2aService state
        // This returns an empty router since A2A routes need their own state
        OpenApiRouter::new()
    }

    fn validate_configuration(&self, config: &Value) -> Result<(), CommonError> {
        // Validate configuration by attempting to deserialize it
        if config.is_null() {
            // Null config is valid - use defaults
            return Ok(());
        }

        serde_json::from_value::<A2aConfiguration>(config.clone()).map_err(|e| {
            CommonError::InvalidRequest {
                msg: format!("Invalid A2A configuration: {e}"),
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
            let provider = A2aInboxProvider::new();
            assert_eq!(provider.id(), "a2a");
        }

        #[test]
        fn test_provider_title() {
            let provider = A2aInboxProvider::new();
            assert_eq!(provider.title(), "A2A Protocol");
        }

        #[test]
        fn test_validate_configuration_null() {
            let provider = A2aInboxProvider::new();
            let result = provider.validate_configuration(&Value::Null);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_configuration_valid() {
            let provider = A2aInboxProvider::new();
            let config = serde_json::json!({
                "agent_id": "agent-123",
                "auto_acknowledge": true
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_configuration_with_extra_fields() {
            let provider = A2aInboxProvider::new();
            // Extra fields should be ignored (serde default behavior)
            let config = serde_json::json!({
                "agent_id": "agent-123",
                "unknown_field": "value"
            });
            let result = provider.validate_configuration(&config);
            assert!(result.is_ok());
        }
    }
}
