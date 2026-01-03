use std::collections::HashSet;

use tracing::{debug, trace};

// Re-export from a2a crate for convenience
pub use a2a::{AgentCache, AgentMetadata, create_agent_cache, get_all_agents};
pub use a2a::logic::agent_cache::{get_agent, get_agents_by_project, get_all_agent_ids, sync_agents_to_cache};

/// Sync agents from SDK metadata to the cache.
/// Clears existing cache entries and adds all agents from SDK metadata.
pub fn sync_agents_from_metadata(cache: &AgentCache, metadata: &sdk_proto::MetadataResponse) {
    debug!(count = metadata.agents.len(), "Syncing agents from SDK");

    // Convert SDK proto agents to a2a AgentMetadata
    let agents: Vec<AgentMetadata> = metadata
        .agents
        .iter()
        .map(|agent| {
            trace!(
                project_id = %agent.project_id,
                agent_id = %agent.id,
                name = %agent.name,
                "Caching agent"
            );
            AgentMetadata {
                id: agent.id.clone(),
                project_id: agent.project_id.clone(),
                name: agent.name.clone(),
                description: agent.description.clone(),
            }
        })
        .collect();

    sync_agents_to_cache(cache, agents);
    trace!(count = metadata.agents.len(), "Agent sync complete");
}

/// Find agents that were in the old set but not in the new metadata.
/// Returns a list of (project_id, agent_id) pairs for agents that should be removed.
pub fn find_removed_agents(
    old_agent_ids: &[(String, String)],
    new_metadata: &sdk_proto::MetadataResponse,
) -> Vec<(String, String)> {
    // Build a set of (project_id, agent_id) from new metadata
    let new_ids: HashSet<(String, String)> = new_metadata
        .agents
        .iter()
        .map(|agent| (agent.project_id.clone(), agent.id.clone()))
        .collect();

    // Find agents in old set that aren't in new set
    old_agent_ids
        .iter()
        .filter(|id| !new_ids.contains(*id))
        .cloned()
        .collect()
}
