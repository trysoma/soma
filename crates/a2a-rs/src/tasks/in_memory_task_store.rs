use derive_builder::Builder;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
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

impl TaskStore for InMemoryTaskStore {
    fn save<'a>(
        &'a self,
        task: &'a Task,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>> {
        Box::pin(async move {
            let id = task.id.clone();
            self.tasks.write().await.insert(id.clone(), task.clone());
            debug!("Task {} saved successfully.", &id);
            Ok(())
        })
    }

    fn get<'a>(
        &'a self,
        id: &'a TaskId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Task>, A2aServerError>> + Send + Sync + 'a>>
    {
        Box::pin(async move {
            debug!("Attempting to get task with id: {}", &id);
            let tasks = self.tasks.read().await;
            let task = tasks.get(&id.clone()).cloned();

            match &task {
                Some(_t) => debug!("Task {} retrieved successfully.", &id),
                None => debug!("Task {} not found.", &id),
            };

            Ok(task)
        })
    }

    fn delete<'a>(
        &'a self,
        id: &'a TaskId,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>> {
        Box::pin(async move {
            debug!("Attempting to delete task with id: {}", &id);
            let mut tasks = self.tasks.write().await;
            let res = tasks.remove(&id.clone());

            match res {
                Some(_) => debug!("Task {} deleted successfully.", &id),
                None => debug!("Attempted to delete nonexistent task with id: {}", &id),
            }

            Ok(())
        })
    }
}
