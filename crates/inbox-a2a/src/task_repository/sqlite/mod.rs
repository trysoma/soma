#![allow(non_camel_case_types)]
mod raw_impl;

#[allow(clippy::all)]
#[allow(dead_code)]
mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::logic::push_notification::{
    CreatePushNotificationConfig, PushNotificationConfigModel, UpdatePushNotificationConfig,
};
use crate::logic::task::{ContextInfo, Task, TaskTimelineItem};
use crate::task_repository::{
    CreateTask, CreateTaskTimelineItem, TaskRepositoryLike, UpdateTaskStatus,
};
use anyhow::Context;
use shared::{
    error::CommonError,
    primitives::{
        decode_pagination_token, PaginatedResponse, PaginationRequest, SqlMigrationLoader,
        WrappedUuidV4,
    },
};
use shared_macros::load_atlas_sql_migrations;
use std::collections::BTreeMap;

/// SQLite repository for A2A task operations
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
            id: &params.id,
            task_id: &params.task_id,
            event_update_type: &params.event_update_type,
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

        let items: Result<Vec<Task>, CommonError> = rows.into_iter().map(Task::try_from).collect();
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
    ) -> Result<PaginatedResponse<ContextInfo>, CommonError> {
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

        let items: Vec<ContextInfo> = rows
            .into_iter()
            .map(|row| ContextInfo {
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

        let items: Result<Vec<Task>, CommonError> = rows.into_iter().map(Task::try_from).collect();
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
            task_id,
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

        let items: Result<Vec<TaskTimelineItem>, CommonError> =
            rows.into_iter().map(TaskTimelineItem::try_from).collect();
        let items = items?;

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

        match row_opt {
            Some(row) => Ok(Some(Task::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn create_push_notification_config(
        &self,
        params: &CreatePushNotificationConfig,
    ) -> Result<(), CommonError> {
        let sqlc_params = insert_push_notification_config_params {
            id: &params.id,
            task_id: &params.task_id,
            url: &params.url,
            token: &params.token,
            authentication: &params.authentication,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_push_notification_config(&self.conn, sqlc_params)
            .await
            .context("Failed to create push notification config")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn update_push_notification_config(
        &self,
        params: &UpdatePushNotificationConfig,
    ) -> Result<(), CommonError> {
        let sqlc_params = update_push_notification_config_params {
            id: &params.id,
            url: &params.url,
            token: &params.token,
            authentication: &params.authentication,
            updated_at: &params.updated_at,
        };

        update_push_notification_config(&self.conn, sqlc_params)
            .await
            .context("Failed to update push notification config")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_push_notification_configs_by_task_id(
        &self,
        task_id: &WrappedUuidV4,
    ) -> Result<Vec<PushNotificationConfigModel>, CommonError> {
        let sqlc_params = get_push_notification_configs_by_task_id_params { task_id };

        let rows = get_push_notification_configs_by_task_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get push notification configs by task id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(rows.into_iter().map(PushNotificationConfigModel::from).collect())
    }

    async fn get_push_notification_config_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<PushNotificationConfigModel>, CommonError> {
        let sqlc_params = get_push_notification_config_by_id_params { id };

        let row_opt = get_push_notification_config_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get push notification config by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(row_opt.map(PushNotificationConfigModel::from))
    }

    async fn delete_push_notification_config(&self, id: &WrappedUuidV4) -> Result<(), CommonError> {
        let sqlc_params = delete_push_notification_config_params { id };

        delete_push_notification_config(&self.conn, sqlc_params)
            .await
            .context("Failed to delete push notification config")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_push_notification_configs_by_task_id(
        &self,
        task_id: &WrappedUuidV4,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_push_notification_configs_by_task_id_params { task_id };

        delete_push_notification_configs_by_task_id(&self.conn, sqlc_params)
            .await
            .context("Failed to delete push notification configs by task id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("dbs/a2a/migrations")
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;
        use crate::logic::task::{
            Message, MessagePart, MessageRole, MessageTaskTimelineItem, Metadata,
            TaskEventUpdateType, TaskStatus, TaskTimelineItemPayload, TextPart,
        };
        use crate::task_repository::{CreateTask, CreateTaskTimelineItem, TaskRepositoryLike};
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

            let new_status = TaskStatus::Working;
            let new_updated_at = WrappedChronoDateTime::now();
            let update_params = UpdateTaskStatus {
                id: task_id.clone(),
                status: new_status.clone(),
                status_timestamp: new_updated_at,
                updated_at: new_updated_at,
            };
            repo.update_task_status(&update_params).await.unwrap();

            let task = repo.get_task_by_id(&task_id).await.unwrap().unwrap();
            assert_eq!(task.status, TaskStatus::Working);
            assert_eq!(task.updated_at, new_updated_at);
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

            let timeline_id = WrappedUuidV4::new();
            let event_type = TaskEventUpdateType::Message;
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

            use std::thread::sleep;
            use std::time::Duration;
            let mut task_ids = vec![];
            for i in 0..5 {
                sleep(Duration::from_millis(10));
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

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let response = repo.get_tasks(&pagination).await.unwrap();

            assert_eq!(response.items.len(), 5);
            assert!(response.next_page_token.is_none());

            let pagination = PaginationRequest {
                page_size: 3,
                next_page_token: None,
            };
            let response = repo.get_tasks(&pagination).await.unwrap();
            assert_eq!(response.items.len(), 3);
            assert!(response.next_page_token.is_some());

            let pagination = PaginationRequest {
                page_size: 3,
                next_page_token: response.next_page_token,
            };
            let response = repo.get_tasks(&pagination).await.unwrap();
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
        async fn test_get_unique_contexts() {
            let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
                .await
                .unwrap();
            let repo = Repository::new(conn);

            let context_id_1 = WrappedUuidV4::new();
            let context_id_2 = WrappedUuidV4::new();

            use std::thread::sleep;
            use std::time::Duration;

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

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let response = repo.get_unique_contexts(&pagination).await.unwrap();

            assert_eq!(response.items.len(), 3);

            let context_ids: Vec<_> = response
                .items
                .iter()
                .map(|c| c.context_id.clone())
                .collect();
            assert!(context_ids.contains(&context_id_1));
            assert!(context_ids.contains(&context_id_2));
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

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let response = repo
                .get_tasks_by_context_id(&context_id_1, &pagination)
                .await
                .unwrap();

            assert_eq!(response.items.len(), 3);

            for task in &response.items {
                assert_eq!(task.context_id, context_id_1);
            }

            let retrieved_ids: Vec<_> = response.items.iter().map(|t| t.id.clone()).collect();
            for task_id in &task_ids_1 {
                assert!(retrieved_ids.contains(task_id));
            }

            let response = repo
                .get_tasks_by_context_id(&context_id_2, &pagination)
                .await
                .unwrap();

            assert_eq!(response.items.len(), 2);

            for task in &response.items {
                assert_eq!(task.context_id, context_id_2);
            }
        }
    }
}
