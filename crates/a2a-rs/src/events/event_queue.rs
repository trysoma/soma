use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing::debug;

use crate::types::{Message, Task, TaskArtifactUpdateEvent, TaskStatusUpdateEvent};

/// Type alias for events that can be enqueued.
#[derive(Clone, Debug)]
pub enum Event {
    Message(Message),
    Task(Task),
    TaskStatusUpdate(TaskStatusUpdateEvent),
    TaskArtifactUpdate(TaskArtifactUpdateEvent),
}

pub const DEFAULT_MAX_QUEUE_SIZE: usize = 1024;

/// Event queue for A2A responses from agent.
///
/// Acts as a buffer between the agent's asynchronous execution and the
/// server's response handling (e.g., streaming via SSE). Supports tapping
/// to create child queues that receive the same events.
pub struct EventQueue {
    sender: broadcast::Sender<Event>,
    receiver: Arc<Mutex<broadcast::Receiver<Event>>>,
    children: Arc<RwLock<Vec<EventQueue>>>,
    is_closed: Arc<RwLock<bool>>,
    max_queue_size: usize,
}

impl EventQueue {
    /// Initializes the EventQueue.
    pub fn new(max_queue_size: usize) -> Self {
        if max_queue_size == 0 {
            panic!("max_queue_size must be greater than 0");
        }

        let (sender, receiver) = broadcast::channel(max_queue_size);

        debug!("EventQueue initialized.");

        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            children: Arc::new(RwLock::new(Vec::new())),
            is_closed: Arc::new(RwLock::new(false)),
            max_queue_size,
        }
    }

    /// Creates a new EventQueue with default max size.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::new(DEFAULT_MAX_QUEUE_SIZE)
    }

    /// Enqueues an event to this queue and all its children.
    pub fn enqueue_event(
        &self,
        event: Event,
    ) -> Pin<
        Box<dyn Future<Output = Result<(), broadcast::error::SendError<Event>>> + Send + Sync + '_>,
    > {
        Box::pin(async move {
            let is_closed = *self.is_closed.read().await;
            if is_closed {
                debug!("Queue is closed. Event will not be enqueued.");
                return Ok(());
            }

            match &event {
                Event::Task(task) => debug!("Enqueuing Task event with task_id: {}", task.id),
                Event::TaskStatusUpdate(update) => debug!(
                    "Enqueuing TaskStatusUpdate event with task_id: {}",
                    update.task_id
                ),
                Event::TaskArtifactUpdate(update) => debug!(
                    "Enqueuing TaskArtifactUpdate event with task_id: {}",
                    update.task_id
                ),
                Event::Message(msg) => {
                    debug!("Enqueuing Message event with task_id: {:?}", msg.task_id)
                }
            }

            // Send to this queue
            self.sender.send(event.clone())?;

            // Send to all children
            let children = self.children.read().await;
            for child in children.iter() {
                let _ = child.enqueue_event(event.clone()).await;
            }

            Ok(())
        })
    }

    /// Dequeues an event from the queue.
    pub async fn dequeue_event(&self, no_wait: bool) -> Result<Event, DequeueError> {
        let is_closed = *self.is_closed.read().await;
        let mut receiver = self.receiver.lock().await;

        if is_closed && receiver.is_empty() {
            debug!("Queue is closed. Event will not be dequeued.");
            return Err(DequeueError::QueueClosed);
        }

        if no_wait {
            debug!("Attempting to dequeue event (no_wait=true).");
            match receiver.try_recv() {
                Ok(event) => {
                    debug!(
                        "Dequeued event (no_wait=true) of type: {:?}",
                        std::mem::discriminant(&event)
                    );
                    Ok(event)
                }
                Err(broadcast::error::TryRecvError::Empty) => Err(DequeueError::QueueEmpty),
                Err(broadcast::error::TryRecvError::Closed) => Err(DequeueError::QueueClosed),
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    // Try again after the lagged messages are skipped
                    match receiver.try_recv() {
                        Ok(event) => Ok(event),
                        Err(_) => Err(DequeueError::QueueEmpty),
                    }
                }
            }
        } else {
            debug!("Attempting to dequeue event (waiting).");
            match receiver.recv().await {
                Ok(event) => {
                    debug!(
                        "Dequeued event (waited) of type: {:?}",
                        std::mem::discriminant(&event)
                    );
                    Ok(event)
                }
                Err(broadcast::error::RecvError::Closed) => Err(DequeueError::QueueClosed),
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Try again after the lagged messages are skipped
                    match receiver.recv().await {
                        Ok(event) => Ok(event),
                        Err(_) => Err(DequeueError::QueueClosed),
                    }
                }
            }
        }
    }

    /// Signals that a formerly enqueued task is complete.
    ///
    /// Note: Since we're using broadcast channels, there's no direct equivalent
    /// to Python's task_done(). This is kept for API compatibility.
    pub fn task_done(&self) {
        debug!("Marking task as done in EventQueue.");
    }

    /// Taps the event queue to create a new child queue that receives all future events.
    pub async fn tap(&self) -> EventQueue {
        debug!("Tapping EventQueue to create a child queue.");

        // Check if parent is already closed
        if *self.is_closed.read().await {
            debug!("Cannot tap a closed EventQueue.");
            // Return a pre-closed queue
            let child = EventQueue {
                sender: self.sender.clone(),
                receiver: Arc::new(Mutex::new(self.sender.subscribe())),
                children: Arc::new(RwLock::new(Vec::new())),
                is_closed: Arc::new(RwLock::new(true)),
                max_queue_size: self.max_queue_size,
            };
            return child;
        }

        let child = EventQueue {
            sender: self.sender.clone(),
            receiver: Arc::new(Mutex::new(self.sender.subscribe())),
            children: Arc::new(RwLock::new(Vec::new())),
            is_closed: Arc::new(RwLock::new(false)),
            max_queue_size: self.max_queue_size,
        };

        self.children.write().await.push(child.clone());
        child
    }

    /// Closes the queue for future push events.
    pub fn close(&self) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + '_>> {
        Box::pin(async move {
            debug!("Closing EventQueue.");

            let mut is_closed = self.is_closed.write().await;
            if *is_closed {
                return;
            }
            *is_closed = true;
            drop(is_closed);

            // Close all children
            let children = self.children.read().await;
            for child in children.iter() {
                child.close().await;
            }
        })
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
            children: self.children.clone(),
            is_closed: self.is_closed.clone(),
            max_queue_size: self.max_queue_size,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DequeueError {
    #[error("Queue is empty")]
    QueueEmpty,
    #[error("Queue is closed")]
    QueueClosed,
}
