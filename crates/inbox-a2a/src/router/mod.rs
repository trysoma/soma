//! A2A inbox router endpoints
//!
//! Provides HTTP endpoints for the A2A protocol:
//! - Task management routes (list, get, update, timeline)
//! - Agent routes (agent card, JSON-RPC)
//! - InboxProvider-based routes for inbox integration

mod agent;
mod task;

pub use agent::{
    route_agent_card, route_a2a_jsonrpc, create_agent_router,
    AgentPathParams, A2aServiceParams, ProxiedAgent,
};
pub use task::{create_task_router, PATH_PREFIX, API_VERSION_1, SERVICE_ROUTE_KEY};

use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;

/// Re-export A2aService and related types for convenience
pub use a2a::{A2aService, AgentCache, ConnectionManager, Repository, TaskRepositoryLike};

/// Creates the complete A2A router with all task and agent endpoints
/// Uses A2aService state from the a2a crate
pub fn create_router() -> OpenApiRouter<Arc<A2aService>> {
    OpenApiRouter::new()
        .merge(task::create_task_router())
        .merge(agent::create_agent_router())
}
