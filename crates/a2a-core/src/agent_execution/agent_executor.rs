use std::future::Future;
use std::pin::Pin;

use crate::{agent_execution::context::RequestContext, events::event_queue::EventQueue};

pub type BoxedFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>
            + Send
            + 'a,
    >,
>;

/// Agent Executor interface.
///
/// Implementations of this interface contain the core logic of the agent,
/// executing tasks based on requests and publishing updates to an event queue.
pub trait AgentExecutor: Send + Sync {
    /// Execute the agent's logic for a given request context.
    ///
    /// The agent should read necessary information from the `context` and
    /// publish `Task` or `Message` events, or `TaskStatusUpdateEvent` /
    /// `TaskArtifactUpdateEvent` to the `event_queue`. This method should
    /// return once the agent's execution for this request is complete or
    /// yields control (e.g., enters an input-required state).
    fn execute<'a>(&'a self, context: RequestContext, event_queue: EventQueue) -> BoxedFuture<'a>;

    /// Request the agent to cancel an ongoing task.
    ///
    /// The agent should attempt to stop the task identified by the task_id
    /// in the context and publish a `TaskStatusUpdateEvent` with state
    /// `TaskState.canceled` to the `event_queue`.
    fn cancel<'a>(&'a self, context: RequestContext, event_queue: EventQueue) -> BoxedFuture<'a>;
}
