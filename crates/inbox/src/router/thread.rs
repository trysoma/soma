//! Thread HTTP endpoints

use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::thread::{
    create_thread, delete_thread, get_thread_with_messages, list_threads, update_thread,
    CreateThreadRequest, CreateThreadResponse, DeleteThreadResponse, GetThreadWithMessagesResponse,
    ListThreadsResponse, UpdateThreadRequest, UpdateThreadResponse,
};
use crate::service::InboxService;
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
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
