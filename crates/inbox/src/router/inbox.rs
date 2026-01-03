//! Inbox HTTP endpoints
//!
//! Endpoints for managing inbox provider instances including creating, updating,
//! enabling/disabling, and listing inboxes. Also includes nested routes for
//! provider-specific functionality.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::Request,
    response::IntoResponse,
    routing::any,
    Json, Router,
};
use tower::ServiceExt;
use shared::adapters::openapi::API_VERSION_TAG;
use std::sync::Arc;
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{API_VERSION_1, PATH_PREFIX, SERVICE_ROUTE_KEY};
use crate::{
    logic::inbox::{
        CreateInboxRequest, CreateInboxResponse, DeleteInboxResponse, GetInboxResponse, Inbox,
        InboxProviderState, InboxStatus, ListInboxesResponse, ListProvidersResponse, ProviderInfo,
        SetInboxStatusRequest, UpdateInboxRequest, UpdateInboxResponse, get_provider_registry,
    },
    repository::{CreateInbox, InboxRepositoryLike, UpdateInbox, UpdateInboxStatus},
    service::InboxService,
};
use shared::{
    adapters::openapi::JsonResponse,
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
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
        .routes(routes!(route_set_inbox_status))
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
    let res = create_inbox(&ctx.repository, request).await;
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
    let res = update_inbox(&ctx.repository, &inbox_id, request).await;
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
    let res = delete_inbox(&ctx.repository, &inbox_id).await;
    trace!(success = res.is_ok(), "Deleting inbox completed");
    JsonResponse::from(res)
}

#[utoipa::path(
    post,
    path = format!("{}/{}/{}/inbox/{{inbox_id}}/status", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("inbox_id" = String, Path, description = "Inbox ID"),
    ),
    request_body = SetInboxStatusRequest,
    responses(
        (status = 200, description = "Update inbox status", body = Inbox),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 404, description = "Not Found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Set inbox status",
    description = "Enable or disable an inbox",
    operation_id = "set-inbox-status",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
async fn route_set_inbox_status(
    State(ctx): State<Arc<InboxService>>,
    Path(inbox_id): Path<String>,
    Json(request): Json<SetInboxStatusRequest>,
) -> JsonResponse<Inbox, CommonError> {
    trace!(inbox_id = %inbox_id, status = %request.status, "Setting inbox status");
    let res = set_inbox_status(&ctx.repository, &inbox_id, request.status).await;
    trace!(success = res.is_ok(), "Setting inbox status completed");
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

    // Check if inbox is enabled
    if !inbox.is_enabled() {
        return CommonError::InvalidRequest {
            msg: format!("Inbox {inbox_id} is disabled"),
            source: None,
        }
        .into_response();
    }

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

    // Create the provider state
    let provider_state = InboxProviderState {
        inbox,
        event_bus: ctx.event_bus.clone(),
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

// --- Logic Functions ---

/// List inboxes with pagination
async fn list_inboxes<R: InboxRepositoryLike>(
    repository: &R,
    pagination: PaginationRequest,
) -> Result<ListInboxesResponse, CommonError> {
    let paginated = repository.get_inboxes(&pagination).await?;
    Ok(ListInboxesResponse {
        inboxes: paginated.items,
        next_page_token: paginated.next_page_token,
    })
}

/// Create a new inbox
async fn create_inbox<R: InboxRepositoryLike>(
    repository: &R,
    request: CreateInboxRequest,
) -> Result<CreateInboxResponse, CommonError> {
    // Verify provider exists
    let registry = get_provider_registry();
    let provider = registry.get(&request.provider_id).ok_or_else(|| {
        CommonError::InvalidRequest {
            msg: format!("Provider {} not found", request.provider_id),
            source: None,
        }
    })?;

    // Validate configuration against provider's schema
    provider.validate_configuration(request.configuration.get_inner())?;

    let now = WrappedChronoDateTime::now();
    let id = request
        .id
        .unwrap_or_else(|| format!("inbox-{}", WrappedUuidV4::new()));

    // Check if inbox with this ID already exists
    if repository.get_inbox_by_id(&id).await?.is_some() {
        return Err(CommonError::InvalidRequest {
            msg: format!("Inbox with id {id} already exists"),
            source: None,
        });
    }

    let settings_json = WrappedJsonValue::new(
        serde_json::to_value(&request.settings).map_err(|e| CommonError::InvalidRequest {
            msg: format!("Failed to serialize settings: {e}"),
            source: Some(e.into()),
        })?,
    );

    let inbox = Inbox {
        id: id.clone(),
        provider_id: request.provider_id.clone(),
        status: InboxStatus::Enabled,
        configuration: request.configuration.clone(),
        settings: request.settings.clone(),
        created_at: now,
        updated_at: now,
    };

    let create_params = CreateInbox {
        id,
        provider_id: request.provider_id,
        status: InboxStatus::Enabled,
        configuration: request.configuration,
        settings: settings_json,
        created_at: now,
        updated_at: now,
    };

    repository.create_inbox(&create_params).await?;

    Ok(inbox)
}

/// Get an inbox by ID
async fn get_inbox<R: InboxRepositoryLike>(
    repository: &R,
    inbox_id: &str,
) -> Result<GetInboxResponse, CommonError> {
    let inbox = repository.get_inbox_by_id(inbox_id).await?;
    inbox.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })
}

/// Update an existing inbox
async fn update_inbox<R: InboxRepositoryLike>(
    repository: &R,
    inbox_id: &str,
    request: UpdateInboxRequest,
) -> Result<UpdateInboxResponse, CommonError> {
    let existing = repository.get_inbox_by_id(inbox_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })?;

    // If configuration is being updated, validate it
    let new_configuration = if let Some(config) = request.configuration {
        let registry = get_provider_registry();
        if let Some(provider) = registry.get(&existing.provider_id) {
            provider.validate_configuration(config.get_inner())?;
        }
        config
    } else {
        existing.configuration.clone()
    };

    let new_settings = request.settings.unwrap_or(existing.settings.clone());
    let now = WrappedChronoDateTime::now();

    let settings_json = WrappedJsonValue::new(
        serde_json::to_value(&new_settings).map_err(|e| CommonError::InvalidRequest {
            msg: format!("Failed to serialize settings: {e}"),
            source: Some(e.into()),
        })?,
    );

    let update_params = UpdateInbox {
        id: inbox_id.to_string(),
        configuration: new_configuration.clone(),
        settings: settings_json,
        updated_at: now,
    };

    repository.update_inbox(&update_params).await?;

    Ok(Inbox {
        id: inbox_id.to_string(),
        provider_id: existing.provider_id,
        status: existing.status,
        configuration: new_configuration,
        settings: new_settings,
        created_at: existing.created_at,
        updated_at: now,
    })
}

/// Delete an inbox
async fn delete_inbox<R: InboxRepositoryLike>(
    repository: &R,
    inbox_id: &str,
) -> Result<DeleteInboxResponse, CommonError> {
    // Verify inbox exists
    let existing = repository.get_inbox_by_id(inbox_id).await?;
    let _ = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })?;

    repository.delete_inbox(inbox_id).await?;

    Ok(DeleteInboxResponse { success: true })
}

/// Set inbox status (enable/disable)
async fn set_inbox_status<R: InboxRepositoryLike>(
    repository: &R,
    inbox_id: &str,
    status: InboxStatus,
) -> Result<Inbox, CommonError> {
    let existing = repository.get_inbox_by_id(inbox_id).await?;
    let existing = existing.ok_or_else(|| CommonError::NotFound {
        msg: format!("Inbox with id {inbox_id} not found"),
        lookup_id: inbox_id.to_string(),
        source: None,
    })?;

    let now = WrappedChronoDateTime::now();

    let update_params = UpdateInboxStatus {
        id: inbox_id.to_string(),
        status: status.clone(),
        updated_at: now,
    };

    repository.update_inbox_status(&update_params).await?;

    Ok(Inbox {
        id: inbox_id.to_string(),
        provider_id: existing.provider_id,
        status,
        configuration: existing.configuration,
        settings: existing.settings,
        created_at: existing.created_at,
        updated_at: now,
    })
}
