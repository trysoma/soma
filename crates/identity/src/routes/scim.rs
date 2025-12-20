//! SCIM 2.0 HTTP endpoints for user and group provisioning
//!
//! Implements the SCIM 2.0 protocol endpoints for:
//! - /Users - User CRUD operations
//! - /Groups - Group CRUD operations

use crate::logic::scim::{
    ScimGroup, ScimGroupListResponse, ScimListParams, ScimPatchRequest, ScimUser,
    ScimUserListResponse, create_group_from_scim, create_user_from_scim, delete_group_scim,
    delete_user_scim, get_group_scim, get_user_scim, list_groups_scim, list_users_scim,
    patch_group_scim, patch_user_scim, replace_group_scim, replace_user_scim,
};
use crate::routes::{IdentityService, PATH_PREFIX, SERVICE_ROUTE_KEY};
use axum::extract::{Json, Path, Query, State};
use http::{HeaderMap, StatusCode};
use shared::identity::Identity;
use shared::{adapters::openapi::API_VERSION_TAG, error::CommonError};
use tracing::trace;
use utoipa_axum::{router::OpenApiRouter, routes};

pub const SCIM_VERSION: &str = "v2";

/// Create the SCIM router with all SCIM endpoints
pub fn create_scim_router() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        // User endpoints
        .routes(routes!(route_list_users))
        .routes(routes!(route_create_user))
        .routes(routes!(route_get_user))
        .routes(routes!(route_replace_user))
        .routes(routes!(route_patch_user))
        .routes(routes!(route_delete_user))
        // Group endpoints
        .routes(routes!(route_list_groups))
        .routes(routes!(route_create_group))
        .routes(routes!(route_get_group))
        .routes(routes!(route_replace_group))
        .routes(routes!(route_patch_group))
        .routes(routes!(route_delete_group))
}

// ============================================================================
// Helper function to get base URL for SCIM resources
// ============================================================================

fn get_scim_base_url() -> String {
    // In production, this should be configured via environment variable
    std::env::var("SCIM_BASE_URL")
        .unwrap_or_else(|_| format!("{PATH_PREFIX}/{SERVICE_ROUTE_KEY}/scim/{SCIM_VERSION}"))
}

// ============================================================================
// SCIM User Endpoints
// ============================================================================

/// SCIM response wrapper that returns 201 for creation
pub struct ScimCreatedResponse<T>(pub T);

impl<T: serde::Serialize> axum::response::IntoResponse for ScimCreatedResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

/// SCIM response for successful operations
pub struct ScimOkResponse<T>(pub T);

impl<T: serde::Serialize> axum::response::IntoResponse for ScimOkResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

/// SCIM response for no content (delete)
pub struct ScimNoContentResponse;

impl axum::response::IntoResponse for ScimNoContentResponse {
    fn into_response(self) -> axum::response::Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

/// SCIM error response
pub struct ScimErrorResponse(pub CommonError);

impl axum::response::IntoResponse for ScimErrorResponse {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

/// Result type for SCIM endpoints
pub type ScimResult<T> = Result<T, ScimErrorResponse>;

#[utoipa::path(
    get,
    path = format!("{}/{}/scim/{}/Users", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(ScimListParams),
    responses(
        (status = 200, description = "List of users", body = ScimUserListResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List SCIM Users",
    description = "List all users with SCIM pagination. Supports filtering by various attributes.",
    operation_id = "scim-list-users",
)]
async fn route_list_users(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Query(params): Query<ScimListParams>,
) -> ScimResult<ScimOkResponse<ScimUserListResponse>> {
    trace!(
        start_index = params.start_index,
        count = params.count,
        "Listing SCIM users"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = list_users_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        params,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Listing SCIM users completed");
    match result {
        Ok(users) => Ok(ScimOkResponse(users)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/scim/{}/Users", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    request_body = ScimUser,
    responses(
        (status = 201, description = "User created", body = ScimUser),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 409, description = "Conflict - User already exists", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create SCIM User",
    description = "Create a new user from a SCIM User payload. The user will be created as a federated_user type.",
    operation_id = "scim-create-user",
)]
async fn route_create_user(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Json(scim_user): Json<ScimUser>,
) -> ScimResult<ScimCreatedResponse<ScimUser>> {
    trace!(user_name = %scim_user.user_name, external_id = ?scim_user.external_id, "Creating SCIM user");
    let identity_placeholder = Identity::Unauthenticated;
    let result = create_user_from_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        scim_user,
    )
    .await;
    trace!(success = result.is_ok(), "Creating SCIM user completed");
    match result {
        Ok(user) => Ok(ScimCreatedResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    get,
    path = format!("{}/{}/scim/{}/Users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User found", body = ScimUser),
        (status = 404, description = "User not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get SCIM User",
    description = "Retrieve a user by their unique identifier in SCIM format.",
    operation_id = "scim-get-user",
)]
async fn route_get_user(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> ScimResult<ScimOkResponse<ScimUser>> {
    trace!(user_id = %user_id, "Getting SCIM user");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = get_user_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &user_id,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Getting SCIM user completed");
    match result {
        Ok(user) => Ok(ScimOkResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    put,
    path = format!("{}/{}/scim/{}/Users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body = ScimUser,
    responses(
        (status = 200, description = "User replaced", body = ScimUser),
        (status = 404, description = "User not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Replace SCIM User",
    description = "Replace a user's attributes with the provided SCIM User payload (PUT operation).",
    operation_id = "scim-replace-user",
)]
async fn route_replace_user(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Json(scim_user): Json<ScimUser>,
) -> ScimResult<ScimOkResponse<ScimUser>> {
    trace!(user_id = %user_id, "Replacing SCIM user");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = replace_user_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &user_id,
        scim_user,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Replacing SCIM user completed");
    match result {
        Ok(user) => Ok(ScimOkResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/scim/{}/Users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body = ScimPatchRequest,
    responses(
        (status = 200, description = "User patched", body = ScimUser),
        (status = 404, description = "User not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Patch SCIM User",
    description = "Partially update a user's attributes using SCIM PATCH operations (add, remove, replace).",
    operation_id = "scim-patch-user",
)]
async fn route_patch_user(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Json(patch_request): Json<ScimPatchRequest>,
) -> ScimResult<ScimOkResponse<ScimUser>> {
    trace!(user_id = %user_id, operation_count = patch_request.operations.len(), "Patching SCIM user");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = patch_user_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &user_id,
        patch_request,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Patching SCIM user completed");
    match result {
        Ok(user) => Ok(ScimOkResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/scim/{}/Users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete SCIM User",
    description = "Delete a user by their unique identifier. This also removes the user from all groups.",
    operation_id = "scim-delete-user",
)]
async fn route_delete_user(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> ScimResult<ScimNoContentResponse> {
    trace!(user_id = %user_id, "Deleting SCIM user");
    let identity_placeholder = Identity::Unauthenticated;
    let result = delete_user_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &user_id,
    )
    .await;
    trace!(success = result.is_ok(), "Deleting SCIM user completed");
    match result {
        Ok(()) => Ok(ScimNoContentResponse),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

// ============================================================================
// SCIM Group Endpoints
// ============================================================================

#[utoipa::path(
    get,
    path = format!("{}/{}/scim/{}/Groups", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(ScimListParams),
    responses(
        (status = 200, description = "List of groups", body = ScimGroupListResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "List SCIM Groups",
    description = "List all groups with SCIM pagination. Supports filtering by various attributes.",
    operation_id = "scim-list-groups",
)]
async fn route_list_groups(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Query(params): Query<ScimListParams>,
) -> ScimResult<ScimOkResponse<ScimGroupListResponse>> {
    trace!(
        start_index = params.start_index,
        count = params.count,
        "Listing SCIM groups"
    );
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = list_groups_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        params,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Listing SCIM groups completed");
    match result {
        Ok(groups) => Ok(ScimOkResponse(groups)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    post,
    path = format!("{}/{}/scim/{}/Groups", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    request_body = ScimGroup,
    responses(
        (status = 201, description = "Group created", body = ScimGroup),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 409, description = "Conflict - Group already exists", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Create SCIM Group",
    description = "Create a new group from a SCIM Group payload. Members can be specified during creation.",
    operation_id = "scim-create-group",
)]
async fn route_create_group(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Json(scim_group): Json<ScimGroup>,
) -> ScimResult<ScimCreatedResponse<ScimGroup>> {
    trace!(display_name = %scim_group.display_name, external_id = ?scim_group.external_id, member_count = scim_group.members.len(), "Creating SCIM group");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = create_group_from_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        scim_group,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Creating SCIM group completed");
    match result {
        Ok(group) => Ok(ScimCreatedResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    get,
    path = format!("{}/{}/scim/{}/Groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("group_id" = String, Path, description = "Group ID"),
    ),
    responses(
        (status = 200, description = "Group found", body = ScimGroup),
        (status = 404, description = "Group not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Get SCIM Group",
    description = "Retrieve a group by its unique identifier in SCIM format, including its members.",
    operation_id = "scim-get-group",
)]
async fn route_get_group(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(group_id): Path<String>,
) -> ScimResult<ScimOkResponse<ScimGroup>> {
    trace!(group_id = %group_id, "Getting SCIM group");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = get_group_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &group_id,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Getting SCIM group completed");
    match result {
        Ok(group) => Ok(ScimOkResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    put,
    path = format!("{}/{}/scim/{}/Groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("group_id" = String, Path, description = "Group ID"),
    ),
    request_body = ScimGroup,
    responses(
        (status = 200, description = "Group replaced", body = ScimGroup),
        (status = 404, description = "Group not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Replace SCIM Group",
    description = "Replace a group's attributes and members with the provided SCIM Group payload (PUT operation).",
    operation_id = "scim-replace-group",
)]
async fn route_replace_group(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(group_id): Path<String>,
    Json(scim_group): Json<ScimGroup>,
) -> ScimResult<ScimOkResponse<ScimGroup>> {
    trace!(group_id = %group_id, member_count = scim_group.members.len(), "Replacing SCIM group");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = replace_group_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &group_id,
        scim_group,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Replacing SCIM group completed");
    match result {
        Ok(group) => Ok(ScimOkResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    patch,
    path = format!("{}/{}/scim/{}/Groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("group_id" = String, Path, description = "Group ID"),
    ),
    request_body = ScimPatchRequest,
    responses(
        (status = 200, description = "Group patched", body = ScimGroup),
        (status = 404, description = "Group not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Patch SCIM Group",
    description = "Partially update a group's attributes or members using SCIM PATCH operations (add, remove, replace).",
    operation_id = "scim-patch-group",
)]
async fn route_patch_group(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(group_id): Path<String>,
    Json(patch_request): Json<ScimPatchRequest>,
) -> ScimResult<ScimOkResponse<ScimGroup>> {
    trace!(group_id = %group_id, operation_count = patch_request.operations.len(), "Patching SCIM group");
    let identity_placeholder = Identity::Unauthenticated;
    let base_url = get_scim_base_url();
    let result = patch_group_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &group_id,
        patch_request,
        &base_url,
    )
    .await;
    trace!(success = result.is_ok(), "Patching SCIM group completed");
    match result {
        Ok(group) => Ok(ScimOkResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

#[utoipa::path(
    delete,
    path = format!("{}/{}/scim/{}/Groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, SCIM_VERSION),
    tags = ["scim", API_VERSION_TAG],
    params(
        ("group_id" = String, Path, description = "Group ID"),
    ),
    responses(
        (status = 204, description = "Group deleted"),
        (status = 404, description = "Group not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Delete SCIM Group",
    description = "Delete a group by its unique identifier. This also removes all group memberships.",
    operation_id = "scim-delete-group",
)]
async fn route_delete_group(
    State(ctx): State<IdentityService>,
    headers: HeaderMap,
    Path(group_id): Path<String>,
) -> ScimResult<ScimNoContentResponse> {
    trace!(group_id = %group_id, "Deleting SCIM group");
    let identity_placeholder = Identity::Unauthenticated;
    let result = delete_group_scim(
        ctx.auth_client(),
        headers,
        identity_placeholder,
        ctx.repository(),
        &group_id,
    )
    .await;
    trace!(success = result.is_ok(), "Deleting SCIM group completed");
    match result {
        Ok(()) => Ok(ScimNoContentResponse),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}
