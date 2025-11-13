use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

use crate::{
    errors::A2aServerError,
    types::{
        DeleteTaskPushNotificationConfigParams, GetTaskPushNotificationConfigParams,
        ListTaskPushNotificationConfigParams, MessageSendParams, SendMessageSuccessResponseResult,
        SendStreamingMessageSuccessResponseResult, Task, TaskIdParams, TaskPushNotificationConfig,
        TaskQueryParams,
    },
};

type TaskSubscriptionStream = Pin<
    Box<
        dyn Stream<Item = Result<SendStreamingMessageSuccessResponseResult, A2aServerError>> + Send,
    >,
>;

/// A2A request handler interface.
///
/// This interface defines the methods that an A2A server implementation must
/// provide to handle incoming JSON-RPC requests.
#[async_trait]
pub trait RequestHandler {
    /// Handles the 'tasks/get' method.
    ///
    /// Retrieves the state and history of a specific task.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters specifying the task ID and optionally history length.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// The `Task` object if found, otherwise `None`.
    async fn on_get_task(&self, params: TaskQueryParams) -> Result<Option<Task>, A2aServerError>;
    /// Handles the 'tasks/cancel' method.
    ///
    /// Requests the agent to cancel an ongoing task.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters specifying the task ID.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// The `Task` object with its status updated to canceled, or `None` if the task was not found.
    async fn on_cancel_task(&self, params: TaskIdParams) -> Result<Option<Task>, A2aServerError>;
    /// Handles the 'message/send' method (non-streaming).
    ///
    /// Sends a message to the agent to create, continue, or restart a task,
    /// and waits for the final result (Task or Message).
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the message and configuration.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// The final `Task` object or a final `Message` object.
    async fn on_message_send(
        &self,
        params: MessageSendParams,
    ) -> Result<SendMessageSuccessResponseResult, A2aServerError>;
    /// Handles the 'message/stream' method (streaming).
    ///
    /// Sends a message to the agent and yields stream events as they are
    /// produced (Task updates, Message chunks, Artifact updates).
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the message and configuration.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// A stream of `Event` objects from the agent's execution.
    ///
    /// # Errors
    ///
    /// Returns `A2aServerError::UnsupportedOperationError` by default if not implemented.
    async fn on_message_send_stream(
        &self,
        _params: MessageSendParams,
    ) -> Result<TaskSubscriptionStream, A2aServerError>;

    /// Handles the 'tasks/pushNotificationConfig/set' method.
    ///
    /// Sets or updates the push notification configuration for a task.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the task ID and push notification configuration.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// The provided `TaskPushNotificationConfig` upon success.
    async fn on_set_task_push_notification_config(
        &self,
        params: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2aServerError>;

    /// Handles the 'tasks/pushNotificationConfig/get' method.
    ///
    /// Retrieves the current push notification configuration for a task.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the task ID.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// The `TaskPushNotificationConfig` for the task.
    async fn on_get_task_push_notification_config(
        &self,
        params: GetTaskPushNotificationConfigParams,
    ) -> Result<TaskPushNotificationConfig, A2aServerError>;

    /// Handles the 'tasks/resubscribe' method.
    ///
    /// Allows a client to re-subscribe to a running streaming task's event stream.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the task ID.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// A stream of `Event` objects from the agent's ongoing execution for the specified task.
    ///
    /// # Errors
    ///
    /// Returns `A2aServerError::UnsupportedOperationError` by default if not implemented.
    fn on_resubscribe_to_task(
        &self,
        _params: TaskIdParams,
    ) -> Result<TaskSubscriptionStream, A2aServerError>;

    /// Handles the 'tasks/pushNotificationConfig/list' method.
    ///
    /// Retrieves the current push notification configurations for a task.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the task ID.
    /// * `context` - Context provided by the server.
    ///
    /// # Returns
    ///
    /// The `Vec<TaskPushNotificationConfig>` for the task.
    async fn on_list_task_push_notification_config(
        &self,
        params: ListTaskPushNotificationConfigParams,
    ) -> Result<Vec<TaskPushNotificationConfig>, A2aServerError>;

    /// Handles the 'tasks/pushNotificationConfig/delete' method.
    ///
    /// Deletes a push notification configuration associated with a task.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters including the task ID.
    /// * `context` - Context provided by the server.
    async fn on_delete_task_push_notification_config(
        &self,
        params: DeleteTaskPushNotificationConfigParams,
    ) -> Result<(), A2aServerError>;
}
