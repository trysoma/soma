//! Push notification logic for A2A protocol
//!
//! Provides functions for managing push notification configurations and
//! sending task update notifications to configured webhooks per A2A spec section 3.17.

use crate::a2a_core::types::{PushNotificationConfig, Task, TaskArtifactUpdateEvent, TaskStatusUpdateEvent};
use crate::task_repository::{Repository, TaskRepositoryLike};
use shared::error::CommonError;
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use tracing::{debug, error, warn};

/// Domain model for push notification configuration
#[derive(Debug, Clone)]
pub struct PushNotificationConfigModel {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub url: String,
    pub token: Option<String>,
    pub authentication: Option<serde_json::Value>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<PushNotificationConfigModel> for PushNotificationConfig {
    fn from(model: PushNotificationConfigModel) -> Self {
        PushNotificationConfig {
            id: Some(model.id.to_string()),
            url: model.url,
            token: model.token,
            authentication: model.authentication.and_then(|v| serde_json::from_value(v).ok()),
        }
    }
}

/// Parameters for creating a push notification configuration
#[derive(Debug)]
pub struct CreatePushNotificationConfig {
    pub id: WrappedUuidV4,
    pub task_id: WrappedUuidV4,
    pub url: String,
    pub token: Option<String>,
    pub authentication: Option<WrappedJsonValue>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

/// Parameters for updating a push notification configuration
#[derive(Debug)]
pub struct UpdatePushNotificationConfig {
    pub id: WrappedUuidV4,
    pub url: String,
    pub token: Option<String>,
    pub authentication: Option<WrappedJsonValue>,
    pub updated_at: WrappedChronoDateTime,
}

/// Set or update a push notification configuration for a task
pub async fn set_push_notification_config(
    repository: &Repository,
    task_id: &WrappedUuidV4,
    config: &PushNotificationConfig,
) -> Result<PushNotificationConfig, CommonError> {
    let now = WrappedChronoDateTime::now();
    let config_id = config
        .id
        .as_ref()
        .map(|s| WrappedUuidV4::try_from(s.clone()))
        .transpose()?
        .unwrap_or_else(WrappedUuidV4::new);

    // Check if config already exists
    let existing = repository.get_push_notification_config_by_id(&config_id).await?;

    let authentication = config
        .authentication
        .as_ref()
        .map(|a| WrappedJsonValue::new(serde_json::to_value(a).unwrap_or_default()));

    if existing.is_some() {
        // Update existing config
        repository
            .update_push_notification_config(&UpdatePushNotificationConfig {
                id: config_id.clone(),
                url: config.url.clone(),
                token: config.token.clone(),
                authentication,
                updated_at: now,
            })
            .await?;
    } else {
        // Create new config
        repository
            .create_push_notification_config(&CreatePushNotificationConfig {
                id: config_id.clone(),
                task_id: task_id.clone(),
                url: config.url.clone(),
                token: config.token.clone(),
                authentication,
                created_at: now,
                updated_at: now,
            })
            .await?;
    }

    Ok(PushNotificationConfig {
        id: Some(config_id.to_string()),
        url: config.url.clone(),
        token: config.token.clone(),
        authentication: config.authentication.clone(),
    })
}

/// Get all push notification configurations for a task
pub async fn get_push_notification_configs(
    repository: &Repository,
    task_id: &WrappedUuidV4,
) -> Result<Vec<PushNotificationConfig>, CommonError> {
    let configs = repository.get_push_notification_configs_by_task_id(task_id).await?;
    Ok(configs.into_iter().map(|c| c.into()).collect())
}

/// Delete a specific push notification configuration
pub async fn delete_push_notification_config(
    repository: &Repository,
    task_id: &WrappedUuidV4,
    config_id: Option<&WrappedUuidV4>,
) -> Result<(), CommonError> {
    if let Some(id) = config_id {
        repository.delete_push_notification_config(id).await
    } else {
        repository.delete_push_notification_configs_by_task_id(task_id).await
    }
}

/// Send push notification for a task status update
pub async fn send_task_status_notification(
    client: &reqwest::Client,
    repository: &Repository,
    task_id: &WrappedUuidV4,
    status_update: &TaskStatusUpdateEvent,
) -> Result<(), CommonError> {
    let configs = repository.get_push_notification_configs_by_task_id(task_id).await?;
    if configs.is_empty() {
        debug!(task_id = %task_id, "No push notification configs for task, skipping notification");
        return Ok(());
    }

    for config in configs {
        send_notification(client, &config.url, config.token.as_deref(), status_update).await;
    }

    Ok(())
}

/// Send push notification for a task artifact update
pub async fn send_task_artifact_notification(
    client: &reqwest::Client,
    repository: &Repository,
    task_id: &WrappedUuidV4,
    artifact_update: &TaskArtifactUpdateEvent,
) -> Result<(), CommonError> {
    let configs = repository.get_push_notification_configs_by_task_id(task_id).await?;
    if configs.is_empty() {
        debug!(task_id = %task_id, "No push notification configs for task, skipping notification");
        return Ok(());
    }

    for config in configs {
        send_notification(client, &config.url, config.token.as_deref(), artifact_update).await;
    }

    Ok(())
}

/// Send push notification for a full task update
pub async fn send_task_notification(
    client: &reqwest::Client,
    repository: &Repository,
    task: &Task,
) -> Result<(), CommonError> {
    let task_id = WrappedUuidV4::try_from(task.id.clone())?;
    let configs = repository.get_push_notification_configs_by_task_id(&task_id).await?;
    if configs.is_empty() {
        debug!(task_id = %task.id, "No push notification configs for task, skipping notification");
        return Ok(());
    }

    for config in configs {
        send_notification(client, &config.url, config.token.as_deref(), task).await;
    }

    Ok(())
}

/// Internal helper to send a notification to a single endpoint
async fn send_notification<T: serde::Serialize>(
    client: &reqwest::Client,
    url: &str,
    token: Option<&str>,
    payload: &T,
) {
    let mut request = client.post(url).json(payload);

    // Add authorization header if token is provided
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    match request.send().await {
        Ok(response) => {
            if response.status().is_success() {
                debug!(url = %url, "Push notification sent successfully");
            } else {
                error!(
                    url = %url,
                    status = %response.status(),
                    "Push notification failed"
                );
            }
        }
        Err(e) => {
            warn!(url = %url, error = %e, "Error sending push notification");
        }
    }
}
