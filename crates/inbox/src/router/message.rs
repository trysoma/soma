//! Message HTTP endpoints

use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::{
    logic::{
        event::InboxEvent,
        message::{
            CreateMessageRequest, CreateMessageResponse, DeleteMessageResponse,
            GetMessageResponse, ListMessagesResponse, UIMessage, UpdateMessageRequest,
            UpdateMessageResponse,
        },
    },
    repository::{CreateMessage, MessageRepositoryLike, ThreadRepositoryLike, UpdateMessage},
    service::InboxService,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
};

/// Create the message router
pub fn create_router() -> OpenApiRouter<Arc<InboxService>> {
    OpenApiRouter::new()
        .routes(routes!(route_list_messages))
        .routes(routes!(route_create_message))
        .routes(routes!(route_get_message))
        .routes(routes!(route_update_message))
        .routes(routes!(route_delete_message))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/message", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(PaginationRequest),
    responses(
        (status = 200, description = "List messages", body = ListMessagesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List messages",
    description = "List all messages with pagination",
    operation_id = "list-messages",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_messages(
    State(ctx): State<Arc<InboxService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListMessagesResponse, CommonError> {
    trace!(page_size = pagination.page_size, "Listing messages");
    let res = list_messages(&ctx.repository, pagination).await;
    trace!(success = res.is_ok(), "Listing messages completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/message", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateMessageRequest,
    responses(
        (status = 200, description = "Create a message", body = CreateMessageResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Thread Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create message",
    description = "Create a new message in a thread",
    operation_id = "create-message",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_create_message(
    State(ctx): State<Arc<InboxService>>,
    Json(request): Json<CreateMessageRequest>,
) -> JsonResponse<CreateMessageResponse, CommonError> {
    trace!(thread_id = %request.thread_id, "Creating message");
    let res = create_message(&ctx.repository, &ctx.event_bus, request).await;
    trace!(success = res.is_ok(), "Creating message completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/message/{{message_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("message_id" = WrappedUuidV4, Path, description = "Message ID"),
    ),
    responses(
        (status = 200, description = "Get message", body = GetMessageResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get message",
    description = "Retrieve a message by its ID",
    operation_id = "get-message",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_message(
    State(ctx): State<Arc<InboxService>>,
    Path(message_id): Path<WrappedUuidV4>,
) -> JsonResponse<GetMessageResponse, CommonError> {
    trace!(message_id = %message_id, "Getting message");
    let res = get_message(&ctx.repository, message_id).await;
    trace!(success = res.is_ok(), "Getting message completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/message/{{message_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("message_id" = WrappedUuidV4, Path, description = "Message ID"),
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Update message", body = UpdateMessageResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update message",
    description = "Update an existing message's parts or metadata",
    operation_id = "update-message",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_update_message(
    State(ctx): State<Arc<InboxService>>,
    Path(message_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateMessageRequest>,
) -> JsonResponse<UpdateMessageResponse, CommonError> {
    trace!(message_id = %message_id, "Updating message");
    let res = update_message(&ctx.repository, &ctx.event_bus, message_id, request).await;
    trace!(success = res.is_ok(), "Updating message completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/message/{{message_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("message_id" = WrappedUuidV4, Path, description = "Message ID"),
    ),
    responses(
        (status = 200, description = "Delete message", body = DeleteMessageResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete message",
    description = "Delete a message by its ID",
    operation_id = "delete-message",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_delete_message(
    State(ctx): State<Arc<InboxService>>,
    Path(message_id): Path<WrappedUuidV4>,
) -> JsonResponse<DeleteMessageResponse, CommonError> {
    trace!(message_id = %message_id, "Deleting message");
    let res = delete_message(&ctx.repository, &ctx.event_bus, message_id).await;
    trace!(success = res.is_ok(), "Deleting message completed");
    JsonResponse::from(res)
}

// --- Logic Functions ---

/// List messages with pagination
async fn list_messages<R: MessageRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListMessagesResponse, CommonError> {
    let paginated = repository.get_messages(&pagination).await?;
    Ok(ListMessagesResponse {
        messages: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// Create a new message
async fn create_message<R: MessageRepositoryLike + ThreadRepositoryLike>(
    repository: &R,
    event_bus: &crate::logic::event::EventBus,
    request: CreateMessageRequest,
) -> Result<CreateMessageResponse, CommonError> {
    // Verify thread exists
    let thread = repository.get_thread_by_id(&request.thread_id).await?;
    let _ = thread.ok_or_else(|| CommonError::NotFound {
        msg: format!("Thread with id {} not found", request.thread_id),
        lookup_id: request.thread_id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();
    let id = WrappedUuidV4::new();

    let parts_json = WrappedJsonValue::new(serde_json::to_value(&request.parts).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Failed to serialize parts: {e}"),
            source: Some(e.into()),
        }
    })?);

    let inbox_settings_json =
        WrappedJsonValue::new(serde_json::to_value(&request.inbox_settings).map_err(|e| {
            CommonError::InvalidRequest {
                msg: format!("Failed to serialize inbox_settings: {e}"),
                source: Some(e.into()),
            }
        })?);

    let message = UIMessage {
        id: id.clone(),
        thread_id: request.thread_id.clone(),
        role: request.role.clone(),
        parts: request.parts.clone(),
        metadata: request.metadata.clone(),
        inbox_settings: request.inbox_settings.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateMessage {
        id,
        thread_id: request.thread_id,
        role: request.role,
        parts: parts_json,
        metadata: request.metadata,
        inbox_settings: inbox_settings_json,
        created_at: now,
        updated_at: now,
    };

    repository.create_message(&create_params).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::message_created(message.clone()));

    Ok(message)
}

/// Get a message by ID
async fn get_message<R: MessageRepositoryLike>(
    repository: &R,
    message_id: WrappedUuidV4,
) -> Result<GetMessageResponse, CommonError> {
    let message = repository.get_message_by_id(&message_id).await?;
    message.ok_or_else(|| CommonError::NotFound {
        msg: format!("Message with id {message_id} not found"),
        lookup_id: message_id.to_string(),
        source: None,
    })
}

/// Update an existing message
async fn update_message<R: MessageRepositoryLike>(
    repository: &R,
    event_bus: &crate::logic::event::EventBus,
    message_id: WrappedUuidV4,
    request: UpdateMessageRequest,
) -> Result<UpdateMessageResponse, CommonError> {
    let existing = repository.get_message_by_id(&message_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Message with id {message_id} not found"),
        lookup_id: message_id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();
    let new_parts = request.parts.unwrap_or(existing.parts.clone());
    let new_metadata = request.metadata.or(existing.metadata.clone());
    let new_inbox_settings = request
        .inbox_settings
        .unwrap_or(existing.inbox_settings.clone());

    let parts_json = WrappedJsonValue::new(serde_json::to_value(&new_parts).map_err(|e| {
        CommonError::InvalidRequest {
            msg: format!("Failed to serialize parts: {e}"),
            source: Some(e.into()),
        }
    })?);

    let inbox_settings_json =
        WrappedJsonValue::new(serde_json::to_value(&new_inbox_settings).map_err(|e| {
            CommonError::InvalidRequest {
                msg: format!("Failed to serialize inbox_settings: {e}"),
                source: Some(e.into()),
            }
        })?);

    let update_params = UpdateMessage {
        id: message_id.clone(),
        parts: parts_json,
        metadata: new_metadata.clone(),
        inbox_settings: inbox_settings_json,
        updated_at: now,
    };

    repository.update_message(&update_params).await?;

    let updated_message = UIMessage {
        id: message_id,
        thread_id: existing.thread_id,
        role: existing.role,
        parts: new_parts,
        metadata: new_metadata,
        inbox_settings: new_inbox_settings,
        created_at: existing.created_at,
        updated_at: now,
    };

    // Publish event
    let _ = event_bus.publish(InboxEvent::message_updated(updated_message.clone()));

    Ok(updated_message)
}

/// Delete a message
async fn delete_message<R: MessageRepositoryLike>(
    repository: &R,
    event_bus: &crate::logic::event::EventBus,
    message_id: WrappedUuidV4,
) -> Result<DeleteMessageResponse, CommonError> {
    // Verify message exists
    let existing = repository.get_message_by_id(&message_id).await?;
    let _ = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Message with id {message_id} not found"),
        lookup_id: message_id.to_string(),
        source: None,
    })?;

    repository.delete_message(&message_id).await?;

    // Publish event
    let _ = event_bus.publish(InboxEvent::message_deleted(message_id));

    Ok(DeleteMessageResponse { success: true })
}
