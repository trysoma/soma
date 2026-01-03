//! Router layer for inbox crate
//! Contains HTTP endpoints for threads, messages, events, and inboxes

pub mod inbox;
pub mod message;
pub mod thread;

use std::sync::Arc;
use utoipa::openapi::OpenApi as OpenApiDoc;
use utoipa_axum::router::OpenApiRouter;

use crate::service::InboxService;

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "inbox";

/// Create the combined inbox router
pub fn create_router() -> OpenApiRouter<Arc<InboxService>> {
    let thread_router = thread::create_router();
    let message_router = message::create_router();
    let inbox_router = inbox::create_router();

    OpenApiRouter::new()
        .merge(thread_router)
        .merge(message_router)
        .merge(inbox_router)
}

/// Get the combined OpenAPI spec for the inbox crate
pub fn get_openapi_spec() -> OpenApiDoc {
    let (_, thread_spec) = thread::create_router().split_for_parts();
    let (_, message_spec) = message::create_router().split_for_parts();
    let (_, inbox_spec) = inbox::create_router().split_for_parts();

    let mut spec = thread_spec;
    spec.merge(message_spec);
    spec.merge(inbox_spec);
    spec
}
