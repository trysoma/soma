use async_trait::async_trait;
use derive_builder::Builder;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::{
    errors::A2aServerError, tasks::push_notification_config_store::PushNotificationConfigStore,
    types::PushNotificationConfig,
};

/// In-memory implementation of PushNotificationConfigStore interface.
/// Stores push notification configurations in memory
#[derive(Builder)]
pub struct InMemoryPushNotificationConfigStore {
    push_notification_infos: Arc<RwLock<HashMap<String, Vec<PushNotificationConfig>>>>,
}

#[async_trait]
impl PushNotificationConfigStore for InMemoryPushNotificationConfigStore {
    async fn set_info(
        &self,
        task_id: &str,
        notification_config: &PushNotificationConfig,
    ) -> Result<(), A2aServerError> {
        let mut infos = self.push_notification_infos.write().await;

        // Get or create the vector for this task_id
        let configs = infos.entry(task_id.to_string()).or_insert_with(Vec::new);

        // Set the id if it's None
        let mut config_to_store = notification_config.clone();
        if config_to_store.id.is_none() {
            config_to_store.id = Some(task_id.to_string());
        }

        // Remove existing config with same id if it exists
        if let Some(ref config_id) = config_to_store.id {
            configs.retain(|c| c.id.as_ref() != Some(config_id));
        }

        // Add the new config
        configs.push(config_to_store);

        debug!(
            "Push notification config for task {} saved successfully.",
            task_id
        );
        Ok(())
    }

    async fn get_info(&self, task_id: &str) -> Result<Vec<PushNotificationConfig>, A2aServerError> {
        debug!(
            "Attempting to get push notification configs for task: {}",
            task_id
        );
        let infos = self.push_notification_infos.read().await;
        let configs = infos.get(task_id).cloned().unwrap_or_default();

        if configs.is_empty() {
            debug!("No push notification configs found for task {}.", task_id);
        } else {
            debug!(
                "{} push notification configs retrieved for task {}.",
                configs.len(),
                task_id
            );
        }

        Ok(configs)
    }

    async fn delete_info(
        &self,
        task_id: &str,
        config_id: Option<&String>,
    ) -> Result<(), A2aServerError> {
        debug!(
            "Attempting to delete push notification config for task: {}",
            task_id
        );
        let mut infos = self.push_notification_infos.write().await;

        let task_id_str = task_id.to_string();
        let config_id_to_delete = config_id.unwrap_or(&task_id_str);

        if let Some(configs) = infos.get_mut(task_id) {
            let initial_len = configs.len();
            configs.retain(|c| c.id.as_ref() != Some(&config_id_to_delete.to_string()));

            if configs.len() < initial_len {
                debug!(
                    "Push notification config {} deleted successfully for task {}.",
                    config_id_to_delete, task_id
                );
            } else {
                debug!(
                    "Push notification config {} not found for task {}.",
                    config_id_to_delete, task_id
                );
            }

            // Remove the entry if no configs remain
            if configs.is_empty() {
                infos.remove(task_id);
            }
        } else {
            debug!("No push notification configs found for task {}.", task_id);
        }

        Ok(())
    }
}
