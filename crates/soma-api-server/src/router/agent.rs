//! Agent management routes
//!
//! Provides endpoints for listing available agents from the agent cache.

use axum::extract::State;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use inbox_a2a::{AgentListItem, ListAgentsResponse};
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
};

use crate::sdk::sdk_agent_sync::{AgentCache, get_all_agents};

pub const PATH_PREFIX: &str = "/api";
pub const SERVICE_ROUTE_KEY: &str = "agent";

/// State for the agent router
pub struct AgentService {
    agent_cache: AgentCache,
}

impl AgentService {
    pub fn new(agent_cache: AgentCache) -> Self {
        Self { agent_cache }
    }
}

/// Creates the agent router with the list agents endpoint
pub fn create_router() -> OpenApiRouter<Arc<AgentService>> {
    OpenApiRouter::new().routes(routes!(route_list_agents))
}

/// GET /api/agent - List all available agents
#[utoipa::path(
    get,
    path = format!("{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    responses(
        (status = 200, description = "List of agents", body = ListAgentsResponse),
    ),
    summary = "List available agents",
    description = "List all available agents from the agent cache",
    operation_id = "list-agents",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_agents(
    State(ctx): State<Arc<AgentService>>,
) -> JsonResponse<ListAgentsResponse, CommonError> {
    trace!("Listing agents");
    let agents = list_agents(&ctx.agent_cache);
    trace!(count = agents.len(), "Listing agents completed");
    JsonResponse::from(Ok(ListAgentsResponse { agents }))
}

/// List all agents from the agent cache
fn list_agents(cache: &AgentCache) -> Vec<AgentListItem> {
    get_all_agents(cache)
        .into_iter()
        .map(|agent| AgentListItem {
            project_id: agent.project_id,
            agent_id: agent.id,
        })
        .collect()
}
