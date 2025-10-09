use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::events::{
    event_queue::EventQueue,
    queue_manager::{NoTaskQueue, QueueManager, TaskQueueExists},
};

/// InMemoryQueueManager is used for a single binary management.
///
/// This implements the `QueueManager` interface using in-memory storage for event
/// queues. It requires all incoming interactions for a given task ID to hit the
/// same binary instance.
///
/// This implementation is suitable for single-instance deployments but needs
/// a distributed approach for scalable deployments.
pub struct InMemoryQueueManager {
    task_queue: Arc<RwLock<HashMap<String, EventQueue>>>,
}

impl InMemoryQueueManager {
    /// Initializes the InMemoryQueueManager.
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryQueueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueManager for InMemoryQueueManager {
    /// Adds a new event queue for a task ID.
    fn add<'a>(
        &'a self,
        task_id: &'a str,
        queue: EventQueue,
    ) -> Pin<Box<dyn Future<Output = Result<(), TaskQueueExists>> + Send + Sync + 'a>> {
        Box::pin(async move {
            let mut task_queue = self.task_queue.write().await;
            if task_queue.contains_key(task_id) {
                return Err(TaskQueueExists);
            }
            task_queue.insert(task_id.to_string(), queue);
            Ok(())
        })
    }

    /// Retrieves the event queue for a task ID.
    fn get<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<EventQueue>> + Send + Sync + 'a>> {
        Box::pin(async move {
            let task_queue = self.task_queue.read().await;
            task_queue.get(task_id).cloned()
        })
    }

    /// Taps the event queue for a task ID to create a child queue.
    fn tap<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<EventQueue>> + Send + Sync + 'a>> {
        Box::pin(async move {
            let task_queue = self.task_queue.read().await;
            if let Some(queue) = task_queue.get(task_id) {
                Some(queue.tap().await)
            } else {
                None
            }
        })
    }

    /// Closes and removes the event queue for a task ID.
    fn close<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), NoTaskQueue>> + Send + Sync + 'a>> {
        Box::pin(async move {
            let mut task_queue = self.task_queue.write().await;
            if let Some(queue) = task_queue.remove(task_id) {
                queue.close().await;
                Ok(())
            } else {
                Err(NoTaskQueue)
            }
        })
    }

    /// Creates a new event queue for a task ID if one doesn't exist, otherwise taps the existing one.
    fn create_or_tap<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = EventQueue> + Send + Sync + 'a>> {
        Box::pin(async move {
            let mut task_queue = self.task_queue.write().await;
            if let Some(queue) = task_queue.get(task_id) {
                debug!("Found existing queue for task_id: {}, tapping it", task_id);
                queue.tap().await
            } else {
                debug!("Creating new queue for task_id: {}", task_id);
                let queue = EventQueue::default();
                task_queue.insert(task_id.to_string(), queue.clone());
                queue
            }
        })
    }
}
