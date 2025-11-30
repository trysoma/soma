use crate::{
    logic::environment_variable::EnvironmentVariable,
    logic::secret::Secret,
    logic::task::{MessagePart, Metadata, TaskTimelineItemPayload, TaskWithDetails},
    repository::{CommonError, Message, Task, TaskTimelineItem},
};
use base64::Engine;
use shared::primitives::WrappedUuidV4;

use super::{
    Row_get_environment_variable_by_id, Row_get_environment_variable_by_key,
    Row_get_environment_variables, Row_get_messages_by_task_id, Row_get_secret_by_id,
    Row_get_secret_by_key, Row_get_secrets, Row_get_task_by_id, Row_get_task_timeline_items,
    Row_get_tasks, Row_get_tasks_by_context_id,
};

// Task conversions
impl TryFrom<Row_get_tasks> for Task {
    type Error = CommonError;
    fn try_from(row: Row_get_tasks) -> Result<Self, Self::Error> {
        let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;
        Ok(Task {
            id: row.id,
            context_id: row.context_id,
            status: row.status,
            status_timestamp: row.status_timestamp,
            status_message_id: row.status_message_id,
            metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_tasks_by_context_id> for Task {
    type Error = CommonError;
    fn try_from(row: Row_get_tasks_by_context_id) -> Result<Self, Self::Error> {
        let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;
        Ok(Task {
            id: row.id,
            context_id: row.context_id,
            status: row.status,
            status_timestamp: row.status_timestamp,
            status_message_id: row.status_message_id,
            metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_task_by_id> for TaskWithDetails {
    type Error = CommonError;
    fn try_from(row: Row_get_task_by_id) -> Result<Self, Self::Error> {
        let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;

        // Parse status_message from JSON string
        let status_message: Option<Message> = if row.status_message == "[]" {
            None
        } else {
            let messages_vec: Vec<Message> = serde_json::from_str(&row.status_message)?;
            messages_vec.into_iter().next()
        };

        // Parse messages from JSON string
        let messages: Vec<Message> = if row.messages == "[]" {
            Vec::new()
        } else {
            serde_json::from_str(&row.messages)?
        };

        let task = Task {
            id: row.id,
            context_id: row.context_id,
            status: row.status,
            status_timestamp: row.status_timestamp,
            status_message_id: row.status_message_id,
            metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        let (messages, messages_next_page_token) = if messages.len() > 100 {
            let last_message = messages.get(100).unwrap();
            let last_message_created_at_string = last_message.created_at.get_inner().to_rfc3339();
            (
                messages[..100].to_vec(),
                Some(
                    base64::engine::general_purpose::STANDARD
                        .encode(last_message_created_at_string.as_bytes()),
                ),
            )
        } else {
            (messages, None)
        };

        Ok(TaskWithDetails {
            task,
            status_message,
            messages,
            messages_next_page_token,
        })
    }
}

// TaskTimelineItem conversions
impl TryFrom<Row_get_task_timeline_items> for TaskTimelineItem {
    type Error = CommonError;
    fn try_from(row: Row_get_task_timeline_items) -> Result<Self, Self::Error> {
        let event_payload: TaskTimelineItemPayload =
            serde_json::from_value(row.event_payload.get_inner().clone())?;
        Ok(TaskTimelineItem {
            id: WrappedUuidV4::try_from(row.id)?,
            task_id: WrappedUuidV4::try_from(row.task_id)?,
            event_payload,
            created_at: row.created_at,
        })
    }
}

// Message conversions
impl TryFrom<Row_get_messages_by_task_id> for Message {
    type Error = CommonError;
    fn try_from(row: Row_get_messages_by_task_id) -> Result<Self, Self::Error> {
        let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;
        let parts: Vec<MessagePart> = serde_json::from_value(row.parts.get_inner().clone())?;
        let reference_task_ids: Vec<WrappedUuidV4> =
            serde_json::from_value(row.reference_task_ids.get_inner().clone())?;
        Ok(Message {
            id: row.id,
            task_id: row.task_id,
            reference_task_ids,
            role: row.role,
            metadata,
            parts,
            created_at: row.created_at,
        })
    }
}

// Secret conversions
impl TryFrom<Row_get_secret_by_id> for Secret {
    type Error = CommonError;
    fn try_from(row: Row_get_secret_by_id) -> Result<Self, Self::Error> {
        Ok(Secret {
            id: row.id,
            key: row.key,
            encrypted_secret: row.encrypted_secret,
            dek_alias: row.dek_alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_secret_by_key> for Secret {
    type Error = CommonError;
    fn try_from(row: Row_get_secret_by_key) -> Result<Self, Self::Error> {
        Ok(Secret {
            id: row.id,
            key: row.key,
            encrypted_secret: row.encrypted_secret,
            dek_alias: row.dek_alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_secrets> for Secret {
    type Error = CommonError;
    fn try_from(row: Row_get_secrets) -> Result<Self, Self::Error> {
        Ok(Secret {
            id: row.id,
            key: row.key,
            encrypted_secret: row.encrypted_secret,
            dek_alias: row.dek_alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// EnvironmentVariable conversions
impl TryFrom<Row_get_environment_variable_by_id> for EnvironmentVariable {
    type Error = CommonError;
    fn try_from(row: Row_get_environment_variable_by_id) -> Result<Self, Self::Error> {
        Ok(EnvironmentVariable {
            id: row.id,
            key: row.key,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_environment_variable_by_key> for EnvironmentVariable {
    type Error = CommonError;
    fn try_from(row: Row_get_environment_variable_by_key) -> Result<Self, Self::Error> {
        Ok(EnvironmentVariable {
            id: row.id,
            key: row.key,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_environment_variables> for EnvironmentVariable {
    type Error = CommonError;
    fn try_from(row: Row_get_environment_variables) -> Result<Self, Self::Error> {
        Ok(EnvironmentVariable {
            id: row.id,
            key: row.key,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
