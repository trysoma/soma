//! Thread HTTP endpoints

use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::{
    logic::{
        event::InboxEvent,
        thread::{
            CreateThreadRequest, CreateThreadResponse, DeleteThreadResponse,
            GetThreadWithMessagesResponse, ListThreadsResponse, Thread, UpdateThreadRequest,
            UpdateThreadResponse,
        },
    },
    repository::{CreateThread, MessageRepositoryLike, ThreadRepositoryLike, UpdateThread},
    service::InboxService,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
};

/// Create the thread router
pub fn create_router() -> OpenApiRouter<Arc<InboxService>> {
    OpenApiRouter::new()
        .routes(routes!(route_list_threads))
        .routes(routes!(route_create_thread))
        .routes(routes!(route_get_thread_with_messages))
        .routes(routes!(route_update_thread))
        .routes(routes!(route_delete_thread))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/thread", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(PaginationRequest),
    responses(
        (status = 200, description = "List threads", body = ListThreadsResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List threads",
    description = "List all threads with pagination",
    operation_id = "list-threads",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_threads(
    State(ctx): State<Arc<InboxService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListThreadsResponse, CommonError> {
    trace!(page_size = pagination.page_size, "Listing threads");
    let res = list_threads(&ctx.repository, pagination).await;
    trace!(success = res.is_ok(), "Listing threads completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/thread", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateThreadRequest,
    responses(
        (status = 200, description = "Create a thread", body = CreateThreadResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create thread",
    description = "Create a new thread for grouping messages",
    operation_id = "create-thread",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_create_thread(
    State(ctx): State<Arc<InboxService>>,
    Json(request): Json<CreateThreadRequest>,
) -> JsonResponse<CreateThreadResponse, CommonError> {
    trace!("Creating thread");
    let res = create_thread(&ctx.repository, &ctx.event_bus, request).await;
    trace!(success = res.is_ok(), "Creating thread completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/thread/{{thread_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("thread_id" = WrappedUuidV4, Path, description = "Thread ID"),
        PaginationRequest,
    ),
    responses(
        (status = 200, description = "Get thread with messages", body = GetThreadWithMessagesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get thread with messages",
    description = "Retrieve a thread and its messages with pagination",
    operation_id = "get-thread-with-messages",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_thread_with_messages(
    State(ctx): State<Arc<InboxService>>,
    Path(thread_id): Path<WrappedUuidV4>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<GetThreadWithMessagesResponse, CommonError> {
    trace!(thread_id = %thread_id, "Getting thread with messages");
    let res = get_thread_with_messages(&ctx.repository, thread_id, pagination).await;
    trace!(success = res.is_ok(), "Getting thread with messages completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/thread/{{thread_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("thread_id" = WrappedUuidV4, Path, description = "Thread ID"),
    ),
    request_body = UpdateThreadRequest,
    responses(
        (status = 200, description = "Update thread", body = UpdateThreadResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update thread",
    description = "Update an existing thread's title or metadata",
    operation_id = "update-thread",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_update_thread(
    State(ctx): State<Arc<InboxService>>,
    Path(thread_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateThreadRequest>,
) -> JsonResponse<UpdateThreadResponse, CommonError> {
    trace!(thread_id = %thread_id, "Updating thread");
    let res = update_thread(&ctx.repository, &ctx.event_bus, thread_id, request).await;
    trace!(success = res.is_ok(), "Updating thread completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/thread/{{thread_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("thread_id" = WrappedUuidV4, Path, description = "Thread ID"),
    ),
    responses(
        (status = 200, description = "Delete thread", body = DeleteThreadResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete thread",
    description = "Delete a thread and all its messages",
    operation_id = "delete-thread",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_delete_thread(
    State(ctx): State<Arc<InboxService>>,
    Path(thread_id): Path<WrappedUuidV4>,
) -> JsonResponse<DeleteThreadResponse, CommonError> {
    trace!(thread_id = %thread_id, "Deleting thread");
    let res = delete_thread(&ctx.repository, &ctx.event_bus, thread_id).await;
    trace!(success = res.is_ok(), "Deleting thread completed");
    JsonResponse::from(res)
}

// --- Logic Functions ---

/// List threads with pagination
async fn list_threads<R: ThreadRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListThreadsResponse, CommonError> {
    let paginated = repository.get_threads(&pagination).await?;
    Ok(ListThreadsResponse {
        threads: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// Create a new thread
async fn create_thread<R: ThreadRepositoryLike>(
    repository: &R,
    event_bus: &crate::logic::event::EventBus,
    request: CreateThreadRequest,
) -> Result<CreateThreadResponse, CommonError> {
    let now = WrappedChronoDateTime::now();
    let id = request.id.unwrap_or_default();

    let inbox_settings_json = WrappedJsonValue::new(serde_json::to_value(&request.inbox_settings)
        .map_err(|e| CommonError::InvalidRequest {
            msg: format!("Failed to serialize inbox_settings: {e}"),
            source: Some(e.into()),
        })?);

    let thread = Thread {
        id: id.clone(),
        title: request.title.clone(),
        metadata: request.metadata.clone(),
        inbox_settings: request.inbox_settings.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateThread {
        id,
        title: request.title,
        metadata: request.metadata,
        inbox_settings: inbox_settings_json,
        created_at: now,
        updated_at: now,
    };

    repository.create_thread(&create_params).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::thread_created(thread.clone()));

    Ok(thread)
}

/// Get a thread with its messages
async fn get_thread_with_messages<R: ThreadRepositoryLike + MessageRepositoryLike>(
    repository: &R,
    thread_id: WrappedUuidV4,
    pagination: PaginationRequest,
) -> Result<GetThreadWithMessagesResponse, CommonError> {
    let thread = repository.get_thread_by_id(&thread_id).await?;
    let thread = thread.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {thread_id} not found"),
        lookup_id: thread_id.to_string(),
        source: None,
    })?;

    let messages = repository
        .get_messages_by_thread(&thread_id, &pagination)
        .await?;

    Ok(GetThreadWithMessagesResponse {
        thread,
        messages: messages.items,
        next_page_token: messages.next_page_token,
    })
}

/// Update an existing thread
async fn update_thread<R: ThreadRepositoryLike>(
    repository: &R,
    event_bus: &crate::logic::event::EventBus,
    thread_id: WrappedUuidV4,
    request: UpdateThreadRequest,
) -> Result<UpdateThreadResponse, CommonError> {
    let existing = repository.get_thread_by_id(&thread_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {thread_id} not found"),
        lookup_id: thread_id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();
    let new_title = request.title.or(existing.title.clone());
    let new_metadata = request.metadata.or(existing.metadata.clone());
    let new_inbox_settings = request.inbox_settings.unwrap_or(existing.inbox_settings.clone());

    let inbox_settings_json = WrappedJsonValue::new(serde_json::to_value(&new_inbox_settings)
        .map_err(|e| CommonError::InvalidRequest {
            msg: format!("Failed to serialize inbox_settings: {e}"),
            source: Some(e.into()),
        })?);

    let update_params = UpdateThread {
        id: thread_id.clone(),
        title: new_title.clone(),
        metadata: new_metadata.clone(),
        inbox_settings: inbox_settings_json,
        updated_at: now,
    };

    repository.update_thread(&update_params).await?;

    let updated_thread = Thread {
        id: thread_id,
        title: new_title,
        metadata: new_metadata,
        inbox_settings: new_inbox_settings,
        created_at: existing.created_at,
        updated_at: now,
    };

    // Publish event
    let _ = event_bus.publish(InboxEvent::thread_updated(updated_thread.clone()));

    Ok(updated_thread)
}

/// Delete a thread
async fn delete_thread<R: ThreadRepositoryLike + MessageRepositoryLike>(
    repository: &R,
    event_bus: &crate::logic::event::EventBus,
    thread_id: WrappedUuidV4,
) -> Result<DeleteThreadResponse, CommonError> {
    // Verify thread exists
    let existing = repository.get_thread_by_id(&thread_id).await?;
    let _ = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {thread_id} not found"),
        lookup_id: thread_id.to_string(),
        source: None,
    })?;

    // Delete all messages in the thread first (cascade should handle this, but be explicit)
    repository.delete_messages_by_thread(&thread_id).await?;

    // Delete the thread
    repository.delete_thread(&thread_id).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::thread_deleted(thread_id));

    Ok(DeleteThreadResponse { success: true })
}
