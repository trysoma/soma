//! Slack API Client
//!
//! HTTP client for interacting with Slack's Web API.

use reqwest::Client;
use serde_json::Value;
use tracing::{error, trace};

use crate::types::{SlackPostMessageRequest, SlackPostMessageResponse, SlackUpdateMessageRequest};

const SLACK_API_BASE: &str = "https://slack.com/api";

/// HTTP client for Slack API
pub struct SlackClient {
    client: Client,
    bot_token: String,
}

impl SlackClient {
    /// Create a new Slack client with the given bot token
    pub fn new(bot_token: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
        }
    }

    /// Post a message to a Slack channel
    pub async fn post_message(
        &self,
        channel: &str,
        text: &str,
        thread_ts: Option<&str>,
    ) -> Result<SlackPostMessageResponse, SlackClientError> {
        let request = SlackPostMessageRequest {
            channel: channel.to_string(),
            text: Some(text.to_string()),
            blocks: None,
            thread_ts: thread_ts.map(String::from),
            reply_broadcast: None,
            metadata: None,
        };

        self.post_message_raw(request).await
    }

    /// Post a message with blocks to a Slack channel
    pub async fn post_message_with_blocks(
        &self,
        channel: &str,
        text: Option<&str>,
        blocks: Vec<Value>,
        thread_ts: Option<&str>,
    ) -> Result<SlackPostMessageResponse, SlackClientError> {
        let request = SlackPostMessageRequest {
            channel: channel.to_string(),
            text: text.map(String::from),
            blocks: Some(blocks),
            thread_ts: thread_ts.map(String::from),
            reply_broadcast: None,
            metadata: None,
        };

        self.post_message_raw(request).await
    }

    /// Post a raw message request to Slack
    pub async fn post_message_raw(
        &self,
        request: SlackPostMessageRequest,
    ) -> Result<SlackPostMessageResponse, SlackClientError> {
        trace!(channel = %request.channel, "Posting message to Slack");

        let response = self
            .client
            .post(format!("{}/chat.postMessage", SLACK_API_BASE))
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&request)
            .send()
            .await
            .map_err(SlackClientError::Request)?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(SlackClientError::Request)?;

        let result: SlackPostMessageResponse =
            serde_json::from_str(&body).map_err(|e| SlackClientError::Parse {
                body: body.clone(),
                error: e,
            })?;

        if !result.ok {
            error!(
                error = ?result.error,
                status = %status,
                "Slack API error"
            );
            return Err(SlackClientError::Api {
                error: result.error.unwrap_or_else(|| "unknown".to_string()),
                response_metadata: result.response_metadata,
            });
        }

        trace!(ts = ?result.ts, "Message posted successfully");
        Ok(result)
    }

    /// Update an existing message in Slack
    pub async fn update_message(
        &self,
        channel: &str,
        ts: &str,
        text: &str,
    ) -> Result<SlackPostMessageResponse, SlackClientError> {
        let request = SlackUpdateMessageRequest {
            channel: channel.to_string(),
            ts: ts.to_string(),
            text: Some(text.to_string()),
            blocks: None,
        };

        self.update_message_raw(request).await
    }

    /// Update a message with blocks
    pub async fn update_message_with_blocks(
        &self,
        channel: &str,
        ts: &str,
        text: Option<&str>,
        blocks: Vec<Value>,
    ) -> Result<SlackPostMessageResponse, SlackClientError> {
        let request = SlackUpdateMessageRequest {
            channel: channel.to_string(),
            ts: ts.to_string(),
            text: text.map(String::from),
            blocks: Some(blocks),
        };

        self.update_message_raw(request).await
    }

    /// Update a raw message request
    pub async fn update_message_raw(
        &self,
        request: SlackUpdateMessageRequest,
    ) -> Result<SlackPostMessageResponse, SlackClientError> {
        trace!(channel = %request.channel, ts = %request.ts, "Updating message in Slack");

        let response = self
            .client
            .post(format!("{}/chat.update", SLACK_API_BASE))
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&request)
            .send()
            .await
            .map_err(SlackClientError::Request)?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(SlackClientError::Request)?;

        let result: SlackPostMessageResponse =
            serde_json::from_str(&body).map_err(|e| SlackClientError::Parse {
                body: body.clone(),
                error: e,
            })?;

        if !result.ok {
            error!(
                error = ?result.error,
                status = %status,
                "Slack API error during update"
            );
            return Err(SlackClientError::Api {
                error: result.error.unwrap_or_else(|| "unknown".to_string()),
                response_metadata: result.response_metadata,
            });
        }

        trace!(ts = ?result.ts, "Message updated successfully");
        Ok(result)
    }

    /// Delete a message from Slack
    pub async fn delete_message(
        &self,
        channel: &str,
        ts: &str,
    ) -> Result<(), SlackClientError> {
        trace!(channel = %channel, ts = %ts, "Deleting message from Slack");

        let response = self
            .client
            .post(format!("{}/chat.delete", SLACK_API_BASE))
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&serde_json::json!({
                "channel": channel,
                "ts": ts
            }))
            .send()
            .await
            .map_err(SlackClientError::Request)?;

        let body = response
            .text()
            .await
            .map_err(SlackClientError::Request)?;

        let result: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| SlackClientError::Parse {
                body: body.clone(),
                error: e,
            })?;

        if result["ok"].as_bool() != Some(true) {
            let error = result["error"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            return Err(SlackClientError::Api {
                error,
                response_metadata: None,
            });
        }

        trace!("Message deleted successfully");
        Ok(())
    }

    /// Add a reaction to a message
    pub async fn add_reaction(
        &self,
        channel: &str,
        ts: &str,
        emoji: &str,
    ) -> Result<(), SlackClientError> {
        trace!(channel = %channel, ts = %ts, emoji = %emoji, "Adding reaction");

        let response = self
            .client
            .post(format!("{}/reactions.add", SLACK_API_BASE))
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json; charset=utf-8")
            .json(&serde_json::json!({
                "channel": channel,
                "timestamp": ts,
                "name": emoji
            }))
            .send()
            .await
            .map_err(SlackClientError::Request)?;

        let body = response
            .text()
            .await
            .map_err(SlackClientError::Request)?;

        let result: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| SlackClientError::Parse {
                body: body.clone(),
                error: e,
            })?;

        if result["ok"].as_bool() != Some(true) {
            let error = result["error"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            // Don't error on "already_reacted"
            if error != "already_reacted" {
                return Err(SlackClientError::Api {
                    error,
                    response_metadata: None,
                });
            }
        }

        Ok(())
    }
}

/// Errors that can occur when interacting with Slack API
#[derive(Debug, thiserror::Error)]
pub enum SlackClientError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Failed to parse response: {error}, body: {body}")]
    Parse {
        body: String,
        #[source]
        error: serde_json::Error,
    },

    #[error("Slack API error: {error}")]
    Api {
        error: String,
        response_metadata: Option<Value>,
    },
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_client_creation() {
            let _client = SlackClient::new("xoxb-test-token".to_string());
            // Just verify it doesn't panic
        }
    }
}
