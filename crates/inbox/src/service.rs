//! Service layer for inbox crate
//! Provides the main service struct that holds all dependencies for inbox operations

use crate::{
    logic::event::EventBus,
    repository::Repository,
};

/// Main service struct for inbox operations
/// Holds all dependencies needed for thread, message, event, and inbox CRUD operations
#[derive(Clone)]
pub struct InboxService {
    pub repository: Repository,
    pub event_bus: EventBus,
}

/// Parameters for creating an InboxService
pub struct InboxServiceParams {
    pub repository: Repository,
    pub event_bus: EventBus,
}

impl InboxService {
    /// Create a new InboxService instance
    pub fn new(params: InboxServiceParams) -> Self {
        Self {
            repository: params.repository,
            event_bus: params.event_bus,
        }
    }

    /// Create a new InboxService with default event bus
    pub fn with_repository(repository: Repository) -> Self {
        Self {
            repository,
            event_bus: EventBus::default(),
        }
    }
}
