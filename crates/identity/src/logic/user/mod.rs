use serde::{Deserialize, Serialize};
use shared::primitives::WrappedChronoDateTime;
use utoipa::ToSchema;

/// User role in the system
#[derive(Debug, Clone, PartialEq, Eq, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Maintainer,
    ReadOnlyMaintainer,
    Agent,
    User,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Maintainer => "maintainer",
            Role::ReadOnlyMaintainer => "read-only-maintainer",
            Role::Agent => "agent",
            Role::User => "user",
        }
    }

    /// Parse a role from string
    pub fn parse(s: &str) -> Option<Role> {
        match s {
            "admin" => Some(Role::Admin),
            "maintainer" => Some(Role::Maintainer),
            "read-only-maintainer" => Some(Role::ReadOnlyMaintainer),
            "agent" => Some(Role::Agent),
            "user" => Some(Role::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
    Machine,
    Human,
}

impl UserType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserType::Machine => "machine",
            UserType::Human => "human",
        }
    }

    pub fn parse(s: &str) -> Option<UserType> {
        match s {
            "machine" => Some(UserType::Machine),
            "human" => Some(UserType::Human),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct User {
    pub id: String,
    pub user_type: UserType,
    pub email: Option<String>,
    pub role: Role,
    pub description: Option<String>,
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
