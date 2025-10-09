use crate::repository::{Task, TaskEventUpdateType, TaskStatus, TaskTimelineItem};
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};

use super::{Row_get_task_by_id, Row_get_task_timeline_items, Row_get_tasks};

// Task conversions
impl From<Row_get_tasks> for Task {
    fn from(row: Row_get_tasks) -> Self {
        Task {
            id: row.id,
            context_id: row.context_id,
            status: row.status,
            metadata: row.metadata,
            created_at: row
                .created_at,
            updated_at: row.updated_at
        }
    }
}

impl From<Row_get_task_by_id> for Task {
    fn from(row: Row_get_task_by_id) -> Self {
        Task {
            id: row.id,
            context_id: row.context_id,
            status: TaskStatus::from(row.status),
            metadata: row.metadata,
            created_at: row
                .created_at,
            updated_at: row
                .updated_at
        }
    }
}

// TaskTimelineItem conversions
impl From<Row_get_task_timeline_items> for TaskTimelineItem {
    fn from(row: Row_get_task_timeline_items) -> Self {
        TaskTimelineItem {
            id: WrappedUuidV4::try_from(row.id).expect("Invalid UUID in database"),
            task_id: WrappedUuidV4::try_from(row.task_id).expect("Invalid UUID in database"),
            event_update_type: TaskEventUpdateType::from(row.event_update_type),
            event_payload: row.event_payload,
            created_at: row
                .created_at,
        }
    }
}
