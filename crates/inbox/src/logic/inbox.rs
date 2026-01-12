//! Inbox provider trait, registry, and logic
//!
//! Defines the abstraction for inbox providers (e.g., A2A, OpenAI Completions, Vercel AI SDK,
//! Gmail webhooks, Slack bots, etc.) and a global registry for registering providers.

use async_trait::async_trait;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::error::CommonError;
use shared::primitives::{
    PaginatedResponse, PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedSchema,
    WrappedUuidV4,
};
use std::sync::Arc;
use tracing::trace;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

use crate::repository::{CreateInbox, InboxRepositoryLike, UpdateInbox};
use super::event::{EventRx, EventTx};
use super::{InboxSerialized, OnConfigChangeTx, OnConfigChangeEvt};

/// Global registry of inbox providers
static INBOX_PROVIDER_REGISTRY: Lazy<InboxProviderRegistry> =
    Lazy::new(InboxProviderRegistry::new);

/// Get the global inbox provider registry
pub fn get_provider_registry() -> &'static InboxProviderRegistry {
    &INBOX_PROVIDER_REGISTRY
}

/// Handle for an inbox instance to interact with the event system
///
/// This handle is provided to inbox providers when an inbox is activated,
/// allowing them to receive events from other sources and publish events.
#[derive(Clone)]
pub struct InboxHandle {
    /// The inbox configuration
    pub inbox: Inbox,
    /// Sender for publishing events (events will be tagged with this inbox as source)
    event_tx: EventTx,
    /// Factory for creating event receivers (filtered to exclude self-published events)
    event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync>,
}

impl InboxHandle {
    /// Create a new inbox handle
    pub fn new(
        inbox: Inbox,
        event_tx: EventTx,
        event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync>,
    ) -> Self {
        Self {
            inbox,
            event_tx,
            event_rx_factory,
        }
    }

    /// Subscribe to receive events for this inbox
    /// Events published by this inbox are automatically filtered out by the service
    pub fn subscribe(&self) -> EventRx {
        (self.event_rx_factory)()
    }

    /// Publish an event from this inbox
    /// The event will be automatically tagged with this inbox as the source
    #[allow(clippy::result_large_err)]
    pub fn publish(&self, event: super::event::InboxEvent) -> Result<usize, tokio::sync::broadcast::error::SendError<super::event::InboxEvent>> {
        let event = event.from_inbox(&self.inbox.id);
        self.event_tx.send(event)
    }

    /// Get the inbox ID
    pub fn id(&self) -> &str {
        &self.inbox.id
    }

    /// Get the raw event sender (for advanced use cases)
    pub fn event_sender(&self) -> EventTx {
        self.event_tx.clone()
    }
}

/// Trait for inbox providers
///
/// Inbox providers implement protocol-specific handling for receiving messages
/// and events. Each provider can define its own routes and configuration schema.
///
/// ## Lifecycle
///
/// When an inbox instance is created/activated:
/// 1. The service calls `on_inbox_activated` with an `InboxHandle`
/// 2. The provider can use the handle to subscribe to events and publish responses
/// 3. When the inbox is deactivated, `on_inbox_deactivated` is called
///
/// ## Event Flow
///
/// - Events published by destinations (agents/workflows) are delivered to all inboxes
/// - Events published by other inboxes are also delivered
/// - Events an inbox publishes itself are NOT delivered back to it
#[async_trait]
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

    /// Called when an inbox instance using this provider is activated
    ///
    /// The provider should use the handle to:
    /// - Subscribe to events: `handle.subscribe()` returns an event receiver
    /// - Publish events: `handle.publish(event)` sends events to other subscribers
    ///
    /// Typically, providers spawn a tokio task to process incoming events:
    /// ```ignore
    /// async fn on_inbox_activated(&self, handle: InboxHandle) {
    ///     let mut rx = handle.subscribe();
    ///     tokio::spawn(async move {
    ///         while let Ok(event) = rx.recv().await {
    ///             // Only process events not from this inbox (already filtered)
    ///             // Handle the event...
    ///         }
    ///     });
    /// }
    /// ```
    async fn on_inbox_activated(&self, _handle: InboxHandle) {
        // Default implementation does nothing
        // Providers that need to receive events should override this
    }

    /// Called when an inbox instance using this provider is deactivated
    ///
    /// Providers should clean up any resources (e.g., cancel spawned tasks).
    /// The inbox_id is provided to identify which instance to clean up.
    async fn on_inbox_deactivated(&self, _inbox_id: &str) {
        // Default implementation does nothing
    }

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
    /// Handle for interacting with the event system
    pub handle: InboxHandle,
    /// Optional repository for providers that need persistence (e.g., thread/message storage)
    pub repository: Option<Arc<crate::repository::Repository>>,
    /// Event bus for publishing events
    pub event_bus: Option<super::event::EventBus>,
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

/// Type of destination an inbox routes messages to
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DestinationType {
    Agent,
    Workflow,
}

impl std::fmt::Display for DestinationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DestinationType::Agent => write!(f, "agent"),
            DestinationType::Workflow => write!(f, "workflow"),
        }
    }
}

impl std::str::FromStr for DestinationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "agent" => Ok(DestinationType::Agent),
            "workflow" => Ok(DestinationType::Workflow),
            _ => Err(format!("Unknown destination type: {s}")),
        }
    }
}

impl libsql::FromValue for DestinationType {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self> {
        match val {
            libsql::Value::Text(s) => s.parse().map_err(|_| libsql::Error::InvalidColumnType),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<DestinationType> for libsql::Value {
    fn from(val: DestinationType) -> Self {
        libsql::Value::Text(val.to_string())
    }
}

/// An inbox instance - a configured instance of an inbox provider
/// Each inbox routes messages to a destination (agent or workflow)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Inbox {
    pub id: String,
    pub provider_id: String,
    /// The type of destination this inbox routes to
    pub destination_type: DestinationType,
    /// The ID of the destination (agent or workflow)
    pub destination_id: String,
    #[schemars(with = "serde_json::Value")]
    #[schema(value_type = Object)]
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
        destination_type: DestinationType,
        destination_id: impl Into<String>,
        configuration: WrappedJsonValue,
    ) -> Self {
        let now = WrappedChronoDateTime::now();
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            destination_type,
            destination_id: destination_id.into(),
            configuration,
            settings: Map::new(),
            created_at: now,
            updated_at: now,
        }
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
    /// The type of destination this inbox routes to
    pub destination_type: DestinationType,
    /// The ID of the destination (agent or workflow)
    pub destination_id: String,
    /// Provider-specific configuration (validated against provider's schema)
    #[schemars(with = "serde_json::Value")]
    #[schema(value_type = Object)]
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
    #[schema(value_type = Option<Object>)]
    pub configuration: Option<WrappedJsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Map<String, Value>>,
}

pub type CreateInboxResponse = Inbox;
pub type UpdateInboxResponse = Inbox;
pub type GetInboxResponse = Inbox;
pub type ListInboxesResponse = PaginatedResponse<Inbox>;

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
                DestinationType::Agent,
                "agent-123",
                WrappedJsonValue::new(serde_json::json!({"api_key": "secret"})),
            );

            assert_eq!(inbox.id, "inbox-1");
            assert_eq!(inbox.provider_id, "test");
            assert_eq!(inbox.destination_type, DestinationType::Agent);
            assert_eq!(inbox.destination_id, "agent-123");
        }

        #[test]
        fn test_destination_type_serialization() {
            let dest = DestinationType::Agent;
            let json = serde_json::to_string(&dest).unwrap();
            assert_eq!(json, "\"agent\"");

            let dest = DestinationType::Workflow;
            let json = serde_json::to_string(&dest).unwrap();
            assert_eq!(json, "\"workflow\"");
        }

        #[test]
        fn test_destination_type_from_str() {
            assert_eq!("agent".parse::<DestinationType>().unwrap(), DestinationType::Agent);
            assert_eq!("workflow".parse::<DestinationType>().unwrap(), DestinationType::Workflow);
            assert_eq!("AGENT".parse::<DestinationType>().unwrap(), DestinationType::Agent);
            assert!("invalid".parse::<DestinationType>().is_err());
        }

        #[test]
        fn test_inbox_serialization() {
            let inbox = Inbox::new(
                "inbox-1",
                "test",
                DestinationType::Workflow,
                "workflow-456",
                WrappedJsonValue::new(serde_json::json!({})),
            );

            let json = serde_json::to_string(&inbox).unwrap();
            assert!(json.contains("\"id\":\"inbox-1\""));
            assert!(json.contains("\"provider_id\":\"test\""));
            assert!(json.contains("\"destination_type\":\"workflow\""));
            assert!(json.contains("\"destination_id\":\"workflow-456\""));
        }
    }
}

// --- Logic Functions ---

/// List inboxes with pagination
pub async fn list_inboxes<R: InboxRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListInboxesResponse, CommonError> {
    let paginated = repository.get_inboxes(&pagination).await?;
    Ok(ListInboxesResponse {
        items: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// Create a new inbox
pub async fn create_inbox<R: InboxRepositoryLike>(
    repository: &R,
    config_change_tx: Option<&OnConfigChangeTx>,
    request: CreateInboxRequest,
) -> Result<CreateInboxResponse, CommonError> {
    // Verify provider exists
    let registry = get_provider_registry();
    let provider = registry.get(&request.provider_id).ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: format!("Provider {} not found", request.provider_id),
            source: None,
        }
    })?;

    // Validate configuration against provider's schema
    provider.validate_configuration(request.configuration.get_inner())?;

    let now = WrappedChronoDateTime::now();
    let id = request
        .id
        .unwrap_or_else(|| format!("inbox-{}", WrappedUuidV4::new()));

    // Check if inbox with this ID already exists
    if repository.get_inbox_by_id(&id).await?.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!("Inbox with id {id} already exists"),
            source: None,
        });
    }

    let settings_json = WrappedJsonValue::new(
        serde_json::to_value(&request.settings).map_err(|e| CommonError::InvalidRequest {
            msg: format!("Failed to serialize settings: {e}"),
            source: Some(e.into()),
        })?,
    );

    let inbox = Inbox {
        id: id.clone(),
        provider_id: request.provider_id.clone(),
        destination_type: request.destination_type.clone(),
        destination_id: request.destination_id.clone(),
        configuration: request.configuration.clone(),
        settings: request.settings.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateInbox {
        id,
        provider_id: request.provider_id,
        destination_type: request.destination_type,
        destination_id: request.destination_id,
        configuration: request.configuration,
        settings: settings_json,
        created_at: now,
        updated_at: now,
    };

    repository.create_inbox(&create_params).await?;

    // Publish config change event
    if let Some(tx) = config_change_tx {
        let serialized: InboxSerialized = inbox.clone().into();
        if let Err(e) = tx.send(OnConfigChangeEvt::InboxAdded(serialized)) {
            trace!(error = %e, "Failed to send inbox config change event (no receivers)");
        }
    }

    Ok(inbox)
}

/// Get an inbox by ID
pub async fn get_inbox<R: InboxRepositoryLike>(
    repository: &R,
    inbox_id: &str,
) -> Result<GetInboxResponse, CommonError> {
    let inbox = repository.get_inbox_by_id(inbox_id).await?;
    inbox.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })
}

/// Update an existing inbox
pub async fn update_inbox<R: InboxRepositoryLike>(
    repository: &R,
    config_change_tx: Option<&OnConfigChangeTx>,
    inbox_id: &str,
    request: UpdateInboxRequest,
) -> Result<UpdateInboxResponse, CommonError> {
    let existing = repository.get_inbox_by_id(inbox_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })?;

    // If configuration is being updated, validate it
    let new_configuration = if let Some(config) = request.configuration {
        let registry = get_provider_registry();
        if let Some(provider) = registry.get(&existing.provider_id) {
            provider.validate_configuration(config.get_inner())?;
        }
        config
    } else {
        existing.configuration.clone()
    };

    let new_settings = request.settings.unwrap_or(existing.settings.clone());
    let now = WrappedChronoDateTime::now();

    let settings_json = WrappedJsonValue::new(
        serde_json::to_value(&new_settings).map_err(|e| CommonError::InvalidRequest {
            msg: format!("Failed to serialize settings: {e}"),
            source: Some(e.into()),
        })?,
    );

    let update_params = UpdateInbox {
        id: inbox_id.to_string(),
        configuration: new_configuration.clone(),
        settings: settings_json,
        updated_at: now,
    };

    repository.update_inbox(&update_params).await?;

    let updated_inbox = Inbox {
        id: inbox_id.to_string(),
        provider_id: existing.provider_id,
        destination_type: existing.destination_type,
        destination_id: existing.destination_id,
        configuration: new_configuration,
        settings: new_settings,
        created_at: existing.created_at,
        updated_at: now,
    };

    // Publish config change event
    if let Some(tx) = config_change_tx {
        let serialized: InboxSerialized = updated_inbox.clone().into();
        if let Err(e) = tx.send(OnConfigChangeEvt::InboxUpdated(serialized)) {
            trace!(error = %e, "Failed to send inbox config change event (no receivers)");
        }
    }

    Ok(updated_inbox)
}

/// Delete an inbox
pub async fn delete_inbox<R: InboxRepositoryLike>(
    repository: &R,
    config_change_tx: Option<&OnConfigChangeTx>,
    inbox_id: &str,
) -> Result<DeleteInboxResponse, CommonError> {
    // Verify inbox exists
    let existing = repository.get_inbox_by_id(inbox_id).await?;
    let _ = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })?;

    repository.delete_inbox(inbox_id).await?;

    // Publish config change event
    if let Some(tx) = config_change_tx {
        if let Err(e) = tx.send(OnConfigChangeEvt::InboxRemoved(inbox_id.to_string())) {
            trace!(error = %e, "Failed to send inbox config change event (no receivers)");
        }
    }

    Ok(DeleteInboxResponse { success: true })
}
