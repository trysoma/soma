use async_trait::async_trait;
use derive_builder::Builder;
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::{
    errors::A2aServerError,
    tasks::{
        push_notification_config_store::PushNotificationConfigStore,
        push_notification_sender::PushNotificationSender,
    },
    types::{PushNotificationConfig, Task},
};

/// Base implementation of PushNotificationSender interface.
#[derive(Builder)]
pub struct BasePushNotificationSender {
    client: Arc<reqwest::Client>,
    config_store: Arc<dyn PushNotificationConfigStore + Send + Sync>,
}

#[async_trait]
impl PushNotificationSender for BasePushNotificationSender {
    async fn send_notification(&self, task: &Task) -> Result<(), A2aServerError> {
        let push_configs = self.config_store.get_info(&task.id).await?;
        if push_configs.is_empty() {
            return Ok(());
        }

        let mut futures = Vec::new();
        for push_info in push_configs {
            futures.push(self.dispatch_notification(task, push_info));
        }

        let results = futures::future::join_all(futures).await;

        if !results.iter().all(|r| *r) {
            warn!(
                "Some push notifications failed to send for task_id={}",
                &task.id
            );
        }

        Ok(())
    }
}

impl BasePushNotificationSender {
    async fn dispatch_notification(&self, task: &Task, push_info: PushNotificationConfig) -> bool {
        let url = &push_info.url;
        match self.client.post(url).json(task).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!(
                        "Push-notification sent for task_id={} to URL: {}",
                        &task.id, url
                    );
                    true
                } else {
                    error!(
                        "Push-notification failed for task_id={} to URL: {}. Status: {}",
                        &task.id,
                        url,
                        response.status()
                    );
                    false
                }
            }
            Err(e) => {
                error!(
                    "Error sending push-notification for task_id={} to URL: {}. Error: {}",
                    &task.id, url, e
                );
                false
            }
        }
    }
}
