//! Service layer for inbox crate
//! Provides the main service struct that holds all dependencies for inbox operations

use std::sync::Arc;

use shared::error::CommonError;

use crate::{
    logic::{
        destination::{
            Destination, DestinationHandle, DestinationRegistry, ListDestinationsResponse,
            RegisterDestinationRequest, UnregisterDestinationResponse,
        },
        event::{EventBus, EventRx},
        inbox::{Inbox, InboxHandle},
        OnConfigChangeTx,
    },
    repository::Repository,
};

/// Main service struct for inbox operations
/// Holds all dependencies needed for thread, message, event, inbox, and destination operations
#[derive(Clone)]
pub struct InboxService {
    pub repository: Repository,
    pub event_bus: EventBus,
    /// In-memory registry for active destinations (agents/workflows)
    pub destination_registry: Arc<DestinationRegistry>,
    /// Optional sender for inbox configuration change events (for syncing to soma.yaml)
    pub config_change_tx: Option<OnConfigChangeTx>,
}

/// Parameters for creating an InboxService
pub struct InboxServiceParams {
    pub repository: Repository,
    pub event_bus: EventBus,
    /// Optional sender for inbox configuration change events (for syncing to soma.yaml)
    pub config_change_tx: Option<OnConfigChangeTx>,
}

impl InboxService {
    /// Create a new InboxService instance
    pub fn new(params: InboxServiceParams) -> Self {
        Self {
            repository: params.repository,
            event_bus: params.event_bus,
            destination_registry: Arc::new(DestinationRegistry::new()),
            config_change_tx: params.config_change_tx,
        }
    }

    // --- Destination Management ---

    /// Register a new destination (agent or workflow)
    ///
    /// Returns a handle that can be used to receive and publish events.
    pub fn register_destination(
        &self,
        request: RegisterDestinationRequest,
    ) -> Result<DestinationHandle, CommonError> {
        let event_tx = self.event_bus.sender();
        let dest_id = request.id.clone();
        let dest_type = request.destination_type.clone();

        // Create a factory that produces filtered event receivers
        let event_bus = self.event_bus.clone();
        let event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync> = Arc::new(move || {
            event_bus.subscribe()
        });

        crate::logic::destination::register_destination(
            &self.destination_registry,
            event_tx,
            event_rx_factory,
            request,
        ).inspect(|_handle| {
            tracing::info!(
                destination_id = %dest_id,
                destination_type = %dest_type,
                "Registered destination"
            );
        })
    }

    /// Unregister a destination
    pub fn unregister_destination(
        &self,
        destination_id: &str,
    ) -> Result<UnregisterDestinationResponse, CommonError> {
        let result = crate::logic::destination::unregister_destination(
            &self.destination_registry,
            destination_id,
        );
        if result.is_ok() {
            tracing::info!(destination_id = %destination_id, "Unregistered destination");
        }
        result
    }

    /// List all registered destinations
    pub fn list_destinations(&self) -> ListDestinationsResponse {
        crate::logic::destination::list_destinations(&self.destination_registry)
    }

    /// Get a destination by ID
    pub fn get_destination(&self, destination_id: &str) -> Result<Destination, CommonError> {
        crate::logic::destination::get_destination(&self.destination_registry, destination_id)
    }

    /// Get a destination handle by ID
    pub fn get_destination_handle(&self, destination_id: &str) -> Option<DestinationHandle> {
        self.destination_registry.get(destination_id)
    }

    // --- Inbox Handle Creation ---

    /// Create an inbox handle for a given inbox configuration
    ///
    /// The handle allows the inbox to receive events and publish events.
    /// Events published by this inbox are automatically tagged with its ID as the source.
    pub fn create_inbox_handle(&self, inbox: Inbox) -> InboxHandle {
        let event_tx = self.event_bus.sender();

        // Create a factory that produces event receivers
        // Note: The actual filtering of self-published events should be done by the consumer
        let event_bus = self.event_bus.clone();
        let event_rx_factory: Arc<dyn Fn() -> EventRx + Send + Sync> = Arc::new(move || {
            event_bus.subscribe()
        });

        InboxHandle::new(inbox, event_tx, event_rx_factory)
    }

    /// Helper to check if an event should be delivered to a destination
    ///
    /// Returns true if the event was not published by the destination itself.
    pub fn should_deliver_to_destination(
        &self,
        event: &crate::logic::event::InboxEvent,
        destination_id: &str,
    ) -> bool {
        if let Some(handle) = self.destination_registry.get(destination_id) {
            event.should_deliver_to_destination(handle.destination_type(), destination_id)
        } else {
            false
        }
    }
}
