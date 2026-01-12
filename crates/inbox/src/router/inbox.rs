//! Inbox HTTP endpoints
//!
//! Endpoints for managing inbox provider instances including creating, updating,
//! deleting, and listing inboxes. Also includes nested routes for
//! provider-specific functionality.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::Request,
    response::IntoResponse,
    routing::any,
    Json, Router,
};
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tower::ServiceExt;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::logic::inbox::{
    create_inbox, delete_inbox, get_inbox, get_provider_registry, list_inboxes, update_inbox,
    CreateInboxRequest, CreateInboxResponse, DeleteInboxResponse, GetInboxResponse,
    InboxProviderState, ListInboxesResponse, ListProvidersResponse, ProviderInfo,
    UpdateInboxRequest, UpdateInboxResponse,
};
use crate::repository::InboxRepositoryLike;
use crate::service::InboxService;
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::PaginationRequest,
};

/// Create the inbox router
pub fn create_router() -> OpenApiRouter<Arc<InboxService>> {
    OpenApiRouter::new()
        .routes(routes!(route_list_providers))
        .routes(routes!(route_list_inboxes))
        .routes(routes!(route_create_inbox))
        .routes(routes!(route_get_inbox))
        .routes(routes!(route_update_inbox))
        .routes(routes!(route_delete_inbox))
}

/// Create the nested router for provider-specific routes
/// This is called separately and mounted at /inbox/v1/inbox/{inbox_id}/*
pub fn create_nested_inbox_router() -> Router<Arc<InboxService>> {
    Router::new().route(
        &format!("{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/{API_VERSION_1}/inbox/{{inbox_id}}/*path"),
        any(handle_nested_inbox_route),
    )
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/inbox/provider", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    responses(
        (status = 200, description = "List registered inbox providers", body = ListProvidersResponse),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List inbox providers",
    description = "List all registered inbox providers with their configuration schemas",
    operation_id = "list-inbox-providers",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_providers(
    State(_ctx): State<Arc<InboxService>>,
) -> JsonResponse<ListProvidersResponse, CommonError> {
    trace!("Listing inbox providers");
    let registry = get_provider_registry();
    let providers: Vec<ProviderInfo> = registry
        .list()
        .iter()
        .map(|p| ProviderInfo {
            id: p.id().to_string(),
            title: p.title().to_string(),
            description: p.description().to_string(),
            configuration_schema: p.configuration_schema(),
        })
        .collect();

    trace!(count = providers.len(), "Listed inbox providers");
    JsonResponse::from(Ok(ListProvidersResponse { providers }))
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/inbox", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(PaginationRequest),
    responses(
        (status = 200, description = "List inboxes", body = ListInboxesResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List inboxes",
    description = "List all configured inbox instances with pagination",
    operation_id = "list-inboxes",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_list_inboxes(
    State(ctx): State<Arc<InboxService>>,
    Query(pagination): Query<PaginationRequest>,
) -> JsonResponse<ListInboxesResponse, CommonError> {
    trace!(page_size = pagination.page_size, "Listing inboxes");
    let res = list_inboxes(&ctx.repository, pagination).await;
    trace!(success = res.is_ok(), "Listing inboxes completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/inbox", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CreateInboxRequest,
    responses(
        (status = 200, description = "Create an inbox", body = CreateInboxResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create inbox",
    description = "Create a new inbox instance from a registered provider",
    operation_id = "create-inbox",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_create_inbox(
    State(ctx): State<Arc<InboxService>>,
    Json(request): Json<CreateInboxRequest>,
) -> JsonResponse<CreateInboxResponse, CommonError> {
    trace!(provider_id = %request.provider_id, "Creating inbox");
    let res = create_inbox(&ctx.repository, ctx.config_change_tx.as_ref(), request).await;
    trace!(success = res.is_ok(), "Creating inbox completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    get,
    path = format!("{}/{}/{}/inbox/{{inbox_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("inbox_id" = String, Path, description = "Inbox ID"),
    ),
    responses(
        (status = 200, description = "Get inbox", body = GetInboxResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get inbox",
    description = "Retrieve an inbox by its ID",
    operation_id = "get-inbox",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_get_inbox(
    State(ctx): State<Arc<InboxService>>,
    Path(inbox_id): Path<String>,
) -> JsonResponse<GetInboxResponse, CommonError> {
    trace!(inbox_id = %inbox_id, "Getting inbox");
    let res = get_inbox(&ctx.repository, &inbox_id).await;
    trace!(success = res.is_ok(), "Getting inbox completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    put,
    path = format!("{}/{}/{}/inbox/{{inbox_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("inbox_id" = String, Path, description = "Inbox ID"),
    ),
    request_body = UpdateInboxRequest,
    responses(
        (status = 200, description = "Update inbox", body = UpdateInboxResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Update inbox",
    description = "Update an existing inbox's configuration or settings",
    operation_id = "update-inbox",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_update_inbox(
    State(ctx): State<Arc<InboxService>>,
    Path(inbox_id): Path<String>,
    Json(request): Json<UpdateInboxRequest>,
) -> JsonResponse<UpdateInboxResponse, CommonError> {
    trace!(inbox_id = %inbox_id, "Updating inbox");
    let res = update_inbox(&ctx.repository, ctx.config_change_tx.as_ref(), &inbox_id, request).await;
    trace!(success = res.is_ok(), "Updating inbox completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/{}/inbox/{{inbox_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("inbox_id" = String, Path, description = "Inbox ID"),
    ),
    responses(
        (status = 200, description = "Delete inbox", body = DeleteInboxResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete inbox",
    description = "Delete an inbox by its ID",
    operation_id = "delete-inbox",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_delete_inbox(
    State(ctx): State<Arc<InboxService>>,
    Path(inbox_id): Path<String>,
) -> JsonResponse<DeleteInboxResponse, CommonError> {
    trace!(inbox_id = %inbox_id, "Deleting inbox");
    let res = delete_inbox(&ctx.repository, ctx.config_change_tx.as_ref(), &inbox_id).await;
    trace!(success = res.is_ok(), "Deleting inbox completed");
    JsonResponse::from(res)
}

/// Handle nested inbox routes by delegating to the provider's router
async fn handle_nested_inbox_route(
    State(ctx): State<Arc<InboxService>>,
    Path((inbox_id, path)): Path<(String, String)>,
    request: Request<Body>,
) -> impl IntoResponse {
    trace!(inbox_id = %inbox_id, path = %path, "Handling nested inbox route");

    // Get the inbox
    let inbox = match ctx.repository.get_inbox_by_id(&inbox_id).await {
        Ok(Some(inbox)) => inbox,
        Ok(None) => {
            return CommonError::NotFound {
                msg: format!("Inbox with id {inbox_id} not found"),
                lookup_id: inbox_id,
                source: None,
            }
            .into_response();
        }
        Err(e) => return e.into_response(),
    };

    // Get the provider
    let registry = get_provider_registry();
    let provider = match registry.get(&inbox.provider_id) {
        Some(provider) => provider,
        None => {
            return CommonError::InvalidRequest {
                msg: format!("Provider {} not found for inbox {inbox_id}", inbox.provider_id),
                source: None,
            }
            .into_response();
        }
    };

    // Create the inbox handle and provider state
    let handle = ctx.create_inbox_handle(inbox.clone());
    let provider_state = InboxProviderState {
        inbox,
        handle,
        repository: Some(Arc::new(ctx.repository.clone())),
        event_bus: Some(ctx.event_bus.clone()),
    };

    // Get the provider's router and handle the request
    let (router, _) = provider.router().split_for_parts();
    let router = router.with_state(provider_state);

    // Forward the request to the provider's router using oneshot
    match router.oneshot(request).await {
        Ok(response) => response,
        Err(err) => {
            // Infallible error - this branch should never be reached
            // but we need to handle it for the type system
            match err {}
        }
    }
}
