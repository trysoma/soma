//! SQLite repository implementation for inbox crate

#![allow(non_camel_case_types)]
mod raw_impl;

#[allow(clippy::all, unused_mut)]
#[allow(dead_code)]
mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::logic::{
    inbox::Inbox,
    message::Message,
    thread::Thread,
};
use crate::logic::inbox::DestinationType;
use crate::repository::{
    CreateEvent, CreateInbox, CreateMessage, CreateThread, EventRepositoryLike,
    InboxRepositoryLike, MessageRepositoryLike, StoredEvent, ThreadRepositoryLike, UpdateInbox,
    UpdateMessage, UpdateThread,
};
use anyhow::Context;
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, SqlMigrationLoader, WrappedChronoDateTime,
        WrappedUuidV4, decode_pagination_token,
    },
};
use shared_macros::load_atlas_sql_migrations;
use std::collections::BTreeMap;
use tracing::trace;

/// SQLite repository for inbox data
#[derive(Clone)]
pub struct Repository {
    conn: shared::libsql::Connection,
}

impl Repository {
    /// Create a new repository instance
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }

    /// Get the underlying connection
    pub fn connection(&self) -> &shared::libsql::Connection {
        &self.conn
    }
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("dbs/inbox/migrations")
    }
}

// --- Helper Functions ---

/// Decode pagination token to datetime cursor
fn decode_cursor(
    pagination: &PaginationRequest,
) -> Result<Option<WrappedChronoDateTime>, CommonError> {
    if let Some(token) = &pagination.next_page_token {
        let decoded_parts = decode_pagination_token(token).map_err(|e| CommonError::Repository {
            msg: format!("Invalid pagination token: {e}"),
            source: Some(e.into()),
        })?;
        if decoded_parts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                    CommonError::Repository {
                        msg: format!("Invalid datetime in pagination token: {e}"),
                        source: Some(e.into()),
                    }
                })?,
            ))
        }
    } else {
        Ok(None)
    }
}

// --- Thread Repository Implementation ---

#[async_trait::async_trait]
impl ThreadRepositoryLike for Repository {
    async fn create_thread(&self, params: &CreateThread) -> Result<(), CommonError> {
        trace!(thread_id = %params.id, "Creating thread");
        let sqlc_params = insert_thread_params {
            id: &params.id,
            title: &params.title,
            metadata: &params.metadata,
            inbox_settings: &params.inbox_settings,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_thread(&self.conn, sqlc_params)
            .await
            .context("Failed to create thread")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(thread_id = %params.id, "Thread created");
        Ok(())
    }

    async fn update_thread(&self, params: &UpdateThread) -> Result<(), CommonError> {
        trace!(thread_id = %params.id, "Updating thread");
        let sqlc_params = update_thread_params {
            id: &params.id,
            title: &params.title,
            metadata: &params.metadata,
            inbox_settings: &params.inbox_settings,
            updated_at: &params.updated_at,
        };

        update_thread(&self.conn, sqlc_params)
            .await
            .context("Failed to update thread")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(thread_id = %params.id, "Thread updated");
        Ok(())
    }

    async fn delete_thread(&self, id: &WrappedUuidV4) -> Result<(), CommonError> {
        trace!(thread_id = %id, "Deleting thread");
        let sqlc_params = delete_thread_params { id };

        delete_thread(&self.conn, sqlc_params)
            .await
            .context("Failed to delete thread")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(thread_id = %id, "Thread deleted");
        Ok(())
    }

    async fn get_thread_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Thread>, CommonError> {
        trace!(thread_id = %id, "Getting thread by ID");
        let sqlc_params = get_thread_by_id_params { id };

        let result = get_thread_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get thread by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let thread = match result {
            Some(row) => Some(Thread::try_from(row)?),
            None => None,
        };
        trace!(thread_id = %id, found = thread.is_some(), "Got thread by ID");
        Ok(thread)
    }

    async fn get_threads(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Thread>, CommonError> {
        trace!(page_size = pagination.page_size, "Listing threads");
        let cursor_datetime = decode_cursor(pagination)?;

        let sqlc_params = get_threads_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_threads(&self.conn, sqlc_params)
            .await
            .context("Failed to get threads")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Thread>, CommonError> =
            rows.into_iter().map(Thread::try_from).collect();
        let items = items?;

        trace!(count = items.len(), "Listed threads");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |thread| vec![thread.created_at.get_inner().to_rfc3339()],
        ))
    }
}

// --- Message Repository Implementation ---

#[async_trait::async_trait]
impl MessageRepositoryLike for Repository {
    async fn create_message(&self, params: &CreateMessage) -> Result<(), CommonError> {
        trace!(message_id = %params.id, thread_id = %params.thread_id, message_type = %params.message_type, "Creating message");
        let sqlc_params = insert_message_params {
            id: &params.id,
            thread_id: &params.thread_id,
            kind: &params.message_type,
            role: &params.role,
            body: &params.body,
            metadata: &params.metadata,
            inbox_settings: &params.inbox_settings,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_message(&self.conn, sqlc_params)
            .await
            .context("Failed to create message")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(message_id = %params.id, "Message created");
        Ok(())
    }

    async fn update_message(&self, params: &UpdateMessage) -> Result<(), CommonError> {
        trace!(message_id = %params.id, "Updating message");
        let sqlc_params = update_message_params {
            id: &params.id,
            body: &params.body,
            metadata: &params.metadata,
            inbox_settings: &params.inbox_settings,
            updated_at: &params.updated_at,
        };

        update_message(&self.conn, sqlc_params)
            .await
            .context("Failed to update message")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(message_id = %params.id, "Message updated");
        Ok(())
    }

    async fn delete_message(&self, id: &WrappedUuidV4) -> Result<(), CommonError> {
        trace!(message_id = %id, "Deleting message");
        let sqlc_params = delete_message_params { id };

        delete_message(&self.conn, sqlc_params)
            .await
            .context("Failed to delete message")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(message_id = %id, "Message deleted");
        Ok(())
    }

    async fn get_message_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<Message>, CommonError> {
        trace!(message_id = %id, "Getting message by ID");
        let sqlc_params = get_message_by_id_params { id };

        let result = get_message_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get message by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let message = match result {
            Some(row) => Some(Message::try_from(row)?),
            None => None,
        };
        trace!(message_id = %id, found = message.is_some(), "Got message by ID");
        Ok(message)
    }

    async fn get_messages(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Message>, CommonError> {
        trace!(page_size = pagination.page_size, "Listing messages");
        let cursor_datetime = decode_cursor(pagination)?;

        let sqlc_params = get_messages_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_messages(&self.conn, sqlc_params)
            .await
            .context("Failed to get messages")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Message>, CommonError> =
            rows.into_iter().map(Message::try_from).collect();
        let items = items?;

        trace!(count = items.len(), "Listed messages");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |message| vec![message.created_at().get_inner().to_rfc3339()],
        ))
    }

    async fn get_messages_by_thread(
        &self,
        thread_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Message>, CommonError> {
        trace!(thread_id = %thread_id, page_size = pagination.page_size, "Listing messages by thread");
        let cursor_datetime = decode_cursor(pagination)?;

        let sqlc_params = get_messages_by_thread_params {
            thread_id,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_messages_by_thread(&self.conn, sqlc_params)
            .await
            .context("Failed to get messages by thread")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Message>, CommonError> =
            rows.into_iter().map(Message::try_from).collect();
        let items = items?;

        trace!(thread_id = %thread_id, count = items.len(), "Listed messages by thread");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |message| vec![message.created_at().get_inner().to_rfc3339()],
        ))
    }

    async fn delete_messages_by_thread(
        &self,
        thread_id: &WrappedUuidV4,
    ) -> Result<(), CommonError> {
        trace!(thread_id = %thread_id, "Deleting messages by thread");
        let sqlc_params = delete_messages_by_thread_params { thread_id };

        delete_messages_by_thread(&self.conn, sqlc_params)
            .await
            .context("Failed to delete messages by thread")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(thread_id = %thread_id, "Deleted messages by thread");
        Ok(())
    }
}

// --- Event Repository Implementation ---

#[async_trait::async_trait]
impl EventRepositoryLike for Repository {
    async fn create_event(&self, params: &CreateEvent) -> Result<(), CommonError> {
        trace!(event_id = %params.id, kind = %params.kind, "Creating event");
        let sqlc_params = insert_event_params {
            id: &params.id,
            kind: &params.kind,
            payload: &params.payload,
            inbox_id: &params.inbox_id,
            inbox_settings: &params.inbox_settings,
            created_at: &params.created_at,
        };

        insert_event(&self.conn, sqlc_params)
            .await
            .context("Failed to create event")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(event_id = %params.id, "Event created");
        Ok(())
    }

    async fn get_event_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<StoredEvent>, CommonError> {
        trace!(event_id = %id, "Getting event by ID");
        let sqlc_params = get_event_by_id_params { id };

        let result = get_event_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get event by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let event = match result {
            Some(row) => Some(StoredEvent::try_from(row)?),
            None => None,
        };
        trace!(event_id = %id, found = event.is_some(), "Got event by ID");
        Ok(event)
    }

    async fn get_events(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<StoredEvent>, CommonError> {
        trace!(page_size = pagination.page_size, "Listing events");
        let cursor_datetime = decode_cursor(pagination)?;

        let sqlc_params = get_events_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_events(&self.conn, sqlc_params)
            .await
            .context("Failed to get events")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<StoredEvent>, CommonError> =
            rows.into_iter().map(StoredEvent::try_from).collect();
        let items = items?;

        trace!(count = items.len(), "Listed events");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |event| vec![event.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_events_by_inbox(
        &self,
        inbox_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<StoredEvent>, CommonError> {
        trace!(inbox_id = %inbox_id, page_size = pagination.page_size, "Listing events by inbox");
        let cursor_datetime = decode_cursor(pagination)?;
        let inbox_id_opt = Some(inbox_id.to_string());

        let sqlc_params = get_events_by_inbox_params {
            inbox_id: &inbox_id_opt,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_events_by_inbox(&self.conn, sqlc_params)
            .await
            .context("Failed to get events by inbox")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<StoredEvent>, CommonError> =
            rows.into_iter().map(StoredEvent::try_from).collect();
        let items = items?;

        trace!(inbox_id = %inbox_id, count = items.len(), "Listed events by inbox");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |event| vec![event.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_events_by_kind(
        &self,
        kind: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<StoredEvent>, CommonError> {
        trace!(kind = %kind, page_size = pagination.page_size, "Listing events by kind");
        let cursor_datetime = decode_cursor(pagination)?;
        let kind_str = kind.to_string();

        let sqlc_params = get_events_by_kind_params {
            kind: &kind_str,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_events_by_kind(&self.conn, sqlc_params)
            .await
            .context("Failed to get events by kind")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<StoredEvent>, CommonError> =
            rows.into_iter().map(StoredEvent::try_from).collect();
        let items = items?;

        trace!(kind = %kind, count = items.len(), "Listed events by kind");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |event| vec![event.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn delete_events_before(
        &self,
        before_date: &WrappedChronoDateTime,
    ) -> Result<(), CommonError> {
        trace!(before = %before_date, "Deleting events before date");
        let sqlc_params = delete_events_before_params {
            before_date,
        };

        delete_events_before(&self.conn, sqlc_params)
            .await
            .context("Failed to delete events before date")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(before = %before_date, "Deleted events before date");
        Ok(())
    }
}

// --- Inbox Repository Implementation ---

#[async_trait::async_trait]
impl InboxRepositoryLike for Repository {
    async fn create_inbox(&self, params: &CreateInbox) -> Result<(), CommonError> {
        trace!(inbox_id = %params.id, provider_id = %params.provider_id, "Creating inbox");
        let sqlc_params = insert_inbox_params {
            id: &params.id,
            provider_id: &params.provider_id,
            destination_type: &params.destination_type,
            destination_id: &params.destination_id,
            configuration: &params.configuration,
            settings: &params.settings,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_inbox(&self.conn, sqlc_params)
            .await
            .context("Failed to create inbox")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(inbox_id = %params.id, "Inbox created");
        Ok(())
    }

    async fn update_inbox(&self, params: &UpdateInbox) -> Result<(), CommonError> {
        trace!(inbox_id = %params.id, "Updating inbox");
        let sqlc_params = update_inbox_params {
            id: &params.id,
            configuration: &params.configuration,
            settings: &params.settings,
            updated_at: &params.updated_at,
        };

        update_inbox(&self.conn, sqlc_params)
            .await
            .context("Failed to update inbox")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(inbox_id = %params.id, "Inbox updated");
        Ok(())
    }

    async fn delete_inbox(&self, id: &str) -> Result<(), CommonError> {
        trace!(inbox_id = %id, "Deleting inbox");
        let id_str = id.to_string();
        let sqlc_params = delete_inbox_params { id: &id_str };

        delete_inbox(&self.conn, sqlc_params)
            .await
            .context("Failed to delete inbox")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(inbox_id = %id, "Inbox deleted");
        Ok(())
    }

    async fn get_inbox_by_id(&self, id: &str) -> Result<Option<Inbox>, CommonError> {
        trace!(inbox_id = %id, "Getting inbox by ID");
        let id_str = id.to_string();
        let sqlc_params = get_inbox_by_id_params { id: &id_str };

        let result = get_inbox_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get inbox by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let inbox = match result {
            Some(row) => Some(Inbox::try_from(row)?),
            None => None,
        };
        trace!(inbox_id = %id, found = inbox.is_some(), "Got inbox by ID");
        Ok(inbox)
    }

    async fn get_inboxes(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError> {
        trace!(page_size = pagination.page_size, "Listing inboxes");
        let cursor_datetime = decode_cursor(pagination)?;

        let sqlc_params = get_inboxes_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_inboxes(&self.conn, sqlc_params)
            .await
            .context("Failed to get inboxes")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Inbox>, CommonError> =
            rows.into_iter().map(Inbox::try_from).collect();
        let items = items?;

        trace!(count = items.len(), "Listed inboxes");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |inbox| vec![inbox.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_inboxes_by_provider(
        &self,
        provider_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError> {
        trace!(provider_id = %provider_id, page_size = pagination.page_size, "Listing inboxes by provider");
        let cursor_datetime = decode_cursor(pagination)?;
        let provider_id_str = provider_id.to_string();

        let sqlc_params = get_inboxes_by_provider_params {
            provider_id: &provider_id_str,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_inboxes_by_provider(&self.conn, sqlc_params)
            .await
            .context("Failed to get inboxes by provider")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Inbox>, CommonError> =
            rows.into_iter().map(Inbox::try_from).collect();
        let items = items?;

        trace!(provider_id = %provider_id, count = items.len(), "Listed inboxes by provider");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |inbox| vec![inbox.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_inboxes_by_destination(
        &self,
        destination_type: &DestinationType,
        destination_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError> {
        trace!(destination_type = %destination_type, destination_id = %destination_id, page_size = pagination.page_size, "Listing inboxes by destination");
        let cursor_datetime = decode_cursor(pagination)?;
        let destination_id_str = destination_id.to_string();

        let sqlc_params = get_inboxes_by_destination_params {
            destination_type,
            destination_id: &destination_id_str,
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_inboxes_by_destination(&self.conn, sqlc_params)
            .await
            .context("Failed to get inboxes by destination")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Inbox>, CommonError> =
            rows.into_iter().map(Inbox::try_from).collect();
        let items = items?;

        trace!(destination_type = %destination_type, destination_id = %destination_id, count = items.len(), "Listed inboxes by destination");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |inbox| vec![inbox.created_at.get_inner().to_rfc3339()],
        ))
    }
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;
        use crate::logic::inbox::DestinationType;
        use crate::logic::message::{MessageRole, MessageType, TextMessageBody, UIMessageBody, UIMessagePart};
        use serde_json::json;
        use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4};

        async fn setup_test_db() -> Repository {
            shared::setup_test!();

            let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                Repository::load_sql_migrations(),
            ])
            .await
            .unwrap();

            Repository::new(conn)
        }

        // ============================================
        // Thread Repository Tests
        // ============================================

        #[tokio::test]
        async fn test_create_and_get_thread() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let thread_id = WrappedUuidV4::new();
            let params = CreateThread {
                id: thread_id.clone(),
                title: Some("Test Thread".to_string()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_thread(&params).await.unwrap();

            let fetched = repo.get_thread_by_id(&thread_id).await.unwrap();
            assert!(fetched.is_some());
            let fetched = fetched.unwrap();
            assert_eq!(fetched.id, thread_id);
            assert_eq!(fetched.title, Some("Test Thread".to_string()));
        }

        #[tokio::test]
        async fn test_get_thread_not_found() {
            let repo = setup_test_db().await;

            let thread_id = WrappedUuidV4::new();
            let fetched = repo.get_thread_by_id(&thread_id).await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_update_thread() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let thread_id = WrappedUuidV4::new();
            let create_params = CreateThread {
                id: thread_id.clone(),
                title: Some("Original Title".to_string()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_thread(&create_params).await.unwrap();

            let update_params = UpdateThread {
                id: thread_id.clone(),
                title: Some("Updated Title".to_string()),
                metadata: Some(WrappedJsonValue::new(json!({"key": "value"}))),
                inbox_settings: WrappedJsonValue::new(json!({"setting": true})),
                updated_at: WrappedChronoDateTime::now(),
            };

            repo.update_thread(&update_params).await.unwrap();

            let fetched = repo.get_thread_by_id(&thread_id).await.unwrap().unwrap();
            assert_eq!(fetched.title, Some("Updated Title".to_string()));
            assert!(fetched.metadata.is_some());
        }

        #[tokio::test]
        async fn test_delete_thread() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let thread_id = WrappedUuidV4::new();
            let params = CreateThread {
                id: thread_id.clone(),
                title: Some("To Be Deleted".to_string()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_thread(&params).await.unwrap();
            repo.delete_thread(&thread_id).await.unwrap();

            let fetched = repo.get_thread_by_id(&thread_id).await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_list_threads_with_pagination() {
            let repo = setup_test_db().await;

            // Create multiple threads
            for i in 1..=5 {
                let now = WrappedChronoDateTime::now();
                let params = CreateThread {
                    id: WrappedUuidV4::new(),
                    title: Some(format!("Thread {i}")),
                    metadata: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_thread(&params).await.unwrap();
            }

            // List all
            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let result = repo.get_threads(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 5);
            assert!(result.next_page_token.is_none());

            // Test pagination with smaller page size
            let pagination = PaginationRequest {
                page_size: 2,
                next_page_token: None,
            };
            let result = repo.get_threads(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.next_page_token.is_some());

            // Get next page
            let pagination = PaginationRequest {
                page_size: 2,
                next_page_token: result.next_page_token,
            };
            let result2 = repo.get_threads(&pagination).await.unwrap();
            assert_eq!(result2.items.len(), 2);
        }

        // ============================================
        // Message Repository Tests
        // ============================================

        async fn create_test_thread(repo: &Repository) -> WrappedUuidV4 {
            let now = WrappedChronoDateTime::now();
            let thread_id = WrappedUuidV4::new();
            let params = CreateThread {
                id: thread_id.clone(),
                title: Some("Test Thread".to_string()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };
            repo.create_thread(&params).await.unwrap();
            thread_id
        }

        #[tokio::test]
        async fn test_create_and_get_text_message() {
            let repo = setup_test_db().await;
            let thread_id = create_test_thread(&repo).await;

            let now = WrappedChronoDateTime::now();
            let message_id = WrappedUuidV4::new();
            let body = TextMessageBody { text: "Hello, world!".to_string() };
            let params = CreateMessage {
                id: message_id.clone(),
                thread_id: thread_id.clone(),
                message_type: MessageType::Text,
                role: MessageRole::User,
                body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_message(&params).await.unwrap();

            let fetched = repo.get_message_by_id(&message_id).await.unwrap();
            assert!(fetched.is_some());
            let fetched = fetched.unwrap();
            assert_eq!(fetched.id(), &message_id);
            assert_eq!(fetched.thread_id(), &thread_id);
            assert_eq!(*fetched.role(), MessageRole::User);
            assert_eq!(fetched.text_content(), "Hello, world!");
        }

        #[tokio::test]
        async fn test_create_and_get_ui_message() {
            let repo = setup_test_db().await;
            let thread_id = create_test_thread(&repo).await;

            let now = WrappedChronoDateTime::now();
            let message_id = WrappedUuidV4::new();
            let body = UIMessageBody {
                parts: vec![UIMessagePart::text("Hello from UI message!")],
            };
            let params = CreateMessage {
                id: message_id.clone(),
                thread_id: thread_id.clone(),
                message_type: MessageType::Ui,
                role: MessageRole::Assistant,
                body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_message(&params).await.unwrap();

            let fetched = repo.get_message_by_id(&message_id).await.unwrap();
            assert!(fetched.is_some());
            let fetched = fetched.unwrap();
            assert_eq!(fetched.id(), &message_id);
            assert_eq!(*fetched.role(), MessageRole::Assistant);
            assert_eq!(fetched.text_content(), "Hello from UI message!");
        }

        #[tokio::test]
        async fn test_get_message_not_found() {
            let repo = setup_test_db().await;

            let message_id = WrappedUuidV4::new();
            let fetched = repo.get_message_by_id(&message_id).await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_update_message() {
            let repo = setup_test_db().await;
            let thread_id = create_test_thread(&repo).await;

            let now = WrappedChronoDateTime::now();
            let message_id = WrappedUuidV4::new();
            let body = TextMessageBody { text: "Original text".to_string() };
            let create_params = CreateMessage {
                id: message_id.clone(),
                thread_id: thread_id.clone(),
                message_type: MessageType::Text,
                role: MessageRole::User,
                body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_message(&create_params).await.unwrap();

            let updated_body = TextMessageBody { text: "Updated text".to_string() };
            let update_params = UpdateMessage {
                id: message_id.clone(),
                body: WrappedJsonValue::new(serde_json::to_value(&updated_body).unwrap()),
                metadata: Some(WrappedJsonValue::new(json!({"edited": true}))),
                inbox_settings: WrappedJsonValue::new(json!({})),
                updated_at: WrappedChronoDateTime::now(),
            };

            repo.update_message(&update_params).await.unwrap();

            let fetched = repo.get_message_by_id(&message_id).await.unwrap().unwrap();
            assert_eq!(fetched.text_content(), "Updated text");
            assert!(fetched.metadata().is_some());
        }

        #[tokio::test]
        async fn test_delete_message() {
            let repo = setup_test_db().await;
            let thread_id = create_test_thread(&repo).await;

            let now = WrappedChronoDateTime::now();
            let message_id = WrappedUuidV4::new();
            let body = TextMessageBody { text: "To be deleted".to_string() };
            let params = CreateMessage {
                id: message_id.clone(),
                thread_id,
                message_type: MessageType::Text,
                role: MessageRole::User,
                body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_message(&params).await.unwrap();
            repo.delete_message(&message_id).await.unwrap();

            let fetched = repo.get_message_by_id(&message_id).await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_list_messages_with_pagination() {
            let repo = setup_test_db().await;
            let thread_id = create_test_thread(&repo).await;

            // Create multiple messages
            for i in 1..=5 {
                let now = WrappedChronoDateTime::now();
                let body = TextMessageBody { text: format!("Message {i}") };
                let params = CreateMessage {
                    id: WrappedUuidV4::new(),
                    thread_id: thread_id.clone(),
                    message_type: MessageType::Text,
                    role: MessageRole::User,
                    body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                    metadata: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_message(&params).await.unwrap();
            }

            // List all
            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let result = repo.get_messages(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 5);

            // Test pagination
            let pagination = PaginationRequest {
                page_size: 2,
                next_page_token: None,
            };
            let result = repo.get_messages(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.next_page_token.is_some());
        }

        #[tokio::test]
        async fn test_get_messages_by_thread() {
            let repo = setup_test_db().await;
            let thread1_id = create_test_thread(&repo).await;
            let thread2_id = create_test_thread(&repo).await;

            // Create messages in thread 1
            for i in 1..=3 {
                let now = WrappedChronoDateTime::now();
                let body = TextMessageBody { text: format!("Thread1 Message {i}") };
                let params = CreateMessage {
                    id: WrappedUuidV4::new(),
                    thread_id: thread1_id.clone(),
                    message_type: MessageType::Text,
                    role: MessageRole::User,
                    body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                    metadata: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_message(&params).await.unwrap();
            }

            // Create messages in thread 2
            for i in 1..=2 {
                let now = WrappedChronoDateTime::now();
                let body = TextMessageBody { text: format!("Thread2 Message {i}") };
                let params = CreateMessage {
                    id: WrappedUuidV4::new(),
                    thread_id: thread2_id.clone(),
                    message_type: MessageType::Text,
                    role: MessageRole::User,
                    body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                    metadata: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_message(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result1 = repo.get_messages_by_thread(&thread1_id, &pagination).await.unwrap();
            assert_eq!(result1.items.len(), 3);

            let result2 = repo.get_messages_by_thread(&thread2_id, &pagination).await.unwrap();
            assert_eq!(result2.items.len(), 2);
        }

        #[tokio::test]
        async fn test_delete_messages_by_thread() {
            let repo = setup_test_db().await;
            let thread_id = create_test_thread(&repo).await;

            // Create messages
            for i in 1..=3 {
                let now = WrappedChronoDateTime::now();
                let body = TextMessageBody { text: format!("Message {i}") };
                let params = CreateMessage {
                    id: WrappedUuidV4::new(),
                    thread_id: thread_id.clone(),
                    message_type: MessageType::Text,
                    role: MessageRole::User,
                    body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                    metadata: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_message(&params).await.unwrap();
            }

            repo.delete_messages_by_thread(&thread_id).await.unwrap();

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let result = repo.get_messages_by_thread(&thread_id, &pagination).await.unwrap();
            assert_eq!(result.items.len(), 0);
        }

        // ============================================
        // Event Repository Tests
        // ============================================

        #[tokio::test]
        async fn test_create_and_get_event() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let event_id = WrappedUuidV4::new();
            let params = CreateEvent {
                id: event_id.clone(),
                kind: "message_created".to_string(),
                payload: WrappedJsonValue::new(json!({"message_id": "msg-123"})),
                inbox_id: Some("inbox-1".to_string()),
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
            };

            repo.create_event(&params).await.unwrap();

            let fetched = repo.get_event_by_id(&event_id).await.unwrap();
            assert!(fetched.is_some());
            let fetched = fetched.unwrap();
            assert_eq!(fetched.id, event_id);
            assert_eq!(fetched.kind, "message_created");
            assert_eq!(fetched.inbox_id, Some("inbox-1".to_string()));
        }

        #[tokio::test]
        async fn test_get_event_not_found() {
            let repo = setup_test_db().await;

            let event_id = WrappedUuidV4::new();
            let fetched = repo.get_event_by_id(&event_id).await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_list_events_with_pagination() {
            let repo = setup_test_db().await;

            // Create multiple events
            for i in 1..=5 {
                let now = WrappedChronoDateTime::now();
                let params = CreateEvent {
                    id: WrappedUuidV4::new(),
                    kind: format!("event_type_{i}"),
                    payload: WrappedJsonValue::new(json!({"index": i})),
                    inbox_id: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                };
                repo.create_event(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let result = repo.get_events(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 5);

            // Test pagination
            let pagination = PaginationRequest {
                page_size: 2,
                next_page_token: None,
            };
            let result = repo.get_events(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.next_page_token.is_some());
        }

        #[tokio::test]
        async fn test_get_events_by_inbox() {
            let repo = setup_test_db().await;

            // Create events for different inboxes
            for i in 1..=3 {
                let now = WrappedChronoDateTime::now();
                let params = CreateEvent {
                    id: WrappedUuidV4::new(),
                    kind: "test_event".to_string(),
                    payload: WrappedJsonValue::new(json!({"index": i})),
                    inbox_id: Some("inbox-1".to_string()),
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                };
                repo.create_event(&params).await.unwrap();
            }

            for i in 1..=2 {
                let now = WrappedChronoDateTime::now();
                let params = CreateEvent {
                    id: WrappedUuidV4::new(),
                    kind: "test_event".to_string(),
                    payload: WrappedJsonValue::new(json!({"index": i})),
                    inbox_id: Some("inbox-2".to_string()),
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                };
                repo.create_event(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result1 = repo.get_events_by_inbox("inbox-1", &pagination).await.unwrap();
            assert_eq!(result1.items.len(), 3);

            let result2 = repo.get_events_by_inbox("inbox-2", &pagination).await.unwrap();
            assert_eq!(result2.items.len(), 2);
        }

        #[tokio::test]
        async fn test_get_events_by_kind() {
            let repo = setup_test_db().await;

            // Create events of different kinds
            for i in 1..=3 {
                let now = WrappedChronoDateTime::now();
                let params = CreateEvent {
                    id: WrappedUuidV4::new(),
                    kind: "message_created".to_string(),
                    payload: WrappedJsonValue::new(json!({"index": i})),
                    inbox_id: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                };
                repo.create_event(&params).await.unwrap();
            }

            for i in 1..=2 {
                let now = WrappedChronoDateTime::now();
                let params = CreateEvent {
                    id: WrappedUuidV4::new(),
                    kind: "thread_updated".to_string(),
                    payload: WrappedJsonValue::new(json!({"index": i})),
                    inbox_id: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                };
                repo.create_event(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result1 = repo.get_events_by_kind("message_created", &pagination).await.unwrap();
            assert_eq!(result1.items.len(), 3);

            let result2 = repo.get_events_by_kind("thread_updated", &pagination).await.unwrap();
            assert_eq!(result2.items.len(), 2);
        }

        #[tokio::test]
        async fn test_delete_events_before() {
            let repo = setup_test_db().await;

            // Create some events
            let now = WrappedChronoDateTime::now();
            for i in 1..=3 {
                let params = CreateEvent {
                    id: WrappedUuidV4::new(),
                    kind: format!("event_{i}"),
                    payload: WrappedJsonValue::new(json!({"index": i})),
                    inbox_id: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                };
                repo.create_event(&params).await.unwrap();
            }

            // Delete events before a future date (should delete all)
            let future = WrappedChronoDateTime::new(
                *now.get_inner() + chrono::Duration::hours(1),
            );
            repo.delete_events_before(&future).await.unwrap();

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let result = repo.get_events(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 0);
        }

        // ============================================
        // Inbox Repository Tests
        // ============================================

        #[tokio::test]
        async fn test_create_and_get_inbox() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let params = CreateInbox {
                id: "inbox-1".to_string(),
                provider_id: "slack".to_string(),
                destination_type: DestinationType::Agent,
                destination_id: "agent-123".to_string(),
                configuration: WrappedJsonValue::new(json!({"channel": "general"})),
                settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_inbox(&params).await.unwrap();

            let fetched = repo.get_inbox_by_id("inbox-1").await.unwrap();
            assert!(fetched.is_some());
            let fetched = fetched.unwrap();
            assert_eq!(fetched.id, "inbox-1");
            assert_eq!(fetched.provider_id, "slack");
            assert_eq!(fetched.destination_type, DestinationType::Agent);
            assert_eq!(fetched.destination_id, "agent-123");
        }

        #[tokio::test]
        async fn test_get_inbox_not_found() {
            let repo = setup_test_db().await;

            let fetched = repo.get_inbox_by_id("nonexistent").await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_update_inbox() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let create_params = CreateInbox {
                id: "inbox-1".to_string(),
                provider_id: "slack".to_string(),
                destination_type: DestinationType::Agent,
                destination_id: "agent-123".to_string(),
                configuration: WrappedJsonValue::new(json!({"channel": "general"})),
                settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_inbox(&create_params).await.unwrap();

            let update_params = UpdateInbox {
                id: "inbox-1".to_string(),
                configuration: WrappedJsonValue::new(json!({"channel": "random"})),
                settings: WrappedJsonValue::new(json!({"priority": "high"})),
                updated_at: WrappedChronoDateTime::now(),
            };

            repo.update_inbox(&update_params).await.unwrap();

            let fetched = repo.get_inbox_by_id("inbox-1").await.unwrap().unwrap();
            let config = fetched.configuration.get_inner();
            assert_eq!(config["channel"], "random");
        }

        #[tokio::test]
        async fn test_delete_inbox() {
            let repo = setup_test_db().await;

            let now = WrappedChronoDateTime::now();
            let params = CreateInbox {
                id: "inbox-1".to_string(),
                provider_id: "slack".to_string(),
                destination_type: DestinationType::Workflow,
                destination_id: "workflow-456".to_string(),
                configuration: WrappedJsonValue::new(json!({})),
                settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };

            repo.create_inbox(&params).await.unwrap();
            repo.delete_inbox("inbox-1").await.unwrap();

            let fetched = repo.get_inbox_by_id("inbox-1").await.unwrap();
            assert!(fetched.is_none());
        }

        #[tokio::test]
        async fn test_list_inboxes_with_pagination() {
            let repo = setup_test_db().await;

            // Create multiple inboxes
            for i in 1..=5 {
                let now = WrappedChronoDateTime::now();
                let params = CreateInbox {
                    id: format!("inbox-{i}"),
                    provider_id: "slack".to_string(),
                    destination_type: if i % 2 == 0 { DestinationType::Workflow } else { DestinationType::Agent },
                    destination_id: format!("dest-{i}"),
                    configuration: WrappedJsonValue::new(json!({})),
                    settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_inbox(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let result = repo.get_inboxes(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 5);

            // Test pagination
            let pagination = PaginationRequest {
                page_size: 2,
                next_page_token: None,
            };
            let result = repo.get_inboxes(&pagination).await.unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.next_page_token.is_some());
        }

        #[tokio::test]
        async fn test_get_inboxes_by_provider() {
            let repo = setup_test_db().await;

            // Create inboxes for different providers
            for i in 1..=3 {
                let now = WrappedChronoDateTime::now();
                let params = CreateInbox {
                    id: format!("slack-inbox-{i}"),
                    provider_id: "slack".to_string(),
                    destination_type: DestinationType::Agent,
                    destination_id: format!("agent-{i}"),
                    configuration: WrappedJsonValue::new(json!({})),
                    settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_inbox(&params).await.unwrap();
            }

            for i in 1..=2 {
                let now = WrappedChronoDateTime::now();
                let params = CreateInbox {
                    id: format!("discord-inbox-{i}"),
                    provider_id: "discord".to_string(),
                    destination_type: DestinationType::Agent,
                    destination_id: format!("agent-{i}"),
                    configuration: WrappedJsonValue::new(json!({})),
                    settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_inbox(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result1 = repo.get_inboxes_by_provider("slack", &pagination).await.unwrap();
            assert_eq!(result1.items.len(), 3);

            let result2 = repo.get_inboxes_by_provider("discord", &pagination).await.unwrap();
            assert_eq!(result2.items.len(), 2);
        }

        #[tokio::test]
        async fn test_get_inboxes_by_destination() {
            let repo = setup_test_db().await;

            // Create inboxes for different destinations
            for i in 1..=3 {
                let now = WrappedChronoDateTime::now();
                let params = CreateInbox {
                    id: format!("agent-inbox-{i}"),
                    provider_id: "slack".to_string(),
                    destination_type: DestinationType::Agent,
                    destination_id: "agent-123".to_string(),
                    configuration: WrappedJsonValue::new(json!({})),
                    settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_inbox(&params).await.unwrap();
            }

            for i in 1..=2 {
                let now = WrappedChronoDateTime::now();
                let params = CreateInbox {
                    id: format!("workflow-inbox-{i}"),
                    provider_id: "slack".to_string(),
                    destination_type: DestinationType::Workflow,
                    destination_id: "workflow-456".to_string(),
                    configuration: WrappedJsonValue::new(json!({})),
                    settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_inbox(&params).await.unwrap();
            }

            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };

            let result1 = repo.get_inboxes_by_destination(&DestinationType::Agent, "agent-123", &pagination).await.unwrap();
            assert_eq!(result1.items.len(), 3);
            for inbox in &result1.items {
                assert_eq!(inbox.destination_type, DestinationType::Agent);
                assert_eq!(inbox.destination_id, "agent-123");
            }

            let result2 = repo.get_inboxes_by_destination(&DestinationType::Workflow, "workflow-456", &pagination).await.unwrap();
            assert_eq!(result2.items.len(), 2);
            for inbox in &result2.items {
                assert_eq!(inbox.destination_type, DestinationType::Workflow);
                assert_eq!(inbox.destination_id, "workflow-456");
            }
        }

        #[tokio::test]
        async fn test_thread_cascade_deletes_messages() {
            let repo = setup_test_db().await;

            // Create thread with messages
            let now = WrappedChronoDateTime::now();
            let thread_id = WrappedUuidV4::new();
            let thread_params = CreateThread {
                id: thread_id.clone(),
                title: Some("Test Thread".to_string()),
                metadata: None,
                inbox_settings: WrappedJsonValue::new(json!({})),
                created_at: now,
                updated_at: now,
            };
            repo.create_thread(&thread_params).await.unwrap();

            // Create messages in the thread
            for i in 1..=3 {
                let body = TextMessageBody { text: format!("Message {i}") };
                let msg_params = CreateMessage {
                    id: WrappedUuidV4::new(),
                    thread_id: thread_id.clone(),
                    message_type: MessageType::Text,
                    role: MessageRole::User,
                    body: WrappedJsonValue::new(serde_json::to_value(&body).unwrap()),
                    metadata: None,
                    inbox_settings: WrappedJsonValue::new(json!({})),
                    created_at: now,
                    updated_at: now,
                };
                repo.create_message(&msg_params).await.unwrap();
            }

            // Verify messages exist
            let pagination = PaginationRequest {
                page_size: 10,
                next_page_token: None,
            };
            let messages = repo.get_messages_by_thread(&thread_id, &pagination).await.unwrap();
            assert_eq!(messages.items.len(), 3);

            // Delete thread - messages should be cascade deleted
            repo.delete_thread(&thread_id).await.unwrap();

            // Verify messages are gone
            let messages = repo.get_messages_by_thread(&thread_id, &pagination).await.unwrap();
            assert_eq!(messages.items.len(), 0);
        }
    }
}
