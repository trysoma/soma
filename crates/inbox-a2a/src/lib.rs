//! A2A Protocol Inbox Provider
//!
//! Implements the InboxProvider trait for the A2A (Agent-to-Agent) protocol,
//! allowing agents to receive and send messages through the A2A protocol.
//!
//! This crate also provides:
//! - A2A core protocol types and implementations (`a2a_core` module)
//! - A2A task and agent HTTP routes (`router` module)
//! - A2A-specific event types for the inbox event bus
//! - Business logic for agent handling (`logic` module)
//! - Task repository for database operations
//! - Service for A2A task management and agent capabilities

pub mod a2a_core;
pub mod events;
pub mod logic;
mod provider;
pub mod router;
mod service;
pub mod task_repository;
mod types;

// Re-export events
pub use events::{
    artifact_created_event, artifact_updated_event, task_created_event, task_status_updated_event,
    A2aArtifact, A2aArtifactPart, A2aTask, A2aTaskState, A2aTaskStatus,
    EVENT_TYPE_ARTIFACT_CREATED, EVENT_TYPE_ARTIFACT_UPDATED, EVENT_TYPE_TASK_CREATED,
    EVENT_TYPE_TASK_STATUS_UPDATED,
};

// Re-export provider
pub use provider::A2aInboxProvider;
pub use types::A2aConfiguration;

// Re-export service
pub use service::{A2aService, A2aServiceParams, AgentListItem, ListAgentsResponse};

// Re-export task repository
pub use task_repository::{
    CreateTask, CreateTaskTimelineItem, Repository, TaskRepositoryLike, UpdateTaskStatus,
};

// Re-export logic components
pub use logic::{construct_agent_card, ConnectionManager, ConstructAgentCardParams};

use inbox::logic::inbox::get_provider_registry;
use std::sync::Arc;

/// Register the A2A inbox provider with the global registry
pub fn register_provider() {
    let registry = get_provider_registry();
    registry.register(Arc::new(A2aInboxProvider::new()));
}
