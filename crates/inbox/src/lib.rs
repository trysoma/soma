//! Inbox crate for managing incoming messages and events
//!
//! This crate provides a unified abstraction for receiving events from various sources
//! such as A2A protocol, OpenAI Completions API, Vercel AI SDK, webhooks (Gmail, Slack), etc.
//!
//! ## Core Concepts
//!
//! - **UIMessage**: Messages following the Vercel AI SDK specification, supporting text,
//!   files, tool invocations, reasoning, and other part types.
//!
//! - **Thread**: A conversation grouping that contains related messages.
//!
//! - **InboxEvent**: Events that occur within the system (message created, updated, etc.)
//!   with support for streaming.
//!
//! - **InboxProvider**: Trait for implementing protocol-specific inbox handlers.
//!   Each provider can define its own routes and configuration schema.
//!
//! - **Inbox**: A configured instance of an inbox provider with its specific settings.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use inbox::{InboxService, InboxServiceParams, Repository};
//! use inbox::logic::{get_provider_registry, InboxProvider};
//!
//! // Register a provider
//! let registry = get_provider_registry();
//! registry.register(Arc::new(MyCustomProvider::new()));
//!
//! // Create service
//! let service = InboxService::new(InboxServiceParams {
//!     repository,
//!     event_bus: EventBus::default(),
//! });
//!
//! // Create router
//! let router = inbox::router::create_router().with_state(Arc::new(service));
//! ```

pub mod logic;
pub mod repository;
pub mod router;
pub mod service;
