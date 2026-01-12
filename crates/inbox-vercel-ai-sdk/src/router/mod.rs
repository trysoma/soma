//! Vercel AI SDK inbox router endpoints
//!
//! Provides HTTP endpoints compatible with the Vercel AI SDK:
//! - POST /ai/ui - Generate UI message response
//! - POST /ai/ui/stream - Stream UI message response (SSE)
//! - POST /ai/text - Generate text message response
//! - POST /ai/text/stream - Stream text message response (SSE)
//!
//! These routes are mounted by the inbox crate at:
//! `/inbox/v1/inbox/{inbox_id}/...`

mod ai;

pub use ai::{API_VERSION_1, SERVICE_ROUTE_KEY, create_router};
