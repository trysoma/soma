use axum::extract::{Json, Path, Query, State};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    logic::task::{
        ConnectionManager, CreateMessageRequest, CreateMessageResponse, GetTaskResponse,
        GetTaskTimelineItemsResponse, ListTasksResponse, ListUniqueContextsResponse,
        UpdateTaskStatusRequest, UpdateTaskStatusResponse, WithContextId, WithTaskId,
        create_message, get_task, get_task_timeline_items, list_tasks, list_tasks_by_context_id,
        list_unique_contexts, update_task_status,
    },
    repository::Repository,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedUuidV4},
};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "task";

pub fn create_router() -> OpenApiRouter<Arc<TaskService>> {
    OpenApiRouter::new()
        .routes(routes!(route_list_tasks))
        .routes(routes!(route_list_contexts))
        .routes(routes!(route_list_tasks_by_context_id))
        .routes(routes!(route_get_task))
        .routes(routes!(route_update_task_status))
        .routes(routes!(route_create_message))
        .routes(routes!(route_get_task_timeline_items))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List tasks", body = ListTasksResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "List tasks",
    description = "List all tasks with pagination",
    operation_id = "list-tasks",
)]
async fn route_list_tasks(
    State(ctx): State<Arc<TaskService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListTasksResponse, CommonError> {
    let res = list_tasks(&ctx.repository, pagination).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/context", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest
    ),
    responses(
        (status = 200, description = "List contexts", body = ListUniqueContextsResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "List contexts",
    description = "List all unique task contexts with pagination",
    operation_id = "list-contexts",
)]
async fn route_list_contexts(
    State(ctx): State<Arc<TaskService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListUniqueContextsResponse, CommonError> {
    let res = list_unique_contexts(&ctx.repository, pagination).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/context/{{context_id}}/task", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest,
        ("context_id" = WrappedUuidV4, Path, description = "Context ID"),
    ),
    responses(
        (status = 200, description = "List tasks", body = ListTasksResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "List tasks by context",
    description = "List all tasks for a specific context ID with pagination",
    operation_id = "list-tasks-by-context-id",
)]
async fn route_list_tasks_by_context_id(
    State(ctx): State<Arc<TaskService>>,
    Query(pagination): Query<PaginationRequest>,
    Path(context_id): Path<WrappedUuidV4>,
) -> JsonResponse<ListTasksResponse, CommonError> {
    let res = list_tasks_by_context_id(
        &ctx.repository,
        WithContextId {
            context_id,
            inner: pagination,
        },
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/{{task_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("task_id" = WrappedUuidV4, Path, description = "Task ID"),
    ),
    responses(
        (status = 200, description = "Get task by id", body = GetTaskResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Get task",
    description = "Retrieve a task by its unique identifier",
    operation_id = "get-task-by-id",
)]
async fn route_get_task(
    State(ctx): State<Arc<TaskService>>,
    Path(task_id): Path<WrappedUuidV4>,
) -> JsonResponse<GetTaskResponse, CommonError> {
    let res = get_task(&ctx.repository, task_id).await;
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/{{task_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("task_id" = WrappedUuidV4, Path, description = "Task ID"),
    ),
    request_body = UpdateTaskStatusRequest,
    responses(
        (status = 200, description = "Update task status", body = UpdateTaskStatusResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Update task status",
    description = "Update the status of a task",
    operation_id = "update-task-status",
)]
async fn route_update_task_status(
    State(ctx): State<Arc<TaskService>>,
    Path(task_id): Path<WrappedUuidV4>,
    Json(request): Json<UpdateTaskStatusRequest>,
) -> JsonResponse<UpdateTaskStatusResponse, CommonError> {
    let res = update_task_status(
        &ctx.repository,
        &ctx.connection_manager,
        None,
        WithTaskId {
            task_id,
            inner: request,
        },
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/{{task_id}}/message", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("task_id" = WrappedUuidV4, Path, description = "Task ID"),
    ),
    request_body = CreateMessageRequest,
    responses(
        (status = 200, description = "Create message", body = CreateMessageResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Send message",
    description = "Send a message to a task",
    operation_id = "send-message",
)]
async fn route_create_message(
    State(ctx): State<Arc<TaskService>>,
    Path(task_id): Path<WrappedUuidV4>,
    Json(request): Json<CreateMessageRequest>,
) -> JsonResponse<CreateMessageResponse, CommonError> {
    let res = create_message(
        &ctx.repository,
        &ctx.connection_manager,
        WithTaskId {
            task_id,
            inner: request,
        },
        false,
    )
    .await;
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/{{task_id}}/timeline", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        PaginationRequest,
        ("task_id" = WrappedUuidV4, Path, description = "Task ID"),
    ),
    responses(
        (status = 200, description = "Get task timeline items", body = GetTaskTimelineItemsResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 401, description = "Unauthorized", body = CommonError),
        (status = 403, description = "Forbidden", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
        (status = 502, description = "Bad Gateway", body = CommonError),
    ),
    summary = "Get task timeline",
    description = "Get the timeline history of a task with pagination",
    operation_id = "task-history",
)]
async fn route_get_task_timeline_items(
    State(ctx): State<Arc<TaskService>>,
    Path(task_id): Path<WrappedUuidV4>,
    Query(request): Query<PaginationRequest>,
) -> JsonResponse<GetTaskTimelineItemsResponse, CommonError> {
    let res = get_task_timeline_items(
        &ctx.repository,
        WithTaskId {
            task_id,
            inner: request,
        },
    )
    .await;
    JsonResponse::from(res)
}

pub struct TaskService {
    repository: Repository,
    connection_manager: ConnectionManager,
}

impl TaskService {
    pub fn new(connection_manager: ConnectionManager, repository: Repository) -> Self {
        Self {
            connection_manager,
            repository,
        }
    }
}
