use async_trait::async_trait;

use crate::events::event_queue::EventQueue;

/// Interface for managing the event queue lifecycles per task.
#[async_trait]
pub trait QueueManager: Send + Sync {
    /// Adds a new event queue associated with a task ID.
    async fn add(&self, task_id: &str, queue: EventQueue) -> Result<(), TaskQueueExists>;

    /// Retrieves the event queue for a task ID.
    async fn get(&self, task_id: &str) -> Option<EventQueue>;

    /// Creates a child event queue (tap) for an existing task ID.
    async fn tap(&self, task_id: &str) -> Option<EventQueue>;

    /// Closes and removes the event queue for a task ID.
    async fn close(&self, task_id: &str) -> Result<(), NoTaskQueue>;

    /// Creates a queue if one doesn't exist, otherwise taps the existing one.
    async fn create_or_tap(&self, task_id: &str) -> EventQueue;
}

/// Exception raised when attempting to add a queue for a task ID that already exists.
#[derive(Debug, thiserror::Error)]
#[error("Task queue already exists")]
pub struct TaskQueueExists;

/// Exception raised when attempting to access or close a queue for a task ID that does not exist.
#[derive(Debug, thiserror::Error)]
#[error("No task queue found")]
pub struct NoTaskQueue;
