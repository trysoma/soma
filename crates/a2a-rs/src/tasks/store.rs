use std::future::Future;
use std::pin::Pin;

use crate::{
    errors::A2aServerError,
    types::{Task, TaskId},
};

pub trait TaskStore: Send + Sync {
    fn save<'a>(
        &'a self,
        task: &'a Task,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>>;

    fn get<'a>(
        &'a self,
        id: &'a TaskId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Task>, A2aServerError>> + Send + Sync + 'a>>;

    fn delete<'a>(
        &'a self,
        id: &'a TaskId,
    ) -> Pin<Box<dyn Future<Output = Result<(), A2aServerError>> + Send + Sync + 'a>>;
}
