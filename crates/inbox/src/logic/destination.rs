//! Destination domain model and in-memory registry
//!
//! Destinations are agents or workflows that can receive and publish events.
//! They are stored in-memory (not persisted to SQLite) and managed by the InboxService.

use dashmap::DashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use super::inbox::DestinationType;
use super::event::{EventRx, EventTx, InboxEvent};

/// A destination that can receive and publish inbox events
///
/// Destinations represent agents or workflows that participate in the inbox event system.
/// Each destination gets a filtered view of events (excluding events it published itself).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct Destination {
    /// Unique identifier for this destination
    pub id: String,
    /// Type of destination (agent or workflow)
    pub destination_type: DestinationType,
    /// Human-readable name for the destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of the destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Destination {
    /// Create a new destination
    pub fn new(id: impl Into<String>, destination_type: DestinationType) -> Self {
        Self {
            id: id.into(),
            destination_type,
            name: None,
            description: None,
        }
    }

    /// Create a new agent destination
    pub fn agent(id: impl Into<String>) -> Self {
        Self::new(id, DestinationType::Agent)
    }

    /// Create a new workflow destination
    pub fn workflow(id: impl Into<String>) -> Self {
        Self::new(id, DestinationType::Workflow)
    }

    /// Set the name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Handle for a registered destination to receive and publish events
#[derive(Clone)]
pub struct DestinationHandle {
    /// The destination configuration
    pub destination: Destination,
    /// Sender to publish events (events will be tagged with this destination as source)
    pub event_tx: EventTx,
    /// Receiver for events (filtered to exclude self-published events)
    event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync>,
}

impl DestinationHandle {
    /// Create a new destination handle
    pub(crate) fn new(
        destination: Destination,
        event_tx: EventTx,
        event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync>,
    ) -> Self {
        Self {
            destination,
            event_tx,
            event_rx_factory,
        }
    }

    /// Subscribe to receive events for this destination
    /// Events published by this destination are automatically filtered out
    pub fn subscribe(&self) -> EventRx {
        (self.event_rx_factory)()
    }

    /// Publish an event from this destination
    /// The event will be automatically tagged with this destination as the source
    #[allow(clippy::result_large_err)]
    pub fn publish(&self, event: InboxEvent) -> Result<usize, tokio::sync::broadcast::error::SendError<InboxEvent>> {
        let event = event.from_destination(
            self.destination.destination_type.clone(),
            &self.destination.id,
        );
        self.event_tx.send(event)
    }

    /// Get the destination ID
    pub fn id(&self) -> &str {
        &self.destination.id
    }

    /// Get the destination type
    pub fn destination_type(&self) -> &DestinationType {
        &self.destination.destination_type
    }
}

/// In-memory registry of active destinations
pub struct DestinationRegistry {
    destinations: DashMap<String, DestinationHandle>,
}

impl DestinationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            destinations: DashMap::new(),
        }
    }

    /// Register a new destination and return a handle for it
    pub fn register(
        &self,
        destination: Destination,
        event_tx: EventTx,
        event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync>,
    ) -> DestinationHandle {
        let handle = DestinationHandle::new(destination.clone(), event_tx, event_rx_factory);
        self.destinations.insert(destination.id.clone(), handle.clone());
        handle
    }

    /// Unregister a destination
    pub fn unregister(&self, id: &str) -> Option<DestinationHandle> {
        self.destinations.remove(id).map(|(_, handle)| handle)
    }

    /// Get a destination handle by ID
    pub fn get(&self, id: &str) -> Option<DestinationHandle> {
        self.destinations.get(id).map(|r| r.value().clone())
    }

    /// Check if a destination exists
    pub fn exists(&self, id: &str) -> bool {
        self.destinations.contains_key(id)
    }

    /// List all registered destinations
    pub fn list(&self) -> Vec<Destination> {
        self.destinations
            .iter()
            .map(|r| r.value().destination.clone())
            .collect()
    }

    /// Get the count of registered destinations
    pub fn count(&self) -> usize {
        self.destinations.len()
    }

    /// Get all destination handles (for broadcasting events)
    pub fn handles(&self) -> Vec<DestinationHandle> {
        self.destinations.iter().map(|r| r.value().clone()).collect()
    }
}

impl Default for DestinationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to register a destination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct RegisterDestinationRequest {
    /// Unique identifier for this destination
    pub id: String,
    /// Type of destination (agent or workflow)
    pub destination_type: DestinationType,
    /// Human-readable name for the destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of the destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response for registering a destination
pub type RegisterDestinationResponse = Destination;

/// Response for listing destinations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ListDestinationsResponse {
    pub destinations: Vec<Destination>,
}

/// Response for unregistering a destination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UnregisterDestinationResponse {
    pub success: bool,
}

// --- Logic Functions ---

use shared::error::CommonError;

/// Register a new destination
///
/// Returns a handle that can be used to receive and publish events.
pub fn register_destination(
    registry: &DestinationRegistry,
    event_tx: EventTx,
    event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync>,
    request: RegisterDestinationRequest,
) -> Result<DestinationHandle, CommonError> {
    // Check if destination already exists
    if registry.exists(&request.id) {
        return Err(CommonError::InvalidRequest {
            msg: format!("Destination with id {} already exists", request.id),
            source: None,
        });
    }

    let mut destination = Destination::new(&request.id, request.destination_type);
    if let Some(name) = request.name {
        destination = destination.with_name(name);
    }
    if let Some(description) = request.description {
        destination = destination.with_description(description);
    }

    let handle = registry.register(destination, event_tx, event_rx_factory);
    Ok(handle)
}

/// Unregister a destination
pub fn unregister_destination(
    registry: &DestinationRegistry,
    destination_id: &str,
) -> Result<UnregisterDestinationResponse, CommonError> {
    let removed = registry.unregister(destination_id);
    if removed.is_none() {
        return Err(CommonError::NotFound {
            msg: format!("Destination with id {destination_id} not found"),
            lookup_id: destination_id.to_string(),
            source: None,
        });
    }
    Ok(UnregisterDestinationResponse { success: true })
}

/// List all registered destinations
pub fn list_destinations(registry: &DestinationRegistry) -> ListDestinationsResponse {
    ListDestinationsResponse {
        destinations: registry.list(),
    }
}

/// Get a destination by ID
pub fn get_destination(
    registry: &DestinationRegistry,
    destination_id: &str,
) -> Result<Destination, CommonError> {
    registry
        .get(destination_id)
        .map(|h| h.destination)
        .ok_or_else(|| CommonError::NotFound {
            msg: format!("Destination with id {destination_id} not found"),
            lookup_id: destination_id.to_string(),
            source: None,
        })
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_destination_new() {
            let dest = Destination::new("agent-1", DestinationType::Agent);
            assert_eq!(dest.id, "agent-1");
            assert_eq!(dest.destination_type, DestinationType::Agent);
            assert!(dest.name.is_none());
        }

        #[test]
        fn test_destination_agent() {
            let dest = Destination::agent("my-agent").with_name("My Agent");
            assert_eq!(dest.id, "my-agent");
            assert_eq!(dest.destination_type, DestinationType::Agent);
            assert_eq!(dest.name, Some("My Agent".to_string()));
        }

        #[test]
        fn test_destination_workflow() {
            let dest = Destination::workflow("my-workflow")
                .with_name("My Workflow")
                .with_description("A test workflow");
            assert_eq!(dest.id, "my-workflow");
            assert_eq!(dest.destination_type, DestinationType::Workflow);
            assert_eq!(dest.name, Some("My Workflow".to_string()));
            assert_eq!(dest.description, Some("A test workflow".to_string()));
        }

        #[test]
        fn test_registry_register_and_get() {
            let registry = DestinationRegistry::new();
            let (tx, _) = tokio::sync::broadcast::channel(100);
            let rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync> = Arc::new({
                let tx = tx.clone();
                move || tx.subscribe()
            });

            let dest = Destination::agent("agent-1");
            let handle = registry.register(dest, tx, rx_factory);

            assert_eq!(handle.id(), "agent-1");
            assert!(registry.exists("agent-1"));
            assert_eq!(registry.count(), 1);
        }

        #[test]
        fn test_registry_unregister() {
            let registry = DestinationRegistry::new();
            let (tx, _) = tokio::sync::broadcast::channel(100);
            let rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync> = Arc::new({
                let tx = tx.clone();
                move || tx.subscribe()
            });

            let dest = Destination::agent("agent-1");
            registry.register(dest, tx, rx_factory);

            assert!(registry.exists("agent-1"));
            let removed = registry.unregister("agent-1");
            assert!(removed.is_some());
            assert!(!registry.exists("agent-1"));
        }

        #[test]
        fn test_registry_list() {
            let registry = DestinationRegistry::new();
            let (tx, _) = tokio::sync::broadcast::channel(100);

            let rx_factory1: Arc<dyn Fn() -> EventRx + Send + Sync> = Arc::new({
                let tx = tx.clone();
                move || tx.subscribe()
            });
            let rx_factory2: Arc<dyn Fn() -> EventRx + Send + Sync> = Arc::new({
                let tx = tx.clone();
                move || tx.subscribe()
            });

            registry.register(Destination::agent("agent-1"), tx.clone(), rx_factory1);
            registry.register(Destination::workflow("workflow-1"), tx, rx_factory2);

            let destinations = registry.list();
            assert_eq!(destinations.len(), 2);
        }
    }
}
