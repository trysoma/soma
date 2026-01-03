//! A2A Protocol Inbox Provider
//!
//! Implements the InboxProvider trait for the A2A (Agent-to-Agent) protocol,
//! allowing agents to receive and send messages through the A2A protocol.
//!
//! This crate also provides:
//! - A2A task and agent HTTP routes (`router` module)
//! - A2A-specific event types for the inbox event bus

pub mod events;
mod provider;
pub mod router;
mod types;

pub use events::{
    A2aArtifact, A2aArtifactPart, A2aTask, A2aTaskState, A2aTaskStatus,
    artifact_created_event, artifact_updated_event, task_created_event, task_status_updated_event,
    EVENT_TYPE_ARTIFACT_CREATED, EVENT_TYPE_ARTIFACT_UPDATED,
    EVENT_TYPE_TASK_CREATED, EVENT_TYPE_TASK_STATUS_UPDATED,
};
pub use provider::A2aInboxProvider;
pub use types::A2aConfiguration;

use inbox::logic::inbox::get_provider_registry;
use std::sync::Arc;

/// Register the A2A inbox provider with the global registry
pub fn register_provider() {
    let registry = get_provider_registry();
    registry.register(Arc::new(A2aInboxProvider::new()));
}
