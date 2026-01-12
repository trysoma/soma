//! A2A inbox router endpoints
//!
//! Provides HTTP endpoints for the A2A protocol:
//! - Task management routes (list, get, update, timeline)
//! - Agent routes (agent card, JSON-RPC)
//! - InboxProvider-based routes for inbox integration

mod a2a;
mod task;

pub use a2a::{
    create_agent_router, route_a2a_jsonrpc, route_agent_card, A2aRouterServiceParams,
    AgentPathParams,
};
pub use task::{create_task_router, API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;

/// Re-export A2aService and related types for convenience
pub use crate::{
    A2aService, A2aServiceParams, ConnectionManager, Repository, TaskRepositoryLike,
};

/// Creates the complete A2A router with all task and agent endpoints
pub fn create_router() -> OpenApiRouter<Arc<A2aService>> {
    OpenApiRouter::new()
        .merge(task::create_task_router())
        .merge(a2a::create_agent_router())
}
