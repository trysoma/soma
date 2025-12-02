//! HTTP routes for the identity service
//!
//! This module provides HTTP endpoints for:
//! - SCIM 2.0 user and group provisioning (/scim/v2/Users, /scim/v2/Groups)
//! - Regular user CRUD operations (/users)
//! - Regular group CRUD operations (/groups)

pub mod scim;

use crate::repository::Repository;
use crate::repository::{
    CreateGroup, CreateGroupMembership, CreateUser, Group, GroupMemberWithUser, UpdateUser, User,
    UserGroupWithGroup, UserRepositoryLike,
};
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
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[derive(Clone)]
pub struct IdentityService(pub Arc<IdentityServiceInner>);

impl IdentityService {
    pub fn new(repository: Repository) -> Self {
        Self(Arc::new(IdentityServiceInner::new(repository)))
    }

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

/// Create the main identity router with all endpoints
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

    let create_user = CreateUser {
        id: user_id.clone(),
        user_type: req.user_type,
        email: req.email,
        role: req.role,
        created_at: now.clone(),
        updated_at: now,
    };

    let res = async {
        ctx.repository().create_user(&create_user).await?;
        ctx.repository()
            .get_user_by_id(&user_id)
            .await?
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Failed to retrieve created user")))
    }
    .await;

    JsonResponse::from(res)
}

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
            role: req.role,
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

    let res = ctx
        .repository()
        .list_users(
            &pagination,
            query.user_type.as_deref(),
            query.role.as_deref(),
        )
        .await;

    JsonResponse::from(res)
}

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

    let create_group = CreateGroup {
        id: group_id.clone(),
        name: req.name,
        created_at: now.clone(),
        updated_at: now,
    };

    let res = async {
        ctx.repository().create_group(&create_group).await?;
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

        let membership = CreateGroupMembership {
            group_id: group_id.clone(),
            user_id: req.user_id,
            created_at: now.clone(),
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
                lookup_id: format!("{}:{}", group_id, user_id),
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
