//! HTTP routes for the identity service
//!
//! This module provides HTTP endpoints for:
//! - SCIM 2.0 user and group provisioning (/scim/v2/Users, /scim/v2/Groups)
//! - Regular user CRUD operations (/users)
//! - Regular group CRUD operations (/groups)

pub mod scim;

use crate::logic::user::{Group, GroupMembership, Role, User, UserType};
use crate::repository::Repository;
use crate::repository::{GroupMemberWithUser, UpdateUser, UserGroupWithGroup, UserRepositoryLike};
use axum::extract::{Json, Path, Query, State};
use serde::{Deserialize, Serialize};
use shared::{
    adapters::openapi::{API_VERSION_TAG, JsonResponse},
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime},
};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub const PATH_PREFIX: &str = "/api";
pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "identity";

// ============================================================================
// Service Context
// ============================================================================

pub struct IdentityServiceInner {
    pub repository: Repository,
}

impl IdentityServiceInner {
    /// Creates a new IdentityServiceInner that holds the given repository.
    ///
    /// # Examples
    ///
    /// ```
    /// let repo = Repository::new(/* ... */);
    /// let inner = IdentityServiceInner::new(repo);
    /// ```
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[derive(Clone)]
pub struct IdentityService(pub Arc<IdentityServiceInner>);

impl IdentityService {
    /// Creates a new shared IdentityService that encapsulates the provided `Repository`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Obtain a Repository from your application setup
    /// let repository = /* Repository */ unimplemented!();
    /// let service = IdentityService::new(repository);
    /// ```
    pub fn new(repository: Repository) -> Self {
        Self(Arc::new(IdentityServiceInner::new(repository)))
    }

    /// Access the underlying repository held by the service.
    ///
    /// # Returns
    ///
    /// A reference to the internal `Repository`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::routes::IdentityService;
    /// # use crate::repository::Repository;
    /// // `svc` is an existing `IdentityService`
    /// let repo: &Repository = svc.repository();
    /// ```
    pub fn repository(&self) -> &Repository {
        &self.0.repository
    }
}

// ============================================================================
// Request/Response Types for User CRUD
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// User ID (if not provided, a UUID will be generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// User type: "service_principal" or "federated_user"
    pub user_type: String,
    /// User's email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User's role
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// User's email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User's role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListUsersQuery {
    pub page_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

pub type UserResponse = User;
pub type ListUsersResponse = PaginatedResponse<User>;

// ============================================================================
// Request/Response Types for Group CRUD
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    /// Group ID (if not provided, a UUID will be generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Group name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateGroupRequest {
    /// Group name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListGroupsQuery {
    pub page_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

pub type GroupResponse = Group;
pub type ListGroupsResponse = PaginatedResponse<Group>;

// ============================================================================
// Request/Response Types for Group Membership
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddGroupMemberRequest {
    /// User ID to add to the group
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListGroupMembersQuery {
    pub page_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

pub type ListGroupMembersResponse = PaginatedResponse<GroupMemberWithUser>;
pub type ListUserGroupsResponse = PaginatedResponse<UserGroupWithGroup>;

// ============================================================================
// Router Creation
// ============================================================================

/// Creates the OpenApiRouter configured with all identity HTTP endpoints.
///
/// The router includes user CRUD, group CRUD, group membership endpoints, and merges the SCIM 2.0 provisioning routes.
///
/// # Returns
///
/// An `OpenApiRouter<IdentityService>` with all identity routes registered.
///
/// # Examples
///
/// ```
/// let router = create_router();
/// // `router` is ready to be mounted into an Axum server and serves the identity API.
/// ```
pub fn create_router() -> OpenApiRouter<IdentityService> {
    OpenApiRouter::new()
        // User endpoints
        .routes(routes!(route_create_user))
        .routes(routes!(route_get_user))
        .routes(routes!(route_update_user))
        .routes(routes!(route_delete_user))
        .routes(routes!(route_list_users))
        .routes(routes!(route_list_user_groups))
        // Group endpoints
        .routes(routes!(route_create_group))
        .routes(routes!(route_get_group))
        .routes(routes!(route_update_group))
        .routes(routes!(route_delete_group))
        .routes(routes!(route_list_groups))
        // Group membership endpoints
        .routes(routes!(route_add_group_member))
        .routes(routes!(route_remove_group_member))
        .routes(routes!(route_list_group_members))
        // Merge SCIM routes
        .merge(scim::create_scim_router())
}

// ============================================================================
// User CRUD Endpoints
// ============================================================================

/// Creates a new user with the provided attributes and returns the created user.
///
/// The endpoint generates an `id` when omitted, assigns default `user_type` and `role` when parsing fails,
/// persists the user, and returns the stored user record on success.
///
/// # Returns
///
/// `JsonResponse<UserResponse, CommonError>` containing the created `User` on success, or a `CommonError` on failure.
///
/// # Examples
///
/// ```
/// use crate::routes::CreateUserRequest;
///
/// #[test]
/// fn build_create_user_request() {
///     let req = CreateUserRequest {
///         id: None,
///         user_type: "human".to_string(),
///         email: Some("alice@example.com".to_string()),
///         role: "user".to_string(),
///     };
///     assert_eq!(req.email.as_deref(), Some("alice@example.com"));
/// }
/// ```
#[utoipa::path(
post,
path = format!("{}/{}/{}/users", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
request_body = CreateUserRequest,
responses(
(status = 200, description = "User created", body = UserResponse),
(status = 400, description = "Bad Request", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Create user",
description = "Create a new user with the specified attributes",
operation_id = "create-user",
)]
async fn route_create_user(
    State(ctx): State<IdentityService>,
    Json(req): Json<CreateUserRequest>,
) -> JsonResponse<UserResponse, CommonError> {
    let now = WrappedChronoDateTime::now();
    let user_id = req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let user_type = UserType::parse(&req.user_type).unwrap_or(UserType::Human);
    let role = Role::parse(&req.role).unwrap_or(Role::User);

    let user = User {
        id: user_id.clone(),
        user_type,
        email: req.email,
        role,
        description: None,
        created_at: now,
        updated_at: now,
    };

    let res = async {
        ctx.repository().create_user(&user).await?;
        ctx.repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Failed to retrieve created user")))
    }
    .await;

    JsonResponse::from(res)
}

/// Retrieve a user by their unique identifier.
///
/// Returns a JSON response that contains the user when found or a `CommonError::NotFound` when no user exists with the given id.
///
/// # Examples
///
/// ```rust
/// let path = format!("{}/{}/{}/users/{}", "api", "identity", "v1", "user123");
/// assert_eq!(path, "/api/identity/v1/users/user123");
/// ```
#[utoipa::path(
get,
path = format!("{}/{}/{}/users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("user_id" = String, Path, description = "User ID"),
),
responses(
(status = 200, description = "User found", body = UserResponse),
(status = 404, description = "User not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Get user",
description = "Retrieve a user by their unique identifier",
operation_id = "get-user",
)]
async fn route_get_user(
    State(ctx): State<IdentityService>,
    Path(user_id): Path<String>,
) -> JsonResponse<UserResponse, CommonError> {
    let res = async {
        ctx.repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "User not found".to_string(),
                lookup_id: user_id.clone(),
                source: None,
            })
    }
    .await;

    JsonResponse::from(res)
}

/// Update an existing user's attributes.
///
/// Attempts to apply the provided changes (email and/or role) to the user identified by `user_id`
/// and returns the freshly fetched user record after the update.
///
/// # Parameters
/// - `user_id` — ID of the user to update.
/// - `req` — update payload containing optional `email` and `role`.
///
/// # Returns
/// The updated `User` on success. Returns `CommonError::NotFound` if no user exists with the given `user_id`.
///
/// # Examples
///
/// ```
/// use crate::routes::UpdateUserRequest;
///
/// let req = UpdateUserRequest {
///     email: Some("new@example.com".to_string()),
///     role: Some("admin".to_string()),
/// };
///
/// // The handler is an async HTTP endpoint; in integration tests you would send `req`
/// // to the route and expect the returned User to reflect the updated fields.
/// ```
#[utoipa::path(
patch,
path = format!("{}/{}/{}/users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("user_id" = String, Path, description = "User ID"),
),
request_body = UpdateUserRequest,
responses(
(status = 200, description = "User updated", body = UserResponse),
(status = 404, description = "User not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Update user",
description = "Update a user's attributes",
operation_id = "update-user",
)]
async fn route_update_user(
    State(ctx): State<IdentityService>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> JsonResponse<UserResponse, CommonError> {
    let res = async {
        // Check if user exists
        let _existing = ctx
            .repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "User not found".to_string(),
                lookup_id: user_id.clone(),
                source: None,
            })?;

        let update_user = UpdateUser {
            email: req.email,
            role: req.role.and_then(|r| Role::parse(&r)),
            description: None,
        };

        ctx.repository().update_user(&user_id, &update_user).await?;

        ctx.repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Failed to retrieve updated user")))
    }
    .await;

    JsonResponse::from(res)
}

/// Removes the user identified by `user_id` and deletes the user's associated group memberships and API keys.
///
/// On success the handler produces an empty (unit) JSON response. If the user does not exist, the handler returns a `NotFound` error.
///
/// # Examples
///
/// ```
/// let path = format!("{}/{}/{}/users/{}", "/api", "identity", "v1", "user123");
/// assert!(path.ends_with("/users/user123"));
/// ```
#[utoipa::path(
delete,
path = format!("{}/{}/{}/users/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("user_id" = String, Path, description = "User ID"),
),
responses(
(status = 200, description = "User deleted"),
(status = 404, description = "User not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Delete user",
description = "Delete a user by their unique identifier",
operation_id = "delete-user",
)]
async fn route_delete_user(
    State(ctx): State<IdentityService>,
    Path(user_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    let res = async {
        // Check if user exists
        let _existing = ctx
            .repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "User not found".to_string(),
                lookup_id: user_id.clone(),
                source: None,
            })?;

        // Delete user's group memberships
        ctx.repository()
            .delete_group_memberships_by_user_id(&user_id)
            .await?;
        // Delete user's API keys
        ctx.repository()
            .delete_api_keys_by_user_id(&user_id)
            .await?;
        // Delete the user
        ctx.repository().delete_user(&user_id).await?;

        Ok(())
    }
    .await;

    JsonResponse::from(res)
}

/// Lists users, supporting pagination and optional filtering by user type and role.
///
/// # Returns
///
/// A `JsonResponse` containing a `PaginatedResponse<User>` on success, or a `CommonError` on failure.
///
/// # Examples
///
/// ```
/// # use crate::routes::ListUsersQuery;
/// // Construct a query to request the first page of up to 50 users.
/// let query = ListUsersQuery {
///     page_size: 50,
///     next_page_token: None,
///     user_type: None,
///     role: None,
/// };
/// // The handler is normally invoked by the web framework (axum); this shows building the query.
/// ```
#[utoipa::path(
get,
path = format!("{}/{}/{}/users", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(ListUsersQuery),
responses(
(status = 200, description = "List of users", body = ListUsersResponse),
(status = 400, description = "Bad Request", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "List users",
description = "List all users with pagination and optional filtering",
operation_id = "list-users",
)]
async fn route_list_users(
    State(ctx): State<IdentityService>,
    Query(query): Query<ListUsersQuery>,
) -> JsonResponse<ListUsersResponse, CommonError> {
    let pagination = PaginationRequest {
        page_size: query.page_size,
        next_page_token: query.next_page_token,
    };

    let user_type_filter = query.user_type.as_ref().and_then(|s| UserType::parse(s));
    let role_filter = query.role.as_ref().and_then(|s| Role::parse(s));

    let res = ctx
        .repository()
        .list_users(&pagination, user_type_filter.as_ref(), role_filter.as_ref())
        .await;

    JsonResponse::from(res)
}

/// List all groups that a specific user belongs to.
///
/// Returns a paginated list of groups for the given `user_id`. If the user does not exist,
/// a `CommonError::NotFound` is returned; other failures produce a `CommonError::Internal`.
///
/// # Examples
///
/// ```
/// // Integration tests should call this handler through the router or via an HTTP client.
/// let query = ListGroupMembersQuery { page_size: 25, next_page_token: None };
/// // Example: perform a GET to /api/identity/v1/users/{user_id}/groups with the query parameters.
/// ```
#[utoipa::path(
get,
path = format!("{}/{}/{}/users/{{user_id}}/groups", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("user_id" = String, Path, description = "User ID"),
ListGroupMembersQuery,
),
responses(
(status = 200, description = "List of user's groups", body = ListUserGroupsResponse),
(status = 404, description = "User not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "List user groups",
description = "List all groups that a user belongs to",
operation_id = "list-user-groups",
)]
async fn route_list_user_groups(
    State(ctx): State<IdentityService>,
    Path(user_id): Path<String>,
    Query(query): Query<ListGroupMembersQuery>,
) -> JsonResponse<ListUserGroupsResponse, CommonError> {
    let res = async {
        // Check if user exists
        let _existing = ctx
            .repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "User not found".to_string(),
                lookup_id: user_id.clone(),
                source: None,
            })?;

        let pagination = PaginationRequest {
            page_size: query.page_size,
            next_page_token: query.next_page_token,
        };

        ctx.repository()
            .list_user_groups(&user_id, &pagination)
            .await
    }
    .await;

    JsonResponse::from(res)
}

// ============================================================================
// Group CRUD Endpoints
// ============================================================================

/// Creates a new group with the given name and returns the created Group.
///
/// The resulting Group will have an id (generated if not provided) and timestamps for `created_at` and `updated_at` set to the current time.
///
/// # Returns
///
/// The created `Group` on success, or a `CommonError` on failure.
///
/// # Examples
///
/// ```
/// use crate::routes::CreateGroupRequest;
///
/// let req = CreateGroupRequest { id: None, name: "admins".to_string() };
/// // When invoked as an HTTP handler within the service, this request will create and return the new group.
/// ```
#[utoipa::path(
post,
path = format!("{}/{}/{}/groups", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
request_body = CreateGroupRequest,
responses(
(status = 200, description = "Group created", body = GroupResponse),
(status = 400, description = "Bad Request", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Create group",
description = "Create a new group with the specified name",
operation_id = "create-group",
)]
async fn route_create_group(
    State(ctx): State<IdentityService>,
    Json(req): Json<CreateGroupRequest>,
) -> JsonResponse<GroupResponse, CommonError> {
    let now = WrappedChronoDateTime::now();
    let group_id = req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let group = Group {
        id: group_id.clone(),
        name: req.name,
        created_at: now,
        updated_at: now,
    };

    let res = async {
        ctx.repository().create_group(&group).await?;
        ctx.repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!("Failed to retrieve created group"))
            })
    }
    .await;

    JsonResponse::from(res)
}

/// Retrieve a group by its unique identifier.
///
/// # Returns
/// `JsonResponse<GroupResponse, CommonError>`: the requested group on success; a `CommonError::NotFound` when the group does not exist; or a `CommonError` for internal errors.
///
/// # Examples
///
/// ```
/// // Called from an async context (e.g., an integration test or another handler).
/// // let resp = route_get_group(State(identity_service), Path("group-id".to_string())).await;
/// ```
#[utoipa::path(
get,
path = format!("{}/{}/{}/groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("group_id" = String, Path, description = "Group ID"),
),
responses(
(status = 200, description = "Group found", body = GroupResponse),
(status = 404, description = "Group not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Get group",
description = "Retrieve a group by its unique identifier",
operation_id = "get-group",
)]
async fn route_get_group(
    State(ctx): State<IdentityService>,
    Path(group_id): Path<String>,
) -> JsonResponse<GroupResponse, CommonError> {
    let res = async {
        ctx.repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "Group not found".to_string(),
                lookup_id: group_id.clone(),
                source: None,
            })
    }
    .await;

    JsonResponse::from(res)
}

/// Updates the name of an existing group and returns the updated group.
///
/// Attempts to find the group by `group_id`, apply the new name from the request, and return the persisted group record.
///
/// # Returns
///
/// Updated `Group` on success; `CommonError::NotFound` if no group with `group_id` exists; other `CommonError` variants for repository or internal failures.
///
/// # Examples
///
/// ```
/// // Illustrative example; in real code these values come from the framework's extractors.
/// use crates::identity::routes::{route_update_group, UpdateGroupRequest};
/// use axum::extract::{State, Path, Json};
///
/// # async fn _example(ctx: crates::identity::IdentityService) {
/// let state = State(ctx);
/// let path = Path("group-123".to_string());
/// let body = Json(UpdateGroupRequest { name: "New Name".to_string() });
/// let _resp = route_update_group(state, path, body).await;
/// # }
/// ```
#[utoipa::path(
patch,
path = format!("{}/{}/{}/groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("group_id" = String, Path, description = "Group ID"),
),
request_body = UpdateGroupRequest,
responses(
(status = 200, description = "Group updated", body = GroupResponse),
(status = 404, description = "Group not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Update group",
description = "Update a group's name",
operation_id = "update-group",
)]
async fn route_update_group(
    State(ctx): State<IdentityService>,
    Path(group_id): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> JsonResponse<GroupResponse, CommonError> {
    let res = async {
        // Check if group exists
        let _existing = ctx
            .repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "Group not found".to_string(),
                lookup_id: group_id.clone(),
                source: None,
            })?;

        ctx.repository().update_group(&group_id, &req.name).await?;

        ctx.repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| {
                CommonError::Unknown(anyhow::anyhow!("Failed to retrieve updated group"))
            })
    }
    .await;

    JsonResponse::from(res)
}

/// Deletes a group and all of its memberships by group ID.
///
/// Attempts to remove the group identified by `group_id`; if the group exists its memberships are deleted first and then the group itself is removed. Returns a NotFound error when no group with the given ID exists.
///
/// # Examples
///
/// ```no_run
/// // Sends a DELETE request to the service to remove a group and its memberships:
/// // DELETE /api/identity/v1/groups/{group_id}
/// ```
#[utoipa::path(
delete,
path = format!("{}/{}/{}/groups/{{group_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("group_id" = String, Path, description = "Group ID"),
),
responses(
(status = 200, description = "Group deleted"),
(status = 404, description = "Group not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Delete group",
description = "Delete a group by its unique identifier",
operation_id = "delete-group",
)]
async fn route_delete_group(
    State(ctx): State<IdentityService>,
    Path(group_id): Path<String>,
) -> JsonResponse<(), CommonError> {
    let res = async {
        // Check if group exists
        let _existing = ctx
            .repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "Group not found".to_string(),
                lookup_id: group_id.clone(),
                source: None,
            })?;

        // Delete group memberships
        ctx.repository()
            .delete_group_memberships_by_group_id(&group_id)
            .await?;
        // Delete the group
        ctx.repository().delete_group(&group_id).await?;

        Ok(())
    }
    .await;

    JsonResponse::from(res)
}

/// Lists groups using pagination and returns a paginated response.
///
/// The handler reads pagination parameters from the query and returns a `PaginatedResponse<Group>`
/// on success or a `CommonError` on failure.
///
/// # Examples
///
/// ```ignore
/// // Query example: GET /api/identity/v1/groups?page_size=50
/// let query = ListGroupsQuery { page_size: 50, next_page_token: None };
/// // The router will call `route_list_groups` and return a `ListGroupsResponse`
/// ```
#[utoipa::path(
get,
path = format!("{}/{}/{}/groups", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(ListGroupsQuery),
responses(
(status = 200, description = "List of groups", body = ListGroupsResponse),
(status = 400, description = "Bad Request", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "List groups",
description = "List all groups with pagination",
operation_id = "list-groups",
)]
async fn route_list_groups(
    State(ctx): State<IdentityService>,
    Query(query): Query<ListGroupsQuery>,
) -> JsonResponse<ListGroupsResponse, CommonError> {
    let pagination = PaginationRequest {
        page_size: query.page_size,
        next_page_token: query.next_page_token,
    };

    let res = ctx.repository().list_groups(&pagination).await;

    JsonResponse::from(res)
}

// ============================================================================
// Group Membership Endpoints
// ============================================================================

/// Adds a user to the specified group.
///
/// Attempts to create a group membership linking the given user to the group identified by `group_id`.
/// On success returns an empty success response; on failure returns a `CommonError` (for example if the group or user does not exist or the user is already a member).
///
/// # Examples
///
/// ```
/// use crates::identity::routes::AddGroupMemberRequest;
///
/// let req = AddGroupMemberRequest { user_id: "user-123".to_string() };
/// assert_eq!(req.user_id, "user-123");
/// ```
#[utoipa::path(
post,
path = format!("{}/{}/{}/groups/{{group_id}}/members", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("group_id" = String, Path, description = "Group ID"),
),
request_body = AddGroupMemberRequest,
responses(
(status = 200, description = "Member added"),
(status = 404, description = "Group or user not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Add group member",
description = "Add a user to a group",
operation_id = "add-group-member",
)]
async fn route_add_group_member(
    State(ctx): State<IdentityService>,
    Path(group_id): Path<String>,
    Json(req): Json<AddGroupMemberRequest>,
) -> JsonResponse<(), CommonError> {
    let now = WrappedChronoDateTime::now();

    let res = async {
        // Check if group exists
        let _group = ctx
            .repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "Group not found".to_string(),
                lookup_id: group_id.clone(),
                source: None,
            })?;

        // Check if user exists
        let _user = ctx
            .repository()
            .get_user_by_id(&req.user_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "User not found".to_string(),
                lookup_id: req.user_id.clone(),
                source: None,
            })?;

        // Check if membership already exists
        if ctx
            .repository()
            .get_group_membership(&group_id, &req.user_id)
            .await?
            .is_some()
        {
            return Err(CommonError::InvalidRequest {
                msg: "User is already a member of this group".to_string(),
                source: None,
            });
        }

        let membership = GroupMembership {
            group_id: group_id.clone(),
            user_id: req.user_id,
            created_at: now,
            updated_at: now,
        };

        ctx.repository()
            .create_group_membership(&membership)
            .await?;

        Ok(())
    }
    .await;

    JsonResponse::from(res)
}

/// Remove a user from a group.
///
/// Removes the membership linking the specified `user_id` to the specified `group_id`.
/// Returns an error if the membership, group, or user does not exist.
///
/// # Examples
///
/// ```
/// // Example (illustrative): call the handler with a service state and path parameters.
/// // let resp = route_remove_group_member(State(service), Path((group_id.to_string(), user_id.to_string()))).await;
/// ```
#[utoipa::path(
delete,
path = format!("{}/{}/{}/groups/{{group_id}}/members/{{user_id}}", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("group_id" = String, Path, description = "Group ID"),
("user_id" = String, Path, description = "User ID"),
),
responses(
(status = 200, description = "Member removed"),
(status = 404, description = "Group, user, or membership not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "Remove group member",
description = "Remove a user from a group",
operation_id = "remove-group-member",
)]
async fn route_remove_group_member(
    State(ctx): State<IdentityService>,
    Path((group_id, user_id)): Path<(String, String)>,
) -> JsonResponse<(), CommonError> {
    let res = async {
        // Check if membership exists
        let _membership = ctx
            .repository()
            .get_group_membership(&group_id, &user_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "Group membership not found".to_string(),
                lookup_id: format!("{group_id}:{user_id}"),
                source: None,
            })?;

        ctx.repository()
            .delete_group_membership(&group_id, &user_id)
            .await?;

        Ok(())
    }
    .await;

    JsonResponse::from(res)
}

/// Lists members of the specified group in a paginated response.
///
/// On success returns a `ListGroupMembersResponse` containing the group's members and pagination metadata. On failure returns a `CommonError` (for example when the group is not found or an internal error occurs).
///
/// # Examples
///
/// ```no_run
/// // Example HTTP request:
/// // GET /api/identity/v1/groups/123/members?page_size=20
/// ```
#[utoipa::path(
get,
path = format!("{}/{}/{}/groups/{{group_id}}/members", PATH_PREFIX, SERVICE_ROUTE_KEY, API_VERSION_1),
tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
params(
("group_id" = String, Path, description = "Group ID"),
ListGroupMembersQuery,
),
responses(
(status = 200, description = "List of group members", body = ListGroupMembersResponse),
(status = 404, description = "Group not found", body = CommonError),
(status = 500, description = "Internal Server Error", body = CommonError),
),
summary = "List group members",
description = "List all members of a group",
operation_id = "list-group-members",
)]
async fn route_list_group_members(
    State(ctx): State<IdentityService>,
    Path(group_id): Path<String>,
    Query(query): Query<ListGroupMembersQuery>,
) -> JsonResponse<ListGroupMembersResponse, CommonError> {
    let res = async {
        // Check if group exists
        let _group = ctx
            .repository()
            .get_group_by_id(&group_id)
            .await?
            .ok_or_else(|| CommonError::NotFound {
                msg: "Group not found".to_string(),
                lookup_id: group_id.clone(),
                source: None,
            })?;

        let pagination = PaginationRequest {
            page_size: query.page_size,
            next_page_token: query.next_page_token,
        };

        ctx.repository()
            .list_group_members(&group_id, &pagination)
            .await
    }
    .await;

    JsonResponse::from(res)
}