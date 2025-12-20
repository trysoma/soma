pub use shared::identity::{Group, User};
use shared::primitives::WrappedChronoDateTime;
use utoipa::ToSchema;

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
