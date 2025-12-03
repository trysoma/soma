mod sqlite;

use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime},
};
use utoipa::ToSchema;

pub use sqlite::Repository;

// User types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct User {
    pub id: String,
    pub user_type: String,
    pub email: Option<String>,
    pub role: String,
    pub description: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct CreateUser {
    pub id: String,
    pub user_type: String,
    pub email: Option<String>,
    pub role: String,
    pub description: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Default)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
}

// API key types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ApiKey {
    pub id: String,
    pub hashed_value: String,
    pub description: Option<String>,
    pub user_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ApiKeyWithUser {
    pub api_key: ApiKey,
    pub user: User,
}

#[derive(Debug)]
pub struct CreateApiKey {
    pub id: String,
    pub hashed_value: String,
    pub description: Option<String>,
    pub user_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Group types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct CreateGroup {
    pub id: String,
    pub name: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Group membership types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct GroupMembership {
    pub group_id: String,
    pub user_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct GroupMemberWithUser {
    pub membership: GroupMembership,
    pub user: User,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserGroupWithGroup {
    pub membership: GroupMembership,
    pub group: Group,
}

#[derive(Debug)]
pub struct CreateGroupMembership {
    pub group_id: String,
    pub user_id: String,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// JWT signing key types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct JwtSigningKey {
    pub kid: String,
    pub encrypted_private_key: String,
    pub expires_at: WrappedChronoDateTime,
    pub public_key: String,
    pub dek_alias: String,
    pub invalidated: bool,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct CreateJwtSigningKey {
    pub kid: String,
    pub encrypted_private_key: String,
    pub expires_at: WrappedChronoDateTime,
    pub public_key: String,
    pub dek_alias: String,
    pub invalidated: bool,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug, Default)]
pub struct UpdateJwtSigningKey {
    pub encrypted_private_key: Option<String>,
    pub expires_at: Option<WrappedChronoDateTime>,
    pub public_key: Option<String>,
    pub dek_alias: Option<String>,
}

// Repository trait for users and API keys
#[allow(async_fn_in_trait)]
pub trait UserRepositoryLike {
    // User methods
    async fn create_user(&self, params: &CreateUser) -> Result<(), CommonError>;

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, CommonError>;

    async fn update_user(&self, id: &str, params: &UpdateUser) -> Result<(), CommonError>;

    async fn delete_user(&self, id: &str) -> Result<(), CommonError>;

    async fn list_users(
        &self,
        pagination: &PaginationRequest,
        user_type: Option<&str>,
        role: Option<&str>,
    ) -> Result<PaginatedResponse<User>, CommonError>;

    // API key methods
    async fn create_api_key(&self, params: &CreateApiKey) -> Result<(), CommonError>;

    async fn get_api_key_by_hashed_value(
        &self,
        hashed_value: &str,
    ) -> Result<Option<ApiKeyWithUser>, CommonError>;

    async fn get_api_key_by_id(&self, id: &str) -> Result<Option<ApiKeyWithUser>, CommonError>;

    async fn delete_api_key(&self, id: &str) -> Result<(), CommonError>;

    async fn list_api_keys(
        &self,
        pagination: &PaginationRequest,
        user_id: Option<&str>,
    ) -> Result<PaginatedResponse<ApiKey>, CommonError>;

    async fn delete_api_keys_by_user_id(&self, user_id: &str) -> Result<(), CommonError>;

    // Group methods
    async fn create_group(&self, params: &CreateGroup) -> Result<(), CommonError>;

    async fn get_group_by_id(&self, id: &str) -> Result<Option<Group>, CommonError>;

    async fn update_group(&self, id: &str, name: &str) -> Result<(), CommonError>;

    async fn delete_group(&self, id: &str) -> Result<(), CommonError>;

    async fn list_groups(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Group>, CommonError>;

    // Group membership methods
    async fn create_group_membership(
        &self,
        params: &CreateGroupMembership,
    ) -> Result<(), CommonError>;

    async fn get_group_membership(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<Option<GroupMembership>, CommonError>;

    async fn delete_group_membership(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<(), CommonError>;

    async fn list_group_members(
        &self,
        group_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<GroupMemberWithUser>, CommonError>;

    async fn list_user_groups(
        &self,
        user_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UserGroupWithGroup>, CommonError>;

    async fn delete_group_memberships_by_group_id(&self, group_id: &str)
    -> Result<(), CommonError>;

    async fn delete_group_memberships_by_user_id(&self, user_id: &str) -> Result<(), CommonError>;

    // JWT signing key methods
    async fn create_jwt_signing_key(&self, params: &CreateJwtSigningKey)
    -> Result<(), CommonError>;

    async fn get_jwt_signing_key_by_kid(
        &self,
        kid: &str,
    ) -> Result<Option<JwtSigningKey>, CommonError>;

    async fn invalidate_jwt_signing_key(&self, kid: &str) -> Result<(), CommonError>;

    async fn list_jwt_signing_keys(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<JwtSigningKey>, CommonError>;
}
