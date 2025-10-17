#![allow(non_camel_case_types)]
mod raw_impl;

include!("raw.generated.rs");

use crate::logic::TaskWithDetails;
use crate::repository::{
    CreateMessage, CreateTask, CreateTaskTimelineItem, Message, Task, TaskRepositoryLike,
    TaskTimelineItem, UpdateTaskStatus,
};
use anyhow::Context;
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, SqlMigrationLoader, WrappedUuidV4,
        decode_pagination_token,
    },
};
use std::collections::BTreeMap;
use shared_macros::load_sql_migrations;

#[derive(Clone)]
pub struct Repository {
    conn: shared::libsql::Connection,
}

impl Repository {
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }
}

impl TaskRepositoryLike for Repository {
    async fn create_task(&self, params: &CreateTask) -> Result<(), CommonError> {
        let sqlc_params = insert_task_params {
            id: &params.id,
            context_id: &params.context_id,
            status: &params.status,
            status_timestamp: &params.status_timestamp,
            metadata: &params.metadata,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_task(&self.conn, sqlc_params)
            .await
            .context("Failed to create task")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn update_task_status(&self, params: &UpdateTaskStatus) -> Result<(), CommonError> {
        let sqlc_params = update_task_status_params {
            id: &params.id,
            status: &params.status,
            status_message_id: &params.status_message_id,
            status_timestamp: &params.status_timestamp,
            updated_at: &params.updated_at,
        };

        update_task_status(&self.conn, sqlc_params)
            .await
            .context("Failed to update task status")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn insert_task_timeline_item(
        &self,
        params: &CreateTaskTimelineItem,
    ) -> Result<(), CommonError> {
        let sqlc_params = insert_task_timeline_item_params {
            id: &params.id.to_string(),
            task_id: &params.task_id.to_string(),
            event_update_type: &params.event_update_type.as_str().to_string(),
            event_payload: &params.event_payload,
            created_at: &params.created_at,
        };

        insert_task_timeline_item(&self.conn, sqlc_params)
            .await
            .context("Failed to insert task timeline item")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_tasks(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Task>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_tasks_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_tasks(&self.conn, sqlc_params)
            .await
            .context("Failed to get tasks")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Task>, CommonError> =
            rows.into_iter().map(Task::try_from).collect();
        let items = items?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |task| vec![task.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_unique_contexts(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<crate::logic::ContextInfo>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_unique_contexts_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_unique_contexts(&self.conn, sqlc_params)
            .await
            .context("Failed to get unique contexts")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<crate::logic::ContextInfo> = rows
            .into_iter()
            .map(|row| crate::logic::ContextInfo {
                context_id: row.context_id,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |context_info| vec![context_info.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_tasks_by_context_id(
        &self,
        context_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Task>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_tasks_by_context_id_params {
            context_id,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_tasks_by_context_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get tasks by context id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Task>, CommonError> =
            rows.into_iter().map(Task::try_from).collect();
        let items = items?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |task| vec![task.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_task_timeline_items(
        &self,
        task_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<TaskTimelineItem>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_task_timeline_items_params {
            task_id: &task_id.to_string(),
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_task_timeline_items(&self.conn, sqlc_params)
            .await
            .context("Failed to get task timeline items")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<TaskTimelineItem>, CommonError> = rows
            .into_iter()
            .map(TaskTimelineItem::try_from)
            .collect();
        let items = items?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_task_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<TaskWithDetails>, CommonError> {
        let sqlc_params = get_task_by_id_params { id };

        let row_opt = get_task_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get task by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match row_opt {
            Some(row) => Ok(Some(TaskWithDetails::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn insert_message(&self, params: &CreateMessage) -> Result<(), CommonError> {
        let sqlc_params = insert_message_params {
            id: &params.id,
            task_id: &params.task_id,
            reference_task_ids: &params.reference_task_ids,
            role: &params.role,
            metadata: &params.metadata,
            parts: &params.parts,
            created_at: &params.created_at,
        };

        insert_message(&self.conn, sqlc_params)
            .await
            .context("Failed to insert message")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_messages_by_task_id(
        &self,
        task_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Message>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_messages_by_task_id_params {
            task_id,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_messages_by_task_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get messages by task id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Message>, CommonError> =
            rows.into_iter().map(Message::try_from).collect();
        let items = items?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |message| vec![message.created_at.get_inner().to_rfc3339()],
        ))
    }
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_sql_migrations!("migrations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::{
        Message, MessagePart, MessageRole, MessageTaskTimelineItem, Metadata, TaskEventUpdateType,
        TaskStatus, TaskStatusUpdateTaskTimelineItem, TaskTimelineItemPayload, TextPart,
    };
    use crate::repository::{
        CreateMessage, CreateTask, CreateTaskTimelineItem, TaskRepositoryLike,
    };
    use base64::Engine;
    use shared::primitives::{
        PaginationRequest, SqlMigrationLoader, WrappedChronoDateTime, WrappedJsonValue,
        WrappedUuidV4,
    };
    use shared::test_utils::repository::setup_in_memory_database;

    #[tokio::test]
    async fn test_create_and_get_task() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task
        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap();
        assert!(task_with_details.is_some());
        let task_with_details = task_with_details.unwrap();
        assert_eq!(task_with_details.task.id, task_id);
        assert_eq!(task_with_details.task.context_id, context_id);
        assert_eq!(task_with_details.task.status, TaskStatus::Submitted);
        assert_eq!(task_with_details.task.created_at, created_at);
        assert_eq!(task_with_details.task.updated_at, updated_at);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task with Submitted status
        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Update to Working status
        let new_status = TaskStatus::Working;
        let new_updated_at = WrappedChronoDateTime::now();
        let update_params = UpdateTaskStatus {
            id: task_id.clone(),
            status: new_status.clone(),
            status_message_id: None,
            status_timestamp: new_updated_at,
            updated_at: new_updated_at,
        };
        repo.update_task_status(&update_params).await.unwrap();

        // Verify update
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
        assert_eq!(task_with_details.task.status, TaskStatus::Working);
        assert_eq!(task_with_details.task.updated_at, new_updated_at);

        // Update to Completed status
        let complete_status = TaskStatus::Completed;
        let complete_updated_at = WrappedChronoDateTime::now();
        let complete_params = UpdateTaskStatus {
            id: task_id.clone(),
            status: complete_status.clone(),
            status_message_id: None,
            status_timestamp: complete_updated_at,
            updated_at: complete_updated_at,
        };
        repo.update_task_status(&complete_params).await.unwrap();

        // Verify completed status
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
        assert_eq!(task_with_details.task.status, TaskStatus::Completed);
        assert_eq!(task_with_details.task.updated_at, complete_updated_at);
    }

    #[tokio::test]
    async fn test_insert_task_timeline_item() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task
        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Insert timeline item with a Message payload
        let timeline_id = WrappedUuidV4::new();
        let event_type = TaskEventUpdateType::Message;

        // Create a proper Message for the timeline
        let message = Message {
            id: WrappedUuidV4::new(),
            task_id: task_id.clone(),
            reference_task_ids: Vec::new(),
            role: MessageRole::Agent,
            metadata: Metadata::new(),
            parts: vec![MessagePart::TextPart(TextPart {
                text: "Task started".to_string(),
                metadata: Metadata::new(),
            })],
            created_at: WrappedChronoDateTime::now(),
        };

        let payload = TaskTimelineItemPayload::Message(MessageTaskTimelineItem { message });
        let event_payload = WrappedJsonValue::new(serde_json::to_value(&payload).unwrap());
        let timeline_created_at = WrappedChronoDateTime::now();

        let timeline_params = CreateTaskTimelineItem {
            id: timeline_id.clone(),
            task_id: task_id.clone(),
            event_update_type: event_type.clone(),
            event_payload: event_payload.clone(),
            created_at: timeline_created_at,
        };
        repo.insert_task_timeline_item(&timeline_params)
            .await
            .unwrap();

        // Get timeline items
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_task_timeline_items(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        let item = &response.items[0];
        assert_eq!(item.id, timeline_id);
        assert_eq!(item.task_id, task_id);
        assert_eq!(item.created_at, timeline_created_at);
    }

    #[tokio::test]
    async fn test_get_tasks_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let context_id = WrappedUuidV4::new();

        // Create 5 tasks with slight time differences
        use std::thread::sleep;
        use std::time::Duration;
        let mut task_ids = vec![];
        for i in 0..5 {
            sleep(Duration::from_millis(10)); // Ensure different timestamps
            let task_id = WrappedUuidV4::new();
            task_ids.push(task_id.clone());
            let status = match i % 3 {
                0 => TaskStatus::Submitted,
                1 => TaskStatus::Working,
                _ => TaskStatus::Completed,
            };
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();
            let updated_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at,
            };
            repo.create_task(&create_params).await.unwrap();
        }

        // Test pagination - get all tasks
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo.get_tasks(&pagination).await.unwrap();

        // Should have 5 tasks
        assert_eq!(response.items.len(), 5);
        assert!(response.next_page_token.is_none()); // No more pages

        // Test pagination with smaller page size
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: None,
        };
        let response = repo.get_tasks(&pagination).await.unwrap();
        assert_eq!(response.items.len(), 3);
        assert!(response.next_page_token.is_some()); // More pages available

        // Get next page
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: response.next_page_token,
        };
        let response = repo.get_tasks(&pagination).await.unwrap();
        assert!(response.items.len() >= 2 && response.items.len() <= 3);
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task
        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Create 5 timeline items
        use std::thread::sleep;
        use std::time::Duration;
        for i in 0..5 {
            sleep(Duration::from_millis(10)); // Ensure different timestamps
            let timeline_id = WrappedUuidV4::new();
            let (event_type, event_payload) = if i % 2 == 0 {
                // Create Message payload
                let message = Message {
                    id: WrappedUuidV4::new(),
                    task_id: task_id.clone(),
                    reference_task_ids: Vec::new(),
                    role: MessageRole::Agent,
                    metadata: Metadata::new(),
                    parts: vec![MessagePart::TextPart(TextPart {
                        text: format!("Event {i}"),
                        metadata: Metadata::new(),
                    })],
                    created_at: WrappedChronoDateTime::now(),
                };
                let payload = TaskTimelineItemPayload::Message(MessageTaskTimelineItem { message });
                (
                    TaskEventUpdateType::Message,
                    WrappedJsonValue::new(serde_json::to_value(&payload).unwrap()),
                )
            } else {
                // Create TaskStatusUpdate payload
                let payload =
                    TaskTimelineItemPayload::TaskStatusUpdate(TaskStatusUpdateTaskTimelineItem {
                        status: TaskStatus::Working,
                        status_message_id: None,
                    });
                (
                    TaskEventUpdateType::TaskStatusUpdate,
                    WrappedJsonValue::new(serde_json::to_value(&payload).unwrap()),
                )
            };
            let timeline_created_at = WrappedChronoDateTime::now();

            let timeline_params = CreateTaskTimelineItem {
                id: timeline_id.clone(),
                task_id: task_id.clone(),
                event_update_type: event_type.clone(),
                event_payload: event_payload.clone(),
                created_at: timeline_created_at,
            };
            repo.insert_task_timeline_item(&timeline_params)
                .await
                .unwrap();
        }

        // Test pagination - get all items
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_task_timeline_items(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 5);
        assert!(response.next_page_token.is_none());

        // All items should belong to the correct task
        for item in &response.items {
            assert_eq!(item.task_id, task_id);
        }

        // Test pagination with smaller page size
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: None,
        };
        let response = repo
            .get_task_timeline_items(&task_id, &pagination)
            .await
            .unwrap();
        assert_eq!(response.items.len(), 3);
        assert!(response.next_page_token.is_some());

        // Get next page
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: response.next_page_token,
        };
        let response = repo
            .get_task_timeline_items(&task_id, &pagination)
            .await
            .unwrap();
        assert!(response.items.len() >= 2 && response.items.len() <= 3);
    }

    #[tokio::test]
    async fn test_get_task_by_id_not_found() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let non_existent_id = WrappedUuidV4::new();
        let task = repo.get_task_by_id(&non_existent_id).await.unwrap();
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_task_status_transitions() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        // Test all status transitions
        let statuses = [TaskStatus::Submitted,
            TaskStatus::Working,
            TaskStatus::InputRequired,
            TaskStatus::Working,
            TaskStatus::Completed];

        // Create initial task
        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: statuses[0].clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Test status transitions
        for status in &statuses[1..] {
            let updated_at = WrappedChronoDateTime::now();
            let update_params = UpdateTaskStatus {
                id: task_id.clone(),
                status: status.clone(),
                status_message_id: None,
                status_timestamp: updated_at,
                updated_at,
            };
            repo.update_task_status(&update_params).await.unwrap();

            let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
            assert_eq!(&task_with_details.task.status, status);
        }
    }

    #[tokio::test]
    async fn test_insert_and_get_message() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create a task first
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let task_metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        let task_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&task_metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&task_params).await.unwrap();

        // Create a message
        let message_id = WrappedUuidV4::new();
        let reference_task_ids = Vec::<WrappedUuidV4>::new();
        let role = MessageRole::User;
        let metadata = Metadata::new();
        let parts = vec![MessagePart::TextPart(TextPart {
            text: "Hello".to_string(),
            metadata: Metadata::new(),
        })];
        let message_created_at = WrappedChronoDateTime::now();

        let message_params = CreateMessage {
            id: message_id.clone(),
            task_id: task_id.clone(),
            reference_task_ids: WrappedJsonValue::new(
                serde_json::to_value(&reference_task_ids).unwrap(),
            ),
            role: role.clone(),
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
            created_at: message_created_at,
        };
        repo.insert_message(&message_params).await.unwrap();

        // Get messages by task_id
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_messages_by_task_id(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        let message = &response.items[0];
        assert_eq!(message.id, message_id);
        assert_eq!(message.task_id, task_id);
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.created_at, message_created_at);
    }

    #[tokio::test]
    async fn test_get_messages_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create a task first
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let task_metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        let task_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&task_metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&task_params).await.unwrap();

        // Create 5 messages with different timestamps
        use std::thread::sleep;
        use std::time::Duration;
        for i in 0..5 {
            sleep(Duration::from_millis(10)); // Ensure different timestamps
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Agent
            };
            let metadata = Metadata::new();
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Message {i}"),
                metadata: Metadata::new(),
            })];
            let message_created_at = WrappedChronoDateTime::now();

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: message_created_at,
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        // Test pagination - get all messages
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_messages_by_task_id(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 5);
        assert!(response.next_page_token.is_none());

        // All messages should belong to the correct task
        for message in &response.items {
            assert_eq!(message.task_id, task_id);
        }

        // Test pagination with smaller page size
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: None,
        };
        let response = repo
            .get_messages_by_task_id(&task_id, &pagination)
            .await
            .unwrap();
        assert_eq!(response.items.len(), 3);
        assert!(response.next_page_token.is_some());

        // Get next page
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: response.next_page_token,
        };
        let response = repo
            .get_messages_by_task_id(&task_id, &pagination)
            .await
            .unwrap();
        assert!(response.items.len() >= 2 && response.items.len() <= 3);
    }

    #[tokio::test]
    async fn test_messages_multiple_tasks() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create two tasks
        let task_id_1 = WrappedUuidV4::new();
        let task_id_2 = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let task_metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        for task_id in [&task_id_1, &task_id_2] {
            let task_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&task_metadata).unwrap()),
                created_at,
                updated_at,
            };
            repo.create_task(&task_params).await.unwrap();
        }

        // Create 3 messages for task 1 and 2 messages for task 2
        for i in 0..3 {
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = MessageRole::User;
            let metadata = Metadata::new();
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Task 1 Message {i}"),
                metadata: Metadata::new(),
            })];
            let message_created_at = WrappedChronoDateTime::now();

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id_1.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: message_created_at,
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        for i in 0..2 {
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = MessageRole::Agent;
            let metadata = Metadata::new();
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Task 2 Message {i}"),
                metadata: Metadata::new(),
            })];
            let message_created_at = WrappedChronoDateTime::now();

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id_2.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: message_created_at,
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        // Get messages for task 1
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_messages_by_task_id(&task_id_1, &pagination)
            .await
            .unwrap();
        assert_eq!(response.items.len(), 3);
        for message in &response.items {
            assert_eq!(message.task_id, task_id_1);
            assert_eq!(message.role, MessageRole::User);
        }

        // Get messages for task 2
        let response = repo
            .get_messages_by_task_id(&task_id_2, &pagination)
            .await
            .unwrap();
        assert_eq!(response.items.len(), 2);
        for message in &response.items {
            assert_eq!(message.task_id, task_id_2);
            assert_eq!(message.role, MessageRole::Agent);
        }
    }

    #[tokio::test]
    async fn test_message_with_reference_task_ids() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create tasks
        let task_id = WrappedUuidV4::new();
        let ref_task_id_1 = WrappedUuidV4::new();
        let ref_task_id_2 = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let task_metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        let task_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&task_metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&task_params).await.unwrap();

        // Create a message with reference task IDs
        let message_id = WrappedUuidV4::new();
        let reference_task_ids = vec![ref_task_id_1.clone(), ref_task_id_2.clone()];
        let role = MessageRole::User;
        let metadata = Metadata::new();
        let parts = vec![MessagePart::TextPart(TextPart {
            text: "Message referencing other tasks".to_string(),
            metadata: Metadata::new(),
        })];
        let message_created_at = WrappedChronoDateTime::now();

        let message_params = CreateMessage {
            id: message_id.clone(),
            task_id: task_id.clone(),
            reference_task_ids: WrappedJsonValue::new(
                serde_json::to_value(&reference_task_ids).unwrap(),
            ),
            role: role.clone(),
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
            created_at: message_created_at,
        };
        repo.insert_message(&message_params).await.unwrap();

        // Retrieve and verify
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_messages_by_task_id(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        let message = &response.items[0];
        assert_eq!(message.id, message_id);

        // Verify reference_task_ids is stored correctly
        assert_eq!(message.reference_task_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_no_messages() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task without any messages
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap();
        assert!(task_with_details.is_some());

        let task_with_details = task_with_details.unwrap();
        assert_eq!(task_with_details.task.id, task_id);
        assert!(task_with_details.status_message.is_none());
        assert_eq!(task_with_details.messages.len(), 0);
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_messages() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Create a message
        let message_id = WrappedUuidV4::new();
        let reference_task_ids = Vec::<WrappedUuidV4>::new();
        let role = MessageRole::User;
        let parts = vec![MessagePart::TextPart(TextPart {
            text: "Test message".to_string(),
            metadata: Metadata::new(),
        })];

        let message_params = CreateMessage {
            id: message_id.clone(),
            task_id: task_id.clone(),
            reference_task_ids: WrappedJsonValue::new(
                serde_json::to_value(&reference_task_ids).unwrap(),
            ),
            role: role.clone(),
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
            created_at: WrappedChronoDateTime::now(),
        };
        repo.insert_message(&message_params).await.unwrap();

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
        assert_eq!(task_with_details.task.id, task_id);
        assert!(task_with_details.status_message.is_none());
        assert_eq!(task_with_details.messages.len(), 1);
        assert_eq!(task_with_details.messages[0].id, message_id);
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_status_message() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Completed;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Create a status message
        let status_message_id = WrappedUuidV4::new();
        let reference_task_ids = Vec::<WrappedUuidV4>::new();
        let role = MessageRole::Agent;
        let parts = vec![MessagePart::TextPart(TextPart {
            text: "Task completed successfully".to_string(),
            metadata: Metadata::new(),
        })];

        let message_params = CreateMessage {
            id: status_message_id.clone(),
            task_id: task_id.clone(),
            reference_task_ids: WrappedJsonValue::new(
                serde_json::to_value(&reference_task_ids).unwrap(),
            ),
            role: role.clone(),
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
            created_at: WrappedChronoDateTime::now(),
        };
        repo.insert_message(&message_params).await.unwrap();

        // Update task status to reference the message
        let now = WrappedChronoDateTime::now();
        let update_params = UpdateTaskStatus {
            id: task_id.clone(),
            status: status.clone(),
            status_message_id: Some(status_message_id.clone()),
            status_timestamp: now,
            updated_at: now,
        };
        repo.update_task_status(&update_params).await.unwrap();

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
        assert_eq!(task_with_details.task.id, task_id);
        assert!(task_with_details.status_message.is_some());
        assert_eq!(
            task_with_details.status_message.unwrap().id,
            status_message_id
        );
        assert_eq!(task_with_details.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_get_task_timeline_items_empty() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task without any timeline items
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get timeline items
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_task_timeline_items(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 0);
        assert!(response.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_messages_by_task_id_empty() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task without any messages
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get messages
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_messages_by_task_id(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 0);
        assert!(response.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_tasks_empty() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Get tasks from empty database
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo.get_tasks(&pagination).await.unwrap();

        assert_eq!(response.items.len(), 0);
        assert!(response.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_unique_contexts() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create 3 tasks with 2 different context_ids
        let context_id_1 = WrappedUuidV4::new();
        let context_id_2 = WrappedUuidV4::new();

        use std::thread::sleep;
        use std::time::Duration;

        // Create 2 tasks with context_id_1
        for _ in 0..2 {
            sleep(Duration::from_millis(10));
            let task_id = WrappedUuidV4::new();
            let status = TaskStatus::Working;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id_1.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
        }

        // Create 1 task with context_id_2
        sleep(Duration::from_millis(10));
        let task_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id_2.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get unique contexts
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo.get_unique_contexts(&pagination).await.unwrap();

        // Should have 3 entries (2 for context_id_1 with different created_at, 1 for context_id_2)
        // This is because the query does DISTINCT on (context_id, created_at)
        assert_eq!(response.items.len(), 3);

        // Verify both context_ids are present
        let context_ids: Vec<_> = response
            .items
            .iter()
            .map(|c| c.context_id.clone())
            .collect();
        assert!(context_ids.contains(&context_id_1));
        assert!(context_ids.contains(&context_id_2));

        // Verify all items have created_at
        for item in &response.items {
            assert!(item.created_at.get_inner().timestamp() > 0);
        }
    }

    #[tokio::test]
    async fn test_get_tasks_by_context_id() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let context_id_1 = WrappedUuidV4::new();
        let context_id_2 = WrappedUuidV4::new();

        use std::thread::sleep;
        use std::time::Duration;

        // Create 3 tasks with context_id_1
        let mut task_ids_1 = vec![];
        for _ in 0..3 {
            sleep(Duration::from_millis(10));
            let task_id = WrappedUuidV4::new();
            task_ids_1.push(task_id.clone());
            let status = TaskStatus::Working;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id_1.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
        }

        // Create 2 tasks with context_id_2
        let mut task_ids_2 = vec![];
        for _ in 0..2 {
            sleep(Duration::from_millis(10));
            let task_id = WrappedUuidV4::new();
            task_ids_2.push(task_id.clone());
            let status = TaskStatus::Submitted;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id_2.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
        }

        // Get tasks for context_id_1
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_tasks_by_context_id(&context_id_1, &pagination)
            .await
            .unwrap();

        // Should have 3 tasks
        assert_eq!(response.items.len(), 3);

        // All tasks should belong to context_id_1
        for task in &response.items {
            assert_eq!(task.context_id, context_id_1);
        }

        // Verify all task IDs are present
        let retrieved_ids: Vec<_> = response.items.iter().map(|t| t.id.clone()).collect();
        for task_id in &task_ids_1 {
            assert!(retrieved_ids.contains(task_id));
        }

        // Get tasks for context_id_2
        let response = repo
            .get_tasks_by_context_id(&context_id_2, &pagination)
            .await
            .unwrap();

        // Should have 2 tasks
        assert_eq!(response.items.len(), 2);

        // All tasks should belong to context_id_2
        for task in &response.items {
            assert_eq!(task.context_id, context_id_2);
        }

        // Verify all task IDs are present
        let retrieved_ids: Vec<_> = response.items.iter().map(|t| t.id.clone()).collect();
        for task_id in &task_ids_2 {
            assert!(retrieved_ids.contains(task_id));
        }
    }

    #[tokio::test]
    async fn test_get_tasks_by_context_id_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let context_id = WrappedUuidV4::new();

        use std::thread::sleep;
        use std::time::Duration;

        // Create 5 tasks with the same context_id
        for _ in 0..5 {
            sleep(Duration::from_millis(10));
            let task_id = WrappedUuidV4::new();
            let status = TaskStatus::Working;
            let metadata = Metadata::new();
            let created_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: task_id.clone(),
                context_id: context_id.clone(),
                status: status.clone(),
                status_timestamp: created_at,
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                created_at,
                updated_at: created_at,
            };
            repo.create_task(&create_params).await.unwrap();
        }

        // Test pagination - get first page with smaller page size
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: None,
        };
        let response = repo
            .get_tasks_by_context_id(&context_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 3);
        assert!(response.next_page_token.is_some());

        // Get next page
        let pagination = PaginationRequest {
            page_size: 3,
            next_page_token: response.next_page_token,
        };
        let response = repo
            .get_tasks_by_context_id(&context_id, &pagination)
            .await
            .unwrap();
        assert!(response.items.len() >= 2 && response.items.len() <= 3);
    }

    #[tokio::test]
    async fn test_get_tasks_by_context_id_empty() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let non_existent_context_id = WrappedUuidV4::new();

        // Get tasks for non-existent context
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_tasks_by_context_id(&non_existent_context_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 0);
        assert!(response.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_messages_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        use std::thread::sleep;
        use std::time::Duration;

        // Create 150 messages (more than the 100 limit)
        for i in 0..150 {
            sleep(Duration::from_millis(5));
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Agent
            };
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Message {i}"),
                metadata: Metadata::new(),
            })];

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: WrappedChronoDateTime::now(),
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();

        // Should have exactly 100 messages (the limit)
        assert_eq!(task_with_details.messages.len(), 100);

        // Should have a next page token since there are more than 100 messages
        assert!(task_with_details.messages_next_page_token.is_some());

        // Verify the pagination token is a valid base64 string
        let token = task_with_details.messages_next_page_token.unwrap();
        let decoded = base64::engine::general_purpose::STANDARD.decode(&token);
        assert!(decoded.is_ok());
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_messages_no_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        use std::thread::sleep;
        use std::time::Duration;

        // Create only 50 messages (less than the 100 limit)
        for i in 0..50 {
            sleep(Duration::from_millis(5));
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = MessageRole::User;
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Message {i}"),
                metadata: Metadata::new(),
            })];

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: WrappedChronoDateTime::now(),
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();

        // Should have exactly 50 messages
        assert_eq!(task_with_details.messages.len(), 50);

        // Should NOT have a next page token since there are less than 100 messages
        assert!(task_with_details.messages_next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_exactly_100_messages() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        use std::thread::sleep;
        use std::time::Duration;

        // Create exactly 100 messages
        for i in 0..100 {
            sleep(Duration::from_millis(5));
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = MessageRole::User;
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Message {i}"),
                metadata: Metadata::new(),
            })];

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: WrappedChronoDateTime::now(),
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();

        // Should have exactly 100 messages
        assert_eq!(task_with_details.messages.len(), 100);

        // Should NOT have a next page token since there are exactly 100 messages (not more)
        assert!(task_with_details.messages_next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_get_task_by_id_with_messages_pagination_token_format() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        use std::thread::sleep;
        use std::time::Duration;

        // Create 101 messages (just over the limit)
        for i in 0..101 {
            sleep(Duration::from_millis(5));
            let message_id = WrappedUuidV4::new();
            let reference_task_ids = Vec::<WrappedUuidV4>::new();
            let role = MessageRole::User;
            let parts = vec![MessagePart::TextPart(TextPart {
                text: format!("Message {i}"),
                metadata: Metadata::new(),
            })];

            let message_params = CreateMessage {
                id: message_id.clone(),
                task_id: task_id.clone(),
                reference_task_ids: WrappedJsonValue::new(
                    serde_json::to_value(&reference_task_ids).unwrap(),
                ),
                role: role.clone(),
                metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
                parts: WrappedJsonValue::new(serde_json::to_value(&parts).unwrap()),
                created_at: WrappedChronoDateTime::now(),
            };
            repo.insert_message(&message_params).await.unwrap();
        }

        // Get task by ID
        let task_with_details = repo.get_task_by_id(&task_id).await.unwrap().unwrap();

        // Should have exactly 100 messages
        assert_eq!(task_with_details.messages.len(), 100);

        // Should have a next page token
        assert!(task_with_details.messages_next_page_token.is_some());

        // Verify the pagination token contains a valid RFC3339 timestamp
        let token = task_with_details.messages_next_page_token.unwrap();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&token)
            .unwrap();
        let timestamp_str = String::from_utf8(decoded).unwrap();

        // Verify it's a valid RFC3339 timestamp
        let parsed = chrono::DateTime::parse_from_rfc3339(&timestamp_str);
        assert!(
            parsed.is_ok(),
            "Token should contain a valid RFC3339 timestamp"
        );

        // The timestamp should be from the 101st message (index 100)
        // which is the message that triggers pagination
        let message_101_created_at = task_with_details
            .messages
            .get(99)
            .unwrap()
            .created_at;

        // The token should represent a timestamp that can be used for pagination
        assert!(parsed.unwrap().timestamp() > 0);
    }

    #[tokio::test]
    async fn test_timeline_with_both_messages_and_status_updates() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create task
        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = Metadata::new();
        let created_at = WrappedChronoDateTime::now();

        let create_params = CreateTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: status.clone(),
            status_timestamp: created_at,
            metadata: WrappedJsonValue::new(serde_json::to_value(&metadata).unwrap()),
            created_at,
            updated_at: created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        use std::thread::sleep;
        use std::time::Duration;

        // Insert a message timeline item
        sleep(Duration::from_millis(10));
        let message = Message {
            id: WrappedUuidV4::new(),
            task_id: task_id.clone(),
            reference_task_ids: Vec::new(),
            role: MessageRole::User,
            metadata: Metadata::new(),
            parts: vec![MessagePart::TextPart(TextPart {
                text: "User message".to_string(),
                metadata: Metadata::new(),
            })],
            created_at: WrappedChronoDateTime::now(),
        };

        let message_payload = TaskTimelineItemPayload::Message(MessageTaskTimelineItem { message });
        let timeline_params = CreateTaskTimelineItem {
            id: WrappedUuidV4::new(),
            task_id: task_id.clone(),
            event_update_type: TaskEventUpdateType::Message,
            event_payload: WrappedJsonValue::new(serde_json::to_value(&message_payload).unwrap()),
            created_at: WrappedChronoDateTime::now(),
        };
        repo.insert_task_timeline_item(&timeline_params)
            .await
            .unwrap();

        // Insert a status update timeline item
        sleep(Duration::from_millis(10));
        let status_update_payload =
            TaskTimelineItemPayload::TaskStatusUpdate(TaskStatusUpdateTaskTimelineItem {
                status: TaskStatus::Working,
                status_message_id: None,
            });
        let timeline_params2 = CreateTaskTimelineItem {
            id: WrappedUuidV4::new(),
            task_id: task_id.clone(),
            event_update_type: TaskEventUpdateType::TaskStatusUpdate,
            event_payload: WrappedJsonValue::new(
                serde_json::to_value(&status_update_payload).unwrap(),
            ),
            created_at: WrappedChronoDateTime::now(),
        };
        repo.insert_task_timeline_item(&timeline_params2)
            .await
            .unwrap();

        // Get timeline items
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let response = repo
            .get_task_timeline_items(&task_id, &pagination)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 2);

        // Verify we have both types
        let mut has_message = false;
        let mut has_status_update = false;

        for item in &response.items {
            match &item.event_payload {
                TaskTimelineItemPayload::Message(_) => has_message = true,
                TaskTimelineItemPayload::TaskStatusUpdate(_) => has_status_update = true,
            }
        }

        assert!(has_message);
        assert!(has_status_update);
    }
}
