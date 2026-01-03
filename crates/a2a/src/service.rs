//! A2A Service - Core service for task management and agent-related functionality
//!
//! This module provides the A2aService which manages:
//! - Task storage and retrieval
//! - Connection management for real-time updates
//! - Agent card discovery
//! - Integration with Restate for agent execution

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use a2a_core::events::InMemoryQueueManager;
use a2a_core::tasks::in_memory_push_notification_config_store::{
    InMemoryPushNotificationConfigStore, InMemoryPushNotificationConfigStoreBuilder,
};
use shared::restate::admin_client::AdminClient;
use shared::restate::invoke::RestateIngressClient;
use shared::soma_agent_definition::SomaAgentDefinitionLike;

use crate::logic::agent::RepositoryTaskStore;
use crate::logic::agent_cache::AgentCache;
use crate::logic::ConnectionManager;
use crate::repository::Repository;

/// Parameters for creating the A2aService with agent capabilities
pub struct A2aServiceParams {
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub host: Url,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub restate_ingress_client: RestateIngressClient,
    pub restate_admin_client: AdminClient,
    pub agent_cache: AgentCache,
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
    task_store: Arc<RepositoryTaskStore>,
    queue_manager: Arc<InMemoryQueueManager>,
    config_store: Arc<InMemoryPushNotificationConfigStore>,
    // Agent-specific fields (optional for task-only usage)
    soma_definition: Option<Arc<dyn SomaAgentDefinitionLike>>,
    host: Option<Url>,
    restate_ingress_client: Option<RestateIngressClient>,
    restate_admin_client: Option<AdminClient>,
    agent_cache: AgentCache,
}

impl A2aService {
    /// Create a new A2aService with the given connection manager and repository (task-only mode)
    pub fn new(connection_manager: ConnectionManager, repository: Repository) -> Self {
        let task_store = Arc::new(RepositoryTaskStore::new(repository.clone()));
        let config_store = Arc::new(
            InMemoryPushNotificationConfigStoreBuilder::default()
                .push_notification_infos(Arc::new(RwLock::new(HashMap::new())))
                .build()
                .unwrap(),
        );
        let queue_manager = Arc::new(InMemoryQueueManager::new());
        let agent_cache = crate::logic::agent_cache::create_agent_cache();

        Self {
            connection_manager,
            repository,
            task_store,
            queue_manager,
            config_store,
            soma_definition: None,
            host: None,
            restate_ingress_client: None,
            restate_admin_client: None,
            agent_cache,
        }
    }

    /// Create a new A2aService with full agent capabilities
    pub fn new_with_agent_support(params: A2aServiceParams) -> Self {
        let A2aServiceParams {
            soma_definition,
            host,
            connection_manager,
            repository,
            restate_ingress_client,
            restate_admin_client,
            agent_cache,
        } = params;

        let task_store = Arc::new(RepositoryTaskStore::new(repository.clone()));
        let config_store = Arc::new(
            InMemoryPushNotificationConfigStoreBuilder::default()
                .push_notification_infos(Arc::new(RwLock::new(HashMap::new())))
                .build()
                .unwrap(),
        );
        let queue_manager = Arc::new(InMemoryQueueManager::new());

        Self {
            connection_manager,
            repository,
            task_store,
            queue_manager,
            config_store,
            soma_definition: Some(soma_definition),
            host: Some(host),
            restate_ingress_client: Some(restate_ingress_client),
            restate_admin_client: Some(restate_admin_client),
            agent_cache,
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

    /// Get a reference to the task store (for A2A protocol integration)
    pub fn task_store(&self) -> Arc<RepositoryTaskStore> {
        self.task_store.clone()
    }

    /// Get a reference to the queue manager
    pub fn queue_manager(&self) -> Arc<InMemoryQueueManager> {
        self.queue_manager.clone()
    }

    /// Get a reference to the config store
    pub fn config_store(&self) -> Arc<InMemoryPushNotificationConfigStore> {
        self.config_store.clone()
    }

    /// Get a reference to the soma definition (optional, only available with agent support)
    pub fn soma_definition(&self) -> Option<Arc<dyn SomaAgentDefinitionLike>> {
        self.soma_definition.clone()
    }

    /// Get a reference to the host URL (optional, only available with agent support)
    pub fn host(&self) -> Option<&Url> {
        self.host.as_ref()
    }

    /// Get a reference to the Restate ingress client (optional, only available with agent support)
    pub fn restate_ingress_client(&self) -> Option<RestateIngressClient> {
        self.restate_ingress_client.clone()
    }

    /// Get a reference to the Restate admin client (optional, only available with agent support)
    pub fn restate_admin_client(&self) -> Option<AdminClient> {
        self.restate_admin_client.clone()
    }

    /// Get a reference to the agent cache
    pub fn agent_cache(&self) -> &AgentCache {
        &self.agent_cache
    }
}
