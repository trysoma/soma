use std::sync::Arc;

use futures::stream::Stream;
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

use crate::{
    errors::{A2aServerError, ErrorBuilder},
    events::{event_consumer::EventConsumer, event_queue::Event},
    tasks::TaskManager,
    types::{Message, Task, TaskState},
};

/// Extension trait for EventConsumer
#[allow(async_fn_in_trait)]
pub trait EventConsumerExt {
    async fn recv(&mut self) -> Option<Event>;
}

impl EventConsumerExt for EventConsumer {
    /// Helper method for ResultAggregator to receive events
    async fn recv(&mut self) -> Option<Event> {
        match self.consume_one_blocking().await {
            Ok(event) => Some(event),
            Err(e) => {
                // Only log as debug for queue closed, as this is expected
                if e.to_string().contains("Queue is closed") {
                    debug!("Queue closed, stopping event stream");
                } else {
                    warn!("Error consuming event: {:?}", e);
                }
                None
            }
        }
    }
}

/// The final result of an agent execution
#[derive(Clone, Debug)]
pub enum AggregatedResult {
    Task(Task),
    Message(Message),
}

/// Aggregates events from an agent execution and produces a final result.
/// Manages task lifecycle based on received events.
pub struct ResultAggregator {
    task_manager: Mutex<TaskManager>,
    current_result: Mutex<Option<AggregatedResult>>,
}

impl ResultAggregator {
    /// Creates a new ResultAggregator with the given TaskManager
    pub fn new(task_manager: TaskManager) -> Self {
        Self {
            task_manager: Mutex::new(task_manager),
            current_result: Mutex::new(None),
        }
    }

    /// Get the current result
    pub async fn current_result(&self) -> Option<AggregatedResult> {
        self.current_result.lock().await.clone()
    }

    /// Process a single event and update the aggregated result
    async fn process_event(&self, event: Event) -> Result<AggregatedResult, A2aServerError> {
        let mut task_manager = self.task_manager.lock().await;

        match event {
            Event::Task(task) => {
                debug!("Processing Task event for task_id: {}", task.id);
                let saved_task = task_manager.save_task(task).await?;
                Ok(AggregatedResult::Task(saved_task))
            }
            Event::Message(message) => {
                debug!("Processing Message event");
                Ok(AggregatedResult::Message(message))
            }
            Event::TaskStatusUpdate(status_update) => {
                debug!(
                    "Processing TaskStatusUpdate event for task_id: {} with state: {:?}",
                    status_update.task_id, status_update.status.state
                );
                let updated_task = task_manager.save_task_status_update(status_update).await?;
                Ok(AggregatedResult::Task(updated_task))
            }
            Event::TaskArtifactUpdate(artifact_update) => {
                debug!(
                    "Processing TaskArtifactUpdate event for task_id: {} with artifact_id: {}",
                    artifact_update.task_id, artifact_update.artifact.artifact_id
                );
                let updated_task = task_manager
                    .save_task_artifact_update(artifact_update)
                    .await?;
                Ok(AggregatedResult::Task(updated_task))
            }
        }
    }

    /// Check if the result indicates a terminal state
    fn is_terminal_state(result: &AggregatedResult) -> bool {
        match result {
            AggregatedResult::Task(task) => matches!(
                task.status.state,
                TaskState::Completed
                    | TaskState::Canceled
                    | TaskState::Failed
                    | TaskState::Rejected
            ),
            AggregatedResult::Message(_) => true, // Messages are always terminal
        }
    }

    /// Check if the result indicates an interrupt state
    fn is_interrupt_state(result: &AggregatedResult) -> bool {
        match result {
            AggregatedResult::Task(task) => task.status.state == TaskState::InputRequired,
            AggregatedResult::Message(_) => false,
        }
    }

    /// Consume all events from the consumer and return the final result
    pub async fn consume_all(
        &self,
        mut consumer: EventConsumer,
    ) -> Result<AggregatedResult, A2aServerError> {
        use EventConsumerExt;

        let mut last_result = None;

        while let Some(event) = consumer.recv().await {
            match self.process_event(event).await {
                Ok(result) => {
                    let is_terminal = Self::is_terminal_state(&result);
                    last_result = Some(result.clone());
                    *self.current_result.lock().await = Some(result);

                    if is_terminal {
                        debug!("Reached terminal state, stopping consumption");
                        break;
                    }
                }
                Err(e) => {
                    error!("Error processing event: {:?}", e);
                    return Err(e);
                }
            }
        }

        last_result.ok_or_else(|| {
            A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message("No events received from agent".to_string())
                    .build()
                    .unwrap(),
            )
        })
    }

    /// Consume events and break on interrupt (action required) state
    pub async fn consume_and_break_on_interrupt(
        &self,
        mut consumer: EventConsumer,
    ) -> Result<(Option<AggregatedResult>, bool), A2aServerError> {
        use EventConsumerExt;

        let mut last_result = None;
        let mut interrupted = false;

        while let Some(event) = consumer.recv().await {
            match self.process_event(event).await {
                Ok(result) => {
                    let is_terminal = Self::is_terminal_state(&result);
                    let is_interrupt = Self::is_interrupt_state(&result);

                    last_result = Some(result.clone());
                    *self.current_result.lock().await = Some(result);

                    if is_terminal {
                        debug!("Reached terminal state, stopping consumption");
                        break;
                    }

                    if is_interrupt {
                        debug!("Reached interrupt state (action required), pausing consumption");
                        interrupted = true;
                        break;
                    }
                }
                Err(e) => {
                    error!("Error processing event: {:?}", e);
                    return Err(e);
                }
            }
        }

        Ok((last_result, interrupted))
    }

    /// Consume events and yield them as a stream
    pub fn consume_and_emit(
        self: Arc<Self>,
        mut consumer: EventConsumer,
    ) -> impl Stream<Item = Event> + 'static {
        async_stream::stream! {
            use EventConsumerExt;

            // Add a small delay to allow the agent to start executing
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            while let Some(event) = consumer.recv().await {
                let event_clone = event.clone();

                // Process the event asynchronously
                match self.process_event(event_clone.clone()).await {
                    Ok(result) => {
                        if Self::is_terminal_state(&result) {
                            debug!("Terminal state reached in stream");
                        }
                        // Only yield the event if it was processed successfully
                        yield event;
                    }
                    Err(e) => {
                        // Log the error but continue processing other events
                        let error_msg = e.to_string();
                        if error_msg.contains("Task not found") ||
                           error_msg.contains("task_id is not set") {
                            debug!("Skipping event due to missing task information: {:?}", e);
                        } else {
                            error!("Error processing event in stream: {:?}", e);
                        }
                        // Don't yield events that failed to process
                    }
                }
            }
        }
    }
}
