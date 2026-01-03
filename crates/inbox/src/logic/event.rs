//! Inbox event domain model
//!
//! Events represent occurrences within the inbox system such as new messages,
//! thread updates, or other inbox-specific events. Events support streaming
//! for real-time updates.
//!
//! Protocol-specific events (e.g., A2A tasks, artifacts) should use the `Custom`
//! event kind with their own serialized data.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use utoipa::ToSchema;

use super::message::UIMessage;
use super::thread::Thread;

/// Types of inbox events
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InboxEventKind {
    /// A new message was created
    MessageCreated { message: UIMessage },
    /// An existing message was updated (e.g., streaming completion)
    MessageUpdated { message: UIMessage },
    /// A message was deleted
    MessageDeleted { message_id: WrappedUuidV4 },
    /// A message part is being streamed (incremental update)
    MessageStreaming {
        message_id: WrappedUuidV4,
        thread_id: WrappedUuidV4,
        /// The part index being updated
        part_index: usize,
        /// The incremental content delta
        delta: String,
    },
    /// A new thread was created
    ThreadCreated { thread: Thread },
    /// An existing thread was updated
    ThreadUpdated { thread: Thread },
    /// A thread was deleted
    ThreadDeleted { thread_id: WrappedUuidV4 },
    /// Custom event from an inbox provider
    ///
    /// Protocol-specific events (e.g., A2A tasks, artifacts) should use this
    /// variant with their own event_type and serialized data.
    Custom {
        event_type: String,
        #[schemars(with = "serde_json::Value")]
        data: WrappedJsonValue,
    },
}

/// An inbox event with metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct InboxEvent {
    pub id: WrappedUuidV4,
    pub kind: InboxEventKind,
    /// Optional inbox ID if event originated from a specific inbox
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_id: Option<String>,
    /// Inbox-specific settings/metadata
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub inbox_settings: Map<String, Value>,
    pub created_at: WrappedChronoDateTime,
}

impl InboxEvent {
    /// Create a new message created event
    pub fn message_created(message: UIMessage) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::MessageCreated { message },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a new message updated event
    pub fn message_updated(message: UIMessage) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::MessageUpdated { message },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a message deleted event
    pub fn message_deleted(message_id: WrappedUuidV4) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::MessageDeleted { message_id },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a message streaming event (incremental text update)
    pub fn message_streaming(
        message_id: WrappedUuidV4,
        thread_id: WrappedUuidV4,
        part_index: usize,
        delta: impl Into<String>,
    ) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::MessageStreaming {
                message_id,
                thread_id,
                part_index,
                delta: delta.into(),
            },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a thread created event
    pub fn thread_created(thread: Thread) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::ThreadCreated { thread },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a thread updated event
    pub fn thread_updated(thread: Thread) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::ThreadUpdated { thread },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a thread deleted event
    pub fn thread_deleted(thread_id: WrappedUuidV4) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::ThreadDeleted { thread_id },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Create a custom event
    ///
    /// Use this for protocol-specific events (e.g., A2A tasks, artifacts).
    pub fn custom(event_type: impl Into<String>, data: WrappedJsonValue) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind: InboxEventKind::Custom {
                event_type: event_type.into(),
                data,
            },
            inbox_id: None,
            inbox_settings: Map::new(),
            created_at: WrappedChronoDateTime::now(),
        }
    }

    /// Set the inbox ID for this event
    pub fn with_inbox_id(mut self, inbox_id: impl Into<String>) -> Self {
        self.inbox_id = Some(inbox_id.into());
        self
    }

    /// Add inbox settings to this event
    pub fn with_inbox_settings(mut self, settings: Map<String, Value>) -> Self {
        self.inbox_settings = settings;
        self
    }
}

/// Channel types for event broadcasting
pub type EventTx = tokio::sync::broadcast::Sender<InboxEvent>;
pub type EventRx = tokio::sync::broadcast::Receiver<InboxEvent>;

/// Create a new event broadcast channel
pub fn create_event_channel(capacity: usize) -> (EventTx, EventRx) {
    tokio::sync::broadcast::channel(capacity)
}

/// Multi-producer, multi-consumer event bus for inbox events
#[derive(Clone)]
pub struct EventBus {
    tx: EventTx,
}

impl EventBus {
    /// Create a new event bus with the specified capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    /// Get a sender handle for publishing events
    pub fn sender(&self) -> EventTx {
        self.tx.clone()
    }

    /// Subscribe to the event bus
    pub fn subscribe(&self) -> EventRx {
        self.tx.subscribe()
    }

    /// Publish an event to all subscribers
    #[allow(clippy::result_large_err)]
    pub fn publish(&self, event: InboxEvent) -> Result<usize, tokio::sync::broadcast::error::SendError<InboxEvent>> {
        self.tx.send(event)
    }

    /// Get the number of active subscribers
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_event_message_created() {
            let thread_id = WrappedUuidV4::new();
            let message = UIMessage::user_text(thread_id, "Hello!");
            let event = InboxEvent::message_created(message.clone());

            match event.kind {
                InboxEventKind::MessageCreated { message: m } => {
                    assert_eq!(m.text_content(), "Hello!");
                }
                _ => panic!("Expected MessageCreated event"),
            }
        }

        #[test]
        fn test_event_message_streaming() {
            let message_id = WrappedUuidV4::new();
            let thread_id = WrappedUuidV4::new();
            let event = InboxEvent::message_streaming(message_id.clone(), thread_id.clone(), 0, "Hello ");

            match event.kind {
                InboxEventKind::MessageStreaming {
                    message_id: m_id,
                    thread_id: t_id,
                    part_index,
                    delta,
                } => {
                    assert_eq!(m_id, message_id);
                    assert_eq!(t_id, thread_id);
                    assert_eq!(part_index, 0);
                    assert_eq!(delta, "Hello ");
                }
                _ => panic!("Expected MessageStreaming event"),
            }
        }

        #[test]
        fn test_event_with_inbox_id() {
            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread).with_inbox_id("inbox-123");

            assert_eq!(event.inbox_id, Some("inbox-123".to_string()));
        }

        #[test]
        fn test_event_bus_subscribe() {
            let bus = EventBus::new(100);
            let _rx1 = bus.subscribe();
            let _rx2 = bus.subscribe();

            assert_eq!(bus.receiver_count(), 2);
        }

        #[test]
        fn test_event_serialization() {
            let thread_id = WrappedUuidV4::new();
            let message = UIMessage::user_text(thread_id, "Test");
            let event = InboxEvent::message_created(message);

            let json = serde_json::to_string(&event).unwrap();
            assert!(json.contains("\"kind\":\"message_created\""));
        }

        #[test]
        fn test_custom_event() {
            let data = WrappedJsonValue::new(serde_json::json!({"task_id": "123", "status": "completed"}));
            let event = InboxEvent::custom("a2a.task_status_updated", data);

            match event.kind {
                InboxEventKind::Custom { event_type, data } => {
                    assert_eq!(event_type, "a2a.task_status_updated");
                    assert!(data.get_inner()["task_id"].as_str() == Some("123"));
                }
                _ => panic!("Expected Custom event"),
            }
        }

        #[tokio::test]
        async fn test_event_bus_publish() {
            let bus = EventBus::new(100);
            let mut rx = bus.subscribe();

            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread);
            let result = bus.publish(event);
            assert!(result.is_ok());

            let received = rx.recv().await;
            assert!(received.is_ok());
            match received.unwrap().kind {
                InboxEventKind::ThreadCreated { thread } => {
                    assert_eq!(thread.title, Some("Test".to_string()));
                }
                _ => panic!("Expected ThreadCreated event"),
            }
        }
    }
}
