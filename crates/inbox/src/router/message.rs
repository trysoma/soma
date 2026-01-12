//! Message HTTP endpoints

use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::message::{
    create_message, delete_message, get_message, list_messages, update_message,
    CreateMessageRequest, CreateMessageResponse, DeleteMessageResponse, GetMessageResponse,
    ListMessagesResponse, UpdateMessageRequest, UpdateMessageResponse,
};
use crate::service::InboxService;
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
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
    let thread_id = match &request {
        CreateMessageRequest::Text(r) => &r.thread_id,
        CreateMessageRequest::Ui(r) => &r.thread_id,
    };
    trace!(thread_id = %thread_id, "Creating message");
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
    description = "Update an existing message's body or metadata",
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
