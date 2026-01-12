//! Event handler for Slack inbox provider
//!
//! Listens to inbox events and sends messages to Slack in response.

use std::sync::Arc;

use inbox::logic::event::{InboxEventKind, MessageStreamingDelta, UiMessageDelta};
use inbox::logic::inbox::InboxHandle;
use inbox::logic::message::MessageRole;
use tracing::{error, trace, warn};

use crate::logic::SlackClient;
use crate::types::{SLACK_CHANNEL_KEY, SLACK_THREAD_TS_KEY, SLACK_TS_KEY};

/// Run the event handler for a Slack inbox
///
/// Subscribes to the event bus and sends messages to Slack when new messages
/// are created or streamed for this inbox.
pub async fn run_event_handler(handle: InboxHandle, client: Arc<SlackClient>) {
    let inbox_id = handle.inbox.id.clone();
    let mut rx = handle.subscribe();

    trace!(inbox_id = %inbox_id, "Starting Slack event handler");

    // Track streaming messages to update them
    let pending_messages: dashmap::DashMap<String, PendingMessage> = dashmap::DashMap::new();

    loop {
        match rx.recv().await {
            Ok(event) => {
                // Skip events from this inbox (prevent loops)
                if !event.should_deliver_to_inbox(&inbox_id) {
                    continue;
                }

                match &event.kind {
                    InboxEventKind::MessageCreated { message } => {
                        // Only send assistant messages to Slack
                        if message.role() != &MessageRole::Assistant {
                            continue;
                        }

                        // Get Slack metadata from inbox_settings
                        let settings = message.inbox_settings();
                        let channel = match settings.get(SLACK_CHANNEL_KEY) {
                            Some(serde_json::Value::String(c)) => c.clone(),
                            _ => {
                                trace!("No Slack channel in message settings, skipping");
                                continue;
                            }
                        };

                        let thread_ts = settings
                            .get(SLACK_THREAD_TS_KEY)
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .or_else(|| {
                                settings
                                    .get(SLACK_TS_KEY)
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                            });

                        let text = message.text_content();
                        if text.is_empty() {
                            continue;
                        }

                        // Check if we have a pending streamed message to update
                        let message_id = message.id().to_string();
                        if let Some((_, pending)) = pending_messages.remove(&message_id) {
                            // Update the existing message with final content
                            if let Err(e) = client
                                .update_message(&pending.channel, &pending.ts, &text)
                                .await
                            {
                                error!(error = %e, "Failed to update Slack message");
                            }
                        } else {
                            // Send new message to Slack
                            if let Err(e) = client
                                .post_message(&channel, &text, thread_ts.as_deref())
                                .await
                            {
                                error!(error = %e, "Failed to send message to Slack");
                            }
                        }
                    }

                    InboxEventKind::MessageStreaming {
                        message_id,
                        delta,
                        ..
                    } => {
                        let message_id_str = message_id.to_string();

                        // Extract text from delta
                        let text_delta = match delta {
                            MessageStreamingDelta::Text(t) => Some(t.delta.clone()),
                            MessageStreamingDelta::Ui(UiMessageDelta::TextDelta { delta }) => {
                                Some(delta.clone())
                            }
                            _ => None,
                        };

                        if let Some(text) = text_delta {
                            // Update or create pending message
                            let mut pending = pending_messages
                                .entry(message_id_str.clone())
                                .or_insert_with(|| PendingMessage::new());

                            pending.accumulated_text.push_str(&text);

                            // If we don't have a Slack message yet, create one
                            // This requires we have channel info from somewhere
                            // For now, we'll just accumulate and send on completion
                        }
                    }

                    InboxEventKind::MessageUpdated { message } => {
                        // Handle message updates if needed
                        if message.role() != &MessageRole::Assistant {
                            continue;
                        }

                        let settings = message.inbox_settings();
                        let channel = match settings.get(SLACK_CHANNEL_KEY) {
                            Some(serde_json::Value::String(c)) => c.clone(),
                            _ => continue,
                        };

                        // If we have a Slack timestamp, update that message
                        if let Some(serde_json::Value::String(ts)) = settings.get(SLACK_TS_KEY) {
                            let text = message.text_content();
                            if let Err(e) = client.update_message(&channel, ts, &text).await {
                                error!(error = %e, "Failed to update Slack message");
                            }
                        }
                    }

                    _ => {
                        // Ignore other event types
                    }
                }
            }

            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!(skipped = n, "Event handler lagged, skipped events");
            }

            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                trace!("Event channel closed, stopping handler");
                break;
            }
        }
    }

    trace!(inbox_id = %inbox_id, "Slack event handler stopped");
}

/// Represents a message being streamed that hasn't been finalized yet
#[allow(dead_code)]
struct PendingMessage {
    /// Accumulated text content
    accumulated_text: String,
    /// Slack channel ID (if known)
    channel: String,
    /// Slack message timestamp (if we've posted it)
    ts: String,
    /// Thread timestamp to reply to
    thread_ts: Option<String>,
}

impl PendingMessage {
    fn new() -> Self {
        Self {
            accumulated_text: String::new(),
            channel: String::new(),
            ts: String::new(),
            thread_ts: None,
        }
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_pending_message_creation() {
            let pending = PendingMessage::new();
            assert!(pending.accumulated_text.is_empty());
            assert!(pending.channel.is_empty());
            assert!(pending.ts.is_empty());
            assert!(pending.thread_ts.is_none());
        }
    }
}
