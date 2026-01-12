//! Slack inbox router endpoints
//!
//! Provides HTTP endpoints for Slack integration:
//! - Webhook endpoint for receiving Slack events and publishing to event bus

mod webhook;

pub use webhook::{create_router, API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};

use inbox::logic::inbox::InboxProviderState;
use utoipa_axum::router::OpenApiRouter;

/// Creates the Slack provider router with all endpoints
/// These routes use InboxProviderState and are mounted by the inbox crate
pub fn create_slack_router() -> OpenApiRouter<InboxProviderState> {
    webhook::create_router()
}
