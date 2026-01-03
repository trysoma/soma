//! SCIM (System for Cross-domain Identity Management) sync logic
//!
//! This module provides logic functions for syncing external IDP users and groups
//! into our repository based on the SCIM 2.0 specification.

use crate::logic::user::GroupMembership;
use crate::repository::{GroupMemberWithUser, UpdateUser, UserRepositoryLike};
use serde::{Deserialize, Serialize};
use shared::identity::{Group, Role, User, UserType};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime},
};
use shared_macros::{authn, authz_role};
use tracing::{debug, info, trace, warn};
use utoipa::{IntoParams, ToSchema};

// ============================================================================
// SCIM 2.0 Core Schema Definitions
// ============================================================================

/// SCIM 2.0 Meta object containing resource metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimMeta {
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// SCIM 2.0 Name object for user's name components
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScimName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honorific_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honorific_suffix: Option<String>,
}

/// SCIM 2.0 Email object for multi-valued email addresses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimEmail {
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub email_type: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

/// SCIM 2.0 Group member reference
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimGroupMember {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub member_type: Option<String>,
}

/// SCIM 2.0 User group reference (for user's group membership)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimUserGroup {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub group_type: Option<String>,
}

// ============================================================================
// SCIM 2.0 User Resource
// ============================================================================

/// SCIM 2.0 User resource
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimUser {
    /// SCIM schema URIs
    #[serde(default = "default_user_schemas")]
    pub schemas: Vec<String>,
    /// Unique identifier (our internal ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// External identifier from the IDP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Username (unique identifier for the user)
    pub user_name: String,
    /// User's name components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ScimName>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Email addresses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<ScimEmail>,
    /// Whether the user is active
    #[serde(default = "default_true")]
    pub active: bool,
    /// Groups the user belongs to (read-only)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<ScimUserGroup>,
    /// Resource metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ScimMeta>,
}

fn default_user_schemas() -> Vec<String> {
    vec!["urn:ietf:params:scim:schemas:core:2.0:User".to_string()]
}

fn default_true() -> bool {
    true
}

// ============================================================================
// SCIM 2.0 Group Resource
// ============================================================================

/// SCIM 2.0 Group resource
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimGroup {
    /// SCIM schema URIs
    #[serde(default = "default_group_schemas")]
    pub schemas: Vec<String>,
    /// Unique identifier (our internal ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// External identifier from the IDP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Display name for the group
    pub display_name: String,
    /// Group members
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<ScimGroupMember>,
    /// Resource metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ScimMeta>,
}

fn default_group_schemas() -> Vec<String> {
    vec!["urn:ietf:params:scim:schemas:core:2.0:Group".to_string()]
}

// ============================================================================
// SCIM 2.0 List Response
// ============================================================================

/// SCIM 2.0 ListResponse for paginated results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimListResponse<T> {
    /// SCIM schema URIs
    pub schemas: Vec<String>,
    /// Total number of results
    pub total_results: i64,
    /// Number of items per page
    pub items_per_page: i64,
    /// Start index (1-based)
    pub start_index: i64,
    /// The resources
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

impl<T> ScimListResponse<T> {
    pub fn new(
        resources: Vec<T>,
        total_results: i64,
        start_index: i64,
        items_per_page: i64,
    ) -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:ListResponse".to_string()],
            total_results,
            items_per_page,
            start_index,
            resources,
        }
    }
}

// ============================================================================
// SCIM 2.0 PATCH Operation
// ============================================================================

/// SCIM 2.0 PATCH operation type
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScimPatchOp {
    Add,
    Remove,
    Replace,
}

/// SCIM 2.0 PATCH operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimPatchOperation {
    pub op: ScimPatchOp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// SCIM 2.0 PATCH request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimPatchRequest {
    pub schemas: Vec<String>,
    #[serde(rename = "Operations")]
    pub operations: Vec<ScimPatchOperation>,
}

// ============================================================================
// SCIM 2.0 Error Response
// ============================================================================

/// SCIM 2.0 Error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scim_type: Option<String>,
    pub detail: String,
}

impl ScimError {
    pub fn new(status: u16, detail: impl Into<String>, scim_type: Option<String>) -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            status: status.to_string(),
            scim_type,
            detail: detail.into(),
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, detail, None)
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, detail, Some("invalidValue".to_string()))
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self::new(409, detail, Some("uniqueness".to_string()))
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

/// SCIM query parameters for list endpoints
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ScimListParams {
    /// Filter expression (e.g., userName eq "john")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Attribute(s) to sort by
    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    /// Sort order (ascending or descending)
    #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
    /// 1-based index of the first result
    #[serde(rename = "startIndex", default = "default_start_index")]
    pub start_index: i64,
    /// Number of results per page
    #[serde(default = "default_count")]
    pub count: i64,
}

fn default_start_index() -> i64 {
    1
}

fn default_count() -> i64 {
    100
}

impl Default for ScimListParams {
    fn default() -> Self {
        Self {
            filter: None,
            sort_by: None,
            sort_order: None,
            start_index: default_start_index(),
            count: default_count(),
        }
    }
}

// ============================================================================
// Response Types
// ============================================================================

pub type ScimUserResponse = ScimUser;
pub type ScimGroupResponse = ScimGroup;
pub type ScimUserListResponse = ScimListResponse<ScimUser>;
pub type ScimGroupListResponse = ScimListResponse<ScimGroup>;

// ============================================================================
// Conversion Functions
// ============================================================================

/// Convert our internal User to a SCIM User
pub fn user_to_scim(user: &User, base_url: &str) -> ScimUser {
    let primary_email = user.email.clone();
    let emails = primary_email
        .map(|email| {
            vec![ScimEmail {
                value: email,
                email_type: Some("work".to_string()),
                primary: true,
            }]
        })
        .unwrap_or_default();

    ScimUser {
        schemas: default_user_schemas(),
        id: Some(user.id.clone()),
        external_id: Some(user.id.clone()),
        user_name: user.email.clone().unwrap_or_else(|| user.id.clone()),
        name: None,
        display_name: user.email.clone(),
        emails,
        active: true,
        groups: vec![],
        meta: Some(ScimMeta {
            resource_type: "User".to_string(),
            created: Some(user.created_at.to_string()),
            last_modified: Some(user.updated_at.to_string()),
            location: Some(format!("{}/Users/{}", base_url, user.id)),
            version: None,
        }),
    }
}

/// Convert our internal Group to a SCIM Group
pub fn group_to_scim(group: &Group, members: &[GroupMemberWithUser], base_url: &str) -> ScimGroup {
    let scim_members: Vec<ScimGroupMember> = members
        .iter()
        .map(|m| ScimGroupMember {
            value: m.user.id.clone(),
            ref_uri: Some(format!("{}/Users/{}", base_url, m.user.id)),
            display: m.user.email.clone(),
            member_type: Some("User".to_string()),
        })
        .collect();

    ScimGroup {
        schemas: default_group_schemas(),
        id: Some(group.id.clone()),
        external_id: Some(group.id.clone()),
        display_name: group.name.clone(),
        members: scim_members,
        meta: Some(ScimMeta {
            resource_type: "Group".to_string(),
            created: Some(group.created_at.to_string()),
            last_modified: Some(group.updated_at.to_string()),
            location: Some(format!("{}/Groups/{}", base_url, group.id)),
            version: None,
        }),
    }
}

// ============================================================================
// SCIM User Logic Functions
// ============================================================================

/// Create a user from a SCIM User payload
#[authz_role(Admin, permission = "scim_user:write")]
#[authn]
pub async fn create_user_from_scim(

    repo: &impl UserRepositoryLike,
    scim_user: ScimUser,
) -> Result<ScimUser, CommonError> {
    let now = WrappedChronoDateTime::now();

    // Use external_id if provided, otherwise generate a UUID
    let user_id = scim_user
        .external_id
        .clone()
        .or_else(|| scim_user.id.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    debug!(user_id = %user_id, user_name = %scim_user.user_name, "Resolved user ID for SCIM user creation");

    // Extract primary email
    let email = scim_user
        .emails
        .iter()
        .find(|e| e.primary)
        .or_else(|| scim_user.emails.first())
        .map(|e| e.value.clone())
        .or_else(|| {
            // If no emails, use userName if it looks like an email
            if scim_user.user_name.contains('@') {
                debug!(user_name = %scim_user.user_name, "Using userName as email");
                Some(scim_user.user_name.clone())
            } else {
                None
            }
        });

    // Check if user already exists
    if let Some(existing) = repo.get_user_by_id(&user_id).await? {
        debug!(user_id = %existing.id, "User already exists");
        return Err(CommonError::InvalidRequest {
            msg: format!("User with id '{}' already exists", existing.id),
            source: None,
        });
    }

    let user = User {
        id: user_id.clone(),
        user_type: UserType::Human,
        email,
        role: Role::User,
        description: None,
        created_at: now,
        updated_at: now,
    };

    trace!(user_id = %user_id, "Inserting SCIM user into repository");
    repo.create_user(&user).await?;

    // Fetch the created user and return as SCIM
    let user = repo
        .get_user_by_id(&user_id)
        .await?
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Failed to retrieve created user")))?;

    info!(user_id = %user_id, "SCIM user created");
    Ok(user_to_scim(&user, ""))
}

/// Get a user by ID and return as SCIM User
#[authz_role(Admin, permission = "scim_user:read")]
#[authn]
pub async fn get_user_scim(

    repo: &impl UserRepositoryLike,
    user_id: &str,
    base_url: &str,
) -> Result<ScimUser, CommonError> {
    get_user_scim_internal(repo, user_id, base_url).await
}

/// Internal function to get a user as SCIM without auth check.
/// Used by other SCIM functions that have already authenticated.
async fn get_user_scim_internal(
    repo: &impl UserRepositoryLike,
    user_id: &str,
    base_url: &str,
) -> Result<ScimUser, CommonError> {
    let user = repo
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User not found".to_string(),
            lookup_id: user_id.to_string(),
            source: None,
        })?;

    // Get user's groups
    let groups_response = repo
        .list_user_groups(
            user_id,
            &PaginationRequest {
                page_size: 100,
                next_page_token: None,
            },
        )
        .await?;

    let mut scim_user = user_to_scim(&user, base_url);
    scim_user.groups = groups_response
        .items
        .iter()
        .map(|ug| ScimUserGroup {
            value: ug.group.id.clone(),
            ref_uri: Some(format!("{}/Groups/{}", base_url, ug.group.id)),
            display: Some(ug.group.name.clone()),
            group_type: Some("direct".to_string()),
        })
        .collect();

    Ok(scim_user)
}

/// List users with SCIM pagination
///
/// Note: Our repository uses cursor-based pagination without a total count.
/// Per SCIM 2.0 spec, `totalResults` should reflect the total matching resources.
/// Since we don't have this information, we return the current page count when
/// there's no more data, or indicate unknown (-1) when pagination continues.
#[authz_role(Admin, permission = "scim_user:list")]
#[authn]
pub async fn list_users_scim(

    repo: &impl UserRepositoryLike,
    params: ScimListParams,
    base_url: &str,
) -> Result<ScimUserListResponse, CommonError> {
    let pagination = PaginationRequest {
        page_size: params.count,
        next_page_token: None,
    };

    let users_response = repo.list_users(&pagination, None, None).await?;

    let scim_users: Vec<ScimUser> = users_response
        .items
        .iter()
        .map(|u| user_to_scim(u, base_url))
        .collect();

    // If there's no next page, total is the current count; otherwise unknown
    let total = if users_response.next_page_token.is_none() {
        scim_users.len() as i64
    } else {
        // SCIM spec doesn't require totalResults, but -1 indicates unknown
        -1
    };

    Ok(ScimListResponse::new(
        scim_users,
        total,
        params.start_index,
        params.count,
    ))
}

/// Replace a user (PUT operation)
#[authz_role(Admin, permission = "scim_user:write")]
#[authn]
pub async fn replace_user_scim(

    repo: &impl UserRepositoryLike,
    user_id: &str,
    scim_user: ScimUser,
    base_url: &str,
) -> Result<ScimUser, CommonError> {
    // Check if user exists
    let _existing = repo
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User not found".to_string(),
            lookup_id: user_id.to_string(),
            source: None,
        })?;

    // Extract primary email
    let email = scim_user
        .emails
        .iter()
        .find(|e| e.primary)
        .or_else(|| scim_user.emails.first())
        .map(|e| e.value.clone())
        .or_else(|| {
            if scim_user.user_name.contains('@') {
                Some(scim_user.user_name.clone())
            } else {
                None
            }
        });

    let update_user = UpdateUser {
        email,
        role: None,
        description: None,
    };

    trace!(user_id = %user_id, "Updating user attributes");
    repo.update_user(user_id, &update_user).await?;

    debug!(user_id = %user_id, "SCIM user replaced");
    // Return updated user
    get_user_scim_internal(repo, user_id, base_url).await
}

/// Patch a user (PATCH operation)
#[authz_role(Admin, permission = "scim_user:write")]
#[authn]
pub async fn patch_user_scim(

    repo: &impl UserRepositoryLike,
    user_id: &str,
    patch_request: ScimPatchRequest,
    base_url: &str,
) -> Result<ScimUser, CommonError> {
    // Check if user exists
    let existing = repo
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User not found".to_string(),
            lookup_id: user_id.to_string(),
            source: None,
        })?;

    let mut email = existing.email.clone();
    let role: Option<Role> = None; // SCIM doesn't update roles

    for op in patch_request.operations {
        match op.op {
            ScimPatchOp::Replace | ScimPatchOp::Add => {
                if let Some(path) = &op.path {
                    match path.as_str() {
                        "emails" | "emails[type eq \"work\"].value" => {
                            if let Some(value) = &op.value {
                                if let Some(email_str) = value.as_str() {
                                    email = Some(email_str.to_string());
                                } else if let Some(emails) = value.as_array() {
                                    if let Some(first) = emails.first() {
                                        if let Some(val) =
                                            first.get("value").and_then(|v| v.as_str())
                                        {
                                            email = Some(val.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        "active" => {
                            // We don't have an active field, but we could handle deactivation
                            // by updating role or similar
                        }
                        _ => {}
                    }
                } else if let Some(value) = &op.value {
                    // No path means replace entire resource attributes
                    if let Some(emails) = value.get("emails").and_then(|v| v.as_array()) {
                        if let Some(first) = emails.first() {
                            if let Some(val) = first.get("value").and_then(|v| v.as_str()) {
                                email = Some(val.to_string());
                            }
                        }
                    }
                }
            }
            ScimPatchOp::Remove => {
                if let Some(path) = &op.path {
                    if path == "emails" {
                        email = None;
                    }
                }
            }
        }
    }

    let update_user = UpdateUser {
        email,
        role,
        description: None,
    };
    trace!(user_id = %user_id, "Updating user attributes");
    repo.update_user(user_id, &update_user).await?;

    debug!(user_id = %user_id, "SCIM user patched");
    get_user_scim_internal(repo, user_id, base_url).await
}

/// Delete a user
#[authz_role(Admin, permission = "scim_user:delete")]
#[authn]
pub async fn delete_user_scim(

    repo: &impl UserRepositoryLike,
    user_id: &str,
) -> Result<(), CommonError> {
    // Check if user exists
    let _existing = repo
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "User not found".to_string(),
            lookup_id: user_id.to_string(),
            source: None,
        })?;

    // Delete user's group memberships first
    trace!(user_id = %user_id, "Deleting user group memberships");
    repo.delete_group_memberships_by_user_id(user_id).await?;
    // Delete user's API keys
    trace!(user_id = %user_id, "Deleting user API keys");
    repo.delete_api_keys_by_user_id(user_id).await?;
    // Delete the user
    trace!(user_id = %user_id, "Deleting user record");
    repo.delete_user(user_id).await?;

    info!(user_id = %user_id, "SCIM user deleted");
    Ok(())
}

// ============================================================================
// SCIM Group Logic Functions
// ============================================================================

/// Create a group from a SCIM Group payload
#[authz_role(Admin, permission = "scim_group:write")]
#[authn]
pub async fn create_group_from_scim(

    repo: &impl UserRepositoryLike,
    scim_group: ScimGroup,
    base_url: &str,
) -> Result<ScimGroup, CommonError> {
    let now = WrappedChronoDateTime::now();

    // Use external_id if provided, otherwise generate a UUID
    let group_id = scim_group
        .external_id
        .clone()
        .or_else(|| scim_group.id.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    debug!(group_id = %group_id, display_name = %scim_group.display_name, "Resolved group ID for SCIM group creation");

    // Check if group already exists
    if let Some(existing) = repo.get_group_by_id(&group_id).await? {
        debug!(group_id = %existing.id, "Group already exists");
        return Err(CommonError::InvalidRequest {
            msg: format!("Group with id '{}' already exists", existing.id),
            source: None,
        });
    }

    let group = Group {
        id: group_id.clone(),
        name: scim_group.display_name.clone(),
        created_at: now,
        updated_at: now,
    };

    trace!(group_id = %group_id, "Inserting SCIM group into repository");
    repo.create_group(&group).await?;

    // Add members if provided
    let member_count = scim_group.members.len();
    let mut added_count = 0;
    for member in &scim_group.members {
        // Verify the user exists
        if repo.get_user_by_id(&member.value).await?.is_some() {
            let membership = GroupMembership {
                group_id: group_id.clone(),
                user_id: member.value.clone(),
                created_at: now,
                updated_at: now,
            };
            // Ignore errors for member creation (user might not exist)
            if repo.create_group_membership(&membership).await.is_ok() {
                added_count += 1;
            }
        } else {
            warn!(group_id = %group_id, user_id = %member.value, "Skipping member: user not found");
        }
    }

    if member_count > 0 {
        debug!(group_id = %group_id, requested = member_count, added = added_count, "Added group members");
    }

    info!(group_id = %group_id, display_name = %scim_group.display_name, "SCIM group created");
    // Return the created group
    get_group_scim_internal(repo, &group_id, base_url).await
}

/// Get a group by ID and return as SCIM Group
#[authz_role(Admin, permission = "scim_group:read")]
#[authn]
pub async fn get_group_scim(

    repo: &impl UserRepositoryLike,
    group_id: &str,
    base_url: &str,
) -> Result<ScimGroup, CommonError> {
    get_group_scim_internal(repo, group_id, base_url).await
}

/// Internal function to get a group as SCIM without auth check.
/// Used by other SCIM functions that have already authenticated.
async fn get_group_scim_internal(
    repo: &impl UserRepositoryLike,
    group_id: &str,
    base_url: &str,
) -> Result<ScimGroup, CommonError> {
    let group = repo
        .get_group_by_id(group_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "Group not found".to_string(),
            lookup_id: group_id.to_string(),
            source: None,
        })?;

    // Get group members
    let members_response = repo
        .list_group_members(
            group_id,
            &PaginationRequest {
                page_size: 1000,
                next_page_token: None,
            },
        )
        .await?;

    Ok(group_to_scim(&group, &members_response.items, base_url))
}

/// List groups with SCIM pagination
///
/// Note: See `list_users_scim` for comments on totalResults limitations.
#[authz_role(Admin, permission = "scim_group:list")]
#[authn]
pub async fn list_groups_scim(

    repo: &impl UserRepositoryLike,
    params: ScimListParams,
    base_url: &str,
) -> Result<ScimGroupListResponse, CommonError> {
    let pagination = PaginationRequest {
        page_size: params.count,
        next_page_token: None,
    };

    let groups_response = repo.list_groups(&pagination).await?;

    let mut scim_groups = Vec::new();
    for group in &groups_response.items {
        let members_response = repo
            .list_group_members(
                &group.id,
                &PaginationRequest {
                    page_size: 1000,
                    next_page_token: None,
                },
            )
            .await?;
        scim_groups.push(group_to_scim(group, &members_response.items, base_url));
    }

    // If there's no next page, total is the current count; otherwise unknown
    let total = if groups_response.next_page_token.is_none() {
        scim_groups.len() as i64
    } else {
        -1
    };

    Ok(ScimListResponse::new(
        scim_groups,
        total,
        params.start_index,
        params.count,
    ))
}

/// Replace a group (PUT operation)
#[authz_role(Admin, permission = "scim_group:write")]
#[authn]
pub async fn replace_group_scim(

    repo: &impl UserRepositoryLike,
    group_id: &str,
    scim_group: ScimGroup,
    base_url: &str,
) -> Result<ScimGroup, CommonError> {
    let now = WrappedChronoDateTime::now();

    // Check if group exists
    let _existing = repo
        .get_group_by_id(group_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "Group not found".to_string(),
            lookup_id: group_id.to_string(),
            source: None,
        })?;

    // Update group name
    trace!(group_id = %group_id, display_name = %scim_group.display_name, "Updating group name");
    repo.update_group(group_id, &scim_group.display_name)
        .await?;

    // Replace members: delete all existing and add new ones
    trace!(group_id = %group_id, "Removing existing group memberships");
    repo.delete_group_memberships_by_group_id(group_id).await?;

    let mut added_count = 0;
    for member in &scim_group.members {
        if repo.get_user_by_id(&member.value).await?.is_some() {
            let membership = GroupMembership {
                group_id: group_id.to_string(),
                user_id: member.value.clone(),
                created_at: now,
                updated_at: now,
            };
            if repo.create_group_membership(&membership).await.is_ok() {
                added_count += 1;
            }
        } else {
            warn!(group_id = %group_id, user_id = %member.value, "Skipping member: user not found");
        }
    }

    debug!(group_id = %group_id, member_count = added_count, "SCIM group replaced");
    get_group_scim_internal(repo, group_id, base_url).await
}

/// Patch a group (PATCH operation)
#[authz_role(Admin, permission = "scim_group:write")]
#[authn]
pub async fn patch_group_scim(

    repo: &impl UserRepositoryLike,
    group_id: &str,
    patch_request: ScimPatchRequest,
    base_url: &str,
) -> Result<ScimGroup, CommonError> {
    let now = WrappedChronoDateTime::now();

    // Check if group exists
    let existing = repo
        .get_group_by_id(group_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "Group not found".to_string(),
            lookup_id: group_id.to_string(),
            source: None,
        })?;

    let mut display_name = existing.name.clone();

    for op in patch_request.operations {
        match op.op {
            ScimPatchOp::Replace | ScimPatchOp::Add => {
                if let Some(path) = &op.path {
                    match path.as_str() {
                        "displayName" => {
                            if let Some(value) = &op.value {
                                if let Some(name) = value.as_str() {
                                    display_name = name.to_string();
                                }
                            }
                        }
                        "members" => {
                            if let Some(value) = &op.value {
                                if let Some(members) = value.as_array() {
                                    for member_val in members {
                                        if let Some(user_id) =
                                            member_val.get("value").and_then(|v| v.as_str())
                                        {
                                            if repo.get_user_by_id(user_id).await?.is_some() {
                                                // Check if membership already exists
                                                if repo
                                                    .get_group_membership(group_id, user_id)
                                                    .await?
                                                    .is_none()
                                                {
                                                    let membership = GroupMembership {
                                                        group_id: group_id.to_string(),
                                                        user_id: user_id.to_string(),
                                                        created_at: now,
                                                        updated_at: now,
                                                    };
                                                    let _ = repo
                                                        .create_group_membership(&membership)
                                                        .await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Some(value) = &op.value {
                    // No path - look for displayName or members in value
                    if let Some(name) = value.get("displayName").and_then(|v| v.as_str()) {
                        display_name = name.to_string();
                    }
                    if let Some(members) = value.get("members").and_then(|v| v.as_array()) {
                        for member_val in members {
                            if let Some(user_id) = member_val.get("value").and_then(|v| v.as_str())
                            {
                                if repo.get_user_by_id(user_id).await?.is_some()
                                    && repo
                                        .get_group_membership(group_id, user_id)
                                        .await?
                                        .is_none()
                                {
                                    let membership = GroupMembership {
                                        group_id: group_id.to_string(),
                                        user_id: user_id.to_string(),
                                        created_at: now,
                                        updated_at: now,
                                    };
                                    let _ = repo.create_group_membership(&membership).await;
                                }
                            }
                        }
                    }
                }
            }
            ScimPatchOp::Remove => {
                if let Some(path) = &op.path {
                    // Handle member removal: members[value eq "user_id"]
                    if path.starts_with("members[") {
                        // Parse the filter to extract user_id
                        // Format: members[value eq "user_id"]
                        if let Some(start) = path.find("\"") {
                            if let Some(end) = path.rfind("\"") {
                                if start < end {
                                    let user_id = &path[start + 1..end];
                                    let _ = repo.delete_group_membership(group_id, user_id).await;
                                }
                            }
                        }
                    } else if path == "members" {
                        // Remove all members
                        repo.delete_group_memberships_by_group_id(group_id).await?;
                    }
                }
            }
        }
    }

    // Update display name if changed
    if display_name != existing.name {
        trace!(group_id = %group_id, new_name = %display_name, "Updating group display name");
        repo.update_group(group_id, &display_name).await?;
    }

    debug!(group_id = %group_id, "SCIM group patched");
    get_group_scim_internal(repo, group_id, base_url).await
}

/// Delete a group
#[authz_role(Admin, permission = "scim_group:delete")]
#[authn]
pub async fn delete_group_scim(

    repo: &impl UserRepositoryLike,
    group_id: &str,
) -> Result<(), CommonError> {
    // Check if group exists
    let _existing = repo
        .get_group_by_id(group_id)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: "Group not found".to_string(),
            lookup_id: group_id.to_string(),
            source: None,
        })?;

    // Delete group memberships first
    trace!(group_id = %group_id, "Deleting group memberships");
    repo.delete_group_memberships_by_group_id(group_id).await?;
    // Delete the group
    trace!(group_id = %group_id, "Deleting group record");
    repo.delete_group(group_id).await?;

    info!(group_id = %group_id, "SCIM group deleted");
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;
        use crate::repository::Repository;
        use shared::identity::Role;
        use shared::primitives::SqlMigrationLoader;
        use shared::test_utils::helpers::MockAuthClient;

        async fn setup_test_repo() -> Repository {
            shared::setup_test!();

            let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
                Repository::load_sql_migrations(),
            ])
            .await
            .unwrap();

            Repository::new(conn)
        }

        /// Create a mock auth client that returns an authenticated admin identity
        fn mock_admin_auth_client() -> MockAuthClient {
            MockAuthClient::new(shared::test_utils::helpers::test_admin_machine())
        }

        #[tokio::test]
        async fn test_create_user_from_scim() {
            let repo = setup_test_repo().await;

            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("ext-123".to_string()),
                user_name: "john.doe@example.com".to_string(),
                name: Some(ScimName {
                    given_name: Some("John".to_string()),
                    family_name: Some("Doe".to_string()),
                    ..Default::default()
                }),
                display_name: Some("John Doe".to_string()),
                emails: vec![ScimEmail {
                    value: "john.doe@example.com".to_string(),
                    email_type: Some("work".to_string()),
                    primary: true,
                }],
                active: true,
                groups: vec![],
                meta: None,
            };

            let auth_client = mock_admin_auth_client();
            let result = create_user_from_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await;
            assert!(result.is_ok());

            let created = result.unwrap();
            assert_eq!(created.id, Some("ext-123".to_string()));
            assert_eq!(created.user_name, "john.doe@example.com");
            assert!(!created.emails.is_empty());
            assert_eq!(created.emails[0].value, "john.doe@example.com");
        }

        #[tokio::test]
        async fn test_create_user_from_scim_duplicate() {
            let repo = setup_test_repo().await;

            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("ext-456".to_string()),
                user_name: "jane.doe@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![ScimEmail {
                    value: "jane.doe@example.com".to_string(),
                    email_type: None,
                    primary: true,
                }],
                active: true,
                groups: vec![],
                meta: None,
            };

            // Create first user
            let auth_client = mock_admin_auth_client();
            let result1 = create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user.clone(),
            )
            .await;
            assert!(result1.is_ok());

            // Try to create duplicate
            let result2 = create_user_from_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await;
            assert!(result2.is_err());
        }

        #[tokio::test]
        async fn test_get_user_scim() {
            let repo = setup_test_repo().await;

            // Create a user first
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("get-test-user".to_string()),
                user_name: "get.test@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![ScimEmail {
                    value: "get.test@example.com".to_string(),
                    email_type: None,
                    primary: true,
                }],
                active: true,
                groups: vec![],
                meta: None,
            };

            let auth_client = mock_admin_auth_client();
            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Get the user
            let result = get_user_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "get-test-user",
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let user = result.unwrap();
            assert_eq!(user.id, Some("get-test-user".to_string()));
            assert!(user.meta.is_some());
            assert!(
                user.meta
                    .unwrap()
                    .location
                    .unwrap()
                    .contains("get-test-user")
            );
        }

        #[tokio::test]
        async fn test_get_user_scim_not_found() {
            let repo = setup_test_repo().await;

            let auth_client = mock_admin_auth_client();
            let result = get_user_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "nonexistent",
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_list_users_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create some users
            for i in 0..3 {
                let scim_user = ScimUser {
                    schemas: default_user_schemas(),
                    id: None,
                    external_id: Some(format!("list-user-{i}")),
                    user_name: format!("user{i}@example.com"),
                    name: None,
                    display_name: None,
                    emails: vec![ScimEmail {
                        value: format!("user{i}@example.com"),
                        email_type: None,
                        primary: true,
                    }],
                    active: true,
                    groups: vec![],
                    meta: None,
                };
                create_user_from_scim(
                    auth_client.clone(), http::HeaderMap::new(),
                    &repo,
                    scim_user,
                )
                .await
                .unwrap();
            }

            let result = list_users_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                ScimListParams::default(),
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let list = result.unwrap();
            assert_eq!(list.total_results, 3);
            assert_eq!(list.resources.len(), 3);
        }

        #[tokio::test]
        async fn test_replace_user_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a user
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("replace-user".to_string()),
                user_name: "replace@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![ScimEmail {
                    value: "replace@example.com".to_string(),
                    email_type: None,
                    primary: true,
                }],
                active: true,
                groups: vec![],
                meta: None,
            };

            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Replace the user
            let updated_user = ScimUser {
                schemas: default_user_schemas(),
                id: Some("replace-user".to_string()),
                external_id: Some("replace-user".to_string()),
                user_name: "updated@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![ScimEmail {
                    value: "updated@example.com".to_string(),
                    email_type: None,
                    primary: true,
                }],
                active: true,
                groups: vec![],
                meta: None,
            };

            let result = replace_user_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "replace-user",
                updated_user,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let user = result.unwrap();
            assert!(!user.emails.is_empty());
            assert_eq!(user.emails[0].value, "updated@example.com");
        }

        #[tokio::test]
        async fn test_patch_user_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a user
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("patch-user".to_string()),
                user_name: "patch@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![ScimEmail {
                    value: "patch@example.com".to_string(),
                    email_type: None,
                    primary: true,
                }],
                active: true,
                groups: vec![],
                meta: None,
            };

            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Patch the user
            let patch_request = ScimPatchRequest {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:PatchOp".to_string()],
                operations: vec![ScimPatchOperation {
                    op: ScimPatchOp::Replace,
                    path: Some("emails".to_string()),
                    value: Some(
                        serde_json::json!([{"value": "patched@example.com", "primary": true}]),
                    ),
                }],
            };

            let result = patch_user_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "patch-user",
                patch_request,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let user = result.unwrap();
            assert!(!user.emails.is_empty());
            assert_eq!(user.emails[0].value, "patched@example.com");
        }

        #[tokio::test]
        async fn test_delete_user_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a user
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("delete-user".to_string()),
                user_name: "delete@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![],
                active: true,
                groups: vec![],
                meta: None,
            };

            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Delete the user
            let result = delete_user_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                "delete-user",
            )
            .await;
            assert!(result.is_ok());

            // Verify deletion
            let get_result = get_user_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "delete-user",
                "",
            )
            .await;
            assert!(get_result.is_err());
        }

        #[tokio::test]
        async fn test_create_group_from_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("group-123".to_string()),
                display_name: "Engineering".to_string(),
                members: vec![],
                meta: None,
            };

            let result = create_group_from_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                scim_group,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let created = result.unwrap();
            assert_eq!(created.id, Some("group-123".to_string()));
            assert_eq!(created.display_name, "Engineering");
        }

        #[tokio::test]
        async fn test_create_group_with_members() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a user first
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("member-user".to_string()),
                user_name: "member@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![],
                active: true,
                groups: vec![],
                meta: None,
            };
            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Create group with member
            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("group-with-members".to_string()),
                display_name: "Team".to_string(),
                members: vec![ScimGroupMember {
                    value: "member-user".to_string(),
                    ref_uri: None,
                    display: None,
                    member_type: Some("User".to_string()),
                }],
                meta: None,
            };

            let result = create_group_from_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                scim_group,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let created = result.unwrap();
            assert_eq!(created.members.len(), 1);
            assert_eq!(created.members[0].value, "member-user");
        }

        #[tokio::test]
        async fn test_get_group_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a group
            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("get-group".to_string()),
                display_name: "Test Group".to_string(),
                members: vec![],
                meta: None,
            };
            create_group_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_group,
                "",
            )
            .await
            .unwrap();

            // Get the group
            let result = get_group_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "get-group",
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let group = result.unwrap();
            assert_eq!(group.id, Some("get-group".to_string()));
            assert_eq!(group.display_name, "Test Group");
        }

        #[tokio::test]
        async fn test_list_groups_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create some groups
            for i in 0..3 {
                let scim_group = ScimGroup {
                    schemas: default_group_schemas(),
                    id: None,
                    external_id: Some(format!("list-group-{i}")),
                    display_name: format!("Group {i}"),
                    members: vec![],
                    meta: None,
                };
                create_group_from_scim(
                    auth_client.clone(), http::HeaderMap::new(),
                    &repo,
                    scim_group,
                    "",
                )
                .await
                .unwrap();
            }

            let result = list_groups_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                ScimListParams::default(),
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let list = result.unwrap();
            assert_eq!(list.total_results, 3);
            assert_eq!(list.resources.len(), 3);
        }

        #[tokio::test]
        async fn test_replace_group_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a group
            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("replace-group".to_string()),
                display_name: "Original Name".to_string(),
                members: vec![],
                meta: None,
            };
            create_group_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_group,
                "",
            )
            .await
            .unwrap();

            // Replace the group
            let updated_group = ScimGroup {
                schemas: default_group_schemas(),
                id: Some("replace-group".to_string()),
                external_id: Some("replace-group".to_string()),
                display_name: "Updated Name".to_string(),
                members: vec![],
                meta: None,
            };

            let result = replace_group_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "replace-group",
                updated_group,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let group = result.unwrap();
            assert_eq!(group.display_name, "Updated Name");
        }

        #[tokio::test]
        async fn test_patch_group_scim_add_member() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a user
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("patch-member".to_string()),
                user_name: "patch-member@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![],
                active: true,
                groups: vec![],
                meta: None,
            };
            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Create a group
            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("patch-group".to_string()),
                display_name: "Patch Group".to_string(),
                members: vec![],
                meta: None,
            };
            create_group_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_group,
                "",
            )
            .await
            .unwrap();

            // Patch to add member
            let patch_request = ScimPatchRequest {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:PatchOp".to_string()],
                operations: vec![ScimPatchOperation {
                    op: ScimPatchOp::Add,
                    path: Some("members".to_string()),
                    value: Some(serde_json::json!([{"value": "patch-member"}])),
                }],
            };

            let result = patch_group_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "patch-group",
                patch_request,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let group = result.unwrap();
            assert_eq!(group.members.len(), 1);
            assert_eq!(group.members[0].value, "patch-member");
        }

        #[tokio::test]
        async fn test_patch_group_scim_remove_member() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a user
            let scim_user = ScimUser {
                schemas: default_user_schemas(),
                id: None,
                external_id: Some("remove-member".to_string()),
                user_name: "remove@example.com".to_string(),
                name: None,
                display_name: None,
                emails: vec![],
                active: true,
                groups: vec![],
                meta: None,
            };
            create_user_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_user,
            )
            .await
            .unwrap();

            // Create a group with the member
            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("remove-member-group".to_string()),
                display_name: "Remove Member Group".to_string(),
                members: vec![ScimGroupMember {
                    value: "remove-member".to_string(),
                    ref_uri: None,
                    display: None,
                    member_type: None,
                }],
                meta: None,
            };
            create_group_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_group,
                "",
            )
            .await
            .unwrap();

            // Patch to remove member
            let patch_request = ScimPatchRequest {
                schemas: vec!["urn:ietf:params:scim:api:messages:2.0:PatchOp".to_string()],
                operations: vec![ScimPatchOperation {
                    op: ScimPatchOp::Remove,
                    path: Some("members[value eq \"remove-member\"]".to_string()),
                    value: None,
                }],
            };

            let result = patch_group_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "remove-member-group",
                patch_request,
                "https://example.com/scim/v2",
            )
            .await;
            assert!(result.is_ok());

            let group = result.unwrap();
            assert!(group.members.is_empty());
        }

        #[tokio::test]
        async fn test_delete_group_scim() {
            let repo = setup_test_repo().await;
            let auth_client = mock_admin_auth_client();

            // Create a group
            let scim_group = ScimGroup {
                schemas: default_group_schemas(),
                id: None,
                external_id: Some("delete-group".to_string()),
                display_name: "Delete Group".to_string(),
                members: vec![],
                meta: None,
            };
            create_group_from_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                scim_group,
                "",
            )
            .await
            .unwrap();

            // Delete the group
            let result = delete_group_scim(
                auth_client.clone(), http::HeaderMap::new(),
                &repo,
                "delete-group",
            )
            .await;
            assert!(result.is_ok());

            // Verify deletion
            let get_result = get_group_scim(
                auth_client, http::HeaderMap::new(),
                &repo,
                "delete-group",
                "",
            )
            .await;
            assert!(get_result.is_err());
        }

        #[tokio::test]
        async fn test_user_to_scim_conversion() {
            let user = User {
                id: "user-123".to_string(),
                user_type: UserType::Human,
                email: Some("test@example.com".to_string()),
                role: Role::User,
                description: None,
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
            };

            let scim = user_to_scim(&user, "https://example.com/scim/v2");

            assert_eq!(scim.id, Some("user-123".to_string()));
            assert_eq!(scim.user_name, "test@example.com");
            assert!(!scim.emails.is_empty());
            assert_eq!(scim.emails[0].value, "test@example.com");
            assert!(scim.meta.is_some());
        }

        #[tokio::test]
        async fn test_group_to_scim_conversion() {
            let group = Group {
                id: "group-123".to_string(),
                name: "Test Group".to_string(),
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
            };

            let scim = group_to_scim(&group, &[], "https://example.com/scim/v2");

            assert_eq!(scim.id, Some("group-123".to_string()));
            assert_eq!(scim.display_name, "Test Group");
            assert!(scim.members.is_empty());
            assert!(scim.meta.is_some());
        }

        #[tokio::test]
        async fn test_scim_error_creation() {
            let not_found = ScimError::not_found("User not found");
            assert_eq!(not_found.status, "404");
            assert_eq!(not_found.detail, "User not found");

            let bad_request = ScimError::bad_request("Invalid email");
            assert_eq!(bad_request.status, "400");
            assert_eq!(bad_request.scim_type, Some("invalidValue".to_string()));

            let conflict = ScimError::conflict("User already exists");
            assert_eq!(conflict.status, "409");
            assert_eq!(conflict.scim_type, Some("uniqueness".to_string()));
        }
    }
}
