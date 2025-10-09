use std::future::Future;
use std::pin::Pin;

use crate::{errors::A2aServerError, types::PushNotificationConfig};

///Interface for storing and retrieving push notification configurations for tasks."
pub trait PushNotificationConfigStore: Send + Sync {
    ///Sets or updates the push notification configuration for a task.
    fn set_info<'a>(
        &'a self,
        task_id: &'a String,
        notification_config: &'a PushNotificationConfig,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>>;
    ///Retrieves the push notification configuration for a task
    fn get_info<'a>(
        &'a self,
        task_id: &'a String,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<PushNotificationConfig>, A2aServerError>>
                + Send
                + Sync
                + 'a,
        >,
    >;
    ///Deletes the push notification configuration for a task
    fn delete_info<'a>(
        &'a self,
        task_id: &'a String,
        config_id: Option<&'a String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>>;
}
