use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{debug, error, trace};

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
        trace!("EventConsumer initialized");
        Self {
            queue,
            timeout_duration: Duration::from_millis(500),
            exception: Arc::new(Mutex::new(None)),
        }
    }

    /// Consume one event from the agent event queue non-blocking.
    pub async fn consume_one(&self) -> Result<Event, A2aServerError> {
        trace!("Consuming event (non-blocking)");

        match self.queue.dequeue_event(true).await {
            Ok(event) => {
                trace!(event = ?event, "Dequeued event");
                self.queue.task_done();
                Ok(event)
            }
            Err(DequeueError::QueueEmpty) => {
                debug!("Queue empty during consume_one");
                Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Agent did not return any response".to_string())
                        .build()
                        .unwrap(),
                ))
            }
            Err(DequeueError::QueueClosed) => {
                debug!("Queue closed during consume_one");
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
        trace!("Consuming event (blocking)");

        match self.queue.dequeue_event(false).await {
            Ok(event) => {
                trace!(event = ?event, "Dequeued event (blocking)");
                self.queue.task_done();
                Ok(event)
            }
            Err(DequeueError::QueueEmpty) => {
                debug!("Unexpected empty queue in blocking dequeue");
                Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Unexpected empty queue".to_string())
                        .build()
                        .unwrap(),
                ))
            }
            Err(DequeueError::QueueClosed) => {
                trace!("Queue closed during blocking consume");
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
        trace!("Starting consume_all");

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
                        trace!(event = ?event, "Dequeued event in consume_all");
                        queue.task_done();

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
                            trace!("Final event received, closing queue");
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
                        continue;
                    }
                    Err(_) => {
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
        trace!("Agent task callback triggered");

        match agent_task.await {
            Ok(Ok(())) => {
                // Task completed successfully
            }
            Ok(Err(e)) => {
                *self.exception.lock().await = Some(e);
            }
            Err(e) => {
                error!(error = ?e, "Agent task panicked or cancelled");
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
