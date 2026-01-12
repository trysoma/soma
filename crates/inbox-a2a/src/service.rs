//! A2A Service - Core service for task management and agent-related functionality
//!
//! This module provides the A2aService which manages:
//! - Task storage and retrieval
//! - Connection management for real-time updates
//! - Agent card discovery

use std::sync::Arc;
use url::Url;

use crate::a2a_core::events::QueueManager;
use inbox::logic::event::EventBus;
use shared::soma_agent_definition::SomaAgentDefinitionLike;

use crate::logic::ConnectionManager;
use crate::task_repository::Repository;

/// Parameters for creating the A2aService with agent capabilities
pub struct A2aServiceParams {
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub host: Url,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    /// Optional event bus for inbox integration
    pub event_bus: Option<EventBus>,
}

/// Agent list item for list response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AgentListItem {
    /// The project ID
    pub project_id: String,
    /// The agent ID
    pub agent_id: String,
}

/// Response for listing agents
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ListAgentsResponse {
    /// List of agents
    pub agents: Vec<AgentListItem>,
}

/// Unified A2A service that provides task management and agent-related functionality
pub struct A2aService {
    repository: Repository,
    connection_manager: ConnectionManager,
    queue_manager: Arc<QueueManager>,
    // Agent-specific fields (optional for task-only usage)
    soma_definition: Option<Arc<dyn SomaAgentDefinitionLike>>,
    host: Option<Url>,
    event_bus: Option<EventBus>,
}

impl A2aService {
    /// Create a new A2aService with the given connection manager and repository (task-only mode)
    pub fn new(connection_manager: ConnectionManager, repository: Repository) -> Self {
        let queue_manager = Arc::new(QueueManager::new());

        Self {
            connection_manager,
            repository,
            queue_manager,
            soma_definition: None,
            host: None,
            event_bus: None,
        }
    }

    /// Create a new A2aService with full agent capabilities
    pub fn new_with_agent_support(params: A2aServiceParams) -> Self {
        let A2aServiceParams {
            soma_definition,
            host,
            connection_manager,
            repository,
            event_bus,
        } = params;

        let queue_manager = Arc::new(QueueManager::new());

        Self {
            connection_manager,
            repository,
            queue_manager,
            soma_definition: Some(soma_definition),
            host: Some(host),
            event_bus,
        }
    }

    /// Get a reference to the repository
    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    /// Get a reference to the connection manager
    pub fn connection_manager(&self) -> &ConnectionManager {
        &self.connection_manager
    }

    /// Get a reference to the queue manager
    pub fn queue_manager(&self) -> Arc<QueueManager> {
        self.queue_manager.clone()
    }

    /// Get a reference to the soma definition (optional, only available with agent support)
    pub fn soma_definition(&self) -> Option<Arc<dyn SomaAgentDefinitionLike>> {
        self.soma_definition.clone()
    }

    /// Get a reference to the host URL (optional, only available with agent support)
    pub fn host(&self) -> Option<&Url> {
        self.host.as_ref()
    }

    /// Get a reference to the event bus (optional, only available with agent support)
    pub fn event_bus(&self) -> Option<EventBus> {
        self.event_bus.clone()
    }
}
