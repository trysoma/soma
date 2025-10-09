#![allow(non_camel_case_types)]
mod raw_impl;

include!("raw.generated.rs");

use crate::repository::{
    CreateTask, CreateTaskTimelineItem, Task, TaskRepositoryLike, TaskTimelineItem,
    UpdateTaskStatus,
};
use shared::{
    error::CommonError,
    primitives::{
        decode_pagination_token, PaginatedResponse, PaginationRequest, SqlMigrationLoader,
        WrappedUuidV4,
    },
};
use anyhow::Context;
use std::collections::BTreeMap;

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
    async fn create_task(&self, params: &CreateTask<'_>) -> Result<(), CommonError> {
        let sqlc_params = insert_task_params {
            id: params.id,
            context_id: params.context_id,
            status: params.status,
            metadata: params.metadata,
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

    async fn update_task_status(&self, params: &UpdateTaskStatus<'_>) -> Result<(), CommonError> {
        let sqlc_params = update_task_status_params {
            id: params.id,
            status: params.status,
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
        params: &CreateTaskTimelineItem<'_>,
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
                    msg: format!("Invalid pagination token: {}", e),
                    source: Some(e),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(
                        decoded_parts[0].as_str(),
                    )
                    .map_err(|e| CommonError::Repository {
                        msg: format!("Invalid datetime in pagination token: {}", e),
                        source: Some(e),
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

        let items: Vec<Task> = rows.into_iter().map(Task::from).collect();

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
                    msg: format!("Invalid pagination token: {}", e),
                    source: Some(e),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(
                        decoded_parts[0].as_str(),
                    )
                    .map_err(|e| CommonError::Repository {
                        msg: format!("Invalid datetime in pagination token: {}", e),
                        source: Some(e),
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

        let items: Vec<TaskTimelineItem> = rows.into_iter().map(TaskTimelineItem::from).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_task_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Task>, CommonError> {
        let sqlc_params = get_task_by_id_params { id };

        let row_opt = get_task_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get task by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(row_opt.map(Task::from))
    }
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        let mut all_migrations = BTreeMap::new();
        let mut sqlite_migrations = BTreeMap::new();

        // 0_init migration
        sqlite_migrations.insert("0_init.up.sql", include_str!("../../../migrations/0_init.up.sql"));
        sqlite_migrations.insert("0_init.down.sql", include_str!("../../../migrations/0_init.down.sql"));

        all_migrations.insert("sqlite", sqlite_migrations);

        all_migrations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{
        CreateTask, CreateTaskTimelineItem, TaskEventUpdateType, TaskRepositoryLike, TaskStatus, UpdateTaskStatus
    };
    use shared::primitives::{PaginationRequest, SqlMigrationLoader, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};
    use shared::test_utils::repository::setup_in_memory_database;

    #[tokio::test]
    async fn test_create_and_get_task() {
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = WrappedJsonValue::new(serde_json::json!({"key": "value"}));
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task
        let create_params = CreateTask {
            id: &task_id,
            context_id: &context_id,
            status: &status,
            metadata: &metadata,
            created_at: &created_at,
            updated_at: &updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Get task by ID
        let task = repo.get_task_by_id(&task_id).await.unwrap();
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.context_id, context_id);
        assert_eq!(task.status, TaskStatus::Submitted);
        assert_eq!(task.created_at, created_at);
        assert_eq!(task.updated_at, updated_at);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Submitted;
        let metadata = WrappedJsonValue::new(serde_json::json!({"key": "value"}));
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task with Submitted status
        let create_params = CreateTask {
            id: &task_id,
            context_id: &context_id,
            status: &status,
            metadata: &metadata,
            created_at: &created_at,
            updated_at: &updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Update to Working status
        let new_status = TaskStatus::Working;
        let new_updated_at = WrappedChronoDateTime::now();
        let update_params = UpdateTaskStatus {
            id: &task_id,
            status: &new_status,
            updated_at: &new_updated_at,
        };
        repo.update_task_status(&update_params).await.unwrap();

        // Verify update
        let task = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Working);
        assert_eq!(task.updated_at, new_updated_at);

        // Update to Completed status
        let complete_status = TaskStatus::Completed;
        let complete_updated_at = WrappedChronoDateTime::now();
        let complete_params = UpdateTaskStatus {
            id: &task_id,
            status: &complete_status,
            updated_at: &complete_updated_at,
        };
        repo.update_task_status(&complete_params).await.unwrap();

        // Verify completed status
        let task = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.updated_at, complete_updated_at);
    }

    #[tokio::test]
    async fn test_insert_task_timeline_item() {
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = WrappedJsonValue::new(serde_json::json!({"key": "value"}));
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task
        let create_params = CreateTask {
            id: &task_id,
            context_id: &context_id,
            status: &status,
            metadata: &metadata,
            created_at: &created_at,
            updated_at: &updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Insert timeline item
        let timeline_id = WrappedUuidV4::new();
        let event_type = TaskEventUpdateType::Message;
        let event_payload = WrappedJsonValue::new(serde_json::json!({"message": "Task started"}));
        let timeline_created_at = WrappedChronoDateTime::now();

        let timeline_params = CreateTaskTimelineItem {
            id: &timeline_id,
            task_id: &task_id,
            event_update_type: &event_type,
            event_payload: &event_payload,
            created_at: &timeline_created_at,
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
        assert_eq!(item.event_update_type, TaskEventUpdateType::Message);
        assert_eq!(item.created_at, timeline_created_at);
    }

    #[tokio::test]
    async fn test_get_tasks_pagination() {
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
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
            let metadata = WrappedJsonValue::new(serde_json::json!({"index": i}));
            let created_at = WrappedChronoDateTime::now();
            let updated_at = WrappedChronoDateTime::now();

            let create_params = CreateTask {
                id: &task_id,
                context_id: &context_id,
                status: &status,
                metadata: &metadata,
                created_at: &created_at,
                updated_at: &updated_at,
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
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let status = TaskStatus::Working;
        let metadata = WrappedJsonValue::new(serde_json::json!({"key": "value"}));
        let created_at = WrappedChronoDateTime::now();
        let updated_at = WrappedChronoDateTime::now();

        // Create task
        let create_params = CreateTask {
            id: &task_id,
            context_id: &context_id,
            status: &status,
            metadata: &metadata,
            created_at: &created_at,
            updated_at: &updated_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Create 5 timeline items
        use std::thread::sleep;
        use std::time::Duration;
        for i in 0..5 {
            sleep(Duration::from_millis(10)); // Ensure different timestamps
            let timeline_id = WrappedUuidV4::new();
            let event_type = if i % 2 == 0 {
                TaskEventUpdateType::Message
            } else {
                TaskEventUpdateType::TaskStatusUpdate
            };
            let event_payload =
                WrappedJsonValue::new(serde_json::json!({"message": format!("Event {}", i)}));
            let timeline_created_at = WrappedChronoDateTime::now();

            let timeline_params = CreateTaskTimelineItem {
                id: &timeline_id,
                task_id: &task_id,
                event_update_type: &event_type,
                event_payload: &event_payload,
                created_at: &timeline_created_at,
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
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
        let repo = Repository::new(conn);

        let non_existent_id = WrappedUuidV4::new();
        let task = repo.get_task_by_id(&non_existent_id).await.unwrap();
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_task_status_transitions() {
        let (_db, conn) =
            setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
        let repo = Repository::new(conn);

        let task_id = WrappedUuidV4::new();
        let context_id = WrappedUuidV4::new();
        let metadata = WrappedJsonValue::new(serde_json::json!({}));
        let created_at = WrappedChronoDateTime::now();

        // Test all status transitions
        let statuses = vec![
            TaskStatus::Submitted,
            TaskStatus::Working,
            TaskStatus::InputRequired,
            TaskStatus::Working,
            TaskStatus::Completed,
        ];

        // Create initial task
        let create_params = CreateTask {
            id: &task_id,
            context_id: &context_id,
            status: &statuses[0],
            metadata: &metadata,
            created_at: &created_at,
            updated_at: &created_at,
        };
        repo.create_task(&create_params).await.unwrap();

        // Test status transitions
        for status in &statuses[1..] {
            let updated_at = WrappedChronoDateTime::now();
            let update_params = UpdateTaskStatus {
                id: &task_id,
                status,
                updated_at: &updated_at,
            };
            repo.update_task_status(&update_params).await.unwrap();

            let task = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
            assert_eq!(&task.status, status);
        }
    }
}
