use serde_json::Map;
use std::sync::Arc;

use crate::{
    errors::{A2aServerError, ErrorBuilder},
    tasks::store::TaskStore,
    types::{ContextId, Message, Task, TaskArtifactUpdateEvent, TaskId, TaskStatusUpdateEvent},
};
use derive_builder::Builder;
use tracing::debug;

/// Helps manage a task's lifecycle during execution of a request.
/// Responsible for retrieving, saving, and updating the `Task` object based on
/// events received from the agent.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TaskManager {
    task_store: Arc<dyn TaskStore + Send + Sync>,
    task_id: Option<TaskId>,
    context_id: Option<ContextId>,
    #[allow(dead_code)]
    initial_message: Option<Message>,
    #[builder(default)]
    current_task: Option<Task>,
}

impl TaskManager {
    /// Creates a new TaskManager instance
    pub fn new(
        task_store: Arc<dyn TaskStore + Send + Sync>,
        task_id: Option<TaskId>,
        context_id: Option<ContextId>,
        initial_message: Option<Message>,
    ) -> Self {
        Self {
            task_store,
            task_id,
            context_id,
            initial_message,
            current_task: None,
        }
    }

    /// Retrieves the current task object, either from memory or the store.
    /// If `task_id` is set, it first checks the in-memory `current_task`,
    /// then attempts to load it from the `task_store`.
    pub async fn get_task(&mut self) -> Result<Option<Task>, A2aServerError> {
        let task_id = match self.task_id.as_ref() {
            None => {
                debug!("task_id is not set, cannot get task.");
                return Ok(None);
            }
            Some(id) => id.clone(),
        };

        if let Some(task) = &self.current_task {
            return Ok(Some(task.clone()));
        }

        debug!("Attempting to get task from store with id: {}", &task_id);
        let task = self.task_store.get(&task_id).await?;

        match task.as_ref() {
            Some(_) => debug!("Task {} retrieved successfully", &task_id),
            None => debug!("Task {} not found", task_id),
        }

        self.current_task = task.clone();
        Ok(task)
    }

    /// Updates the task with a new message
    pub fn update_with_message(&mut self, _message: Message, task: Task) -> Task {
        let updated_task = task;

        // Add the message to the task's messages if it has a messages field
        // This is a simplified version - you may need to adjust based on your Task struct

        self.current_task = Some(updated_task.clone());
        updated_task
    }

    /// Processes a Task event and saves the updated task state.
    pub async fn save_task(&mut self, task: Task) -> Result<Task, A2aServerError> {
        // Validate task ID matches if we have one
        if let Some(ref task_id) = self.task_id {
            if task_id != &task.id {
                return Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Task manager task ID does not match event Task ID".to_string())
                        .build()
                        .unwrap(),
                ));
            }
        } else {
            // If we don't have a task_id yet, set it from the incoming task
            debug!("Setting task_id in TaskManager to: {}", task.id);
            self.task_id = Some(task.id.clone());
        }

        // Validate context ID matches if we have one
        if let Some(ref context_id) = self.context_id {
            if context_id != &task.context_id {
                return Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message(
                            "Task manager context ID does not match event context ID".to_string(),
                        )
                        .build()
                        .unwrap(),
                ));
            }
        } else {
            // If we don't have a context_id yet, set it from the incoming task
            debug!("Setting context_id in TaskManager to: {}", task.context_id);
            self.context_id = Some(task.context_id.clone());
        }

        debug!(
            "Saving task with id: {}, context_id: {}",
            task.id, task.context_id
        );

        self.task_store.save(&task).await?;
        self.current_task = Some(task.clone());

        Ok(task)
    }

    /// Processes a TaskStatusUpdateEvent and updates the task state
    pub async fn save_task_status_update(
        &mut self,
        event: TaskStatusUpdateEvent,
    ) -> Result<Task, A2aServerError> {
        // Validate task ID matches if we have one
        if let Some(ref task_id) = self.task_id {
            if task_id != &event.task_id {
                debug!(
                    "Task ID mismatch: TaskManager has task_id={}, but event has task_id={}",
                    task_id, event.task_id
                );
                return Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Task manager task ID does not match event Task ID".to_string())
                        .build()
                        .unwrap(),
                ));
            }
        } else if !event.task_id.is_empty() {
            // If we don't have a task_id yet and the event has one, set it
            debug!(
                "Setting task_id in TaskManager from TaskStatusUpdate to: {}",
                event.task_id
            );
            self.task_id = Some(event.task_id.clone());
        }

        // Validate context ID matches if we have one
        if let Some(ref context_id) = self.context_id {
            if context_id != &event.context_id {
                return Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message(
                            "Task manager context ID does not match event context ID".to_string(),
                        )
                        .build()
                        .unwrap(),
                ));
            }
        } else if !event.context_id.is_empty() {
            // If we don't have a context_id yet and the event has one, set it
            debug!(
                "Setting context_id in TaskManager from TaskStatusUpdate to: {}",
                event.context_id
            );
            self.context_id = Some(event.context_id.clone());
        }

        // Get the current task - if it doesn't exist and we have IDs, create a minimal one
        let mut task = match self.get_task().await? {
            Some(t) => t,
            None => {
                // If we have task_id but no task in store, create a minimal task from the event
                if let (Some(task_id), Some(context_id)) = (&self.task_id, &self.context_id) {
                    debug!("Creating new task from TaskStatusUpdate event");
                    Task {
                        id: task_id.clone(),
                        context_id: context_id.clone(),
                        kind: "task".to_string(),
                        status: event.status.clone(),
                        history: vec![],
                        metadata: Map::new(),
                        artifacts: vec![],
                    }
                } else {
                    return Err(A2aServerError::InternalError(
                        ErrorBuilder::default()
                            .message(
                                "Task not found and cannot create from status update".to_string(),
                            )
                            .build()
                            .unwrap(),
                    ));
                }
            }
        };

        // Update the task status
        task.status = event.status;

        // Save the updated task
        self.task_store.save(&task).await?;
        self.current_task = Some(task.clone());

        Ok(task)
    }

    /// Processes a TaskArtifactUpdateEvent and updates the task state
    pub async fn save_task_artifact_update(
        &mut self,
        event: TaskArtifactUpdateEvent,
    ) -> Result<Task, A2aServerError> {
        // Validate task ID matches if we have one
        if let Some(ref task_id) = self.task_id {
            if task_id != &event.task_id {
                return Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message("Task manager task ID does not match event Task ID".to_string())
                        .build()
                        .unwrap(),
                ));
            }
        } else if !event.task_id.is_empty() {
            // If we don't have a task_id yet and the event has one, set it
            debug!(
                "Setting task_id in TaskManager from TaskArtifactUpdate to: {}",
                event.task_id
            );
            self.task_id = Some(event.task_id.clone());
        }

        // Validate context ID matches if we have one
        if let Some(ref context_id) = self.context_id {
            if context_id != &event.context_id {
                return Err(A2aServerError::InternalError(
                    ErrorBuilder::default()
                        .message(
                            "Task manager context ID does not match event context ID".to_string(),
                        )
                        .build()
                        .unwrap(),
                ));
            }
        } else if !event.context_id.is_empty() {
            // If we don't have a context_id yet and the event has one, set it
            debug!(
                "Setting context_id in TaskManager from TaskArtifactUpdate to: {}",
                event.context_id
            );
            self.context_id = Some(event.context_id.clone());
        }

        // Get the current task
        let mut task = self.get_task().await?.ok_or_else(|| {
            A2aServerError::InternalError(
                ErrorBuilder::default()
                    .message("Task not found when processing artifact update".to_string())
                    .build()
                    .unwrap(),
            )
        })?;

        // Update the task artifacts
        let artifact = event.artifact;

        // Add or update the artifact
        // Find if artifact already exists and update it, otherwise add it
        if let Some(existing) = task
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact.artifact_id)
        {
            *existing = artifact;
        } else {
            task.artifacts.push(artifact);
        }

        // Save the updated task
        self.task_store.save(&task).await?;
        self.current_task = Some(task.clone());

        Ok(task)
    }

    /// Get the current task ID
    pub fn task_id(&self) -> Option<&TaskId> {
        self.task_id.as_ref()
    }

    /// Get the current context ID
    pub fn context_id(&self) -> Option<&ContextId> {
        self.context_id.as_ref()
    }
}
