//! Inbox provider trait and registry
//!
//! Defines the abstraction for inbox providers (e.g., A2A, OpenAI Completions, Vercel AI SDK,
//! Gmail webhooks, Slack bots, etc.) and a global registry for registering providers.

use dashmap::DashMap;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::error::CommonError;
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedSchema};
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

/// Global registry of inbox providers
static INBOX_PROVIDER_REGISTRY: Lazy<InboxProviderRegistry> =
    Lazy::new(InboxProviderRegistry::new);

/// Get the global inbox provider registry
pub fn get_provider_registry() -> &'static InboxProviderRegistry {
    &INBOX_PROVIDER_REGISTRY
}

/// Trait for inbox providers
///
/// Inbox providers implement protocol-specific handling for receiving messages
/// and events. Each provider can define its own routes and configuration schema.
pub trait InboxProvider: Send + Sync {
    /// Unique identifier for this provider (e.g., "a2a", "openai-completions", "slack")
    fn id(&self) -> &str;

    /// Human-readable title for the provider
    fn title(&self) -> &str;

    /// Description of what this provider does
    fn description(&self) -> &str;

    /// JSON Schema for the configuration required by this provider
    fn configuration_schema(&self) -> WrappedSchema;

    /// Get the Axum router for this provider's endpoints
    /// The router will be mounted at /inbox/v1/inbox/{inbox_id}/...
    fn router(&self) -> OpenApiRouter<InboxProviderState>;

    /// Validate the provided configuration against the schema
    fn validate_configuration(&self, config: &Value) -> Result<(), CommonError> {
        // Default implementation does basic validation
        // Providers can override for more specific validation
        let schema = self.configuration_schema();
        let schema_value = serde_json::to_value(schema.get_inner())
            .map_err(|e| CommonError::InvalidRequest {
                msg: format!("Failed to serialize schema: {e}"),
                source: Some(e.into()),
            })?;

        // Use jsonschema for validation if needed
        // For now, just check that config is an object
        if !config.is_object() && !config.is_null() {
            return Err(CommonError::InvalidRequest {
                msg: "Configuration must be an object".to_string(),
                source: None,
            });
        }

        let _ = schema_value; // Placeholder for actual validation
        Ok(())
    }
}

/// State passed to inbox provider routers
#[derive(Clone)]
pub struct InboxProviderState {
    /// The inbox instance configuration
    pub inbox: Inbox,
    /// The event bus for publishing events
    pub event_bus: super::event::EventBus,
}

/// Registry for inbox providers
pub struct InboxProviderRegistry {
    providers: DashMap<String, Arc<dyn InboxProvider>>,
}

impl InboxProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
        }
    }

    /// Register a new provider
    pub fn register(&self, provider: Arc<dyn InboxProvider>) {
        let id = provider.id().to_string();
        tracing::info!(provider_id = %id, "Registering inbox provider");
        self.providers.insert(id, provider);
    }

    /// Get a provider by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn InboxProvider>> {
        self.providers.get(id).map(|r| r.value().clone())
    }

    /// List all registered providers
    pub fn list(&self) -> Vec<Arc<dyn InboxProvider>> {
        self.providers.iter().map(|r| r.value().clone()).collect()
    }

    /// Check if a provider exists
    pub fn exists(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// Remove a provider (mainly for testing)
    pub fn remove(&self, id: &str) -> Option<Arc<dyn InboxProvider>> {
        self.providers.remove(id).map(|(_, v)| v)
    }
}

impl Default for InboxProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of an inbox instance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InboxStatus {
    Enabled,
    Disabled,
}

impl std::fmt::Display for InboxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InboxStatus::Enabled => write!(f, "enabled"),
            InboxStatus::Disabled => write!(f, "disabled"),
        }
    }
}

impl std::str::FromStr for InboxStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "enabled" => Ok(InboxStatus::Enabled),
            "disabled" => Ok(InboxStatus::Disabled),
            _ => Err(format!("Unknown inbox status: {s}")),
        }
    }
}

impl libsql::FromValue for InboxStatus {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => {
                s.parse().map_err(|_| libsql::Error::InvalidColumnType)
            }
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<InboxStatus> for libsql::Value {
    fn from(val: InboxStatus) -> Self {
        libsql::Value::Text(val.to_string())
    }
}

/// An inbox instance - a configured instance of an inbox provider
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Inbox {
    pub id: String,
    pub provider_id: String,
    pub status: InboxStatus,
    #[schemars(with = "serde_json::Value")]
    pub configuration: WrappedJsonValue,
    /// Inbox-specific settings
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub settings: Map<String, Value>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl Inbox {
    /// Create a new inbox instance
    pub fn new(
        id: impl Into<String>,
        provider_id: impl Into<String>,
        configuration: WrappedJsonValue,
    ) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            status: InboxStatus::Enabled,
            configuration,
            settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the inbox is enabled
    pub fn is_enabled(&self) -> bool {
        self.status == InboxStatus::Enabled
    }
}

/// Request to create a new inbox
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateInboxRequest {
    /// Optional ID (auto-generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Provider ID (must match a registered provider)
    pub provider_id: String,
    /// Provider-specific configuration (validated against provider's schema)
    #[schemars(with = "serde_json::Value")]
    pub configuration: WrappedJsonValue,
    /// Optional settings
    #[serde(default)]
    pub settings: Map<String, Value>,
}

/// Request to update an inbox
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateInboxRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub configuration: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Map<String, Value>>,
}

/// Request to change inbox status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct SetInboxStatusRequest {
    pub status: InboxStatus,
}

pub type CreateInboxResponse = Inbox;
pub type UpdateInboxResponse = Inbox;
pub type GetInboxResponse = Inbox;

/// Response for listing inboxes
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListInboxesResponse {
    pub inboxes: Vec<Inbox>,
    pub next_page_token: Option<String>,
}

/// Response for deleting an inbox
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct DeleteInboxResponse {
    pub success: bool,
}

/// Information about a registered provider
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ProviderInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub configuration_schema: WrappedSchema,
}

/// Response for listing providers
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListProvidersResponse {
    pub providers: Vec<ProviderInfo>,
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        struct TestProvider;

        impl InboxProvider for TestProvider {
            fn id(&self) -> &str {
                "test"
            }

            fn title(&self) -> &str {
                "Test Provider"
            }

            fn description(&self) -> &str {
                "A test inbox provider"
            }

            fn configuration_schema(&self) -> WrappedSchema {
                WrappedSchema::new(schemars::schema_for!(TestConfig))
            }

            fn router(&self) -> OpenApiRouter<InboxProviderState> {
                OpenApiRouter::new()
            }
        }

        #[derive(JsonSchema)]
        struct TestConfig {
            api_key: String,
        }

        #[test]
        fn test_registry_register_and_get() {
            let registry = InboxProviderRegistry::new();
            let provider = Arc::new(TestProvider);

            registry.register(provider);

            let retrieved = registry.get("test");
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().id(), "test");
        }

        #[test]
        fn test_registry_list() {
            let registry = InboxProviderRegistry::new();
            registry.register(Arc::new(TestProvider));

            let providers = registry.list();
            assert_eq!(providers.len(), 1);
        }

        #[test]
        fn test_registry_exists() {
            let registry = InboxProviderRegistry::new();
            registry.register(Arc::new(TestProvider));

            assert!(registry.exists("test"));
            assert!(!registry.exists("nonexistent"));
        }

        #[test]
        fn test_inbox_new() {
            let inbox = Inbox::new(
                "inbox-1",
                "test",
                WrappedJsonValue::new(serde_json::json!({"api_key": "secret"})),
            );

            assert_eq!(inbox.id, "inbox-1");
            assert_eq!(inbox.provider_id, "test");
            assert!(inbox.is_enabled());
        }

        #[test]
        fn test_inbox_status_serialization() {
            let status = InboxStatus::Enabled;
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, "\"enabled\"");

            let status = InboxStatus::Disabled;
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, "\"disabled\"");
        }

        #[test]
        fn test_inbox_serialization() {
            let inbox = Inbox::new(
                "inbox-1",
                "test",
                WrappedJsonValue::new(serde_json::json!({})),
            );

            let json = serde_json::to_string(&inbox).unwrap();
            assert!(json.contains("\"id\":\"inbox-1\""));
            assert!(json.contains("\"provider_id\":\"test\""));
            assert!(json.contains("\"status\":\"enabled\""));
        }
    }
}
