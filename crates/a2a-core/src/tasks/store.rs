use async_trait::async_trait;

use crate::{
    errors::A2aServerError,
    types::{Task, TaskId},
};

#[async_trait]
pub trait TaskStore: Send + Sync {
    async fn save(&self, task: &Task) -> Result<(), A2aServerError>;

    async fn get(&self, id: &TaskId) -> Result<Option<Task>, A2aServerError>;

    async fn delete(&self, id: &TaskId) -> Result<(), A2aServerError>;
}
