//! SCIM (System for Cross-domain Identity Management) sync logic
//!
//! This module provides logic functions for syncing external IDP users and groups
//! into our repository based on the SCIM 2.0 specification.

use crate::logic::user::{Group, GroupMembership, Role, User, UserType};
use crate::repository::{GroupMemberWithUser, UpdateUser, UserRepositoryLike};
use serde::{Deserialize, Serialize};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime},
};
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

/// Provides the SCIM core User schema URIs used for User resources.
///
/// # Examples
///
/// ```
/// let schemas = default_user_schemas();
/// assert_eq!(schemas, vec!["urn:ietf:params:scim:schemas:core:2.0:User".to_string()]);
/// ```
fn default_user_schemas() -> Vec<String> {
    vec!["urn:ietf:params:scim:schemas:core:2.0:User".to_string()]
}

/// Returns the default `true` value for fields that should default to true.
///
/// # Examples
///
/// ```
/// assert_eq!(default_true(), true);
/// ```
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

/// Default SCIM core schema URIs for Group resources.
///
/// # Examples
///
/// ```
/// let schemas = default_group_schemas();
/// assert_eq!(schemas, vec!["urn:ietf:params:scim:schemas:core:2.0:Group".to_string()]);
/// ```
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
    /// Constructs a SCIM ListResponse containing the given resources and pagination metadata.
    ///
    /// The response's `schemas` field is set to the standard SCIM ListResponse schema.
    ///
    —
    /// # Parameters
    ///
    /// - `resources`: the page of resources to include in this response.
    /// - `total_results`: the total number of matching results across all pages.
    /// - `start_index`: the 1-based index of the first resource in `resources` within the total result set.
    /// - `items_per_page`: the maximum number of items returned in this page (page size).
    ///
    /// # Examples
    ///
    /// ```
    /// let resp = ScimListResponse::new(vec![1, 2, 3], 10, 1, 3);
    /// assert_eq!(resp.total_results, 10);
    /// assert_eq!(resp.start_index, 1);
    /// assert_eq!(resp.items_per_page, 3);
    /// assert_eq!(resp.resources, vec![1, 2, 3]);
    /// ```
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
    /// Constructs a SCIM error response with the standard SCIM Error schema.
    ///
    /// The returned value contains the SCIM Error schema URN in `schemas`, the HTTP
    /// status code as a string in `status`, the provided `detail` message, and the
    /// optional `scim_type`.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = ScimError::new(404, "User not found", None);
    /// assert_eq!(err.status, "404");
    /// assert!(err.schemas.contains(&"urn:ietf:params:scim:api:messages:2.0:Error".to_string()));
    /// ```
    pub fn new(status: u16, detail: impl Into<String>, scim_type: Option<String>) -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            status: status.to_string(),
            scim_type,
            detail: detail.into(),
        }
    }

    /// Create a SCIM error representing HTTP 404 Not Found.
    ///
    /// The returned `ScimError` has `status` set to 404, no `scimType`, and `detail` set to the provided message.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = ScimError::not_found("user not found");
    /// assert_eq!(err.status, 404);
    /// assert!(err.scim_type.is_none());
    /// assert_eq!(err.detail, "user not found");
    /// ```
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, detail, None)
    }

    /// Creates a SCIM 2.0 Bad Request error (`status` 400) with SCIM type `invalidValue`.
    ///
    /// The provided `detail` is used as the human-readable error message.
    ///
    /// # Parameters
    ///
    /// - `detail`: Detail message to include in the error.
    ///
    /// # Returns
    ///
    /// A `ScimError` representing an HTTP 400 Bad Request with `scimType` set to `"invalidValue"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = ScimError::bad_request("invalid email format");
    /// assert_eq!(err.scim_type.as_deref(), Some("invalidValue"));
    /// assert_eq!(err.detail, "invalid email format");
    /// ```
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, detail, Some("invalidValue".to_string()))
    }

    /// Constructs a SCIM error representing a resource conflict (HTTP 409) with scimType "uniqueness".
    ///
    /// The returned error uses status 409 and sets `scimType` to `"uniqueness"`, with `detail` set to the provided message.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = ScimError::conflict("group already exists");
    /// assert_eq!(err.status, 409);
    /// assert_eq!(err.scim_type.as_deref(), Some("uniqueness"));
    /// assert!(err.detail.contains("group already exists"));
    /// ```
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

/// Default start index for SCIM list queries.
///
/// Returns the 1-based start index used when no start index is provided.
///
/// # Examples
///
/// ```
/// assert_eq!(default_start_index(), 1);
/// ```
fn default_start_index() -> i64 {
    1
}

/// Provides the default item count for SCIM list requests.
///
/// Returns `100` as the default count.
///
/// # Examples
///
/// ```
/// let c = default_count();
/// assert_eq!(c, 100);
/// ```
fn default_count() -> i64 {
    100
}

impl Default for ScimListParams {
    /// Creates a ScimListParams instance populated with standard default query values.
    ///
    /// # Examples
    ///
    /// ```
    /// let params = ScimListParams::default();
    /// assert_eq!(params.start_index, 1);
    /// assert_eq!(params.count, 100);
    /// assert!(params.filter.is_none());
    /// ```
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

/// Convert an internal `User` into its SCIM `ScimUser` representation.
///
/// The returned `ScimUser` contains the user's ID, external ID, username (falls back to ID if no email),
/// a single `work` email if the internal user has an email, an Active flag set to true, and SCIM `meta`
/// populated with resource type, timestamps, and a `location` built from `base_url`.
///
/// # Examples
///
/// ```
/// // Construct a `User` (fields shown for illustration; adapt to your `User` type).
/// let user = User {
///     id: "user-123".to_string(),
///     email: Some("alice@example.com".to_string()),
///     created_at: chrono::Utc::now(),
///     updated_at: chrono::Utc::now(),
///     ..Default::default()
/// };
/// let scim = user_to_scim(&user, "https://scim.example.com");
/// assert_eq!(scim.meta.as_ref().unwrap().resource_type, "User");
/// assert_eq!(scim.id.as_deref(), Some("user-123"));
/// assert_eq!(scim.emails.len(), 1);
/// assert_eq!(scim.emails[0].email_type.as_deref(), Some("work"));
/// ```
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

/// Convert an internal `Group` and its members into a SCIM 2.0 `ScimGroup`.
///
/// # Examples
///
/// ```
/// // Construct minimal test values (types shown here exist in the crate).
/// let group = Group {
///     id: "group-1".to_string(),
///     name: "Engineering".to_string(),
///     created_at: chrono::Utc::now(),
///     updated_at: chrono::Utc::now(),
/// };
///
/// let user = User {
///     id: "user-1".to_string(),
///     email: "alice@example.com".to_string(),
///     ..Default::default()
/// };
///
/// let member = GroupMemberWithUser { user };
///
/// let scim = group_to_scim(&group, &[member], "https://api.example.com");
///
/// assert_eq!(scim.display_name, "Engineering");
/// assert_eq!(scim.members.len(), 1);
/// assert_eq!(scim.members[0].value, "user-1");
/// assert_eq!(scim.meta.as_ref().unwrap().resource_type, "Group");
/// ```
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

/// Creates an internal user from a SCIM User payload and returns the created SCIM representation.
///
/// The function determines the new user's id using `external_id` if present, then `id`, and finally a generated UUID.
/// It selects the primary email (or first email) from the SCIM payload, or falls back to `userName` if it looks like an email.
/// If a user with the chosen id already exists this returns `CommonError::InvalidRequest`. On success the created
/// user is returned converted to a `ScimUser`.
///
/// # Returns
///
/// The created user converted to a `ScimUser`.
///
/// # Errors
///
/// Returns `CommonError::InvalidRequest` if a user with the resolved id already exists, `CommonError::Unknown` if the
/// created user cannot be retrieved after creation, or other repository errors propagated from the repository.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let repo = /* impl UserRepositoryLike */ unimplemented!();
/// let scim_user = ScimUser {
///     schemas: vec![String::from("urn:ietf:params:scim:schemas:core:2.0:User")],
///     id: None,
///     external_id: Some("alice-ext".into()),
///     user_name: "alice@example.com".into(),
///     name: None,
///     display_name: None,
///     emails: vec![ScimEmail { value: "alice@example.com".into(), r#type: Some("work".into()), primary: true }],
///     active: true,
///     groups: None,
///     meta: None,
/// };
/// let created = create_user_from_scim(&repo, scim_user).await;
/// # });
/// ```
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
                Some(scim_user.user_name.clone())
            } else {
                None
            }
        });

    // Check if user already exists
    if let Some(existing) = repo.get_user_by_id(&user_id).await? {
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

    repo.create_user(&user).await?;

    // Fetch the created user and return as SCIM
    let user = repo
        .get_user_by_id(&user_id)
        .await?
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Failed to retrieve created user")))?;

    Ok(user_to_scim(&user, ""))
}

/// Retrieve a user by ID and convert it to a SCIM User with group memberships populated.
///
/// The returned SCIM user includes metadata and a `groups` list built from the user's group memberships.
///
/// # Examples
///
/// ```
/// // Example usage (requires an async runtime and a repository implementing `UserRepositoryLike`).
/// // let repo = ...; // implementor of UserRepositoryLike
/// // let scim_user = tokio::runtime::Runtime::new().unwrap().block_on(async {
/// //     get_user_scim(&repo, "user-id", "https://example.com").await.unwrap()
/// // });
/// ```
/* No outer attributes */
pub async fn get_user_scim(
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

/// Lists users from the repository and returns a SCIM ListResponse constructed from the provided parameters.
///
/// Converts repository user records to `ScimUser` resources, then wraps them in a `ScimListResponse` whose
/// `total_results`, `start_index`, and `items_per_page` are derived from the given `params`.
///
/// # Returns
///
/// A `ScimListResponse<ScimUser>` containing the converted users and pagination metadata.
///
/// # Examples
///
/// ```
/// # async fn example(repo: &impl UserRepositoryLike, base_url: &str) -> Result<(), CommonError> {
/// let params = ScimListParams::default();
/// let list = list_users_scim(repo, params, base_url).await?;
/// assert!(list.total_results >= 0);
/// # Ok(()) }
/// ```
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

    let total = scim_users.len() as i64;

    Ok(ScimListResponse::new(
        scim_users,
        total,
        params.start_index,
        params.count,
    ))
}

/// Replaces a user's attributes using the provided SCIM User representation.
///
/// This updates the stored user's attributes according to the SCIM payload (for example, the primary
/// email or the first email value; `userName` is used as an email fallback when it contains '@'),
/// and returns the updated resource as a SCIM User. Returns `CommonError::NotFound` if the target
/// user does not exist; other repository errors are propagated.
///
/// # Returns
///
/// `ScimUser` containing the user's current state after the replacement.
///
/// # Examples
///
/// ```
/// // async context (e.g. inside a #[tokio::test] async fn)
/// // let repo = ...; // an implementation of UserRepositoryLike
/// // let scim_user = ScimUser { user_name: "alice@example.com".into(), emails: vec![ScimEmail { value: "alice@example.com".into(), ..Default::default() }], ..Default::default() };
/// // let updated = replace_user_scim(&repo, "user-id", scim_user, "https://example.com").await.unwrap();
/// // assert_eq!(updated.user_name, "alice@example.com");
/// ```
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

    repo.update_user(user_id, &update_user).await?;

    // Return updated user
    get_user_scim(repo, user_id, base_url).await
}

/// Apply a SCIM PATCH request to update a user's mutable SCIM attributes.
///
/// This applies the provided `ScimPatchRequest` to the user identified by `user_id`,
/// updating attributes that SCIM exposes for modification (currently email). The function
/// interprets `add`/`replace` operations targeting `emails` (or an emails value payload)
/// to set the user's primary email, and handles `remove` on `emails` to clear it.
/// SCIM-driven role changes are ignored.
///
/// # Returns
///
/// Updated `ScimUser` representation of the user after the patch.
///
/// # Examples
///
/// ```
/// # async fn example(repo: &impl UserRepositoryLike, base_url: &str) -> Result<(), CommonError> {
/// use serde_json::json;
///
/// let patch = ScimPatchRequest {
///     schemas: vec!["urn:ietf:params:scim:api:messages:2.0:PatchOp".into()],
///     operations: vec![ScimPatchOperation {
///         op: ScimPatchOp::Replace,
///         path: Some("emails".into()),
///         value: Some(json!([ { "value": "new@example.com", "type": "work", "primary": true } ])),
///     }],
/// };
///
/// let updated = patch_user_scim(repo, "user-id-123", patch, base_url).await?;
/// assert_eq!(updated.emails.first().and_then(|e| Some(e.value.clone())), Some("new@example.com".into()));
/// # Ok(()) }
/// ```
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
    repo.update_user(user_id, &update_user).await?;

    get_user_scim(repo, user_id, base_url).await
}

/// Delete a user and its related resources (group memberships and API keys).
///
/// # Examples
///
/// ```
/// // Example assumes an async runtime and a `repo` implementing `UserRepositoryLike`.
/// // Run inside an async test or runtime (e.g. `#[tokio::test]`).
/// # async fn example(repo: &impl UserRepositoryLike) {
/// delete_user_scim(repo, "user-id").await.unwrap();
/// # }
/// ```
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
    repo.delete_group_memberships_by_user_id(user_id).await?;
    // Delete user's API keys
    repo.delete_api_keys_by_user_id(user_id).await?;
    // Delete the user
    repo.delete_user(user_id).await?;

    Ok(())
}

// ============================================================================
// SCIM Group Logic Functions
// ============================================================================

/// Create a new group from a SCIM Group payload and persist it in the repository.
///
/// Uses the group's `external_id` or `id` from the SCIM payload as the group identifier; if neither
/// is present a new UUID is generated. If the payload includes members, each member is added to
/// the group only if the referenced user exists; membership creation errors are ignored for
/// individual members.
///
/// # Returns
///
/// The created group represented as a `ScimGroup`.
///
/// # Examples
///
/// ```rust
/// # use std::sync::Arc;
/// # async fn example() {
/// // `repo` must implement `UserRepositoryLike` (test double or repository instance).
/// let repo = /* test repo */ unimplemented!();
/// let scim_group = ScimGroup {
///     display_name: Some("Engineering".to_string()),
///     members: vec![],
///     ..Default::default()
/// };
/// let created = create_group_from_scim(&repo, scim_group, "https://example.com")
///     .await
///     .unwrap();
/// assert_eq!(created.display_name.as_deref(), Some("Engineering"));
/// # }
/// ```
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

    // Check if group already exists
    if let Some(existing) = repo.get_group_by_id(&group_id).await? {
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

    repo.create_group(&group).await?;

    // Add members if provided
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
            let _ = repo.create_group_membership(&membership).await;
        }
    }

    // Return the created group
    get_group_scim(repo, &group_id, base_url).await
}

/// Retrieve a group by its ID and return it as a SCIM `ScimGroup`.
///
/// Returns a NotFound `CommonError` if the group does not exist. The returned
/// SCIM group includes populated member entries and metadata derived from the
/// stored group record and the provided `base_url`.
///
/// # Examples
///
/// ```no_run
/// # async fn doc_example(repo: &impl UserRepositoryLike) -> Result<(), CommonError> {
/// let scim_group = get_group_scim(repo, "group-123", "https://example.com").await?;
/// assert_eq!(scim_group.id.as_deref(), Some("group-123"));
/// # Ok(())
/// # }
/// ```
pub async fn get_group_scim(
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

/// Return a paginated SCIM-formatted list of groups including their members.
///
/// Converts repository groups to SCIM Group resources by listing groups with the requested
/// pagination, loading each group's members, and assembling a ScimListResponse populated
/// with the provided start index and count values.
///
/// # Returns
///
/// A ScimListResponse containing SCIM Group resources and pagination metadata derived from `params`.
///
/// # Examples
///
/// ```no_run
/// # use crate::logic::scim::{list_groups_scim, ScimListParams};
/// # async fn example(repo: &impl crate::repo::UserRepositoryLike) -> Result<(), Box<dyn std::error::Error>> {
/// let params = ScimListParams::default();
/// let base_url = "https://example.com/scim";
/// let response = list_groups_scim(repo, params, base_url).await?;
/// assert!(response.total_results >= 0);
/// # Ok(())
/// # }
/// ```
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

    let total = scim_groups.len() as i64;

    Ok(ScimListResponse::new(
        scim_groups,
        total,
        params.start_index,
        params.count,
    ))
}

/// Replace an existing group with the provided SCIM group representation.
///
/// Replaces the group's display name and resets its membership list to match `scim_group.members`.
/// Memberships referencing users that do not exist are ignored.
///
/// # Examples
///
/// ```no_run
/// # use your_crate::logic::scim::{replace_group_scim, ScimGroup};
/// # use your_crate::repo::InMemoryRepo;
/// # #[tokio::main]
/// # async fn main() {
/// let repo = InMemoryRepo::new();
/// let scim_group = ScimGroup {
///     id: Some("group-id".into()),
///     external_id: None,
///     display_name: "New Name".into(),
///     members: vec![],
///     meta: None,
///     schemas: vec![],
/// };
/// let updated = replace_group_scim(&repo, "group-id", scim_group, "https://example.com")
///     .await
///     .unwrap();
/// assert_eq!(updated.display_name, "New Name");
/// # }
/// ```
///
/// @returns `ScimGroup` representing the updated group on success.
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
    repo.update_group(group_id, &scim_group.display_name)
        .await?;

    // Replace members: delete all existing and add new ones
    repo.delete_group_memberships_by_group_id(group_id).await?;

    for member in &scim_group.members {
        if repo.get_user_by_id(&member.value).await?.is_some() {
            let membership = GroupMembership {
                group_id: group_id.to_string(),
                user_id: member.value.clone(),
                created_at: now,
                updated_at: now,
            };
            let _ = repo.create_group_membership(&membership).await;
        }
    }

    get_group_scim(repo, group_id, base_url).await
}

/// Applies a SCIM PATCH request to an existing group, updating the group's display name and memberships.
///
/// This function processes each operation in the provided `ScimPatchRequest`:
/// - For `displayName` (via `path` or in an operation `value`), updates the group's display name.
/// - For `members` (via `path` or in an operation `value`), adds provided users as group members if the user exists and is not already a member.
/// - For `Remove` operations on `members[value eq "user_id"]`, removes the specified member; for `Remove` with `path == "members"`, removes all members.
/// User additions ignore members that reference non-existent users and do not error on duplicate memberships.
///
/// Returns the group's SCIM representation after applying the patch or an error if the group does not exist or a repository operation fails.
///
/// # Errors
///
/// Returns `CommonError::NotFound` if the target group cannot be found; other repository errors may be returned as `CommonError`.
///
/// # Examples
///
/// ```
/// # use crate::logic::scim::{patch_group_scim, ScimPatchRequest, ScimPatchOperation, ScimPatchOp};
/// # use crate::repository::InMemoryRepo; // hypothetical test repo
/// # async fn example(repo: &InMemoryRepo, base_url: &str) -> Result<(), crate::errors::CommonError> {
/// let op = ScimPatchOperation {
///     op: ScimPatchOp::Add,
///     path: Some("members".to_string()),
///     value: serde_json::json!([ { "value": "user-123" } ]),
/// };
/// let req = ScimPatchRequest { schemas: vec![], operations: vec![op] };
/// let updated = patch_group_scim(repo, "group-1", req, base_url).await?;
/// assert_eq!(updated.id, "group-1");
/// # Ok(())
/// # }
/// ```
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
        repo.update_group(group_id, &display_name).await?;
    }

    get_group_scim(repo, group_id, base_url).await
}

/// Delete a group and all of its memberships from the repository.
///
/// This verifies the group exists, removes any memberships for the group, and then deletes the group record.
/// Returns a `CommonError::NotFound` when the group does not exist; other repository errors are propagated.
///
/// # Examples
///
/// ```no_run
/// # use crates::identity::logic::scim::delete_group_scim;
/// # async fn example(repo: &impl crates::identity::UserRepositoryLike) -> Result<(), crates::identity::CommonError> {
/// delete_group_scim(repo, "group-id").await?;
/// # Ok(())
/// # }
/// ```
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
    repo.delete_group_memberships_by_group_id(group_id).await?;
    // Delete the group
    repo.delete_group(group_id).await?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use shared::primitives::SqlMigrationLoader;

    /// Sets up an in-memory test Repository initialized with SQL migrations.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run() {
    /// let repo = setup_test_repo().await;
    /// // use `repo` to perform repository operations in tests
    /// # }
    /// ```
    async fn setup_test_repo() -> Repository {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();

        Repository::new(conn)
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

        let result = create_user_from_scim(&repo, scim_user).await;
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
        let result1 = create_user_from_scim(&repo, scim_user.clone()).await;
        assert!(result1.is_ok());

        // Try to create duplicate
        let result2 = create_user_from_scim(&repo, scim_user).await;
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

        create_user_from_scim(&repo, scim_user).await.unwrap();

        // Get the user
        let result = get_user_scim(&repo, "get-test-user", "https://example.com/scim/v2").await;
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

        let result = get_user_scim(&repo, "nonexistent", "https://example.com/scim/v2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_users_scim() {
        let repo = setup_test_repo().await;

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
            create_user_from_scim(&repo, scim_user).await.unwrap();
        }

        let result = list_users_scim(
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

        create_user_from_scim(&repo, scim_user).await.unwrap();

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

        create_user_from_scim(&repo, scim_user).await.unwrap();

        // Patch the user
        let patch_request = ScimPatchRequest {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:PatchOp".to_string()],
            operations: vec![ScimPatchOperation {
                op: ScimPatchOp::Replace,
                path: Some("emails".to_string()),
                value: Some(serde_json::json!([{"value": "patched@example.com", "primary": true}])),
            }],
        };

        let result = patch_user_scim(
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

        create_user_from_scim(&repo, scim_user).await.unwrap();

        // Delete the user
        let result = delete_user_scim(&repo, "delete-user").await;
        assert!(result.is_ok());

        // Verify deletion
        let get_result = get_user_scim(&repo, "delete-user", "").await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_create_group_from_scim() {
        let repo = setup_test_repo().await;

        let scim_group = ScimGroup {
            schemas: default_group_schemas(),
            id: None,
            external_id: Some("group-123".to_string()),
            display_name: "Engineering".to_string(),
            members: vec![],
            meta: None,
        };

        let result = create_group_from_scim(&repo, scim_group, "https://example.com/scim/v2").await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.id, Some("group-123".to_string()));
        assert_eq!(created.display_name, "Engineering");
    }

    #[tokio::test]
    async fn test_create_group_with_members() {
        let repo = setup_test_repo().await;

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
        create_user_from_scim(&repo, scim_user).await.unwrap();

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

        let result = create_group_from_scim(&repo, scim_group, "https://example.com/scim/v2").await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.members.len(), 1);
        assert_eq!(created.members[0].value, "member-user");
    }

    #[tokio::test]
    async fn test_get_group_scim() {
        let repo = setup_test_repo().await;

        // Create a group
        let scim_group = ScimGroup {
            schemas: default_group_schemas(),
            id: None,
            external_id: Some("get-group".to_string()),
            display_name: "Test Group".to_string(),
            members: vec![],
            meta: None,
        };
        create_group_from_scim(&repo, scim_group, "").await.unwrap();

        // Get the group
        let result = get_group_scim(&repo, "get-group", "https://example.com/scim/v2").await;
        assert!(result.is_ok());

        let group = result.unwrap();
        assert_eq!(group.id, Some("get-group".to_string()));
        assert_eq!(group.display_name, "Test Group");
    }

    #[tokio::test]
    async fn test_list_groups_scim() {
        let repo = setup_test_repo().await;

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
            create_group_from_scim(&repo, scim_group, "").await.unwrap();
        }

        let result = list_groups_scim(
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

        // Create a group
        let scim_group = ScimGroup {
            schemas: default_group_schemas(),
            id: None,
            external_id: Some("replace-group".to_string()),
            display_name: "Original Name".to_string(),
            members: vec![],
            meta: None,
        };
        create_group_from_scim(&repo, scim_group, "").await.unwrap();

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
        create_user_from_scim(&repo, scim_user).await.unwrap();

        // Create a group
        let scim_group = ScimGroup {
            schemas: default_group_schemas(),
            id: None,
            external_id: Some("patch-group".to_string()),
            display_name: "Patch Group".to_string(),
            members: vec![],
            meta: None,
        };
        create_group_from_scim(&repo, scim_group, "").await.unwrap();

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
        create_user_from_scim(&repo, scim_user).await.unwrap();

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
        create_group_from_scim(&repo, scim_group, "").await.unwrap();

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

        // Create a group
        let scim_group = ScimGroup {
            schemas: default_group_schemas(),
            id: None,
            external_id: Some("delete-group".to_string()),
            display_name: "Delete Group".to_string(),
            members: vec![],
            meta: None,
        };
        create_group_from_scim(&repo, scim_group, "").await.unwrap();

        // Delete the group
        let result = delete_group_scim(&repo, "delete-group").await;
        assert!(result.is_ok());

        // Verify deletion
        let get_result = get_group_scim(&repo, "delete-group", "").await;
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