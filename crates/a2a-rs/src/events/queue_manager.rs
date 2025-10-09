use std::future::Future;
use std::pin::Pin;

use crate::events::event_queue::EventQueue;

/// Interface for managing the event queue lifecycles per task.
pub trait QueueManager: Send + Sync {
    /// Adds a new event queue associated with a task ID.
    fn add<'a>(
        &'a self,
        task_id: &'a str,
        queue: EventQueue,
    ) -> Pin<Box<dyn Future<Output = Result<(), TaskQueueExists>> + Send + Sync + 'a>>;

    /// Retrieves the event queue for a task ID.
    fn get<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<EventQueue>> + Send + Sync + 'a>>;

    /// Creates a child event queue (tap) for an existing task ID.
    fn tap<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<EventQueue>> + Send + Sync + 'a>>;

    /// Closes and removes the event queue for a task ID.
    fn close<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), NoTaskQueue>> + Send + Sync + 'a>>;

    /// Creates a queue if one doesn't exist, otherwise taps the existing one.
    fn create_or_tap<'a>(
        &'a self,
        task_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = EventQueue> + Send + Sync + 'a>>;
}

/// Exception raised when attempting to add a queue for a task ID that already exists.
#[derive(Debug, thiserror::Error)]
#[error("Task queue already exists")]
pub struct TaskQueueExists;

/// Exception raised when attempting to access or close a queue for a task ID that does not exist.
#[derive(Debug, thiserror::Error)]
#[error("No task queue found")]
pub struct NoTaskQueue;
