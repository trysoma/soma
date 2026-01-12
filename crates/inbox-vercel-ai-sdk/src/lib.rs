//! Vercel AI SDK Inbox Provider
//!
//! Provides HTTP endpoints compatible with the Vercel AI SDK for:
//! - UI message generation (POST /ai/ui and SSE /ai/ui/stream)
//! - Text message generation (POST /ai/text and SSE /ai/text/stream)
//!
//! This crate implements the `InboxProvider` trait, allowing the inbox system
//! to dynamically mount these routes when an inbox using this provider is created.
//!
//! ## Integration
//!
//! The routes use `InboxProviderState` to:
//! - Publish incoming messages to the event bus
//! - Subscribe to events and wait for responses from destinations (agents/workflows)
//! - Convert event responses back to HTTP responses
//!
//! ## Endpoints
//!
//! When an inbox is created with provider_id "vercel-ai-sdk", the following routes
//! are mounted at `/inbox/v1/inbox/{inbox_id}/`:
//!
//! - `POST /ai/ui` - Generate UI message response (request/response)
//! - `POST /ai/ui/stream` - Stream UI message response (SSE)
//! - `POST /ai/text` - Generate text message response (request/response)
//! - `POST /ai/text/stream` - Stream text message response (SSE)

pub mod logic;
mod provider;
pub mod router;
mod types;

// Re-export provider
pub use provider::VercelAiSdkInboxProvider;
pub use types::VercelAiSdkConfiguration;

// Re-export logic components
pub use logic::{GenerateParams, GenerateResponse, StreamItem};

use inbox::logic::inbox::get_provider_registry;
use std::sync::Arc;

/// Register the Vercel AI SDK inbox provider with the global registry
pub fn register_provider() {
    let registry = get_provider_registry();
    registry.register(Arc::new(VercelAiSdkInboxProvider::new()));
}
