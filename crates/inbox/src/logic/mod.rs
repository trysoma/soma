//! Logic layer for inbox crate
//! Contains domain models and business logic for messages, threads, events, inboxes, and destinations

pub mod destination;
pub mod event;
pub mod inbox;
pub mod message;
pub mod thread;

// Re-export commonly used types
pub use destination::{Destination, DestinationHandle, DestinationRegistry};
pub use event::{
    // Core event types
    InboxEvent, InboxEventKind, EventSource,
    // Topic-based filtering
    Topic,
    // Channel types
    create_event_channel, EventTx, EventRx,
    // Event bus and queue infrastructure
    EventBus, EventQueue, DequeueError, DEFAULT_EVENT_QUEUE_CAPACITY,
    // Subscription management
    SubscriptionManager,
    // Consumer for streaming
    EventConsumer,
};
pub use inbox::{Inbox, InboxHandle, InboxProvider, InboxProviderRegistry, InboxProviderState, DestinationType};
pub use message::{UIMessage, UIMessagePart, MessageRole};
pub use thread::Thread;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

// --- Config Change Events ---
// Used to notify when inbox configuration changes (for syncing to soma.yaml)

/// Serialized inbox data for config change events
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct InboxSerialized {
    pub id: String,
    pub provider_id: String,
    pub destination_type: DestinationType,
    pub destination_id: String,
    pub configuration: Value,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub settings: Map<String, Value>,
}

impl From<Inbox> for InboxSerialized {
    fn from(inbox: Inbox) -> Self {
        Self {
            id: inbox.id,
            provider_id: inbox.provider_id,
            destination_type: inbox.destination_type,
            destination_id: inbox.destination_id,
            configuration: inbox.configuration.get_inner().clone(),
            settings: inbox.settings,
        }
    }
}

/// Events emitted when inbox configuration changes
#[derive(Clone, Debug)]
pub enum OnConfigChangeEvt {
    /// An inbox was created
    InboxAdded(InboxSerialized),
    /// An inbox was updated
    InboxUpdated(InboxSerialized),
    /// An inbox was removed (inbox_id)
    InboxRemoved(String),
}

/// Sender for config change events
pub type OnConfigChangeTx = tokio::sync::broadcast::Sender<OnConfigChangeEvt>;
/// Receiver for config change events
pub type OnConfigChangeRx = tokio::sync::broadcast::Receiver<OnConfigChangeEvt>;
