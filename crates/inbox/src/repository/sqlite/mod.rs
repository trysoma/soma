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
    message::UIMessage,
    thread::Thread,
};
use crate::repository::{
    CreateEvent, CreateInbox, CreateMessage, CreateThread, EventRepositoryLike,
    InboxRepositoryLike, MessageRepositoryLike, StoredEvent, ThreadRepositoryLike, UpdateInbox,
    UpdateInboxStatus, UpdateMessage, UpdateThread,
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
        trace!(message_id = %params.id, thread_id = %params.thread_id, "Creating message");
        let sqlc_params = insert_message_params {
            id: &params.id,
            thread_id: &params.thread_id,
            role: &params.role,
            parts: &params.parts,
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
            parts: &params.parts,
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
    ) -> Result<Option<UIMessage>, CommonError> {
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
            Some(row) => Some(UIMessage::try_from(row)?),
            None => None,
        };
        trace!(message_id = %id, found = message.is_some(), "Got message by ID");
        Ok(message)
    }

    async fn get_messages(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UIMessage>, CommonError> {
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

        let items: Result<Vec<UIMessage>, CommonError> =
            rows.into_iter().map(UIMessage::try_from).collect();
        let items = items?;

        trace!(count = items.len(), "Listed messages");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |message| vec![message.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_messages_by_thread(
        &self,
        thread_id: &WrappedUuidV4,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UIMessage>, CommonError> {
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

        let items: Result<Vec<UIMessage>, CommonError> =
            rows.into_iter().map(UIMessage::try_from).collect();
        let items = items?;

        trace!(thread_id = %thread_id, count = items.len(), "Listed messages by thread");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |message| vec![message.created_at.get_inner().to_rfc3339()],
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
            status: &params.status,
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

    async fn update_inbox_status(&self, params: &UpdateInboxStatus) -> Result<(), CommonError> {
        trace!(inbox_id = %params.id, status = %params.status, "Updating inbox status");
        let sqlc_params = update_inbox_status_params {
            id: &params.id,
            status: &params.status,
            updated_at: &params.updated_at,
        };

        update_inbox_status(&self.conn, sqlc_params)
            .await
            .context("Failed to update inbox status")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        trace!(inbox_id = %params.id, "Inbox status updated");
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

    async fn get_enabled_inboxes(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Inbox>, CommonError> {
        trace!(page_size = pagination.page_size, "Listing enabled inboxes");
        let cursor_datetime = decode_cursor(pagination)?;

        let sqlc_params = get_enabled_inboxes_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_enabled_inboxes(&self.conn, sqlc_params)
            .await
            .context("Failed to get enabled inboxes")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Inbox>, CommonError> =
            rows.into_iter().map(Inbox::try_from).collect();
        let items = items?;

        trace!(count = items.len(), "Listed enabled inboxes");
        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |inbox| vec![inbox.created_at.get_inner().to_rfc3339()],
        ))
    }
}
