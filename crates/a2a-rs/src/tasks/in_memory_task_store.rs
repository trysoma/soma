use derive_builder::Builder;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::async_trait;
use tracing::debug;

use crate::{
    errors::A2aServerError,
    tasks::store::TaskStore,
    types::{Task, TaskId},
};

///In-memory implementation of TaskStore.
/// Stores task objects in a dictionary in memory. Task data is lost when the
/// server process stops.
#[derive(Builder)]
pub struct InMemoryTaskStore {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

#[async_trait]
impl TaskStore for InMemoryTaskStore {
    async fn save(&self, task: &Task) -> Result<(), A2aServerError> {
        let id = task.id.clone();
        self.tasks.write().await.insert(id.clone(), task.clone());
        debug!("Task {} saved successfully.", &id);
        Ok(())
    }

    async fn get(&self, id: &TaskId) -> Result<Option<Task>, A2aServerError> {
        debug!("Attempting to get task with id: {}", &id);
        let tasks = self.tasks.read().await;
        let task = tasks.get(&id.clone()).cloned();

        match &task {
            Some(_t) => debug!("Task {} retrieved successfully.", &id),
            None => debug!("Task {} not found.", &id),
        };

        Ok(task)
    }

    async fn delete(&self, id: &TaskId) -> Result<(), A2aServerError> {
        debug!("Attempting to delete task with id: {}", &id);
        let mut tasks = self.tasks.write().await;
        let res = tasks.remove(&id.clone());

        match res {
            Some(_) => debug!("Task {} deleted successfully.", &id),
            None => debug!("Attempted to delete nonexistent task with id: {}", &id),
        }

        Ok(())
    }
}
