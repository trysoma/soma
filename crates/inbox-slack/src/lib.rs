//! Slack Inbox Provider
//!
//! Implements the InboxProvider trait for Slack integration,
//! allowing agents to receive and send messages through Slack channels
//! and direct messages.
//!
//! This crate provides:
//! - Slack Events API webhook handling (`router` module)
//! - Message sending via Slack Web API (`logic` module)
//! - Event bus integration for real-time updates
//!
//! ## Integration
//!
//! The Slack provider integrates with the inbox system through the `InboxProvider` trait:
//! - `router()` returns routes using `InboxProviderState` (mounted dynamically by inbox crate)
//! - `on_inbox_activated()` spawns a background task to listen for events and send to Slack
//! - `on_inbox_deactivated()` stops the background task
//!
//! The webhook endpoint publishes messages to the event bus and returns 200 immediately.
//! The background event handler sends responses to Slack via reqwest.

pub mod logic;
mod provider;
pub mod router;
mod types;

// Re-export provider
pub use provider::SlackInboxProvider;
pub use types::SlackConfiguration;

// Re-export logic components
pub use logic::SlackClient;

use inbox::logic::inbox::get_provider_registry;
use std::sync::Arc;

/// Register the Slack inbox provider with the global registry
pub fn register_provider() {
    let registry = get_provider_registry();
    registry.register(Arc::new(SlackInboxProvider::new()));
}
