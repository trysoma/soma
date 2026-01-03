//! Logic layer for inbox crate
//! Contains domain models and business logic for messages, threads, events, and inboxes

pub mod event;
pub mod inbox;
pub mod message;
pub mod thread;

// Re-export commonly used types
pub use event::{InboxEvent, InboxEventKind, create_event_channel, EventTx, EventRx, EventBus};
pub use inbox::{Inbox, InboxProvider, InboxProviderRegistry};
pub use message::{UIMessage, UIMessagePart, MessageRole};
pub use thread::Thread;
