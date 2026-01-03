use async_trait::async_trait;
use futures::future::join_all;
use std::sync::Arc;

use crate::{
    agent_execution::{context::RequestContext, request_context_builder::RequestContextBuilder},
    tasks::store::TaskStore,
    types::{MessageSendParams, Task},
};

/// Builds request context and populates referred tasks.
pub struct SimpleRequestContextBuilder {
    should_populate_referred_tasks: bool,
    task_store: Option<Arc<dyn TaskStore + Send + Sync>>,
}

impl SimpleRequestContextBuilder {
    /// Initializes the SimpleRequestContextBuilder.
    pub fn new(
        should_populate_referred_tasks: bool,
        task_store: Option<Arc<dyn TaskStore + Send + Sync>>,
    ) -> Self {
        Self {
            should_populate_referred_tasks,
            task_store,
        }
    }
}

#[async_trait]
impl RequestContextBuilder for SimpleRequestContextBuilder {
    /// Builds the request context for an agent execution.
    ///
    /// This method assembles the RequestContext object. If the builder was
    /// initialized with `should_populate_referred_tasks=true`, it fetches all tasks
    /// referenced in `params.message.reference_task_ids` from the `task_store`.
    async fn build(
        &self,
        params: Option<MessageSendParams>,
        task_id: Option<String>,
        context_id: Option<String>,
        task: Option<Task>,
    ) -> Result<RequestContext, Box<dyn std::error::Error + Send + Sync>> {
        let mut related_tasks = None;

        if self.should_populate_referred_tasks {
            if let (Some(task_store), Some(params)) = (&self.task_store, &params) {
                if !params.message.reference_task_ids.is_empty() {
                    let futures: Vec<_> = params
                        .message
                        .reference_task_ids
                        .iter()
                        .map(|id| task_store.get(id))
                        .collect();

                    let tasks = join_all(futures).await;

                    related_tasks = Some(
                        tasks
                            .into_iter()
                            .filter_map(|result| result.ok().flatten())
                            .collect::<Vec<Task>>(),
                    );
                }
            }
        }

        Ok(RequestContext::new(
            params,
            task_id,
            context_id,
            task,
            related_tasks,
        )?)
    }
}
