use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};
use utoipa::ToSchema;

/// Metadata for a registered agent
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentMetadata {
    /// The agent ID
    pub id: String,
    /// The project ID the agent belongs to
    pub project_id: String,
    /// Display name of the agent
    pub name: String,
    /// Description of the agent
    pub description: String,
}

/// Cache for storing agent metadata from SDK
/// Structure: project_id -> (agent_id -> AgentMetadata)
pub type AgentCache = Arc<DashMap<String, DashMap<String, AgentMetadata>>>;

/// Create a new empty agent cache
pub fn create_agent_cache() -> AgentCache {
    Arc::new(DashMap::new())
}

/// Sync agents from a list of agent metadata to the cache.
/// Clears existing cache entries and adds all agents.
pub fn sync_agents_to_cache(cache: &AgentCache, agents: Vec<AgentMetadata>) {
    debug!(count = agents.len(), "Syncing agents to cache");

    // Step 1: Clear existing cache
    cache.clear();

    // Step 2: Add all agents
    for agent in agents {
        let project_id = agent.project_id.clone();
        let agent_id = agent.id.clone();

        // Get or create the inner map for this project
        cache
            .entry(project_id.clone())
            .or_default()
            .insert(agent_id.clone(), agent.clone());

        trace!(
            project_id = %project_id,
            agent_id = %agent_id,
            name = %agent.name,
            "Cached agent"
        );
    }

    trace!("Agent sync complete");
}

/// Get all agents from the cache as a flat list
pub fn get_all_agents(cache: &AgentCache) -> Vec<AgentMetadata> {
    let mut agents = Vec::new();
    for project_entry in cache.iter() {
        for agent_entry in project_entry.value().iter() {
            agents.push(agent_entry.value().clone());
        }
    }
    agents
}

/// Get an agent by project_id and agent_id
pub fn get_agent(cache: &AgentCache, project_id: &str, agent_id: &str) -> Option<AgentMetadata> {
    cache.get(project_id).and_then(|project_agents| {
        project_agents
            .get(agent_id)
            .map(|entry| entry.value().clone())
    })
}

/// Get all agents for a specific project
pub fn get_agents_by_project(cache: &AgentCache, project_id: &str) -> Vec<AgentMetadata> {
    cache
        .get(project_id)
        .map(|project_agents| {
            project_agents
                .iter()
                .map(|entry| entry.value().clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Get all agent identifiers from cache as (project_id, agent_id) pairs.
/// Used to capture state before syncing to detect removed agents.
pub fn get_all_agent_ids(cache: &AgentCache) -> Vec<(String, String)> {
    let mut ids = Vec::new();
    for project_entry in cache.iter() {
        let project_id = project_entry.key().clone();
        for agent_entry in project_entry.value().iter() {
            ids.push((project_id.clone(), agent_entry.key().clone()));
        }
    }
    ids
}
