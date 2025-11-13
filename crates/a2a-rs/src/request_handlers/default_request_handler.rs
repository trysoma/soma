use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use tokio::sync::Mutex;
use tokio::task::{AbortHandle, JoinHandle};
use tracing::{debug, error};

use crate::{
    agent_execution::{
        agent_executor::AgentExecutor, context::RequestContext,
        request_context_builder::RequestContextBuilder,
        simple_request_context_builder::SimpleRequestContextBuilder,
    },
    errors::{A2aServerError, ErrorBuilder},
    events::{
        event_consumer::EventConsumer,
        event_queue::{Event, EventQueue},
        in_memory_queue_manager::InMemoryQueueManager,
        queue_manager::QueueManager,
    },
    request_handlers::request_handler::RequestHandler,
    tasks::{
        AggregatedResult, ResultAggregator, TaskManager,
        push_notification_config_store::PushNotificationConfigStore,
        push_notification_sender::PushNotificationSender, store::TaskStore,
    },
    types::{
        DeleteTaskPushNotificationConfigParams, GetTaskPushNotificationConfigParams,
        ListTaskPushNotificationConfigParams, MessageSendParams, SendMessageSuccessResponseResult,
        SendStreamingMessageSuccessResponseResult, Task, TaskIdParams, TaskPushNotificationConfig,
        TaskQueryParams, TaskState,
    },
};

/// Set of terminal task states
fn is_terminal_state(state: &TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Canceled | TaskState::Failed | TaskState::Rejected
    )
}

/// Default request handler for all incoming requests.
///
/// This handler provides default implementations for all A2A JSON-RPC methods,
/// coordinating between the `AgentExecutor`, `TaskStore`, `QueueManager`,
/// and optional `PushNotifier`.
pub struct DefaultRequestHandler {
    agent_executor: Arc<dyn AgentExecutor + Send + Sync>,
    task_store: Arc<dyn TaskStore + Send + Sync>,
    queue_manager: Arc<dyn QueueManager + Send + Sync>,
    push_config_store: Option<Arc<dyn PushNotificationConfigStore + Send + Sync>>,
    push_sender: Option<Arc<dyn PushNotificationSender + Send + Sync>>,
    request_context_builder: Arc<dyn RequestContextBuilder + Send + Sync>,
    running_agents: Arc<Mutex<HashMap<String, AbortHandle>>>,
    result_aggregators: Arc<Mutex<HashMap<String, Arc<ResultAggregator>>>>,
}

impl DefaultRequestHandler {
    /// Creates a new DefaultRequestHandler instance
    pub fn new(
        agent_executor: Arc<dyn AgentExecutor + Send + Sync>,
        task_store: Arc<dyn TaskStore + Send + Sync>,
        queue_manager: Option<Arc<dyn QueueManager + Send + Sync>>,
        push_config_store: Option<Arc<dyn PushNotificationConfigStore + Send + Sync>>,
        push_sender: Option<Arc<dyn PushNotificationSender + Send + Sync>>,
        request_context_builder: Option<Arc<dyn RequestContextBuilder + Send + Sync>>,
    ) -> Self {
        let queue_manager = queue_manager.unwrap_or_else(|| Arc::new(InMemoryQueueManager::new()));

        let request_context_builder = request_context_builder
            .unwrap_or_else(|| Arc::new(SimpleRequestContextBuilder::new(false, None)));

        Self {
            agent_executor,
            task_store,
            queue_manager,
            push_config_store,
            push_sender,
            request_context_builder,
            running_agents: Arc::new(Mutex::new(HashMap::new())),
            result_aggregators: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Runs the agent's `execute` method and closes the queue afterwards.
    #[allow(dead_code)]
    async fn run_event_stream(&self, request: RequestContext, queue: EventQueue) {
        if let Err(e) = self.agent_executor.execute(request, queue.clone()).await {
            error!("Agent execution failed: {:?}", e);
        }
        queue.close().await;
    }

    /// Common setup logic for both streaming and non-streaming message handling.
    async fn setup_message_execution(
        &self,
        params: MessageSendParams,
    ) -> Result<
        (
            TaskManager,
            String,
            EventQueue,
            Arc<ResultAggregator>,
            JoinHandle<()>,
        ),
        A2aServerError,
    > {
        // Create task manager and validate existing task
        let task_manager = TaskManager::new(
            self.task_store.clone(),
            params.message.task_id.clone(),
            params.message.context_id.clone(),
            Some(params.message.clone()),
        );

        let mut task_manager_mut = task_manager;
        let task = task_manager_mut.get_task().await?;

        if let Some(ref task) = task {
            if is_terminal_state(&task.status.state) {
                return Err(A2aServerError::InvalidParamsError(
                    ErrorBuilder::default()
                        .message(format!(
                            "Task {} is in terminal state: {:?}",
                            task.id, task.status.state
                        ))
                        .build()
                        .unwrap(),
                ));
            }

            let _updated_task =
                task_manager_mut.update_with_message(params.message.clone(), task.clone());

            if self.should_add_push_info(&params) {
                if let (Some(push_config_store), Some(config)) =
                    (&self.push_config_store, &params.configuration)
                {
                    if let Some(ref push_config) = config.push_notification_config {
                        push_config_store.set_info(&task.id, push_config).await?;
                    }
                }
            }
        }

        // Build request context
        let request_context = self
            .request_context_builder
            .build(
                Some(params.clone()),
                task.as_ref().map(|t| t.id.clone()),
                task.as_ref().map(|t| t.context_id.clone()),
                task.clone(),
            )
            .await
            .map_err(|e| {
                A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message(format!("Failed to build request context: {e}"))
                        .build()
                        .unwrap(),
                )
            })?;

        let task_id = request_context
            .task_id()
            .ok_or_else(|| {
                A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Task ID not set in request context".to_string())
                        .build()
                        .unwrap(),
                )
            })?
            .to_string();

        // Create or get the event queue
        let queue = self.queue_manager.create_or_tap(&task_id).await;

        // Create or get the result aggregator for this task
        let result_aggregator = {
            let mut aggregators = self.result_aggregators.lock().await;
            if let Some(existing) = aggregators.get(&task_id) {
                debug!("Reusing existing ResultAggregator for task_id: {}", task_id);
                existing.clone()
            } else {
                debug!("Creating new ResultAggregator for task_id: {}", task_id);
                let new_aggregator = Arc::new(ResultAggregator::new(task_manager_mut.clone()));
                aggregators.insert(task_id.clone(), new_aggregator.clone());
                new_aggregator
            }
        };

        // Create the agent execution task
        let agent_executor = self.agent_executor.clone();
        let queue_clone = queue.clone();
        let request_context_for_agent = RequestContext::new(
            Some(params),
            Some(task_id.clone()),
            request_context.context_id().map(|s| s.to_string()),
            task,
            None,
        )
        .map_err(|e| {
            A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message(format!("Failed to create request context: {e:?}"))
                    .build()
                    .unwrap(),
            )
        })?;

        let producer_task = tokio::spawn(async move {
            tracing::info!("Starting agent execution task");
            if let Err(e) = agent_executor
                .execute(request_context_for_agent, queue_clone.clone())
                .await
            {
                let error_msg = format!("{e:?}");
                // Check if this is a connection closed error (happens during server shutdown)
                if error_msg.contains("connection closed before message completed") {
                    tracing::warn!(
                        "Agent execution connection closed (likely due to shutdown): {}",
                        error_msg
                    );
                } else {
                    error!("Agent execution failed: {:?}", e);
                }
            }
            tracing::info!("Agent execution task completed");
            // Don't close the queue here - let the consumer close it when it receives a final event
        });

        // Register the producer task
        self.register_producer(&task_id, &producer_task).await;

        Ok((
            task_manager_mut,
            task_id,
            queue,
            result_aggregator,
            producer_task,
        ))
    }

    /// Validates that agent-generated task ID matches the expected task ID.
    fn validate_task_id_match(
        &self,
        task_id: &str,
        event_task_id: &str,
    ) -> Result<(), A2aServerError> {
        if task_id != event_task_id {
            error!(
                "Agent generated task_id={} does not match the RequestContext task_id={}.",
                event_task_id, task_id
            );
            Err(A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message("Task ID mismatch in agent response".to_string())
                    .build()
                    .unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    /// Sends push notification if configured and task is available.
    async fn send_push_notification_if_needed(
        &self,
        _task_id: &str,
        result_aggregator: &ResultAggregator,
    ) -> Result<(), A2aServerError> {
        if let (Some(push_sender), Some(result)) =
            (&self.push_sender, result_aggregator.current_result().await)
            && let AggregatedResult::Task(task) = result
        {
            push_sender.send_notification(&task).await?;
        }
        Ok(())
    }

    /// Registers the agent execution task with the handler.
    async fn register_producer(&self, task_id: &str, producer_task: &JoinHandle<()>) {
        let mut running_agents = self.running_agents.lock().await;
        running_agents.insert(task_id.to_string(), producer_task.abort_handle());
    }

    /// Cleans up the agent execution task and queue manager entry.
    async fn cleanup_producer(&self, producer_task: JoinHandle<()>, task_id: &str) {
        debug!("Starting cleanup for task_id: {}", task_id);

        // Wait for the task to complete
        let _ = producer_task.await;

        // Close the queue (this is safe to call even if already closed)
        self.queue_manager.close(task_id).await.ok();

        // Remove from running agents
        let mut running_agents = self.running_agents.lock().await;
        running_agents.remove(task_id);

        // Remove the result aggregator
        let mut aggregators = self.result_aggregators.lock().await;
        aggregators.remove(task_id);

        debug!("Cleanup completed for task_id: {}", task_id);
    }

    /// Determines if push notification info should be set for a task.
    fn should_add_push_info(&self, params: &MessageSendParams) -> bool {
        self.push_config_store.is_some()
            && params.configuration.is_some()
            && params
                .configuration
                .as_ref()
                .unwrap()
                .push_notification_config
                .is_some()
    }
}

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    /// Default handler for 'tasks/get'.
    async fn on_get_task(&self, params: TaskQueryParams) -> Result<Option<Task>, A2aServerError> {
        let task = self.task_store.get(&params.id).await?;
        if task.is_none() {
            return Err(A2aServerError::TaskNotFoundError(
                ErrorBuilder::default().build().unwrap(),
            ));
        }
        Ok(task)
    }

    /// Default handler for 'tasks/cancel'.
    async fn on_cancel_task(&self, params: TaskIdParams) -> Result<Option<Task>, A2aServerError> {
        let task = self.task_store.get(&params.id).await?;
        let task = task.ok_or_else(|| {
            A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap())
        })?;

        let task_manager = TaskManager::new(
            self.task_store.clone(),
            Some(task.id.clone()),
            Some(task.context_id.clone()),
            None,
        );

        let result_aggregator = Arc::new(ResultAggregator::new(task_manager));

        let queue = self
            .queue_manager
            .tap(&task.id)
            .await
            .unwrap_or_else(|| EventQueue::new(1000));

        self.agent_executor
            .cancel(
                RequestContext::new(
                    None,
                    Some(task.id.clone()),
                    Some(task.context_id.clone()),
                    Some(task.clone()),
                    None,
                )?,
                queue.clone(),
            )
            .await
            .map_err(|e| {
                A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message(format!("Failed to cancel task: {e}"))
                        .build()
                        .unwrap(),
                )
            })?;

        // Cancel the ongoing task, if one exists
        if let Some(abort_handle) = self.running_agents.lock().await.get(&task.id) {
            abort_handle.abort();
        }

        let consumer = EventConsumer::new(queue);
        match result_aggregator.consume_all(consumer).await? {
            AggregatedResult::Task(task) => Ok(Some(task)),
            _ => Err(A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message("Agent did not return valid response for cancel".to_string())
                    .build()
                    .unwrap(),
            )),
        }
    }

    /// Default handler for 'message/send' interface (non-streaming).
    async fn on_message_send(
        &self,
        params: MessageSendParams,
    ) -> Result<SendMessageSuccessResponseResult, A2aServerError> {
        let (_task_manager, task_id, queue, result_aggregator, producer_task) =
            self.setup_message_execution(params).await?;

        let consumer = EventConsumer::new(queue);

        let (result, interrupted) = result_aggregator
            .consume_and_break_on_interrupt(consumer)
            .await?;

        let result = result.ok_or_else(|| {
            A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message("No result from agent execution".to_string())
                    .build()
                    .unwrap(),
            )
        })?;

        if let AggregatedResult::Task(task) = &result {
            self.validate_task_id_match(&task_id, &task.id)?;
        }

        self.send_push_notification_if_needed(&task_id, &result_aggregator)
            .await?;

        // Clean up
        if interrupted {
            // Track this disconnected cleanup task
            let self_clone = self.clone();
            let task_id_clone = task_id.clone();
            tokio::spawn(async move {
                self_clone
                    .cleanup_producer(producer_task, &task_id_clone)
                    .await;
            });
        } else {
            self.cleanup_producer(producer_task, &task_id).await;
        }

        // Convert to SendMessageSuccessResponseResult
        match result {
            AggregatedResult::Task(task) => Ok(SendMessageSuccessResponseResult::Task(task)),
            AggregatedResult::Message(message) => {
                Ok(SendMessageSuccessResponseResult::Message(message))
            }
        }
    }

    /// Default handler for 'message/stream' (streaming).
    async fn on_message_send_stream(
        &self,
        params: MessageSendParams,
    ) -> Result<
        Pin<
            Box<
                dyn Stream<Item = Result<SendStreamingMessageSuccessResponseResult, A2aServerError>>
                    + Send,
            >,
        >,
        A2aServerError,
    > {
        let self_clone = self.clone();
        let params_clone = params.clone();

        // Create an async stream
        let stream = async_stream::try_stream! {
            let (
                _task_manager,
                task_id,
                queue,
                result_aggregator,
                producer_task,
            ) = self_clone.setup_message_execution(params_clone.clone()).await?;

            let consumer = EventConsumer::new(queue);

            let push_config_store = self_clone.push_config_store.clone();
            let push_config = params_clone.configuration
                .and_then(|c| c.push_notification_config);

            let event_stream = result_aggregator.clone().consume_and_emit(consumer);
            tokio::pin!(event_stream);

            while let Some(event) = event_stream.next().await {
                // Validate task ID for Task events
                if let Event::Task(ref task) = event {
                    if let Err(e) = self_clone.validate_task_id_match(&task_id, &task.id) {
                        error!("Task ID validation failed: {:?}", e);
                    }
                }

                // Store push config if needed
                if let (Some(store), Some(config)) = (&push_config_store, &push_config) {
                    if let Err(e) = store.set_info(&task_id, config).await {
                        error!("Failed to store push config: {:?}", e);
                    }
                }

                // Send push notification if needed
                if let Err(e) = self_clone.send_push_notification_if_needed(
                    &task_id,
                    &result_aggregator,
                ).await {
                    error!("Failed to send push notification: {:?}", e);
                }

                // Convert Event to SendStreamingMessageSuccessResponseResult
                let result = match event {
                    Event::Task(task) => SendStreamingMessageSuccessResponseResult::Task(task),
                    Event::Message(message) => SendStreamingMessageSuccessResponseResult::Message(message),
                    Event::TaskStatusUpdate(update) => SendStreamingMessageSuccessResponseResult::TaskStatusUpdateEvent(update),
                    Event::TaskArtifactUpdate(update) => SendStreamingMessageSuccessResponseResult::TaskArtifactUpdateEvent(update),
                };

                yield result;
            }

            // Clean up when stream is done
            self_clone.cleanup_producer(producer_task, &task_id).await;
        };

        Ok(Box::pin(stream))
    }

    /// Default handler for 'tasks/pushNotificationConfig/set'.
    async fn on_set_task_push_notification_config(
        &self,
        params: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2aServerError> {
        let push_config_store = self.push_config_store.as_ref().ok_or_else(|| {
            A2aServerError::UnsupportedOperationError(ErrorBuilder::default().build().unwrap())
        })?;

        let _task = self.task_store.get(&params.task_id).await?.ok_or_else(|| {
            A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap())
        })?;

        push_config_store
            .set_info(&params.task_id, &params.push_notification_config)
            .await?;

        Ok(params)
    }

    /// Default handler for 'tasks/pushNotificationConfig/get'.
    async fn on_get_task_push_notification_config(
        &self,
        params: GetTaskPushNotificationConfigParams,
    ) -> Result<TaskPushNotificationConfig, A2aServerError> {
        let push_config_store = self.push_config_store.as_ref().ok_or_else(|| {
            A2aServerError::UnsupportedOperationError(ErrorBuilder::default().build().unwrap())
        })?;

        let _task = self.task_store.get(&params.id).await?.ok_or_else(|| {
            A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap())
        })?;

        let configs = push_config_store.get_info(&params.id).await?;
        let config = configs.into_iter().next().ok_or_else(|| {
            A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message("Push notification config not found".to_string())
                    .build()
                    .unwrap(),
            )
        })?;

        Ok(TaskPushNotificationConfig {
            task_id: params.id,
            push_notification_config: config,
        })
    }

    /// Default handler for 'tasks/resubscribe'.
    fn on_resubscribe_to_task(
        &self,
        params: TaskIdParams,
    ) -> Result<
        Pin<
            Box<
                dyn Stream<Item = Result<SendStreamingMessageSuccessResponseResult, A2aServerError>>
                    + Send,
            >,
        >,
        A2aServerError,
    > {
        // Clone what we need before creating the stream
        let task_store = self.task_store.clone();
        let queue_manager = self.queue_manager.clone();
        let result_aggregators = self.result_aggregators.clone();

        let stream = async_stream::try_stream! {
            let task = task_store.get(&params.id).await?
                .ok_or_else(|| A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap()))?;

            if is_terminal_state(&task.status.state) {
                Err(A2aServerError::InvalidParamsError(ErrorBuilder::default()
                    .message(format!("Task {} is in terminal state: {:?}", task.id, task.status.state))
                    .build()
                    .unwrap()))?;
            }

            // Check if there's already a result aggregator for this task
            let result_aggregator = {
                let aggregators = result_aggregators.lock().await;
                if let Some(existing) = aggregators.get(&task.id) {
                    existing.clone()
                } else {
                    // If not, create a new one
                    drop(aggregators); // Release the lock before creating new aggregator
                    let task_manager = TaskManager::new(
                        task_store.clone(),
                        Some(task.id.clone()),
                        Some(task.context_id.clone()),
                        None,
                    );
                    Arc::new(ResultAggregator::new(task_manager))
                }
            };

            let queue = queue_manager.tap(&task.id).await
                .ok_or_else(|| A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap()))?;

            let consumer = EventConsumer::new(queue);
            let event_stream = result_aggregator.clone().consume_and_emit(consumer);
            tokio::pin!(event_stream);

            while let Some(event) = event_stream.next().await {
                // Convert Event to SendStreamingMessageSuccessResponseResult
                let result = match event {
                    Event::Task(task) => SendStreamingMessageSuccessResponseResult::Task(task),
                    Event::Message(message) => SendStreamingMessageSuccessResponseResult::Message(message),
                    Event::TaskStatusUpdate(update) => SendStreamingMessageSuccessResponseResult::TaskStatusUpdateEvent(update),
                    Event::TaskArtifactUpdate(update) => SendStreamingMessageSuccessResponseResult::TaskArtifactUpdateEvent(update),
                };

                yield result;
            }
        };

        Ok(Box::pin(stream)
            as Pin<
                Box<
                    dyn Stream<
                            Item = Result<
                                SendStreamingMessageSuccessResponseResult,
                                A2aServerError,
                            >,
                        > + Send,
                >,
            >)
    }

    /// Default handler for 'tasks/pushNotificationConfig/list'.
    async fn on_list_task_push_notification_config(
        &self,
        params: ListTaskPushNotificationConfigParams,
    ) -> Result<Vec<TaskPushNotificationConfig>, A2aServerError> {
        let push_config_store = self.push_config_store.as_ref().ok_or_else(|| {
            A2aServerError::UnsupportedOperationError(ErrorBuilder::default().build().unwrap())
        })?;

        let _task = self.task_store.get(&params.id).await?.ok_or_else(|| {
            A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap())
        })?;

        let configs = push_config_store.get_info(&params.id).await?;

        let task_push_notification_configs = configs
            .into_iter()
            .map(|config| TaskPushNotificationConfig {
                task_id: params.id.clone(),
                push_notification_config: config,
            })
            .collect();

        Ok(task_push_notification_configs)
    }

    /// Default handler for 'tasks/pushNotificationConfig/delete'.
    async fn on_delete_task_push_notification_config(
        &self,
        params: DeleteTaskPushNotificationConfigParams,
    ) -> Result<(), A2aServerError> {
        let push_config_store = self.push_config_store.as_ref().ok_or_else(|| {
            A2aServerError::UnsupportedOperationError(ErrorBuilder::default().build().unwrap())
        })?;

        let _task = self.task_store.get(&params.id).await?.ok_or_else(|| {
            A2aServerError::TaskNotFoundError(ErrorBuilder::default().build().unwrap())
        })?;

        push_config_store
            .delete_info(&params.id, Some(&params.push_notification_config_id))
            .await?;

        Ok(())
    }
}

// Implement Clone for DefaultRequestHandler to support self cloning in async contexts
impl Clone for DefaultRequestHandler {
    fn clone(&self) -> Self {
        Self {
            agent_executor: self.agent_executor.clone(),
            task_store: self.task_store.clone(),
            queue_manager: self.queue_manager.clone(),
            push_config_store: self.push_config_store.clone(),
            push_sender: self.push_sender.clone(),
            request_context_builder: self.request_context_builder.clone(),
            running_agents: self.running_agents.clone(),
            result_aggregators: self.result_aggregators.clone(),
        }
    }
}
