use async_trait::async_trait;

use crate::{errors::A2aServerError, types::PushNotificationConfig};

///Interface for storing and retrieving push notification configurations for tasks."
#[async_trait]
pub trait PushNotificationConfigStore: Send + Sync {
    ///Sets or updates the push notification configuration for a task.
    async fn set_info(
        &self,
        task_id: &str,
        notification_config: &PushNotificationConfig,
    ) -> Result<(), A2aServerError>;

    ///Retrieves the push notification configuration for a task
    async fn get_info(&self, task_id: &str) -> Result<Vec<PushNotificationConfig>, A2aServerError>;

    ///Deletes the push notification configuration for a task
    async fn delete_info(
        &self,
        task_id: &str,
        config_id: Option<&String>,
    ) -> Result<(), A2aServerError>;
}
