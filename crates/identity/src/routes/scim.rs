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
use http::StatusCode;
use shared::{adapters::openapi::API_VERSION_TAG, error::CommonError};
use utoipa_axum::{router::OpenApiRouter, routes};

pub const SCIM_VERSION: &str = "v2";

/// Creates the SCIM router that registers all SCIM Users and Groups endpoints.
///
/// # Examples
///
/// ```rust
/// let router = create_scim_router();
/// // router can now be mounted into an Axum application
/// let _ = router;
/// ```
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

/// Constructs the base URL for SCIM resources.
///
/// If the `SCIM_BASE_URL` environment variable is set, its value is returned.
/// Otherwise a default URL is composed from `PATH_PREFIX`, `SERVICE_ROUTE_KEY`, and `SCIM_VERSION`.
///
/// # Examples
///
/// ```
/// use std::env;
/// env::set_var("SCIM_BASE_URL", "https://example.com/scim");
/// assert_eq!(super::get_scim_base_url(), "https://example.com/scim");
/// env::remove_var("SCIM_BASE_URL");
/// let fallback = super::get_scim_base_url();
/// assert!(fallback.contains("/scim/"));
/// ```
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
    /// Convert the wrapped value into an HTTP 201 Created JSON response.
    ///
    /// The response has status `201 Created` and a JSON-serialized body containing the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::http::StatusCode;
    ///
    /// let resp = ScimCreatedResponse("created").into_response();
    /// assert_eq!(resp.status(), StatusCode::CREATED);
    /// ```
    fn into_response(self) -> axum::response::Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

/// SCIM response for successful operations
pub struct ScimOkResponse<T>(pub T);

impl<T: serde::Serialize> axum::response::IntoResponse for ScimOkResponse<T> {
    /// Convert the wrapped value into an HTTP 200 OK JSON response.
    ///
    /// # Returns
    ///
    /// An HTTP response with status code 200 and a JSON-serialized body containing the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::http::StatusCode;
    /// use axum::response::Response;
    ///
    /// let ok = ScimOkResponse("hello");
    /// let resp: Response = ok.into_response();
    /// assert_eq!(resp.status(), StatusCode::OK);
    /// ```
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

/// SCIM response for no content (delete)
pub struct ScimNoContentResponse;

impl axum::response::IntoResponse for ScimNoContentResponse {
    /// Converts the value into an HTTP 204 No Content response.
    ///
    /// # Examples
    ///
    /// ```
    /// // Convert the unit-scoped SCIM no-content value into an HTTP response
    /// let resp = ScimNoContentResponse.into_response();
    /// assert_eq!(resp.status(), axum::http::StatusCode::NO_CONTENT);
    /// ```
    ///
    /// # Returns
    ///
    /// An `axum::response::Response` with HTTP status 204 No Content.
    fn into_response(self) -> axum::response::Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

/// SCIM error response
pub struct ScimErrorResponse(pub CommonError);

impl axum::response::IntoResponse for ScimErrorResponse {
    /// Converts the wrapped value into an Axum HTTP response.
    ///
    /// # Examples
    ///
    /// ```
    /// let resp = ScimCreatedResponse(some_value).into_response();
    /// // `resp` is an `axum::response::Response`
    /// ```
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

/// Result type for SCIM endpoints
pub type ScimResult<T> = Result<T, ScimErrorResponse>;

/// List SCIM users applying pagination and optional filtering.
///
/// Returns a SCIM-formatted list of users according to the supplied `ScimListParams`.
///
/// # Examples
///
/// ```
/// #[tokio::test]
/// async fn example_list_users() {
///     // Construct service state and query params appropriate for your test environment.
///     // `svc` and `params` below are placeholders — replace with real values.
///     let svc = /* IdentityService instance */;
///     let params = /* ScimListParams instance */;
///     let res = route_list_users(State(svc), Query(params)).await;
///     // `res` will be `Ok(ScimOkResponse(ScimUserListResponse))` on success.
///     assert!(res.is_ok());
/// }
/// ```
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
    Query(params): Query<ScimListParams>,
) -> ScimResult<ScimOkResponse<ScimUserListResponse>> {
    let base_url = get_scim_base_url();
    let result = list_users_scim(ctx.repository(), params, &base_url).await;
    match result {
        Ok(users) => Ok(ScimOkResponse(users)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Create a new user from a SCIM User payload.
///
/// The created identity will be stored as a `federated_user`. On success the created SCIM user is returned; on failure a `CommonError` is propagated.
///
/// # Examples
///
/// ```
/// # use crate::scim::{route_create_user, ScimUser};
/// # use axum::extract::State;
/// # use axum::Json;
/// # tokio_test::block_on(async {
/// let payload = ScimUser { /* populate required SCIM fields */ };
/// // In real server execution the framework supplies `State(ctx)` and `Json(payload)`.
/// // let result = route_create_user(State(ctx), Json(payload)).await;
/// # });
/// ```
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
    Json(scim_user): Json<ScimUser>,
) -> ScimResult<ScimCreatedResponse<ScimUser>> {
    let result = create_user_from_scim(ctx.repository(), scim_user).await;
    match result {
        Ok(user) => Ok(ScimCreatedResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Retrieve a SCIM-formatted user by their unique identifier.
///
/// Returns the found SCIM user wrapped in a 200 OK response; if the user cannot be found or an internal
/// error occurs, the function returns an error response describing the failure.
///
/// # Returns
///
/// `ScimOkResponse<ScimUser>` containing the requested user on success.
///
/// # Examples
///
/// ```no_run
/// # use axum::extract::{State, Path};
/// # use scim_module::{route_get_user, IdentityService, ScimResult, ScimOkResponse, ScimUser};
/// # async fn example(ctx: IdentityService) -> Result<(), ()> {
/// let user_id = "alice".to_string();
/// // Call the handler as an async function (in a real app the framework invokes it).
/// let result: ScimResult<ScimOkResponse<ScimUser>> = route_get_user(State(ctx), Path(user_id)).await;
/// match result {
///     Ok(ScimOkResponse(user)) => {
///         // use `user`
///     }
///     Err(err) => {
///         // handle error
///     }
/// }
/// # Ok(()) }
/// ```
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
    Path(user_id): Path<String>,
) -> ScimResult<ScimOkResponse<ScimUser>> {
    let base_url = get_scim_base_url();
    let result = get_user_scim(ctx.repository(), &user_id, &base_url).await;
    match result {
        Ok(user) => Ok(ScimOkResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Replace the SCIM user identified by `user_id` with the provided `ScimUser` payload.
///
/// On success returns an `ScimOkResponse` wrapping the replaced `ScimUser`. Errors are returned
/// as `ScimErrorResponse` (e.g., user not found or internal server error).
///
/// # Examples
///
/// ```
/// // Pseudocode example showing intended usage; actual call occurs within an async web handler:
/// # async fn example(ctx: crate::identity::IdentityService) {
/// # use crate::scim::{ScimUser, route_replace_user};
/// let user_id = "alice@example.com".to_string();
/// let scim_user = ScimUser { /* fields */ };
/// // In an Axum handler the framework provides `State`, `Path`, and `Json`.
/// // The route returns `ScimResult<ScimOkResponse<ScimUser>>`.
/// // let result = route_replace_user(State(ctx), Path(user_id), Json(scim_user)).await;
/// # }
/// ```
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
    Path(user_id): Path<String>,
    Json(scim_user): Json<ScimUser>,
) -> ScimResult<ScimOkResponse<ScimUser>> {
    let base_url = get_scim_base_url();
    let result = replace_user_scim(ctx.repository(), &user_id, scim_user, &base_url).await;
    match result {
        Ok(user) => Ok(ScimOkResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Applies a SCIM PATCH request to partially update the specified user.
///
/// Accepts a SCIM patch document and returns the user's updated representation when the patch succeeds.
///
/// # Returns
///
/// `ScimOkResponse<ScimUser>` with the updated SCIM user on success.
///
/// # Examples
///
/// ```no_run
/// use axum::extract::{State, Path, Json};
/// use crate::scim::{route_patch_user, ScimPatchRequest};
///
/// #[tokio::main]
/// async fn main() {
///     // Example placeholders — in real usage these come from the HTTP layer.
///     let svc_state = State(/* IdentityService instance */);
///     let user_id = Path("user-123".to_string());
///     let patch = Json(ScimPatchRequest { /* ... */ });
///
///     // Call the route handler (would normally be invoked by the router).
///     let _ = route_patch_user(svc_state, user_id, patch).await;
/// }
/// ```
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
    Path(user_id): Path<String>,
    Json(patch_request): Json<ScimPatchRequest>,
) -> ScimResult<ScimOkResponse<ScimUser>> {
    let base_url = get_scim_base_url();
    let result = patch_user_scim(ctx.repository(), &user_id, patch_request, &base_url).await;
    match result {
        Ok(user) => Ok(ScimOkResponse(user)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Delete a SCIM user and remove them from all groups.
///
/// Deletes the user identified by `user_id`. On success the endpoint yields no content.
///
/// # Returns
///
/// `ScimNoContentResponse` on success (HTTP 204). `ScimErrorResponse` on failure with an appropriate `CommonError`.
///
/// # Examples
///
/// ```no_run
/// # use axum::extract::{State, Path};
/// # use identity_service::routes::scim::{route_delete_user, ScimNoContentResponse, ScimErrorResponse};
/// # async fn example(ctx: identity_service::IdentityService, user_id: String) {
/// let response = route_delete_user(State(ctx), Path(user_id)).await;
/// match response {
///     Ok(ScimNoContentResponse) => println!("deleted"),
///     Err(ScimErrorResponse(err)) => eprintln!("error: {:?}", err),
/// }
/// # }
/// ```
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
    Path(user_id): Path<String>,
) -> ScimResult<ScimNoContentResponse> {
    let result = delete_user_scim(ctx.repository(), &user_id).await;
    match result {
        Ok(()) => Ok(ScimNoContentResponse),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

// ============================================================================
// SCIM Group Endpoints
// ============================================================================

/// Lists SCIM groups using pagination and optional filtering, returning a SCIM-formatted group list response.
///
/// The handler queries the identity repository and returns the groups formatted for SCIM list responses.
///
/// # Examples
///
/// ```
/// use crate::scim::{ScimGroupListResponse, ScimOkResponse};
///
/// // Construct a minimal SCIM list response and wrap it as the handler would on success.
/// let resp = ScimGroupListResponse {
///     total_results: 0,
///     start_index: 1,
///     items_per_page: 0,
///     Resources: Vec::new(),
/// };
/// let wrapped = ScimOkResponse(resp);
/// ```
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
    Query(params): Query<ScimListParams>,
) -> ScimResult<ScimOkResponse<ScimGroupListResponse>> {
    let base_url = get_scim_base_url();
    let result = list_groups_scim(ctx.repository(), params, &base_url).await;
    match result {
        Ok(groups) => Ok(ScimOkResponse(groups)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Create a SCIM group from the provided SCIM Group payload.
///
/// On success returns the created SCIM group wrapped for an HTTP 201 Created response;
/// on failure returns a `ScimErrorResponse` describing the error.
///
/// # Examples
///
/// ```
/// use crate::scim::models::ScimGroup;
///
/// // Build a minimal SCIM group payload to send to the create endpoint.
/// let group = ScimGroup {
///     id: None,
///     display_name: "engineering".into(),
///     members: None,
///     // populate other required fields as needed...
/// };
///
/// // The constructed `group` would be passed as the JSON body to `route_create_group`.
/// assert_eq!(group.display_name, "engineering");
/// ```
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
    Json(scim_group): Json<ScimGroup>,
) -> ScimResult<ScimCreatedResponse<ScimGroup>> {
    let base_url = get_scim_base_url();
    let result = create_group_from_scim(ctx.repository(), scim_group, &base_url).await;
    match result {
        Ok(group) => Ok(ScimCreatedResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Retrieve a SCIM group by its unique identifier, returning the group representation including members.
///
/// # Parameters
///
/// - `group_id`: The unique identifier of the group to retrieve in SCIM format.
///
/// # Returns
///
/// `ScimOkResponse<ScimGroup>` containing the requested group on success.
///
/// # Examples
///
/// ```
/// use axum::extract::{State, Path};
/// # async fn example() {
/// // `ctx` would be your IdentityService state and `group_id` the target group's id.
/// // let ctx = State(your_identity_service);
/// // let path = Path("group-id-123".to_string());
/// // let response = route_get_group(ctx, path).await;
/// # }
/// ```
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
    Path(group_id): Path<String>,
) -> ScimResult<ScimOkResponse<ScimGroup>> {
    let base_url = get_scim_base_url();
    let result = get_group_scim(ctx.repository(), &group_id, &base_url).await;
    match result {
        Ok(group) => Ok(ScimOkResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Replace a SCIM group's attributes and membership using the provided SCIM Group representation.
///
/// Replaces the group's attributes and members with those from `scim_group` and returns the updated
/// SCIM group representation on success.
///
/// # Returns
///
/// `ScimOkResponse<ScimGroup>` containing the updated SCIM group on success.
///
/// # Examples
///
/// ```no_run
/// use crate::scim::{route_replace_group, ScimGroup, ScimOkResponse};
/// // This is an illustrative example; in the router the handler is invoked by the framework.
/// let example_group = ScimGroup { /* fields */ };
/// // route_replace_group is an async handler invoked by the HTTP server; call it from an async context.
/// // let resp: Result<ScimOkResponse<ScimGroup>, _> = route_replace_group(state, Path("group-id".into()), Json(example_group)).await;
/// ```
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
    Path(group_id): Path<String>,
    Json(scim_group): Json<ScimGroup>,
) -> ScimResult<ScimOkResponse<ScimGroup>> {
    let base_url = get_scim_base_url();
    let result = replace_group_scim(ctx.repository(), &group_id, scim_group, &base_url).await;
    match result {
        Ok(group) => Ok(ScimOkResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Apply a SCIM PATCH to update a group's attributes or members.
///
/// Partially updates the specified group using SCIM PATCH operations (`add`, `remove`, `replace`).
///
/// # Returns
///
/// On success, a `ScimOkResponse<ScimGroup>` containing the updated group; on failure, a `ScimErrorResponse` describing the error.
///
/// # Examples
///
/// ```no_run
/// use axum::extract::{State, Path, Json};
/// // `ctx`, `group_id` and `patch_request` are provided by your application/runtime.
/// # async fn example(ctx: crate::IdentityService, group_id: String, patch_request: crate::ScimPatchRequest) {
/// let state = State(ctx);
/// let path = Path(group_id);
/// let json = Json(patch_request);
/// let _response = crate::route_patch_group(state, path, json).await;
/// # }
/// ```
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
    Path(group_id): Path<String>,
    Json(patch_request): Json<ScimPatchRequest>,
) -> ScimResult<ScimOkResponse<ScimGroup>> {
    let base_url = get_scim_base_url();
    let result = patch_group_scim(ctx.repository(), &group_id, patch_request, &base_url).await;
    match result {
        Ok(group) => Ok(ScimOkResponse(group)),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}

/// Deletes a SCIM group identified by `group_id` and removes its memberships.
///
/// # Returns
///
/// `Ok(ScimNoContentResponse)` when the group was deleted, `Err(ScimErrorResponse)` with a `CommonError` on failure.
///
/// # Examples
///
/// ```
/// # use crate::scim::{ScimNoContentResponse, ScimResult, ScimErrorResponse};
/// // Simulated successful result as returned by the route handler:
/// let result: ScimResult<ScimNoContentResponse> = Ok(ScimNoContentResponse);
/// assert!(result.is_ok());
/// ```
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
    Path(group_id): Path<String>,
) -> ScimResult<ScimNoContentResponse> {
    let result = delete_group_scim(ctx.repository(), &group_id).await;
    match result {
        Ok(()) => Ok(ScimNoContentResponse),
        Err(e) => Err(ScimErrorResponse(e)),
    }
}