//! Row conversion implementations for SQLC-generated types

use serde_json::{Map, Value};
use shared::error::CommonError;

use crate::logic::{inbox::Inbox, message::{UIMessage, UIMessagePart}, thread::Thread};
use crate::repository::StoredEvent;

use super::{
    Row_get_enabled_inboxes, Row_get_event_by_id, Row_get_events, Row_get_events_by_inbox,
    Row_get_events_by_kind, Row_get_inbox_by_id, Row_get_inboxes, Row_get_inboxes_by_provider,
    Row_get_message_by_id, Row_get_messages, Row_get_messages_by_thread, Row_get_thread_by_id,
    Row_get_threads,
};

// --- Helper Functions ---

/// Convert WrappedJsonValue to Map<String, Value>
fn json_to_map(json: &shared::primitives::WrappedJsonValue) -> Map<String, Value> {
    match json.get_inner() {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    }
}

/// Parse UIMessagePart array from JSON
fn parse_parts(
    parts_json: &shared::primitives::WrappedJsonValue,
) -> Result<Vec<UIMessagePart>, CommonError> {
    serde_json::from_value(parts_json.get_inner().clone()).map_err(|e| CommonError::Repository {
        msg: format!("Failed to parse message parts: {e}"),
        source: Some(e.into()),
    })
}

// --- Thread Conversions ---

impl TryFrom<Row_get_thread_by_id> for Thread {
    type Error = CommonError;
    fn try_from(row: Row_get_thread_by_id) -> Result<Self, Self::Error> {
        Ok(Thread {
            id: row.id,
            title: row.title,
            metadata: row.metadata,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_threads> for Thread {
    type Error = CommonError;
    fn try_from(row: Row_get_threads) -> Result<Self, Self::Error> {
        Ok(Thread {
            id: row.id,
            title: row.title,
            metadata: row.metadata,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// --- Message Conversions ---

impl TryFrom<Row_get_message_by_id> for UIMessage {
    type Error = CommonError;
    fn try_from(row: Row_get_message_by_id) -> Result<Self, Self::Error> {
        Ok(UIMessage {
            id: row.id,
            thread_id: row.thread_id,
            role: row.role,
            parts: parse_parts(&row.parts)?,
            metadata: row.metadata,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_messages> for UIMessage {
    type Error = CommonError;
    fn try_from(row: Row_get_messages) -> Result<Self, Self::Error> {
        Ok(UIMessage {
            id: row.id,
            thread_id: row.thread_id,
            role: row.role,
            parts: parse_parts(&row.parts)?,
            metadata: row.metadata,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_messages_by_thread> for UIMessage {
    type Error = CommonError;
    fn try_from(row: Row_get_messages_by_thread) -> Result<Self, Self::Error> {
        Ok(UIMessage {
            id: row.id,
            thread_id: row.thread_id,
            role: row.role,
            parts: parse_parts(&row.parts)?,
            metadata: row.metadata,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// --- Event Conversions ---

impl TryFrom<Row_get_event_by_id> for StoredEvent {
    type Error = CommonError;
    fn try_from(row: Row_get_event_by_id) -> Result<Self, Self::Error> {
        Ok(StoredEvent {
            id: row.id,
            kind: row.kind,
            payload: row.payload,
            inbox_id: row.inbox_id,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
        })
    }
}

impl TryFrom<Row_get_events> for StoredEvent {
    type Error = CommonError;
    fn try_from(row: Row_get_events) -> Result<Self, Self::Error> {
        Ok(StoredEvent {
            id: row.id,
            kind: row.kind,
            payload: row.payload,
            inbox_id: row.inbox_id,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
        })
    }
}

impl TryFrom<Row_get_events_by_inbox> for StoredEvent {
    type Error = CommonError;
    fn try_from(row: Row_get_events_by_inbox) -> Result<Self, Self::Error> {
        Ok(StoredEvent {
            id: row.id,
            kind: row.kind,
            payload: row.payload,
            inbox_id: row.inbox_id,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
        })
    }
}

impl TryFrom<Row_get_events_by_kind> for StoredEvent {
    type Error = CommonError;
    fn try_from(row: Row_get_events_by_kind) -> Result<Self, Self::Error> {
        Ok(StoredEvent {
            id: row.id,
            kind: row.kind,
            payload: row.payload,
            inbox_id: row.inbox_id,
            inbox_settings: json_to_map(&row.inbox_settings),
            created_at: row.created_at,
        })
    }
}

// --- Inbox Conversions ---

impl TryFrom<Row_get_inbox_by_id> for Inbox {
    type Error = CommonError;
    fn try_from(row: Row_get_inbox_by_id) -> Result<Self, Self::Error> {
        Ok(Inbox {
            id: row.id,
            provider_id: row.provider_id,
            status: row.status,
            configuration: row.configuration,
            settings: json_to_map(&row.settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_inboxes> for Inbox {
    type Error = CommonError;
    fn try_from(row: Row_get_inboxes) -> Result<Self, Self::Error> {
        Ok(Inbox {
            id: row.id,
            provider_id: row.provider_id,
            status: row.status,
            configuration: row.configuration,
            settings: json_to_map(&row.settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_inboxes_by_provider> for Inbox {
    type Error = CommonError;
    fn try_from(row: Row_get_inboxes_by_provider) -> Result<Self, Self::Error> {
        Ok(Inbox {
            id: row.id,
            provider_id: row.provider_id,
            status: row.status,
            configuration: row.configuration,
            settings: json_to_map(&row.settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_enabled_inboxes> for Inbox {
    type Error = CommonError;
    fn try_from(row: Row_get_enabled_inboxes) -> Result<Self, Self::Error> {
        Ok(Inbox {
            id: row.id,
            provider_id: row.provider_id,
            status: row.status,
            configuration: row.configuration,
            settings: json_to_map(&row.settings),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
