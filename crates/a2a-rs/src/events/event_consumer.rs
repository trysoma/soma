use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{debug, error, warn};

use crate::{
    errors::{A2aServerError, ErrorBuilder},
    events::event_queue::{DequeueError, Event, EventQueue},
    types::TaskState,
};

/// Consumer to read events from the agent event queue.
pub struct EventConsumer {
    queue: EventQueue,
    timeout_duration: Duration,
    exception: Arc<Mutex<Option<A2aServerError>>>,
}

impl EventConsumer {
    /// Initializes the EventConsumer.
    pub fn new(queue: EventQueue) -> Self {
        debug!("EventConsumer initialized");
        Self {
            queue,
            timeout_duration: Duration::from_millis(500),
            exception: Arc::new(Mutex::new(None)),
        }
    }

    /// Consume one event from the agent event queue non-blocking.
    pub async fn consume_one(&self) -> Result<Event, A2aServerError> {
        debug!("Attempting to consume one event.");

        match self.queue.dequeue_event(true).await {
            Ok(event) => {
                debug!("Dequeued event of type: {:?} in consume_one.", event);
                self.queue.task_done();
                Ok(event)
            }
            Err(DequeueError::QueueEmpty) => {
                warn!(
                    "Event queue was empty in consume_one. This might be a timing issue where the consumer is trying to read before the agent has enqueued events."
                );
                Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Agent did not return any response".to_string())
                        .build()
                        .unwrap(),
                ))
            }
            Err(DequeueError::QueueClosed) => {
                warn!("Event queue was closed in consume_one.");
                Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Queue is closed".to_string())
                        .build()
                        .unwrap(),
                ))
            }
        }
    }

    /// Consume one event from the agent event queue with blocking.
    pub async fn consume_one_blocking(&self) -> Result<Event, A2aServerError> {
        debug!("Attempting to consume one event (blocking).");

        match self.queue.dequeue_event(false).await {
            Ok(event) => {
                debug!(
                    "Dequeued event of type: {:?} in consume_one_blocking.",
                    &event
                );
                self.queue.task_done();
                Ok(event)
            }
            Err(DequeueError::QueueEmpty) => {
                // This shouldn't happen with blocking dequeue
                warn!("Unexpected empty queue in consume_one_blocking.");
                Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Unexpected empty queue".to_string())
                        .build()
                        .unwrap(),
                ))
            }
            Err(DequeueError::QueueClosed) => {
                debug!("Event queue was closed in consume_one_blocking.");
                Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Queue is closed".to_string())
                        .build()
                        .unwrap(),
                ))
            }
        }
    }

    /// Consume all the generated streaming events from the agent.
    ///
    /// This method yields events as they become available from the queue
    /// until a final event is received or the queue is closed. It also
    /// monitors for exceptions set by the `agent_task_callback`.
    pub async fn consume_all(&self) -> impl futures::Stream<Item = Result<Event, A2aServerError>> {
        debug!("Starting to consume all events from the queue.");

        let queue = self.queue.clone();
        let timeout_duration = self.timeout_duration;
        let exception = self.exception.clone();

        async_stream::stream! {
            loop {
                // Check if exception is set
                if let Some(exc) = exception.lock().await.take() {
                    yield Err(exc);
                    break;
                }

                // Try to dequeue with timeout
                match timeout(timeout_duration, queue.dequeue_event(false)).await {
                    Ok(Ok(event)) => {
                        debug!("Dequeued event of type: {:?} in consume_all.", &event);
                        queue.task_done();
                        debug!("Marked task as done in event queue in consume_all");

                        let is_final_event = match &event {
                            Event::TaskStatusUpdate(update) => update.final_,
                            Event::Message(_) => true,
                            Event::Task(task) => matches!(
                                task.status.state,
                                TaskState::Completed
                                    | TaskState::Canceled
                                    | TaskState::Failed
                                    | TaskState::Rejected
                                    | TaskState::Unknown
                                    | TaskState::InputRequired
                            ),
                            Event::TaskArtifactUpdate(_) => false,
                        };

                        if is_final_event {
                            debug!("Stopping event consumption in consume_all.");
                            queue.close().await;
                            yield Ok(event);
                            break;
                        }

                        yield Ok(event);
                    }
                    Ok(Err(DequeueError::QueueClosed)) => {
                        if queue.is_closed().await {
                            break;
                        }
                    }
                    Ok(Err(DequeueError::QueueEmpty)) => {
                        // This shouldn't happen with no_wait=false, but handle it anyway
                        continue;
                    }
                    Err(_) => {
                        // Timeout occurred, continue polling
                        continue;
                    }
                }
            }
        }
    }

    /// Callback to handle exceptions from the agent's execution task.
    ///
    /// If the agent's asyncio task raises an exception, this callback is
    /// invoked, and the exception is stored to be re-raised by the consumer loop.
    pub async fn agent_task_callback(&self, agent_task: JoinHandle<Result<(), A2aServerError>>) {
        debug!("Agent task callback triggered.");

        match agent_task.await {
            Ok(Ok(())) => {
                // Task completed successfully
            }
            Ok(Err(e)) => {
                // Task returned an error
                *self.exception.lock().await = Some(e);
            }
            Err(e) => {
                // Task panicked or was cancelled
                error!("Agent task panicked or was cancelled: {:?}", e);
                *self.exception.lock().await = Some(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message(format!("Agent task failed: {e}"))
                        .build()
                        .unwrap(),
                ));
            }
        }
    }
}
