use async_trait::async_trait;

use crate::{errors::A2aServerError, types::Task};

///Interface for sending push notifications for tasks.
#[async_trait]
pub trait PushNotificationSender: Send + Sync {
    ///Sends a push notification containing the latest task state
    async fn send_notification(&self, task: &Task) -> Result<(), A2aServerError>;
}
