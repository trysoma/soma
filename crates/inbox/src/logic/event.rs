//! Inbox event domain model
//!
//! Events represent occurrences within the inbox system such as new messages,
//! thread updates, or other inbox-specific events. Events support streaming
//! for real-time updates.
//!
//! ## Architecture
//!
//! The event system provides a robust multi-producer, multi-consumer event bus with:
//! - Topic-based filtering for subscriptions (thread, task, custom topics)
//! - Graceful shutdown with close state
//! - Lag recovery for slow consumers
//! - Streaming consumption with timeout handling
//!
//! ## Topic-Based Subscriptions
//!
//! Events can be tagged with multiple topics, and subscribers can filter by topic:
//! - `Topic::All` - receive all events
//! - `Topic::Thread(id)` - events for a specific thread
//! - `Topic::Message(id)` - events for a specific message
//! - `Topic::Custom(key)` - protocol-specific topics (e.g., "task:{task_id}" for A2A)
//!
//! ## Streaming Events
//!
//! `MessageStreaming` supports two delta types:
//! - `TextMessageDelta`: Simple text delta streaming (just a string)
//! - `UiMessageDelta`: Rich delta streaming based on Vercel AI SDK UIMessageChunk types
//!   See: https://github.com/vercel/ai/blob/main/packages/ai/src/ui-message-stream/ui-message-chunks.ts

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use futures::Stream;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use shared::identity::Identity;
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::time::timeout;
use tracing::{debug, trace};
use utoipa::ToSchema;

use super::inbox::DestinationType;
use super::message::Message;
use super::thread::Thread;

// ============================================================================
// Topic Types
// ============================================================================

/// Topic key for filtering event subscriptions
///
/// Topics allow subscribers to receive only events they care about.
/// Events can belong to multiple topics simultaneously.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Topic {
    /// Matches all events (global subscription)
    All,
    /// Events for a specific thread
    Thread { thread_id: WrappedUuidV4 },
    /// Events for a specific message
    Message { message_id: WrappedUuidV4 },
    /// Events that are replies to a specific message (request-response pattern)
    ///
    /// When a message is published with `reply_to` set, it automatically gets
    /// this topic added. Subscribers can filter for responses to their requests.
    Reply { to_message_id: WrappedUuidV4 },
    /// Custom topic for protocol-specific subscriptions (e.g., "task:{task_id}")
    Custom { key: String },
}

impl Topic {
    /// Create a thread topic
    pub fn thread(thread_id: impl Into<WrappedUuidV4>) -> Self {
        Topic::Thread {
            thread_id: thread_id.into(),
        }
    }

    /// Create a message topic
    pub fn message(message_id: impl Into<WrappedUuidV4>) -> Self {
        Topic::Message {
            message_id: message_id.into(),
        }
    }

    /// Create a custom topic
    pub fn custom(key: impl Into<String>) -> Self {
        Topic::Custom { key: key.into() }
    }

    /// Create a reply topic (for request-response pattern)
    ///
    /// Subscribe to this topic to receive events that are responses to
    /// a specific message you sent.
    pub fn reply(to_message_id: impl Into<WrappedUuidV4>) -> Self {
        Topic::Reply {
            to_message_id: to_message_id.into(),
        }
    }

    /// Create a task topic (convenience for A2A integration)
    pub fn task(task_id: impl AsRef<str>) -> Self {
        Topic::Custom {
            key: format!("task:{}", task_id.as_ref()),
        }
    }
}

// ============================================================================
// Event Types
// ============================================================================

/// Finish reason for message completion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    Error,
    Other,
    Unknown,
}

/// Simple text delta for streaming
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextMessageDelta {
    /// The incremental text content
    pub delta: String,
}

/// Delta types for UI message streaming, based on Vercel AI SDK UIMessageChunk types.
///
/// Reference: https://github.com/vercel/ai/blob/main/packages/ai/src/ui-message-stream/ui-message-chunks.ts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum UiMessageDelta {
    /// Start of a text part
    TextStart,
    /// Incremental text content
    TextDelta { delta: String },
    /// End of a text part
    TextEnd,
    /// Start of a reasoning part
    ReasoningStart,
    /// Incremental reasoning content
    ReasoningDelta { delta: String },
    /// End of a reasoning part
    ReasoningEnd,
    /// Error occurred during streaming
    Error { error_text: String },
    /// Tool input is fully available
    ToolInputAvailable {
        tool_call_id: String,
        tool_name: String,
        #[schemars(with = "serde_json::Value")]
        input: WrappedJsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dynamic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Error parsing tool input
    ToolInputError {
        tool_call_id: String,
        tool_name: String,
        #[schemars(with = "serde_json::Value")]
        input: WrappedJsonValue,
        error_text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dynamic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Request for tool approval
    ToolApprovalRequest {
        approval_id: String,
        tool_call_id: String,
    },
    /// Tool output is available
    ToolOutputAvailable {
        tool_call_id: String,
        #[schemars(with = "serde_json::Value")]
        output: WrappedJsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dynamic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preliminary: Option<bool>,
    },
    /// Error in tool output
    ToolOutputError {
        tool_call_id: String,
        error_text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dynamic: Option<bool>,
    },
    /// Tool execution was denied
    ToolOutputDenied { tool_call_id: String },
    /// Start of tool input streaming
    ToolInputStart {
        tool_call_id: String,
        tool_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dynamic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Incremental tool input
    ToolInputDelta {
        tool_call_id: String,
        input_text_delta: String,
    },
    /// URL source reference
    SourceUrl {
        source_id: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Document source reference
    SourceDocument {
        source_id: String,
        media_type: String,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
    /// File attachment
    File { url: String, media_type: String },
    /// Start of a step
    StartStep,
    /// End of a step
    FinishStep,
    /// Start of the message
    Start {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<serde_json::Value>")]
        message_metadata: Option<WrappedJsonValue>,
    },
    /// End of the message
    Finish {
        #[serde(skip_serializing_if = "Option::is_none")]
        finish_reason: Option<FinishReason>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<serde_json::Value>")]
        message_metadata: Option<WrappedJsonValue>,
    },
    /// Message was aborted
    Abort,
    /// Message metadata update
    MessageMetadata {
        #[schemars(with = "serde_json::Value")]
        message_metadata: WrappedJsonValue,
    },
    /// Custom data chunk (for extensibility)
    Data {
        data_type: String,
        #[schemars(with = "serde_json::Value")]
        data: WrappedJsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        transient: Option<bool>,
    },
}

/// Delta types for message streaming events
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "delta_type", rename_all = "snake_case")]
pub enum MessageStreamingDelta {
    /// Simple text delta
    Text(TextMessageDelta),
    /// Rich UI message delta based on Vercel AI SDK
    Ui(UiMessageDelta),
}

/// Source of an inbox event - identifies who published the event
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "source_type", rename_all = "snake_case")]
pub enum EventSource {
    /// Event originated from a destination (agent or workflow)
    Destination {
        /// Type of destination (agent or workflow)
        destination_type: DestinationType,
        /// Unique identifier of the destination
        destination_id: String,
    },
    /// Event originated from an inbox provider
    Inbox {
        /// The inbox ID
        inbox_id: String,
        /// Optional metadata from the inbox
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<Map<String, Value>>,
        /// Optional identity of the caller (if authenticated)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(skip)]
        #[schema(value_type = Option<Object>)]
        identity: Option<Identity>,
    },
    /// Event originated from the system (internal operations)
    System,
}

impl EventSource {
    /// Create a destination source
    pub fn destination(destination_type: DestinationType, destination_id: impl Into<String>) -> Self {
        EventSource::Destination {
            destination_type,
            destination_id: destination_id.into(),
        }
    }

    /// Create an inbox source
    pub fn inbox(inbox_id: impl Into<String>) -> Self {
        EventSource::Inbox {
            inbox_id: inbox_id.into(),
            metadata: None,
            identity: None,
        }
    }

    /// Create an inbox source with metadata
    pub fn inbox_with_metadata(
        inbox_id: impl Into<String>,
        metadata: Option<Map<String, Value>>,
        identity: Option<Identity>,
    ) -> Self {
        EventSource::Inbox {
            inbox_id: inbox_id.into(),
            metadata,
            identity,
        }
    }

    /// Check if this source matches the given destination
    pub fn is_destination(&self, dest_type: &DestinationType, dest_id: &str) -> bool {
        matches!(self, EventSource::Destination { destination_type, destination_id }
            if destination_type == dest_type && destination_id == dest_id)
    }

    /// Check if this source matches the given inbox
    pub fn is_inbox(&self, id: &str) -> bool {
        matches!(self, EventSource::Inbox { inbox_id, .. } if inbox_id == id)
    }
}

/// Types of inbox events
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InboxEventKind {
    /// A new message was created
    MessageCreated { message: Message },
    /// An existing message was updated (e.g., streaming completion)
    MessageUpdated { message: Message },
    /// A message was deleted
    MessageDeleted { message_id: WrappedUuidV4 },
    /// Message streaming (incremental updates)
    MessageStreaming {
        message_id: WrappedUuidV4,
        thread_id: WrappedUuidV4,
        /// The part ID being updated
        part_id: String,
        /// The streaming delta (text or UI)
        delta: MessageStreamingDelta,
        /// Provider-specific metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<serde_json::Value>")]
        provider_metadata: Option<WrappedJsonValue>,
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
    /// Source of this event - identifies who published it
    pub source: EventSource,
    pub created_at: WrappedChronoDateTime,
    /// Message ID this event is a reply to (for request-response correlation)
    ///
    /// When set, this event automatically gets `Topic::Reply { to_message_id }` added,
    /// allowing subscribers to filter for responses to their requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<WrappedUuidV4>,
    /// Additional custom topics for filtering (e.g., task IDs for A2A)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(skip)]
    #[schema(value_type = Vec<Object>)]
    pub topics: Vec<Topic>,
}

impl InboxEvent {
    /// Create a new event with the given kind and source
    pub fn new(kind: InboxEventKind, source: EventSource) -> Self {
        Self {
            id: WrappedUuidV4::new(),
            kind,
            source,
            created_at: WrappedChronoDateTime::now(),
            reply_to: None,
            topics: Vec::new(),
        }
    }

    /// Get all topics this event belongs to (including implicit topics from the event kind)
    pub fn all_topics(&self) -> Vec<Topic> {
        let mut topics = vec![Topic::All];

        // Add reply topic if this is a response to another message
        if let Some(ref reply_to_id) = self.reply_to {
            topics.push(Topic::reply(reply_to_id.clone()));
        }

        // Add implicit topics based on event kind
        match &self.kind {
            InboxEventKind::MessageCreated { message } | InboxEventKind::MessageUpdated { message } => {
                topics.push(Topic::thread(message.thread_id().clone()));
                topics.push(Topic::message(message.id().clone()));
            }
            InboxEventKind::MessageDeleted { message_id } => {
                topics.push(Topic::message(message_id.clone()));
            }
            InboxEventKind::MessageStreaming {
                thread_id,
                message_id,
                ..
            } => {
                topics.push(Topic::thread(thread_id.clone()));
                topics.push(Topic::message(message_id.clone()));
            }
            InboxEventKind::ThreadCreated { thread } | InboxEventKind::ThreadUpdated { thread } => {
                topics.push(Topic::thread(thread.id.clone()));
            }
            InboxEventKind::ThreadDeleted { thread_id } => {
                topics.push(Topic::thread(thread_id.clone()));
            }
            InboxEventKind::Custom { .. } => {
                // Custom events rely on explicit topics
            }
        }

        // Add explicit custom topics
        topics.extend(self.topics.iter().cloned());

        topics
    }

    /// Check if this event matches any of the given topics
    pub fn matches_topics(&self, filter_topics: &[Topic]) -> bool {
        if filter_topics.is_empty() {
            return true;
        }
        let event_topics = self.all_topics();
        filter_topics.iter().any(|t| event_topics.contains(t))
    }

    /// Create a new message created event (system source by default)
    pub fn message_created(message: Message) -> Self {
        Self::new(InboxEventKind::MessageCreated { message }, EventSource::System)
    }

    /// Create a new message updated event (system source by default)
    pub fn message_updated(message: Message) -> Self {
        Self::new(InboxEventKind::MessageUpdated { message }, EventSource::System)
    }

    /// Create a message deleted event (system source by default)
    pub fn message_deleted(message_id: WrappedUuidV4) -> Self {
        Self::new(InboxEventKind::MessageDeleted { message_id }, EventSource::System)
    }

    /// Create a message streaming event with any delta type (system source by default)
    pub fn message_streaming(
        message_id: WrappedUuidV4,
        thread_id: WrappedUuidV4,
        part_id: impl Into<String>,
        delta: MessageStreamingDelta,
        provider_metadata: Option<WrappedJsonValue>,
    ) -> Self {
        Self::new(
            InboxEventKind::MessageStreaming {
                message_id,
                thread_id,
                part_id: part_id.into(),
                delta,
                provider_metadata,
            },
            EventSource::System,
        )
    }

    /// Create a simple text streaming event (incremental text update)
    pub fn text_message_streaming(
        message_id: WrappedUuidV4,
        thread_id: WrappedUuidV4,
        part_id: impl Into<String>,
        delta: impl Into<String>,
        provider_metadata: Option<WrappedJsonValue>,
    ) -> Self {
        Self::message_streaming(
            message_id,
            thread_id,
            part_id,
            MessageStreamingDelta::Text(TextMessageDelta {
                delta: delta.into(),
            }),
            provider_metadata,
        )
    }

    /// Create a rich UI message streaming event based on Vercel AI SDK UIMessageChunk types
    pub fn ui_message_streaming(
        message_id: WrappedUuidV4,
        thread_id: WrappedUuidV4,
        part_id: impl Into<String>,
        delta: UiMessageDelta,
        provider_metadata: Option<WrappedJsonValue>,
    ) -> Self {
        Self::message_streaming(
            message_id,
            thread_id,
            part_id,
            MessageStreamingDelta::Ui(delta),
            provider_metadata,
        )
    }

    /// Create a thread created event (system source by default)
    pub fn thread_created(thread: Thread) -> Self {
        Self::new(InboxEventKind::ThreadCreated { thread }, EventSource::System)
    }

    /// Create a thread updated event (system source by default)
    pub fn thread_updated(thread: Thread) -> Self {
        Self::new(InboxEventKind::ThreadUpdated { thread }, EventSource::System)
    }

    /// Create a thread deleted event (system source by default)
    pub fn thread_deleted(thread_id: WrappedUuidV4) -> Self {
        Self::new(InboxEventKind::ThreadDeleted { thread_id }, EventSource::System)
    }

    /// Create a custom event (system source by default)
    ///
    /// Use this for protocol-specific events (e.g., A2A tasks, artifacts).
    pub fn custom(event_type: impl Into<String>, data: WrappedJsonValue) -> Self {
        Self::new(
            InboxEventKind::Custom {
                event_type: event_type.into(),
                data,
            },
            EventSource::System,
        )
    }

    /// Set the source for this event
    pub fn with_source(mut self, source: EventSource) -> Self {
        self.source = source;
        self
    }

    /// Add a topic to this event
    pub fn with_topic(mut self, topic: Topic) -> Self {
        self.topics.push(topic);
        self
    }

    /// Add a task topic (convenience for A2A integration)
    pub fn with_task(self, task_id: impl AsRef<str>) -> Self {
        self.with_topic(Topic::task(task_id))
    }

    /// Add a thread topic
    pub fn with_thread(self, thread_id: impl Into<WrappedUuidV4>) -> Self {
        self.with_topic(Topic::thread(thread_id))
    }

    /// Set the reply_to field (for request-response correlation)
    ///
    /// This marks this event as a response to another message.
    /// The event will automatically get `Topic::Reply { to_message_id }` added
    /// to its topics, allowing request originators to filter for responses.
    pub fn in_reply_to(mut self, message_id: impl Into<WrappedUuidV4>) -> Self {
        self.reply_to = Some(message_id.into());
        self
    }

    /// Set the source as a destination
    pub fn from_destination(self, destination_type: DestinationType, destination_id: impl Into<String>) -> Self {
        self.with_source(EventSource::destination(destination_type, destination_id))
    }

    /// Set the source as an inbox
    pub fn from_inbox(self, inbox_id: impl Into<String>) -> Self {
        self.with_source(EventSource::inbox(inbox_id))
    }

    /// Set the source as an inbox with metadata and identity
    pub fn from_inbox_with_metadata(
        self,
        inbox_id: impl Into<String>,
        metadata: Option<Map<String, Value>>,
        identity: Option<Identity>,
    ) -> Self {
        self.with_source(EventSource::inbox_with_metadata(inbox_id, metadata, identity))
    }

    /// Check if this event should be delivered to a destination
    /// (events are not delivered to the source that published them)
    pub fn should_deliver_to_destination(&self, dest_type: &DestinationType, dest_id: &str) -> bool {
        !self.source.is_destination(dest_type, dest_id)
    }

    /// Check if this event should be delivered to an inbox
    /// (events are not delivered to the source that published them)
    pub fn should_deliver_to_inbox(&self, inbox_id: &str) -> bool {
        !self.source.is_inbox(inbox_id)
    }
}

// ============================================================================
// Channel Types
// ============================================================================

/// Channel types for event broadcasting
pub type EventTx = broadcast::Sender<InboxEvent>;
pub type EventRx = broadcast::Receiver<InboxEvent>;

/// Create a new event broadcast channel
pub fn create_event_channel(capacity: usize) -> (EventTx, EventRx) {
    broadcast::channel(capacity)
}

/// Default capacity for event queues
pub const DEFAULT_EVENT_QUEUE_CAPACITY: usize = 1024;

// ============================================================================
// EventQueue - Robust queue with close state and lag handling
// ============================================================================

/// Errors when dequeuing events
#[derive(Debug, thiserror::Error)]
pub enum DequeueError {
    #[error("Queue is empty")]
    QueueEmpty,
    #[error("Queue is closed")]
    QueueClosed,
}

/// A robust event queue with graceful shutdown and lag recovery
///
/// Each EventQueue wraps a broadcast channel and provides:
/// - Explicit close state for graceful shutdown
/// - Lag recovery for slow consumers
/// - `tap()` to create independent subscribers
/// - Topic filtering support
pub struct EventQueue {
    sender: EventTx,
    receiver: Arc<Mutex<EventRx>>,
    is_closed: Arc<RwLock<bool>>,
    /// Topics to filter events (empty means all events)
    filter_topics: Vec<Topic>,
}

impl EventQueue {
    /// Create a new EventQueue with specified capacity
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be greater than 0");
        let (sender, receiver) = broadcast::channel(capacity);
        trace!("EventQueue initialized with capacity {}", capacity);
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            is_closed: Arc::new(RwLock::new(false)),
            filter_topics: Vec::new(),
        }
    }

    /// Create an EventQueue with topic filtering
    pub fn with_topics(capacity: usize, topics: Vec<Topic>) -> Self {
        let mut queue = Self::new(capacity);
        queue.filter_topics = topics;
        queue
    }

    /// Create an EventQueue from an existing sender (for tapping)
    fn from_sender(sender: EventTx, is_closed: Arc<RwLock<bool>>, filter_topics: Vec<Topic>) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(sender.subscribe())),
            sender,
            is_closed,
            filter_topics,
        }
    }

    /// Enqueue an event to this queue
    pub async fn enqueue(&self, event: InboxEvent) -> Result<usize, broadcast::error::SendError<InboxEvent>> {
        if *self.is_closed.read().await {
            trace!("Queue closed, event not enqueued");
            return Ok(0);
        }
        trace!(event_id = %event.id, "Enqueuing event");
        self.sender.send(event)
    }

    /// Dequeue an event from the queue
    ///
    /// If `no_wait` is true, returns immediately with QueueEmpty if no events available.
    /// If `no_wait` is false, blocks until an event is available.
    pub async fn dequeue(&self, no_wait: bool) -> Result<InboxEvent, DequeueError> {
        loop {
            let is_closed = *self.is_closed.read().await;
            let mut receiver = self.receiver.lock().await;

            if is_closed && receiver.is_empty() {
                trace!("Queue closed and empty");
                return Err(DequeueError::QueueClosed);
            }

            let event = if no_wait {
                match receiver.try_recv() {
                    Ok(event) => event,
                    Err(broadcast::error::TryRecvError::Empty) => return Err(DequeueError::QueueEmpty),
                    Err(broadcast::error::TryRecvError::Closed) => return Err(DequeueError::QueueClosed),
                    Err(broadcast::error::TryRecvError::Lagged(_)) => {
                        // Recover from lag by trying again
                        match receiver.try_recv() {
                            Ok(event) => event,
                            Err(_) => return Err(DequeueError::QueueEmpty),
                        }
                    }
                }
            } else {
                match receiver.recv().await {
                    Ok(event) => event,
                    Err(broadcast::error::RecvError::Closed) => return Err(DequeueError::QueueClosed),
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Recover from lag by trying again
                        match receiver.recv().await {
                            Ok(event) => event,
                            Err(_) => return Err(DequeueError::QueueClosed),
                        }
                    }
                }
            };

            // Apply topic filtering
            if self.filter_topics.is_empty() || event.matches_topics(&self.filter_topics) {
                trace!(event_id = %event.id, "Dequeued event");
                return Ok(event);
            }
            // Event didn't match filter, continue loop
        }
    }

    /// Create a new subscriber to this queue with the same filter
    pub fn tap(&self) -> EventQueue {
        trace!("Tapping EventQueue");
        EventQueue::from_sender(self.sender.clone(), self.is_closed.clone(), self.filter_topics.clone())
    }

    /// Create a new subscriber with different topic filters
    pub fn tap_with_topics(&self, topics: Vec<Topic>) -> EventQueue {
        trace!("Tapping EventQueue with custom topics");
        EventQueue::from_sender(self.sender.clone(), self.is_closed.clone(), topics)
    }

    /// Close the queue for future events
    pub async fn close(&self) {
        trace!("Closing EventQueue");
        let mut is_closed = self.is_closed.write().await;
        *is_closed = true;
    }

    /// Check if the queue is closed
    pub async fn is_closed(&self) -> bool {
        *self.is_closed.read().await
    }

    /// Get the sender for publishing events directly
    pub fn sender(&self) -> EventTx {
        self.sender.clone()
    }

    /// Get the number of active receivers
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Clone for EventQueue {
    fn clone(&self) -> Self {
        // Clone creates a new subscriber (like tap)
        Self {
            sender: self.sender.clone(),
            receiver: Arc::new(Mutex::new(self.sender.subscribe())),
            is_closed: self.is_closed.clone(),
            filter_topics: self.filter_topics.clone(),
        }
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new(DEFAULT_EVENT_QUEUE_CAPACITY)
    }
}

// ============================================================================
// SubscriptionManager - Keyed subscriptions for threads, tasks, etc.
// ============================================================================

/// Manager for keyed event subscriptions
///
/// Allows creating and managing subscriptions keyed by any string identifier.
/// Useful for:
/// - Thread-based subscriptions (key = thread_id)
/// - Task-based subscriptions (key = task_id)
/// - Any other keyed subscription pattern
pub struct SubscriptionManager {
    queues: Arc<DashMap<String, EventQueue>>,
    /// Shared sender for all queues (from the main EventBus)
    shared_sender: EventTx,
    /// Shared close state
    shared_is_closed: Arc<RwLock<bool>>,
}

impl SubscriptionManager {
    /// Create a new SubscriptionManager with a shared sender
    pub fn new(shared_sender: EventTx) -> Self {
        Self {
            queues: Arc::new(DashMap::new()),
            shared_sender,
            shared_is_closed: Arc::new(RwLock::new(false)),
        }
    }

    /// Get or create a subscription for a key
    ///
    /// If a subscription already exists, returns a tap of it.
    /// If not, creates a new subscription with the given topics.
    pub fn subscribe(&self, key: &str, topics: Vec<Topic>) -> EventQueue {
        if let Some(queue) = self.queues.get(key) {
            trace!(key, "Tapping existing subscription");
            return queue.tap();
        }

        trace!(key, "Creating new subscription");
        let queue = EventQueue::from_sender(
            self.shared_sender.clone(),
            self.shared_is_closed.clone(),
            topics,
        );
        self.queues.insert(key.to_string(), queue.clone());
        queue
    }

    /// Get or create a thread subscription
    pub fn subscribe_thread(&self, thread_id: &WrappedUuidV4) -> EventQueue {
        let key = format!("thread:{}", thread_id);
        self.subscribe(&key, vec![Topic::thread(thread_id.clone())])
    }

    /// Get or create a task subscription (for A2A)
    pub fn subscribe_task(&self, task_id: &str) -> EventQueue {
        let key = format!("task:{}", task_id);
        self.subscribe(&key, vec![Topic::task(task_id)])
    }

    /// Get an existing subscription without creating
    pub fn get(&self, key: &str) -> Option<EventQueue> {
        self.queues.get(key).map(|q| q.tap())
    }

    /// Remove a subscription
    pub async fn unsubscribe(&self, key: &str) -> Option<EventQueue> {
        if let Some((_, queue)) = self.queues.remove(key) {
            queue.close().await;
            Some(queue)
        } else {
            None
        }
    }

    /// Check if a subscription exists
    pub fn exists(&self, key: &str) -> bool {
        self.queues.contains_key(key)
    }

    /// Get the count of active subscriptions
    pub fn count(&self) -> usize {
        self.queues.len()
    }

    /// Close all subscriptions
    pub async fn close_all(&self) {
        let mut is_closed = self.shared_is_closed.write().await;
        *is_closed = true;

        // Close all individual queues
        for entry in self.queues.iter() {
            entry.close().await;
        }
    }
}

// ============================================================================
// EventConsumer - Streaming consumption with timeout handling
// ============================================================================

/// Consumer for reading events from a queue with streaming support
///
/// Provides convenient methods for consuming events:
/// - `consume_one()` - Non-blocking single event consumption
/// - `consume_one_blocking()` - Blocking single event consumption
/// - `consume_all()` - Async stream of events with timeout handling
pub struct EventConsumer {
    queue: EventQueue,
    timeout_duration: Duration,
}

impl EventConsumer {
    /// Create a new EventConsumer for the given queue
    pub fn new(queue: EventQueue) -> Self {
        trace!("EventConsumer initialized");
        Self {
            queue,
            timeout_duration: Duration::from_millis(500),
        }
    }

    /// Create a consumer with custom timeout
    pub fn with_timeout(queue: EventQueue, timeout: Duration) -> Self {
        Self {
            queue,
            timeout_duration: timeout,
        }
    }

    /// Consume one event from the queue (non-blocking)
    pub async fn consume_one(&self) -> Result<InboxEvent, DequeueError> {
        trace!("Consuming event (non-blocking)");
        self.queue.dequeue(true).await
    }

    /// Consume one event from the queue (blocking)
    pub async fn consume_one_blocking(&self) -> Result<InboxEvent, DequeueError> {
        trace!("Consuming event (blocking)");
        self.queue.dequeue(false).await
    }

    /// Consume all events as an async stream
    ///
    /// The stream yields events until the queue is closed.
    /// Uses timeout to periodically check for close state.
    pub fn consume_all(&self) -> impl Stream<Item = Result<InboxEvent, DequeueError>> + '_ {
        trace!("Starting consume_all stream");
        let queue = self.queue.clone();
        let timeout_duration = self.timeout_duration;

        async_stream::stream! {
            loop {
                match timeout(timeout_duration, queue.dequeue(false)).await {
                    Ok(Ok(event)) => {
                        trace!(event_id = %event.id, "Yielding event from stream");
                        yield Ok(event);
                    }
                    Ok(Err(DequeueError::QueueClosed)) => {
                        if queue.is_closed().await {
                            debug!("Queue closed, ending stream");
                            break;
                        }
                    }
                    Ok(Err(DequeueError::QueueEmpty)) => {
                        // Should not happen in blocking mode, but handle gracefully
                        continue;
                    }
                    Err(_) => {
                        // Timeout - check if we should continue
                        if queue.is_closed().await {
                            debug!("Queue closed during timeout, ending stream");
                            break;
                        }
                        continue;
                    }
                }
            }
        }
    }

    /// Consume events until a predicate returns true (terminal event)
    ///
    /// Useful for protocols like A2A where certain events signal completion.
    pub fn consume_until<F>(&self, is_terminal: F) -> impl Stream<Item = Result<InboxEvent, DequeueError>> + '_
    where
        F: Fn(&InboxEvent) -> bool + 'static,
    {
        let queue = self.queue.clone();
        let timeout_duration = self.timeout_duration;

        async_stream::stream! {
            loop {
                match timeout(timeout_duration, queue.dequeue(false)).await {
                    Ok(Ok(event)) => {
                        let is_final = is_terminal(&event);
                        trace!(event_id = %event.id, is_final, "Yielding event from stream");

                        if is_final {
                            queue.close().await;
                            yield Ok(event);
                            break;
                        }

                        yield Ok(event);
                    }
                    Ok(Err(DequeueError::QueueClosed)) => {
                        if queue.is_closed().await {
                            break;
                        }
                    }
                    Ok(Err(DequeueError::QueueEmpty)) => {
                        continue;
                    }
                    Err(_) => {
                        if queue.is_closed().await {
                            break;
                        }
                        continue;
                    }
                }
            }
        }
    }

    /// Get a reference to the underlying queue
    pub fn queue(&self) -> &EventQueue {
        &self.queue
    }

    /// Close the underlying queue
    pub async fn close(&self) {
        self.queue.close().await;
    }
}

// ============================================================================
// EventBus - Main entry point for the event system
// ============================================================================

/// Multi-producer, multi-consumer event bus for inbox events
///
/// The EventBus is the main entry point for the event system. It provides:
/// - Global event publishing
/// - Topic-based subscriptions
/// - Keyed subscription management via SubscriptionManager
#[derive(Clone)]
pub struct EventBus {
    tx: EventTx,
    is_closed: Arc<RwLock<bool>>,
}

impl EventBus {
    /// Create a new event bus with the specified capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            is_closed: Arc::new(RwLock::new(false)),
        }
    }

    /// Get a sender handle for publishing events
    pub fn sender(&self) -> EventTx {
        self.tx.clone()
    }

    /// Subscribe to all events (unfiltered)
    pub fn subscribe(&self) -> EventRx {
        self.tx.subscribe()
    }

    /// Subscribe with topic filtering
    pub fn subscribe_filtered(&self, topics: Vec<Topic>) -> EventQueue {
        EventQueue::from_sender(self.tx.clone(), self.is_closed.clone(), topics)
    }

    /// Subscribe to a specific thread
    pub fn subscribe_thread(&self, thread_id: impl Into<WrappedUuidV4>) -> EventQueue {
        self.subscribe_filtered(vec![Topic::thread(thread_id)])
    }

    /// Subscribe to a specific task (for A2A)
    pub fn subscribe_task(&self, task_id: impl AsRef<str>) -> EventQueue {
        self.subscribe_filtered(vec![Topic::task(task_id)])
    }

    /// Subscribe to replies to a specific message (for request-response pattern)
    ///
    /// This creates a subscription that only receives events where `reply_to`
    /// matches the given message ID. Useful for waiting for responses to requests.
    pub fn subscribe_replies(&self, to_message_id: impl Into<WrappedUuidV4>) -> EventQueue {
        self.subscribe_filtered(vec![Topic::reply(to_message_id)])
    }

    /// Create a SubscriptionManager for managing keyed subscriptions
    pub fn create_subscription_manager(&self) -> SubscriptionManager {
        SubscriptionManager::new(self.tx.clone())
    }

    /// Create an EventQueue for this bus
    pub fn create_queue(&self) -> EventQueue {
        EventQueue::from_sender(self.tx.clone(), self.is_closed.clone(), Vec::new())
    }

    /// Create an EventQueue with topic filtering
    pub fn create_queue_with_topics(&self, topics: Vec<Topic>) -> EventQueue {
        EventQueue::from_sender(self.tx.clone(), self.is_closed.clone(), topics)
    }

    /// Publish an event to all subscribers
    #[allow(clippy::result_large_err)]
    pub fn publish(&self, event: InboxEvent) -> Result<usize, broadcast::error::SendError<InboxEvent>> {
        self.tx.send(event)
    }

    /// Get the number of active subscribers
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Close the event bus
    pub async fn close(&self) {
        let mut is_closed = self.is_closed.write().await;
        *is_closed = true;
    }

    /// Check if the event bus is closed
    pub async fn is_closed(&self) -> bool {
        *self.is_closed.read().await
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(DEFAULT_EVENT_QUEUE_CAPACITY)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_topic_creation() {
            let thread_topic = Topic::thread(WrappedUuidV4::new());
            assert!(matches!(thread_topic, Topic::Thread { .. }));

            let task_topic = Topic::task("task-123");
            assert!(matches!(task_topic, Topic::Custom { key } if key == "task:task-123"));

            let custom_topic = Topic::custom("my-key");
            assert!(matches!(custom_topic, Topic::Custom { key } if key == "my-key"));
        }

        #[test]
        fn test_event_message_created() {
            use super::super::super::message::{MessageRole, UIMessagePart};

            let thread_id = WrappedUuidV4::new();
            let message = Message::ui(thread_id, MessageRole::User, vec![UIMessagePart::text("Hello!")]);
            let event = InboxEvent::message_created(message.clone());

            match event.kind {
                InboxEventKind::MessageCreated { message: m } => {
                    assert_eq!(m.text_content(), "Hello!");
                }
                _ => panic!("Expected MessageCreated event"),
            }
        }

        #[test]
        fn test_event_with_topics() {
            let thread_id = WrappedUuidV4::new();
            let event = InboxEvent::custom("test", WrappedJsonValue::new(serde_json::json!({})))
                .with_task("task-123")
                .with_thread(thread_id.clone());

            let topics = event.all_topics();
            assert!(topics.contains(&Topic::All));
            assert!(topics.contains(&Topic::task("task-123")));
            assert!(topics.contains(&Topic::thread(thread_id)));
        }

        #[test]
        fn test_event_reply_to() {
            let request_message_id = WrappedUuidV4::new();
            let event = InboxEvent::custom("response", WrappedJsonValue::new(serde_json::json!({})))
                .in_reply_to(request_message_id.clone());

            // Should have reply_to set
            assert_eq!(event.reply_to, Some(request_message_id.clone()));

            // Should include reply topic
            let topics = event.all_topics();
            assert!(topics.contains(&Topic::reply(request_message_id.clone())));

            // Should match reply filter
            assert!(event.matches_topics(&[Topic::reply(request_message_id)]));
        }

        #[test]
        fn test_topic_reply() {
            let message_id = WrappedUuidV4::new();
            let reply_topic = Topic::reply(message_id.clone());
            assert!(matches!(reply_topic, Topic::Reply { to_message_id } if to_message_id == message_id));
        }

        #[test]
        fn test_event_matches_topics() {
            use super::super::super::message::{MessageRole, UIMessagePart};

            let thread_id = WrappedUuidV4::new();
            let message = Message::ui(thread_id.clone(), MessageRole::User, vec![UIMessagePart::text("Test")]);
            let event = InboxEvent::message_created(message);

            // Should match thread topic
            assert!(event.matches_topics(&[Topic::thread(thread_id.clone())]));

            // Should match All
            assert!(event.matches_topics(&[Topic::All]));

            // Should not match different thread
            assert!(!event.matches_topics(&[Topic::thread(WrappedUuidV4::new())]));

            // Empty filter matches all
            assert!(event.matches_topics(&[]));
        }

        #[test]
        fn test_event_text_message_streaming() {
            let message_id = WrappedUuidV4::new();
            let thread_id = WrappedUuidV4::new();
            let event =
                InboxEvent::text_message_streaming(message_id.clone(), thread_id.clone(), "part-0", "Hello ", None);

            match event.kind {
                InboxEventKind::MessageStreaming {
                    message_id: m_id,
                    thread_id: t_id,
                    part_id,
                    delta,
                    provider_metadata,
                } => {
                    assert_eq!(m_id, message_id);
                    assert_eq!(t_id, thread_id);
                    assert_eq!(part_id, "part-0");
                    assert!(provider_metadata.is_none());
                    match delta {
                        MessageStreamingDelta::Text(TextMessageDelta { delta }) => {
                            assert_eq!(delta, "Hello ");
                        }
                        _ => panic!("Expected Text delta"),
                    }
                }
                _ => panic!("Expected MessageStreaming event"),
            }
        }

        #[test]
        fn test_event_ui_message_streaming() {
            let message_id = WrappedUuidV4::new();
            let thread_id = WrappedUuidV4::new();
            let delta = UiMessageDelta::TextDelta {
                delta: "Hello world".to_string(),
            };
            let event =
                InboxEvent::ui_message_streaming(message_id.clone(), thread_id.clone(), "part-1", delta, None);

            match event.kind {
                InboxEventKind::MessageStreaming {
                    message_id: m_id,
                    thread_id: t_id,
                    part_id,
                    delta,
                    provider_metadata,
                } => {
                    assert_eq!(m_id, message_id);
                    assert_eq!(t_id, thread_id);
                    assert_eq!(part_id, "part-1");
                    assert!(provider_metadata.is_none());
                    match delta {
                        MessageStreamingDelta::Ui(UiMessageDelta::TextDelta { delta }) => {
                            assert_eq!(delta, "Hello world");
                        }
                        _ => panic!("Expected Ui TextDelta"),
                    }
                }
                _ => panic!("Expected MessageStreaming event"),
            }
        }

        #[test]
        fn test_event_with_source() {
            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread).from_inbox("inbox-123");

            assert!(event.source.is_inbox("inbox-123"));
            assert!(!event.source.is_inbox("other-inbox"));
        }

        #[test]
        fn test_event_source_destination() {
            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread).from_destination(DestinationType::Agent, "agent-456");

            assert!(event.source.is_destination(&DestinationType::Agent, "agent-456"));
            assert!(!event.source.is_destination(&DestinationType::Workflow, "agent-456"));
            assert!(!event.source.is_destination(&DestinationType::Agent, "other-agent"));
        }

        #[test]
        fn test_event_should_deliver_to_destination() {
            let thread = Thread::new(Some("Test".to_string()));

            // Event from agent-1 should not be delivered to agent-1
            let event = InboxEvent::thread_created(thread.clone()).from_destination(DestinationType::Agent, "agent-1");
            assert!(!event.should_deliver_to_destination(&DestinationType::Agent, "agent-1"));
            assert!(event.should_deliver_to_destination(&DestinationType::Agent, "agent-2"));
            assert!(event.should_deliver_to_destination(&DestinationType::Workflow, "workflow-1"));
        }

        #[test]
        fn test_event_should_deliver_to_inbox() {
            let thread = Thread::new(Some("Test".to_string()));

            // Event from inbox-1 should not be delivered to inbox-1
            let event = InboxEvent::thread_created(thread).from_inbox("inbox-1");
            assert!(!event.should_deliver_to_inbox("inbox-1"));
            assert!(event.should_deliver_to_inbox("inbox-2"));
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
            use super::super::super::message::{MessageRole, UIMessagePart};

            let thread_id = WrappedUuidV4::new();
            let message = Message::ui(thread_id, MessageRole::User, vec![UIMessagePart::text("Test")]);
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

        #[tokio::test]
        async fn test_event_queue_basic() {
            let queue = EventQueue::default();

            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread);
            queue.enqueue(event).await.unwrap();

            let received = queue.dequeue(true).await.unwrap();
            match received.kind {
                InboxEventKind::ThreadCreated { thread } => {
                    assert_eq!(thread.title, Some("Test".to_string()));
                }
                _ => panic!("Expected ThreadCreated event"),
            }
        }

        #[tokio::test]
        async fn test_event_queue_tap() {
            let queue = EventQueue::default();
            let tapped = queue.tap();

            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread);
            queue.enqueue(event).await.unwrap();

            // Both queues should receive the event
            let received1 = queue.dequeue(true).await.unwrap();
            let received2 = tapped.dequeue(true).await.unwrap();

            assert_eq!(received1.id, received2.id);
        }

        #[tokio::test]
        async fn test_event_queue_close() {
            let queue = EventQueue::default();

            assert!(!queue.is_closed().await);
            queue.close().await;
            assert!(queue.is_closed().await);

            // Enqueue should succeed but not actually send
            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread);
            let result = queue.enqueue(event).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }

        #[tokio::test]
        async fn test_event_queue_filtered() {
            use super::super::super::message::{MessageRole, UIMessagePart};

            let thread_id_1 = WrappedUuidV4::new();
            let thread_id_2 = WrappedUuidV4::new();

            let queue = EventQueue::with_topics(100, vec![Topic::thread(thread_id_1.clone())]);

            // Create events for different threads
            let msg1 = Message::ui(thread_id_1.clone(), MessageRole::User, vec![UIMessagePart::text("Thread 1")]);
            let msg2 = Message::ui(thread_id_2.clone(), MessageRole::User, vec![UIMessagePart::text("Thread 2")]);

            let event1 = InboxEvent::message_created(msg1);
            let event2 = InboxEvent::message_created(msg2);

            queue.enqueue(event1).await.unwrap();
            queue.enqueue(event2).await.unwrap();

            // Should only receive event for thread_id_1
            let received = queue.dequeue(true).await.unwrap();
            match &received.kind {
                InboxEventKind::MessageCreated { message } => {
                    assert_eq!(message.thread_id(), &thread_id_1);
                }
                _ => panic!("Expected MessageCreated event"),
            }

            // Second dequeue should be empty (event2 was filtered out)
            let result = queue.dequeue(true).await;
            assert!(matches!(result, Err(DequeueError::QueueEmpty)));
        }

        #[tokio::test]
        async fn test_subscription_manager() {
            let bus = EventBus::default();
            let manager = bus.create_subscription_manager();

            // Create subscriptions
            let _sub1 = manager.subscribe_thread(&WrappedUuidV4::new());
            let _sub2 = manager.subscribe_task("task-123");

            assert_eq!(manager.count(), 2);
            assert!(manager.exists("task:task-123"));
        }

        #[tokio::test]
        async fn test_event_consumer_basic() {
            let queue = EventQueue::default();
            let consumer = EventConsumer::new(queue.clone());

            let thread = Thread::new(Some("Test".to_string()));
            let event = InboxEvent::thread_created(thread);
            queue.enqueue(event).await.unwrap();

            let received = consumer.consume_one().await.unwrap();
            match received.kind {
                InboxEventKind::ThreadCreated { thread } => {
                    assert_eq!(thread.title, Some("Test".to_string()));
                }
                _ => panic!("Expected ThreadCreated event"),
            }
        }
    }
}
