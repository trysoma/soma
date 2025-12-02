use crate::repository::{
    ApiKey, ApiKeyWithUser, Group, GroupMembership, GroupMemberWithUser, User, UserGroupWithGroup,
};

// Import generated Row types from parent module
use super::{
    Row_get_api_key_by_hashed_value, Row_get_api_keys, Row_get_group_by_id,
    Row_get_group_members, Row_get_group_membership, Row_get_groups, Row_get_user_by_id,
    Row_get_user_groups, Row_get_users,
};

// Conversions from generated Row types to domain types

impl From<Row_get_user_by_id> for User {
    fn from(row: Row_get_user_by_id) -> Self {
        User {
            id: row.id,
            user_type: row.user_type,
            email: row.email,
            role: row.role,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_users> for User {
    fn from(row: Row_get_users) -> Self {
        User {
            id: row.id,
            user_type: row.user_type,
            email: row.email,
            role: row.role,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_api_keys> for ApiKey {
    fn from(row: Row_get_api_keys) -> Self {
        ApiKey {
            hashed_value: row.hashed_value,
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_api_key_by_hashed_value> for ApiKeyWithUser {
    fn from(row: Row_get_api_key_by_hashed_value) -> Self {
        ApiKeyWithUser {
            api_key: ApiKey {
                hashed_value: row.hashed_value,
                user_id: row.user_id.clone(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            user: User {
                id: row.user_id_fk,
                user_type: row.user_type,
                email: row.user_email,
                role: row.user_role,
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
            },
        }
    }
}

// Group conversions

impl From<Row_get_group_by_id> for Group {
    fn from(row: Row_get_group_by_id) -> Self {
        Group {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_groups> for Group {
    fn from(row: Row_get_groups) -> Self {
        Group {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_group_membership> for GroupMembership {
    fn from(row: Row_get_group_membership) -> Self {
        GroupMembership {
            group_id: row.group_id,
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Row_get_group_members> for GroupMemberWithUser {
    fn from(row: Row_get_group_members) -> Self {
        GroupMemberWithUser {
            membership: GroupMembership {
                group_id: row.group_id,
                user_id: row.user_id.clone(),
                created_at: row.membership_created_at,
                updated_at: row.membership_updated_at,
            },
            user: User {
                id: row.user_id_fk,
                user_type: row.user_type,
                email: row.user_email,
                role: row.user_role,
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
            },
        }
    }
}

impl From<Row_get_user_groups> for UserGroupWithGroup {
    fn from(row: Row_get_user_groups) -> Self {
        UserGroupWithGroup {
            membership: GroupMembership {
                group_id: row.group_id.clone(),
                user_id: row.user_id,
                created_at: row.membership_created_at,
                updated_at: row.membership_updated_at,
            },
            group: Group {
                id: row.group_id_fk,
                name: row.group_name,
                created_at: row.group_created_at,
                updated_at: row.group_updated_at,
            },
        }
    }
}
