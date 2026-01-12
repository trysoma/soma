//! Event System for A2A Protocol
//!
//! Provides event queue and queue manager for A2A protocol message passing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use shared::error::CommonError;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{debug, error, trace};

use crate::a2a_core::errors::{A2aError, A2aServerError};
use crate::a2a_core::types::{Message, Task, TaskArtifactUpdateEvent, TaskState, TaskStatusUpdateEvent};

/// Type alias for events that can be enqueued.
#[derive(Clone, Debug)]
pub enum Event {
    Message(Message),
    Task(Task),
    TaskStatusUpdate(TaskStatusUpdateEvent),
    TaskArtifactUpdate(TaskArtifactUpdateEvent),
}

pub const DEFAULT_MAX_QUEUE_SIZE: usize = 1024;

/// Errors when dequeuing events
#[derive(Debug, thiserror::Error)]
pub enum DequeueError {
    #[error("Queue is empty")]
    QueueEmpty,
    #[error("Queue is closed")]
    QueueClosed,
}

/// Event queue for A2A responses from agent.
///
/// Acts as a buffer between the agent's asynchronous execution and the
/// server's response handling (e.g., streaming via SSE).
pub struct EventQueue {
    sender: broadcast::Sender<Event>,
    receiver: Arc<Mutex<broadcast::Receiver<Event>>>,
    is_closed: Arc<RwLock<bool>>,
}

impl EventQueue {
    /// Creates a new EventQueue with specified max size.
    pub fn new(max_queue_size: usize) -> Self {
        assert!(max_queue_size > 0, "max_queue_size must be greater than 0");
        let (sender, receiver) = broadcast::channel(max_queue_size);
        trace!("EventQueue initialized");
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            is_closed: Arc::new(RwLock::new(false)),
        }
    }

    /// Enqueues an event to this queue.
    pub async fn enqueue_event(&self, event: Event) -> Result<(), broadcast::error::SendError<Event>> {
        if *self.is_closed.read().await {
            trace!("Queue closed, event not enqueued");
            return Ok(());
        }
        trace!(event = ?event, "Enqueuing event");
        self.sender.send(event)?;
        Ok(())
    }

    /// Dequeues an event from the queue.
    pub async fn dequeue_event(&self, no_wait: bool) -> Result<Event, DequeueError> {
        let is_closed = *self.is_closed.read().await;
        let mut receiver = self.receiver.lock().await;

        if is_closed && receiver.is_empty() {
            trace!("Queue closed and empty");
            return Err(DequeueError::QueueClosed);
        }

        if no_wait {
            match receiver.try_recv() {
                Ok(event) => {
                    trace!(event = ?event, "Dequeued event (no_wait)");
                    Ok(event)
                }
                Err(broadcast::error::TryRecvError::Empty) => Err(DequeueError::QueueEmpty),
                Err(broadcast::error::TryRecvError::Closed) => Err(DequeueError::QueueClosed),
                Err(broadcast::error::TryRecvError::Lagged(_)) => match receiver.try_recv() {
                    Ok(event) => Ok(event),
                    Err(_) => Err(DequeueError::QueueEmpty),
                },
            }
        } else {
            match receiver.recv().await {
                Ok(event) => {
                    trace!(event = ?event, "Dequeued event (waited)");
                    Ok(event)
                }
                Err(broadcast::error::RecvError::Closed) => Err(DequeueError::QueueClosed),
                Err(broadcast::error::RecvError::Lagged(_)) => match receiver.recv().await {
                    Ok(event) => Ok(event),
                    Err(_) => Err(DequeueError::QueueClosed),
                },
            }
        }
    }

    /// Creates a new subscriber to this queue.
    pub fn tap(&self) -> EventQueue {
        trace!("Tapping EventQueue");
        EventQueue {
            sender: self.sender.clone(),
            receiver: Arc::new(Mutex::new(self.sender.subscribe())),
            is_closed: self.is_closed.clone(),
        }
    }

    /// Closes the queue for future push events.
    pub async fn close(&self) {
        trace!("Closing EventQueue");
        let mut is_closed = self.is_closed.write().await;
        *is_closed = true;
    }

    /// Checks if the queue is closed.
    pub async fn is_closed(&self) -> bool {
        *self.is_closed.read().await
    }
}

impl Clone for EventQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: Arc::new(Mutex::new(self.sender.subscribe())),
            is_closed: self.is_closed.clone(),
        }
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_QUEUE_SIZE)
    }
}

/// In-memory queue manager for managing event queues per task.
pub struct QueueManager {
    task_queues: Arc<RwLock<HashMap<String, EventQueue>>>,
}

impl QueueManager {
    /// Creates a new QueueManager.
    pub fn new() -> Self {
        Self {
            task_queues: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds a new event queue for a task ID.
    pub async fn add(&self, task_id: &str, queue: EventQueue) -> Result<(), CommonError> {
        let mut task_queues = self.task_queues.write().await;
        if task_queues.contains_key(task_id) {
            return Err(CommonError::TaskQueueError {
                msg: format!("Task queue already exists for task_id: {task_id}"),
            });
        }
        task_queues.insert(task_id.to_string(), queue);
        Ok(())
    }

    /// Retrieves the event queue for a task ID.
    pub async fn get(&self, task_id: &str) -> Option<EventQueue> {
        let task_queues = self.task_queues.read().await;
        task_queues.get(task_id).cloned()
    }

    /// Taps the event queue for a task ID to create a subscriber.
    pub async fn tap(&self, task_id: &str) -> Option<EventQueue> {
        let task_queues = self.task_queues.read().await;
        task_queues.get(task_id).map(|q| q.tap())
    }

    /// Closes and removes the event queue for a task ID.
    pub async fn close(&self, task_id: &str) -> Result<(), CommonError> {
        let mut task_queues = self.task_queues.write().await;
        if let Some(queue) = task_queues.remove(task_id) {
            queue.close().await;
            Ok(())
        } else {
            Err(CommonError::TaskQueueError {
                msg: format!("No task queue found for task_id: {task_id}"),
            })
        }
    }

    /// Creates a new event queue for a task ID if one doesn't exist, otherwise taps the existing one.
    pub async fn create_or_tap(&self, task_id: &str) -> EventQueue {
        let mut task_queues = self.task_queues.write().await;
        if let Some(queue) = task_queues.get(task_id) {
            trace!(task_id, "Tapping existing queue");
            queue.tap()
        } else {
            trace!(task_id, "Creating new queue");
            let queue = EventQueue::default();
            task_queues.insert(task_id.to_string(), queue.clone());
            queue
        }
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Consumer to read events from the agent event queue.
pub struct EventConsumer {
    queue: EventQueue,
    timeout_duration: Duration,
    exception: Arc<Mutex<Option<A2aServerError>>>,
}

impl EventConsumer {
    /// Creates a new EventConsumer for the given queue.
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
                Ok(event)
            }
            Err(DequeueError::QueueEmpty) => {
                debug!("Queue empty during consume_one");
                Err(A2aServerError::InternalError(A2aError::new(
                    "Agent did not return any response",
                )))
            }
            Err(DequeueError::QueueClosed) => {
                debug!("Queue closed during consume_one");
                Err(A2aServerError::InternalError(A2aError::new("Queue is closed")))
            }
        }
    }

    /// Consume one event from the agent event queue with blocking.
    pub async fn consume_one_blocking(&self) -> Result<Event, A2aServerError> {
        trace!("Consuming event (blocking)");
        match self.queue.dequeue_event(false).await {
            Ok(event) => {
                trace!(event = ?event, "Dequeued event (blocking)");
                Ok(event)
            }
            Err(DequeueError::QueueEmpty) => {
                debug!("Unexpected empty queue in blocking dequeue");
                Err(A2aServerError::InternalError(A2aError::new("Unexpected empty queue")))
            }
            Err(DequeueError::QueueClosed) => {
                trace!("Queue closed during blocking consume");
                Err(A2aServerError::InternalError(A2aError::new("Queue is closed")))
            }
        }
    }

    /// Consume all the generated streaming events from the agent.
    pub async fn consume_all(&self) -> impl futures::Stream<Item = Result<Event, A2aServerError>> {
        trace!("Starting consume_all");
        let queue = self.queue.clone();
        let timeout_duration = self.timeout_duration;
        let exception = self.exception.clone();

        async_stream::stream! {
            loop {
                if let Some(exc) = exception.lock().await.take() {
                    yield Err(exc);
                    break;
                }

                match timeout(timeout_duration, queue.dequeue_event(false)).await {
                    Ok(Ok(event)) => {
                        trace!(event = ?event, "Dequeued event in consume_all");

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
    pub async fn agent_task_callback(&self, agent_task: JoinHandle<Result<(), A2aServerError>>) {
        trace!("Agent task callback triggered");
        match agent_task.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                *self.exception.lock().await = Some(e);
            }
            Err(e) => {
                error!(error = ?e, "Agent task panicked or cancelled");
                *self.exception.lock().await = Some(A2aServerError::InternalError(
                    A2aError::new(format!("Agent task failed: {e}")),
                ));
            }
        }
    }
}
