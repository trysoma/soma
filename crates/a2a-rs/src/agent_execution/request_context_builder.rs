use std::future::Future;
use std::pin::Pin;

use crate::{
    agent_execution::context::RequestContext,
    types::{MessageSendParams, Task},
};

/// Builds request context to be supplied to agent executor.
pub trait RequestContextBuilder: Send + Sync {
    fn build<'a>(
        &'a self,
        params: Option<MessageSendParams>,
        task_id: Option<String>,
        context_id: Option<String>,
        task: Option<Task>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<RequestContext, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + Sync
                + 'a,
        >,
    >;
}
