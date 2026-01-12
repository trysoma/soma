//! Slack webhook routes
//!
//! Provides endpoints for receiving Slack Events API webhooks.
//! The webhook immediately returns 200 and publishes the message to the event bus.
//! The event handler (activated via on_inbox_activated) sends responses to Slack.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use shared::adapters::openapi::API_VERSION_TAG;
use tracing::{trace, warn};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use inbox::logic::event::InboxEvent;
use inbox::logic::inbox::InboxProviderState;
use inbox::logic::message::{Message, MessageRole, UIMessagePart};

use crate::types::{
    SlackEvent, SlackEventEnvelope, SLACK_CHANNEL_KEY, SLACK_THREAD_TS_KEY, SLACK_TS_KEY,
    SLACK_USER_KEY,
};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "slack";

/// Response for Slack URL verification challenge
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UrlVerificationResponse {
    pub challenge: String,
}

/// Creates the webhook router for provider-specific routes
/// These routes are mounted at /api/inbox/v1/inbox/{inbox_id}/slack/...
pub fn create_router() -> OpenApiRouter<InboxProviderState> {
    OpenApiRouter::new().routes(routes!(route_slack_webhook))
}

/// POST /slack/webhook - Slack webhook endpoint
///
/// Receives Slack Events API webhooks and processes them:
/// - URL verification challenges are echoed back immediately
/// - Message events are published to the inbox event bus and return 200 immediately
/// - The event handler (spawned on inbox activation) sends responses to Slack
#[utoipa::path(
    post,
    path = "/slack/webhook",
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = SlackEventEnvelope,
    responses(
        (status = 200, description = "Event acknowledged"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error"),
    ),
    summary = "Slack webhook endpoint",
    description = "Receives Slack Events API webhooks. Returns URL verification challenge for setup, \
                   or acknowledges message events and publishes them to the event bus.",
    operation_id = "slack-webhook",
)]
pub async fn route_slack_webhook(
    State(ctx): State<InboxProviderState>,
    Json(envelope): Json<SlackEventEnvelope>,
) -> impl IntoResponse {
    let inbox_id = &ctx.inbox.id;
    trace!(inbox_id = %inbox_id, "Received Slack webhook");

    match envelope {
        // Handle URL verification challenge (required for Slack app setup)
        SlackEventEnvelope::UrlVerification { challenge, .. } => {
            trace!("Responding to Slack URL verification challenge");
            Json(UrlVerificationResponse { challenge }).into_response()
        }

        // Handle actual events
        SlackEventEnvelope::EventCallback {
            event,
            team_id,
            api_app_id,
            ..
        } => {
            trace!(
                team_id = %team_id,
                api_app_id = %api_app_id,
                "Processing Slack event callback"
            );

            // Process the event and publish to event bus
            match event {
                SlackEvent::Message(msg) => {
                    // Skip bot messages to prevent loops
                    if msg.bot_id.is_some() || msg.subtype.is_some() {
                        trace!("Skipping bot message or subtype");
                        return StatusCode::OK.into_response();
                    }

                    let text = msg.text.clone().unwrap_or_default();
                    if text.is_empty() {
                        return StatusCode::OK.into_response();
                    }

                    // Build inbox settings with Slack metadata
                    let mut inbox_settings = Map::new();
                    inbox_settings.insert(
                        SLACK_CHANNEL_KEY.to_string(),
                        serde_json::Value::String(msg.channel.clone()),
                    );
                    inbox_settings.insert(
                        SLACK_TS_KEY.to_string(),
                        serde_json::Value::String(msg.ts.clone()),
                    );
                    if let Some(ref thread_ts) = msg.thread_ts {
                        inbox_settings.insert(
                            SLACK_THREAD_TS_KEY.to_string(),
                            serde_json::Value::String(thread_ts.clone()),
                        );
                    }
                    if let Some(ref user) = msg.user {
                        inbox_settings.insert(
                            SLACK_USER_KEY.to_string(),
                            serde_json::Value::String(user.clone()),
                        );
                    }

                    // Create the user message with Slack metadata in inbox_settings
                    let thread_id = shared::primitives::WrappedUuidV4::new();
                    let mut message = Message::ui(
                        thread_id,
                        MessageRole::User,
                        vec![UIMessagePart::text(text)],
                    );

                    // Set inbox_settings on the message
                    match &mut message {
                        Message::Ui(ui_msg) => {
                            ui_msg.inbox_settings = inbox_settings;
                        }
                        Message::Text(text_msg) => {
                            text_msg.inbox_settings = inbox_settings;
                        }
                    }

                    // Publish message created event through the inbox handle
                    let event = InboxEvent::message_created(message);
                    if let Err(e) = ctx.handle.publish(event) {
                        warn!(error = %e, "Failed to publish message to event bus");
                    }

                    // Return 200 immediately - the event handler will send the response to Slack
                    StatusCode::OK.into_response()
                }

                SlackEvent::AppMention(mention) => {
                    // Handle @mention of the bot
                    let mut inbox_settings = Map::new();
                    inbox_settings.insert(
                        SLACK_CHANNEL_KEY.to_string(),
                        serde_json::Value::String(mention.channel.clone()),
                    );
                    inbox_settings.insert(
                        SLACK_TS_KEY.to_string(),
                        serde_json::Value::String(mention.ts.clone()),
                    );
                    if let Some(ref thread_ts) = mention.thread_ts {
                        inbox_settings.insert(
                            SLACK_THREAD_TS_KEY.to_string(),
                            serde_json::Value::String(thread_ts.clone()),
                        );
                    }
                    inbox_settings.insert(
                        SLACK_USER_KEY.to_string(),
                        serde_json::Value::String(mention.user.clone()),
                    );

                    let thread_id = shared::primitives::WrappedUuidV4::new();
                    let mut message = Message::ui(
                        thread_id,
                        MessageRole::User,
                        vec![UIMessagePart::text(mention.text.clone())],
                    );

                    match &mut message {
                        Message::Ui(ui_msg) => {
                            ui_msg.inbox_settings = inbox_settings;
                        }
                        Message::Text(text_msg) => {
                            text_msg.inbox_settings = inbox_settings;
                        }
                    }

                    let event = InboxEvent::message_created(message);
                    if let Err(e) = ctx.handle.publish(event) {
                        warn!(error = %e, "Failed to publish mention to event bus");
                    }

                    StatusCode::OK.into_response()
                }

                SlackEvent::Unknown => {
                    trace!("Received unknown Slack event type");
                    StatusCode::OK.into_response()
                }
            }
        }

        // Handle rate limiting
        SlackEventEnvelope::AppRateLimited {
            minute_rate_limited,
            ..
        } => {
            warn!(
                rate_limited_until = minute_rate_limited,
                "Slack app rate limited"
            );
            StatusCode::OK.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_path_constants() {
            assert_eq!(PATH_PREFIX, "/api");
            assert_eq!(SERVICE_ROUTE_KEY, "slack");
            assert_eq!(API_VERSION_1, "v1");
        }

        #[test]
        fn test_url_verification_response_serialization() {
            let response = UrlVerificationResponse {
                challenge: "test_challenge_123".to_string(),
            };
            let json = serde_json::to_string(&response).unwrap();
            assert!(json.contains("\"challenge\":\"test_challenge_123\""));
        }
    }
}
