//! Task Management for A2A Protocol
//!
//! Provides task state management utilities for A2A tasks.

use crate::a2a_core::types::TaskState;

/// Check if a task state is terminal (cannot transition further)
pub fn is_terminal_state(state: &TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Canceled | TaskState::Failed | TaskState::Rejected
    )
}

/// Check if a task state requires user input
pub fn is_interrupt_state(state: &TaskState) -> bool {
    matches!(state, TaskState::InputRequired)
}
