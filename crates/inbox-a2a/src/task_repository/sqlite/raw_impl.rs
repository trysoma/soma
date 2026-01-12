use crate::logic::push_notification::PushNotificationConfigModel;
use crate::logic::task::{Metadata, Task, TaskTimelineItem, TaskTimelineItemPayload};
use shared::error::CommonError;

use super::{
    Row_get_push_notification_config_by_id, Row_get_push_notification_configs_by_task_id,
    Row_get_task_by_id, Row_get_task_timeline_items, Row_get_tasks, Row_get_tasks_by_context_id,
};

impl TryFrom<Row_get_tasks> for Task {
    type Error = CommonError;
    fn try_from(row: Row_get_tasks) -> Result<Self, Self::Error> {
        let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;
        Ok(Task {
            id: row.id,
            context_id: row.context_id,
            status: row.status,
            status_timestamp: row.status_timestamp,
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
            metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_task_by_id> for Task {
    type Error = CommonError;
    fn try_from(row: Row_get_task_by_id) -> Result<Self, Self::Error> {
        let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;

        Ok(Task {
            id: row.id,
            context_id: row.context_id,
            status: row.status,
            status_timestamp: row.status_timestamp,
            metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_task_timeline_items> for TaskTimelineItem {
    type Error = CommonError;
    fn try_from(row: Row_get_task_timeline_items) -> Result<Self, Self::Error> {
        let event_payload: TaskTimelineItemPayload =
            serde_json::from_value(row.event_payload.get_inner().clone())?;
        Ok(TaskTimelineItem {
            id: row.id,
            task_id: row.task_id,
            event_payload,
            created_at: row.created_at,
        })
    }
}

impl From<Row_get_push_notification_configs_by_task_id> for PushNotificationConfigModel {
    fn from(row: Row_get_push_notification_configs_by_task_id) -> Self {
        PushNotificationConfigModel {
            id: row.id,
            task_id: row.task_id,
            url: row.url,
            token: row.token,
            authentication: row.authentication.map(|v| v.get_inner().clone()),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_push_notification_config_by_id> for PushNotificationConfigModel {
    fn from(row: Row_get_push_notification_config_by_id) -> Self {
        PushNotificationConfigModel {
            id: row.id,
            task_id: row.task_id,
            url: row.url,
            token: row.token,
            authentication: row.authentication.map(|v| v.get_inner().clone()),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
