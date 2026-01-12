//! Row conversion implementations for SQLC-generated types

use serde_json::{Map, Value};
use shared::error::CommonError;

use crate::logic::{
    inbox::Inbox,
    message::{Message, MessageType, TextMessage, TextMessageBody, UIMessage, UIMessageBody, UIMessagePart},
    thread::Thread,
};
use crate::repository::StoredEvent;

use super::{
    Row_get_event_by_id, Row_get_events, Row_get_events_by_inbox, Row_get_events_by_kind,
    Row_get_inbox_by_id, Row_get_inboxes, Row_get_inboxes_by_destination,
    Row_get_inboxes_by_provider, Row_get_message_by_id, Row_get_messages,
    Row_get_messages_by_thread, Row_get_thread_by_id, Row_get_threads,
};

// --- Helper Functions ---

/// Convert WrappedJsonValue to Map<String, Value>
fn json_to_map(json: &shared::primitives::WrappedJsonValue) -> Result<Map<String, Value>, CommonError> {
    match json.get_inner() {
        Value::Object(map) => Ok(map.clone()),
        _ => Err(CommonError::InvalidRequest {
            msg: "Expected JSON object but received different value type".to_string(),
            source: None,
        }),
    }
}

/// Parse UIMessagePart array from body JSON (expects {"parts": [...]})
fn parse_ui_message_body(
    body_json: &shared::primitives::WrappedJsonValue,
) -> Result<Vec<UIMessagePart>, CommonError> {
    let body: UIMessageBody =
        serde_json::from_value(body_json.get_inner().clone()).map_err(|e| CommonError::Repository {
            msg: format!("Failed to parse UI message body: {e}"),
            source: Some(e.into()),
        })?;
    Ok(body.parts)
}

/// Parse text from body JSON (expects {"text": "..."})
fn parse_text_message_body(
    body_json: &shared::primitives::WrappedJsonValue,
) -> Result<String, CommonError> {
    let body: TextMessageBody =
        serde_json::from_value(body_json.get_inner().clone()).map_err(|e| CommonError::Repository {
            msg: format!("Failed to parse text message body: {e}"),
            source: Some(e.into()),
        })?;
    Ok(body.text)
}

/// Helper struct containing common message row fields
struct MessageRowData {
    id: shared::primitives::WrappedUuidV4,
    thread_id: shared::primitives::WrappedUuidV4,
    message_type: crate::logic::message::MessageType,
    role: crate::logic::message::MessageRole,
    body: shared::primitives::WrappedJsonValue,
    metadata: Option<shared::primitives::WrappedJsonValue>,
    inbox_settings: shared::primitives::WrappedJsonValue,
    created_at: shared::primitives::WrappedChronoDateTime,
    updated_at: shared::primitives::WrappedChronoDateTime,
}

/// Convert message row data to Message enum based on message type
fn convert_message_row(row: MessageRowData) -> Result<Message, CommonError> {
    match row.message_type {
        MessageType::Text => {
            let text = parse_text_message_body(&row.body)?;
            Ok(Message::Text(TextMessage {
                id: row.id,
                thread_id: row.thread_id,
                role: row.role,
                text,
                metadata: row.metadata,
                provider_metadata: None,
                inbox_settings: json_to_map(&row.inbox_settings)?,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        }
        MessageType::Ui => {
            let parts = parse_ui_message_body(&row.body)?;
            Ok(Message::Ui(UIMessage {
                id: row.id,
                thread_id: row.thread_id,
                role: row.role,
                parts,
                metadata: row.metadata,
                provider_metadata: None,
                inbox_settings: json_to_map(&row.inbox_settings)?,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        }
    }
}

// --- Thread Conversions ---

impl TryFrom<Row_get_thread_by_id> for Thread {
    type Error = CommonError;
    fn try_from(row: Row_get_thread_by_id) -> Result<Self, Self::Error> {
        Ok(Thread {
            id: row.id,
            title: row.title,
            metadata: row.metadata,
            inbox_settings: json_to_map(&row.inbox_settings)?,
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
            inbox_settings: json_to_map(&row.inbox_settings)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// --- Message Conversions ---

impl TryFrom<Row_get_message_by_id> for Message {
    type Error = CommonError;
    fn try_from(row: Row_get_message_by_id) -> Result<Self, Self::Error> {
        convert_message_row(MessageRowData {
            id: row.id,
            thread_id: row.thread_id,
            message_type: row.kind,
            role: row.role,
            body: row.body,
            metadata: row.metadata,
            inbox_settings: row.inbox_settings,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_messages> for Message {
    type Error = CommonError;
    fn try_from(row: Row_get_messages) -> Result<Self, Self::Error> {
        convert_message_row(MessageRowData {
            id: row.id,
            thread_id: row.thread_id,
            message_type: row.kind,
            role: row.role,
            body: row.body,
            metadata: row.metadata,
            inbox_settings: row.inbox_settings,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_messages_by_thread> for Message {
    type Error = CommonError;
    fn try_from(row: Row_get_messages_by_thread) -> Result<Self, Self::Error> {
        convert_message_row(MessageRowData {
            id: row.id,
            thread_id: row.thread_id,
            message_type: row.kind,
            role: row.role,
            body: row.body,
            metadata: row.metadata,
            inbox_settings: row.inbox_settings,
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
            inbox_settings: json_to_map(&row.inbox_settings)?,
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
            inbox_settings: json_to_map(&row.inbox_settings)?,
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
            inbox_settings: json_to_map(&row.inbox_settings)?,
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
            inbox_settings: json_to_map(&row.inbox_settings)?,
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
            destination_type: row.destination_type,
            destination_id: row.destination_id,
            configuration: row.configuration,
            settings: json_to_map(&row.settings)?,
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
            destination_type: row.destination_type,
            destination_id: row.destination_id,
            configuration: row.configuration,
            settings: json_to_map(&row.settings)?,
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
            destination_type: row.destination_type,
            destination_id: row.destination_id,
            configuration: row.configuration,
            settings: json_to_map(&row.settings)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_inboxes_by_destination> for Inbox {
    type Error = CommonError;
    fn try_from(row: Row_get_inboxes_by_destination) -> Result<Self, Self::Error> {
        Ok(Inbox {
            id: row.id,
            provider_id: row.provider_id,
            destination_type: row.destination_type,
            destination_id: row.destination_id,
            configuration: row.configuration,
            settings: json_to_map(&row.settings)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
