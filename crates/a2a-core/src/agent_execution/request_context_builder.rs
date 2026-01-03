use async_trait::async_trait;

use crate::{
    agent_execution::context::RequestContext,
    types::{MessageSendParams, Task},
};

/// Builds request context to be supplied to agent executor.
#[async_trait]
pub trait RequestContextBuilder: Send + Sync {
    async fn build(
        &self,
        params: Option<MessageSendParams>,
        task_id: Option<String>,
        context_id: Option<String>,
        task: Option<Task>,
    ) -> Result<RequestContext, Box<dyn std::error::Error + Send + Sync>>;
}
