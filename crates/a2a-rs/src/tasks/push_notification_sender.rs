use std::future::Future;
use std::pin::Pin;

use crate::{errors::A2aServerError, types::Task};

///Interface for sending push notifications for tasks.
pub trait PushNotificationSender: Send + Sync {
    ///Sends a push notification containing the latest task state
    fn send_notification<'a>(
        &'a self,
        task: &'a Task,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>>;
}
