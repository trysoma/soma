mod sqlite;

use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime},
};
use utoipa::ToSchema;

pub use sqlite::Repository;

// Re-export types from logic modules that are used in repository trait
pub use crate::logic::api_key::{HashedApiKey, HashedApiKeyWithUser};
pub use crate::logic::user::{
    Group, GroupMemberWithUser, GroupMembership, Role, User, UserGroupWithGroup, UserType,
};
use crate::logic::user_auth_flow::config::EncryptedUserAuthFlowConfig;
use crate::logic::{
    internal_token_issuance::JwtSigningKey,
    sts::config::{StsTokenConfig, StsTokenConfigType},
    user_auth_flow::oauth::OAuthState,
};

#[derive(Debug, Default)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub role: Option<Role>,
    pub description: Option<String>,
}

// STS configuration types (raw database format)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct StsConfigurationDb {
    pub config: StsTokenConfig,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// IdP configuration types (for OAuth/OIDC authorization flows)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserAuthFlowConfigDb {
    pub id: String,
    pub config: EncryptedUserAuthFlowConfig,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

// Repository trait for users and API keys
#[allow(async_fn_in_trait)]
pub trait UserRepositoryLike {
    // User methods
    async fn create_user(&self, params: &User) -> Result<(), CommonError>;

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, CommonError>;

    async fn update_user(&self, id: &str, params: &UpdateUser) -> Result<(), CommonError>;

    async fn delete_user(&self, id: &str) -> Result<(), CommonError>;

    async fn list_users(
        &self,
        pagination: &PaginationRequest,
        user_type: Option<&UserType>,
        role: Option<&Role>,
    ) -> Result<PaginatedResponse<User>, CommonError>;

    // API key methods
    async fn create_api_key(&self, params: &HashedApiKey) -> Result<(), CommonError>;

    async fn get_api_key_by_hashed_value(
        &self,
        hashed_value: &str,
    ) -> Result<Option<HashedApiKeyWithUser>, CommonError>;

    async fn get_api_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<HashedApiKeyWithUser>, CommonError>;

    async fn delete_api_key(&self, id: &str) -> Result<(), CommonError>;

    async fn list_api_keys(
        &self,
        pagination: &PaginationRequest,
        user_id: Option<&str>,
    ) -> Result<PaginatedResponse<HashedApiKey>, CommonError>;

    async fn delete_api_keys_by_user_id(&self, user_id: &str) -> Result<(), CommonError>;

    // Group methods
    async fn create_group(&self, params: &Group) -> Result<(), CommonError>;

    async fn get_group_by_id(&self, id: &str) -> Result<Option<Group>, CommonError>;

    async fn update_group(&self, id: &str, name: &str) -> Result<(), CommonError>;

    async fn delete_group(&self, id: &str) -> Result<(), CommonError>;

    async fn list_groups(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Group>, CommonError>;

    // Group membership methods
    async fn create_group_membership(&self, params: &GroupMembership) -> Result<(), CommonError>;

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
    async fn create_jwt_signing_key(&self, params: &JwtSigningKey) -> Result<(), CommonError>;

    async fn get_jwt_signing_key_by_kid(
        &self,
        kid: &str,
    ) -> Result<Option<JwtSigningKey>, CommonError>;

    async fn invalidate_jwt_signing_key(&self, kid: &str) -> Result<(), CommonError>;

    async fn list_jwt_signing_keys(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<JwtSigningKey>, CommonError>;

    // STS configuration methods
    async fn create_sts_configuration(
        &self,
        params: &StsConfigurationDb,
    ) -> Result<(), CommonError>;

    async fn get_sts_configuration_by_id(
        &self,
        id: &str,
    ) -> Result<Option<StsConfigurationDb>, CommonError>;

    async fn delete_sts_configuration(&self, id: &str) -> Result<(), CommonError>;

    async fn list_sts_configurations(
        &self,
        pagination: &PaginationRequest,
        config_type: Option<StsTokenConfigType>,
    ) -> Result<PaginatedResponse<StsConfigurationDb>, CommonError>;

    // IdP configuration methods
    async fn create_user_auth_flow_config(
        &self,
        params: &UserAuthFlowConfigDb,
    ) -> Result<(), CommonError>;

    async fn get_user_auth_flow_config_by_id(
        &self,
        id: &str,
    ) -> Result<Option<UserAuthFlowConfigDb>, CommonError>;

    async fn delete_user_auth_flow_config(&self, id: &str) -> Result<(), CommonError>;

    async fn list_user_auth_flow_configs(
        &self,
        pagination: &PaginationRequest,
        config_type: Option<&str>,
    ) -> Result<PaginatedResponse<UserAuthFlowConfigDb>, CommonError>;

    // OAuth state methods
    async fn create_oauth_state(&self, params: &OAuthState) -> Result<(), CommonError>;

    async fn get_oauth_state_by_state(
        &self,
        state: &str,
    ) -> Result<Option<OAuthState>, CommonError>;

    async fn delete_oauth_state(&self, state: &str) -> Result<(), CommonError>;

    async fn delete_expired_oauth_states(&self) -> Result<u64, CommonError>;
}
