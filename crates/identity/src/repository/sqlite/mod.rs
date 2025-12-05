#![allow(non_camel_case_types)]
#![allow(dead_code)]
mod raw_impl;

#[allow(clippy::all)]
pub mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::logic::sts::config::StsTokenConfigType;
use crate::logic::user::{Role, UserType};
use crate::logic::user_auth_flow::oauth::OAuthState;
use crate::repository::{
    Group, GroupMemberWithUser, GroupMembership, HashedApiKey, HashedApiKeyWithUser, JwtSigningKey,
    StsConfigurationDb, UpdateUser, User, UserAuthFlowConfigDb, UserGroupWithGroup,
    UserRepositoryLike,
};

use anyhow::Context;
use shared::error::CommonError;
use shared::primitives::{
    PaginatedResponse, PaginationRequest, WrappedChronoDateTime, decode_pagination_token,
};
use shared_macros::load_atlas_sql_migrations;

#[derive(Clone)]
pub struct Repository {
    conn: shared::libsql::Connection,
}

impl Repository {
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }

    /// Get the underlying connection
    pub fn connection(&self) -> &shared::libsql::Connection {
        &self.conn
    }
}

use shared::primitives::SqlMigrationLoader;
use std::collections::BTreeMap;

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("dbs/identity/migrations")
    }
}

impl UserRepositoryLike for Repository {
    async fn create_user(&self, params: &User) -> Result<(), CommonError> {
        let sqlc_params = create_user_params {
            id: &params.id,
            user_type: &params.user_type,
            email: &params.email,
            role: &params.role,
            description: &params.description.clone(),
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_user(&self.conn, sqlc_params)
            .await
            .context("Failed to create user")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, CommonError> {
        let sqlc_params = get_user_by_id_params {
            id: &id.to_string(),
        };

        let result = get_user_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get user by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn update_user(&self, id: &str, params: &UpdateUser) -> Result<(), CommonError> {
        // First get the existing user to preserve fields that aren't being updated
        let existing = self.get_user_by_id(id).await?;
        let existing = existing.ok_or_else(|| CommonError::Repository {
            msg: format!("User with id {id} not found"),
            source: None,
        })?;

        let email = params.email.clone().or_else(|| existing.email.clone());
        let role = params.role.clone().unwrap_or_else(|| existing.role.clone());
        let description = params
            .description
            .clone()
            .or_else(|| existing.description.clone());

        let sqlc_params = update_user_params {
            email: &email,
            role: &role,
            description: &description,
            id: &id.to_string(),
        };

        update_user(&self.conn, sqlc_params)
            .await
            .context("Failed to update user")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_user_params {
            id: &id.to_string(),
        };

        delete_user(&self.conn, sqlc_params)
            .await
            .context("Failed to delete user")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_users(
        &self,
        pagination: &PaginationRequest,
        user_type: Option<&UserType>,
        role: Option<&Role>,
    ) -> Result<PaginatedResponse<User>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };
        let user_type_owned = user_type.cloned();
        let role_owned = role.cloned();
        let sqlc_params = get_users_params {
            cursor: &cursor_datetime,
            user_type: &user_type_owned,
            role: &role_owned,
            page_size: &pagination.page_size,
        };

        let rows = get_users(&self.conn, sqlc_params)
            .await
            .context("Failed to list users")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<User> = rows.into_iter().map(|row| row.into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn create_api_key(&self, params: &HashedApiKey) -> Result<(), CommonError> {
        let sqlc_params = create_api_key_params {
            id: &params.id,
            hashed_value: &params.hashed_value,
            description: &params.description,
            user_id: &params.user_id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_api_key(&self.conn, sqlc_params)
            .await
            .context("Failed to create api key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_api_key_by_hashed_value(
        &self,
        hashed_value: &str,
    ) -> Result<Option<HashedApiKeyWithUser>, CommonError> {
        let sqlc_params = get_api_key_by_hashed_value_params {
            hashed_value: &hashed_value.to_string(),
        };

        let result = get_api_key_by_hashed_value(&self.conn, sqlc_params)
            .await
            .context("Failed to get api key by hashed value")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn get_api_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<HashedApiKeyWithUser>, CommonError> {
        let sqlc_params = get_api_key_by_id_params {
            id: &id.to_string(),
        };

        let result = get_api_key_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get api key by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn delete_api_key(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_api_key_params {
            id: &id.to_string(),
        };

        delete_api_key(&self.conn, sqlc_params)
            .await
            .context("Failed to delete api key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_api_keys(
        &self,
        pagination: &PaginationRequest,
        user_id: Option<&str>,
    ) -> Result<PaginatedResponse<HashedApiKey>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };
        let user_id_owned = user_id.map(|s| s.to_string());

        let sqlc_params = get_api_keys_params {
            cursor: &cursor_datetime,
            user_id: &user_id_owned,
            page_size: &pagination.page_size,
        };

        let rows = get_api_keys(&self.conn, sqlc_params)
            .await
            .context("Failed to list api keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<HashedApiKey> = rows.into_iter().map(|row| row.into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn delete_api_keys_by_user_id(&self, user_id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_api_keys_by_user_id_params {
            user_id: &user_id.to_string(),
        };

        delete_api_keys_by_user_id(&self.conn, sqlc_params)
            .await
            .context("Failed to delete api keys by user id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    // Group methods
    async fn create_group(&self, params: &Group) -> Result<(), CommonError> {
        let sqlc_params = create_group_params {
            id: &params.id,
            name: &params.name,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_group(&self.conn, sqlc_params)
            .await
            .context("Failed to create group")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_group_by_id(&self, id: &str) -> Result<Option<Group>, CommonError> {
        let sqlc_params = get_group_by_id_params {
            id: &id.to_string(),
        };

        let result = get_group_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get group by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn update_group(&self, id: &str, name: &str) -> Result<(), CommonError> {
        let sqlc_params = update_group_params {
            name: &name.to_string(),
            id: &id.to_string(),
        };

        update_group(&self.conn, sqlc_params)
            .await
            .context("Failed to update group")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_group(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_group_params {
            id: &id.to_string(),
        };

        delete_group(&self.conn, sqlc_params)
            .await
            .context("Failed to delete group")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_groups(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Group>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_groups_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_groups(&self.conn, sqlc_params)
            .await
            .context("Failed to list groups")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<Group> = rows.into_iter().map(|row| row.into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    // Group membership methods
    async fn create_group_membership(&self, params: &GroupMembership) -> Result<(), CommonError> {
        let sqlc_params = create_group_membership_params {
            group_id: &params.group_id,
            user_id: &params.user_id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_group_membership(&self.conn, sqlc_params)
            .await
            .context("Failed to create group membership")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_group_membership(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<Option<GroupMembership>, CommonError> {
        let sqlc_params = get_group_membership_params {
            group_id: &group_id.to_string(),
            user_id: &user_id.to_string(),
        };

        let result = get_group_membership(&self.conn, sqlc_params)
            .await
            .context("Failed to get group membership")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn delete_group_membership(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_group_membership_params {
            group_id: &group_id.to_string(),
            user_id: &user_id.to_string(),
        };

        delete_group_membership(&self.conn, sqlc_params)
            .await
            .context("Failed to delete group membership")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_group_members(
        &self,
        group_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<GroupMemberWithUser>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_group_members_params {
            group_id: &group_id.to_string(),
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_group_members(&self.conn, sqlc_params)
            .await
            .context("Failed to list group members")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<GroupMemberWithUser> = rows.into_iter().map(|row| row.into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.membership.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn list_user_groups(
        &self,
        user_id: &str,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UserGroupWithGroup>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_user_groups_params {
            user_id: &user_id.to_string(),
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_user_groups(&self.conn, sqlc_params)
            .await
            .context("Failed to list user groups")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<UserGroupWithGroup> = rows.into_iter().map(|row| row.into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.membership.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn delete_group_memberships_by_group_id(
        &self,
        group_id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_group_memberships_by_group_id_params {
            group_id: &group_id.to_string(),
        };

        delete_group_memberships_by_group_id(&self.conn, sqlc_params)
            .await
            .context("Failed to delete group memberships by group id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_group_memberships_by_user_id(&self, user_id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_group_memberships_by_user_id_params {
            user_id: &user_id.to_string(),
        };

        delete_group_memberships_by_user_id(&self.conn, sqlc_params)
            .await
            .context("Failed to delete group memberships by user id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    // JWT signing key methods
    async fn create_jwt_signing_key(&self, params: &JwtSigningKey) -> Result<(), CommonError> {
        let sqlc_params = create_jwt_signing_key_params {
            kid: &params.kid,
            encrypted_private_key: &params.encrypted_private_key,
            expires_at: &params.expires_at,
            public_key: &params.public_key,
            dek_alias: &params.dek_alias,
            invalidated: &params.invalidated,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_jwt_signing_key(&self.conn, sqlc_params)
            .await
            .context("Failed to create jwt signing key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_jwt_signing_key_by_kid(
        &self,
        kid: &str,
    ) -> Result<Option<JwtSigningKey>, CommonError> {
        let sqlc_params = get_jwt_signing_key_by_kid_params {
            kid: &kid.to_string(),
        };

        let result = get_jwt_signing_key_by_kid(&self.conn, sqlc_params)
            .await
            .context("Failed to get jwt signing key by kid")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn invalidate_jwt_signing_key(&self, kid: &str) -> Result<(), CommonError> {
        let sqlc_params = invalidate_jwt_signing_key_params {
            kid: &kid.to_string(),
        };

        invalidate_jwt_signing_key(&self.conn, sqlc_params)
            .await
            .context("Failed to invalidate jwt signing key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_jwt_signing_keys(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<JwtSigningKey>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_jwt_signing_keys_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_jwt_signing_keys(&self.conn, sqlc_params)
            .await
            .context("Failed to list jwt signing keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<JwtSigningKey> = rows.into_iter().map(|row| row.into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    // STS configuration methods
    async fn create_sts_configuration(
        &self,
        params: &StsConfigurationDb,
    ) -> Result<(), CommonError> {
        // Extract id and config_type from the StsTokenConfig
        let (id, config_type, value) = match &params.config {
            crate::logic::sts::config::StsTokenConfig::DevMode(config) => {
                (config.id.clone(), "dev".to_string(), None)
            }
            crate::logic::sts::config::StsTokenConfig::JwtTemplate(config) => {
                let json_value =
                    serde_json::to_value(config).map_err(|e| CommonError::Repository {
                        msg: format!("Failed to serialize jwt_template config: {e}"),
                        source: Some(e.into()),
                    })?;
                (
                    config.id.clone(),
                    "jwt_template".to_string(),
                    Some(shared::primitives::WrappedJsonValue::new(json_value)),
                )
            }
        };

        let sqlc_params = create_sts_configuration_params {
            id: &id,
            config_type: &config_type,
            value: &value,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_sts_configuration(&self.conn, sqlc_params)
            .await
            .context("Failed to create STS configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_sts_configuration_by_id(
        &self,
        id: &str,
    ) -> Result<Option<StsConfigurationDb>, CommonError> {
        let sqlc_params = get_sts_configuration_by_id_params {
            id: &id.to_string(),
        };

        let result = get_sts_configuration_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get STS configuration by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_sts_configuration(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_sts_configuration_params {
            id: &id.to_string(),
        };

        delete_sts_configuration(&self.conn, sqlc_params)
            .await
            .context("Failed to delete STS configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_sts_configurations(
        &self,
        pagination: &PaginationRequest,
        config_type: Option<StsTokenConfigType>,
    ) -> Result<PaginatedResponse<StsConfigurationDb>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };
        let config_type_owned = config_type.map(|ct| match ct {
            StsTokenConfigType::DevMode => "dev".to_string(),
            StsTokenConfigType::JwtTemplate => "jwt_template".to_string(),
        });

        let sqlc_params = get_sts_configurations_params {
            cursor: &cursor_datetime,
            config_type: &config_type_owned,
            page_size: &pagination.page_size,
        };

        let rows = get_sts_configurations(&self.conn, sqlc_params)
            .await
            .context("Failed to list STS configurations")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<StsConfigurationDb> = rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    // User auth flow configuration methods
    async fn create_user_auth_flow_config(
        &self,
        params: &UserAuthFlowConfigDb,
    ) -> Result<(), CommonError> {
        let (config_type, config_json) = params.config.to_db_values()?;

        tracing::info!(
            "Creating user auth flow(s) config with type: {}",
            config_type
        );

        let sqlc_params = create_user_auth_flow_config_params {
            id: &params.id,
            config_type: &config_type,
            config: &config_json,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_user_auth_flow_config(&self.conn, sqlc_params)
            .await
            .context("Failed to create user auth flow configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_user_auth_flow_config_by_id(
        &self,
        id: &str,
    ) -> Result<Option<UserAuthFlowConfigDb>, CommonError> {
        let sqlc_params = get_user_auth_flow_config_by_id_params {
            id: &id.to_string(),
        };

        let result = get_user_auth_flow_config_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get user auth flow configuration by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match result {
            Some(row) => Ok(Some(row.try_into()?)),
            None => Ok(None),
        }
    }

    async fn delete_user_auth_flow_config(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_user_auth_flow_config_params {
            id: &id.to_string(),
        };

        delete_user_auth_flow_config(&self.conn, sqlc_params)
            .await
            .context("Failed to delete user auth flow configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_user_auth_flow_configs(
        &self,
        pagination: &PaginationRequest,
        config_type: Option<&str>,
    ) -> Result<PaginatedResponse<UserAuthFlowConfigDb>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                )
            }
        } else {
            None
        };
        let config_type_owned = config_type.map(|s| s.to_string());

        let sqlc_params = get_user_auth_flow_configs_params {
            cursor: &cursor_datetime,
            config_type: &config_type_owned,
            page_size: &pagination.page_size,
        };

        let rows = get_user_auth_flow_configs(&self.conn, sqlc_params)
            .await
            .context("Failed to list user auth flow configurations")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<UserAuthFlowConfigDb>, CommonError> =
            rows.into_iter().map(|row| row.try_into()).collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items?,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    // OAuth state methods
    async fn create_oauth_state(&self, params: &OAuthState) -> Result<(), CommonError> {
        let sqlc_params = create_oauth_state_params {
            state: &params.state,
            config_id: &params.config_id,
            code_verifier: &params.code_verifier,
            nonce: &params.nonce,
            redirect_uri: &params.redirect_uri,
            created_at: &params.created_at,
            expires_at: &params.expires_at,
        };

        create_oauth_state(&self.conn, sqlc_params)
            .await
            .context("Failed to create OAuth state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_oauth_state_by_state(
        &self,
        state: &str,
    ) -> Result<Option<OAuthState>, CommonError> {
        let sqlc_params = get_oauth_state_by_state_params {
            state: &state.to_string(),
        };

        let result = get_oauth_state_by_state(&self.conn, sqlc_params)
            .await
            .context("Failed to get OAuth state by state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| row.into()))
    }

    async fn delete_oauth_state(&self, state: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_oauth_state_params {
            state: &state.to_string(),
        };

        delete_oauth_state(&self.conn, sqlc_params)
            .await
            .context("Failed to delete OAuth state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_expired_oauth_states(&self) -> Result<u64, CommonError> {
        let now = WrappedChronoDateTime::now();
        let sqlc_params = delete_expired_oauth_states_params { now: &now };

        delete_expired_oauth_states(&self.conn, sqlc_params)
            .await
            .context("Failed to delete expired OAuth states")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })
    }
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::api_key::HashedApiKey;
    use crate::logic::internal_token_issuance::JwtSigningKey;
    use crate::logic::user::{Group, GroupMembership, Role, User, UserType};
    use shared::primitives::WrappedChronoDateTime;

    async fn setup_test_db() -> Repository {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();

        Repository::new(conn)
    }

    fn create_test_user(id: &str, user_type: UserType, email: Option<&str>, role: Role) -> User {
        let now = WrappedChronoDateTime::now();
        User {
            id: id.to_string(),
            user_type,
            email: email.map(|s| s.to_string()),
            role,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_api_key(id: &str, hashed_value: &str, user_id: &str) -> HashedApiKey {
        let now = WrappedChronoDateTime::now();
        HashedApiKey {
            id: id.to_string(),
            hashed_value: hashed_value.to_string(),
            description: Some(format!("Test API key {id}")),
            user_id: user_id.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_group(id: &str, name: &str) -> Group {
        let now = WrappedChronoDateTime::now();
        Group {
            id: id.to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_group_membership(group_id: &str, user_id: &str) -> GroupMembership {
        let now = WrappedChronoDateTime::now();
        GroupMembership {
            group_id: group_id.to_string(),
            user_id: user_id.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_jwt_signing_key(
        kid: &str,
        encrypted_private_key: &str,
        public_key: &str,
        dek_alias: &str,
    ) -> JwtSigningKey {
        let now = WrappedChronoDateTime::now();
        let expires_at = *now.get_inner() + chrono::Duration::days(30);
        JwtSigningKey {
            kid: kid.to_string(),
            encrypted_private_key: encrypted_private_key.to_string(),
            expires_at: WrappedChronoDateTime::new(expires_at),
            public_key: public_key.to_string(),
            dek_alias: dek_alias.to_string(),
            invalidated: false,
            created_at: now,
            updated_at: now,
        }
    }

    // ============================================
    // User tests
    // ============================================

    #[tokio::test]
    async fn test_create_and_get_user() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        let fetched = repo.get_user_by_id("user-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, "user-1");
        assert_eq!(fetched.user_type, UserType::Machine);
        assert_eq!(fetched.email, Some("test@example.com".to_string()));
        assert_eq!(fetched.role, Role::Admin);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo.get_user_by_id("nonexistent").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_update_user() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("old@example.com"),
            Role::User,
        );
        repo.create_user(&user).await.unwrap();

        let update = UpdateUser {
            email: Some("new@example.com".to_string()),
            role: Some(Role::Admin),
            description: None,
        };
        repo.update_user("user-1", &update).await.unwrap();

        let fetched = repo.get_user_by_id("user-1").await.unwrap().unwrap();
        assert_eq!(fetched.email, Some("new@example.com".to_string()));
        assert_eq!(fetched.role, Role::Admin);
    }

    #[tokio::test]
    async fn test_update_user_partial() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("old@example.com"),
            Role::User,
        );
        repo.create_user(&user).await.unwrap();

        // Only update email
        let update = UpdateUser {
            email: Some("new@example.com".to_string()),
            role: None,
            description: None,
        };
        repo.update_user("user-1", &update).await.unwrap();

        let fetched = repo.get_user_by_id("user-1").await.unwrap().unwrap();
        assert_eq!(fetched.email, Some("new@example.com".to_string()));
        assert_eq!(fetched.role, Role::User); // Should be unchanged
    }

    #[tokio::test]
    async fn test_delete_user() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        repo.delete_user("user-1").await.unwrap();

        let fetched = repo.get_user_by_id("user-1").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_users() {
        let repo = setup_test_db().await;

        // Create multiple users with small delay to ensure different timestamps
        for i in 1..=5 {
            let user = create_test_user(
                &format!("user-{i}"),
                if i % 2 == 0 {
                    UserType::Human
                } else {
                    UserType::Machine
                },
                Some(&format!("user{i}@example.com")),
                if i % 2 == 0 { Role::Admin } else { Role::User },
            );
            repo.create_user(&user).await.unwrap();
        }

        // List all
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_users(&pagination, None, None).await.unwrap();
        assert_eq!(result.items.len(), 5);

        // Filter by user_type
        let result = repo
            .list_users(&pagination, Some(&UserType::Machine), None)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);

        // Filter by role
        let result = repo
            .list_users(&pagination, None, Some(&Role::Admin))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
    }

    #[tokio::test]
    async fn test_list_users_pagination() {
        let repo = setup_test_db().await;

        // Create 5 users
        for i in 1..=5 {
            let user = create_test_user(
                &format!("user-{i}"),
                UserType::Machine,
                Some(&format!("user{i}@example.com")),
                Role::User,
            );
            repo.create_user(&user).await.unwrap();
            // Small delay to ensure different created_at timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo.list_users(&pagination, None, None).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_users(&pagination, None, None).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get third page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_users(&pagination, None, None).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());
    }

    // ============================================
    // API Key tests
    // ============================================

    #[tokio::test]
    async fn test_create_and_get_api_key() {
        let repo = setup_test_db().await;

        // Create user first
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        // Create API key
        let api_key = create_test_api_key("api-key-1", "hashed-key-1", "user-1");
        repo.create_api_key(&api_key).await.unwrap();

        let fetched = repo
            .get_api_key_by_hashed_value("hashed-key-1")
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.api_key.id, "api-key-1");
        assert_eq!(fetched.api_key.hashed_value, "hashed-key-1");
        assert_eq!(fetched.api_key.user_id, "user-1");
        assert_eq!(fetched.user.id, "user-1");
        assert_eq!(fetched.user.email, Some("test@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_get_api_key_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo
            .get_api_key_by_hashed_value("nonexistent")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_api_key() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        let api_key = create_test_api_key("api-key-1", "hashed-key-1", "user-1");
        repo.create_api_key(&api_key).await.unwrap();

        repo.delete_api_key("api-key-1").await.unwrap();

        let fetched = repo
            .get_api_key_by_hashed_value("hashed-key-1")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_api_keys() {
        let repo = setup_test_db().await;

        // Create users
        let user1 = create_test_user(
            "user-1",
            UserType::Machine,
            Some("user1@example.com"),
            Role::Admin,
        );
        let user2 = create_test_user(
            "user-2",
            UserType::Human,
            Some("user2@example.com"),
            Role::User,
        );
        repo.create_user(&user1).await.unwrap();
        repo.create_user(&user2).await.unwrap();

        // Create API keys
        for i in 1..=3 {
            let api_key = create_test_api_key(
                &format!("api-key-user1-{i}"),
                &format!("hash-user1-{i}"),
                "user-1",
            );
            repo.create_api_key(&api_key).await.unwrap();
        }
        for i in 1..=2 {
            let api_key = create_test_api_key(
                &format!("api-key-user2-{i}"),
                &format!("hash-user2-{i}"),
                "user-2",
            );
            repo.create_api_key(&api_key).await.unwrap();
        }

        // List all
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_api_keys(&pagination, None).await.unwrap();
        assert_eq!(result.items.len(), 5);

        // Filter by user_id
        let result = repo
            .list_api_keys(&pagination, Some("user-1"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);

        let result = repo
            .list_api_keys(&pagination, Some("user-2"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_api_keys_by_user_id() {
        let repo = setup_test_db().await;

        // Create users
        let user1 = create_test_user(
            "user-1",
            UserType::Machine,
            Some("user1@example.com"),
            Role::Admin,
        );
        let user2 = create_test_user(
            "user-2",
            UserType::Human,
            Some("user2@example.com"),
            Role::User,
        );
        repo.create_user(&user1).await.unwrap();
        repo.create_user(&user2).await.unwrap();

        // Create API keys for both users
        for i in 1..=3 {
            let api_key = create_test_api_key(
                &format!("api-key-user1-{i}"),
                &format!("hash-user1-{i}"),
                "user-1",
            );
            repo.create_api_key(&api_key).await.unwrap();
        }
        for i in 1..=2 {
            let api_key = create_test_api_key(
                &format!("api-key-user2-{i}"),
                &format!("hash-user2-{i}"),
                "user-2",
            );
            repo.create_api_key(&api_key).await.unwrap();
        }

        // Delete user-1's keys
        repo.delete_api_keys_by_user_id("user-1").await.unwrap();

        // Verify user-1's keys are gone
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_api_keys(&pagination, Some("user-1"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 0);

        // Verify user-2's keys still exist
        let result = repo
            .list_api_keys(&pagination, Some("user-2"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
    }

    #[tokio::test]
    async fn test_cascade_delete_api_keys_on_user_delete() {
        let repo = setup_test_db().await;

        // Create user
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        // Create API keys
        for i in 1..=3 {
            let api_key =
                create_test_api_key(&format!("api-key-{i}"), &format!("hash-{i}"), "user-1");
            repo.create_api_key(&api_key).await.unwrap();
        }

        // Delete user (should cascade delete API keys due to foreign key)
        repo.delete_user("user-1").await.unwrap();

        // Verify API keys are gone
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_api_keys(&pagination, Some("user-1"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 0);
    }

    // ============================================
    // Group tests
    // ============================================

    #[tokio::test]
    async fn test_create_and_get_group() {
        let repo = setup_test_db().await;

        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        let fetched = repo.get_group_by_id("group-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, "group-1");
        assert_eq!(fetched.name, "Test Group");
    }

    #[tokio::test]
    async fn test_get_group_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo.get_group_by_id("nonexistent").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_update_group() {
        let repo = setup_test_db().await;

        let group = create_test_group("group-1", "Old Name");
        repo.create_group(&group).await.unwrap();

        repo.update_group("group-1", "New Name").await.unwrap();

        let fetched = repo.get_group_by_id("group-1").await.unwrap().unwrap();
        assert_eq!(fetched.name, "New Name");
    }

    #[tokio::test]
    async fn test_delete_group() {
        let repo = setup_test_db().await;

        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        repo.delete_group("group-1").await.unwrap();

        let fetched = repo.get_group_by_id("group-1").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_groups() {
        let repo = setup_test_db().await;

        // Create multiple groups
        for i in 1..=5 {
            let group = create_test_group(&format!("group-{i}"), &format!("Group {i}"));
            repo.create_group(&group).await.unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_groups(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 5);
    }

    #[tokio::test]
    async fn test_list_groups_pagination() {
        let repo = setup_test_db().await;

        // Create 5 groups with delays
        for i in 1..=5 {
            let group = create_test_group(&format!("group-{i}"), &format!("Group {i}"));
            repo.create_group(&group).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo.list_groups(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_groups(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get third page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_groups(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());
    }

    // ============================================
    // Group membership tests
    // ============================================

    #[tokio::test]
    async fn test_create_and_get_group_membership() {
        let repo = setup_test_db().await;

        // Create user and group first
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        // Create membership
        let membership = create_test_group_membership("group-1", "user-1");
        repo.create_group_membership(&membership).await.unwrap();

        let fetched = repo
            .get_group_membership("group-1", "user-1")
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.group_id, "group-1");
        assert_eq!(fetched.user_id, "user-1");
    }

    #[tokio::test]
    async fn test_get_group_membership_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo
            .get_group_membership("nonexistent", "nonexistent")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_group_membership() {
        let repo = setup_test_db().await;

        // Create user, group, and membership
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        let membership = create_test_group_membership("group-1", "user-1");
        repo.create_group_membership(&membership).await.unwrap();

        // Delete membership
        repo.delete_group_membership("group-1", "user-1")
            .await
            .unwrap();

        let fetched = repo
            .get_group_membership("group-1", "user-1")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_group_members() {
        let repo = setup_test_db().await;

        // Create group
        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        // Create users and add them to the group
        for i in 1..=5 {
            let user = create_test_user(
                &format!("user-{i}"),
                UserType::Machine,
                Some(&format!("user{i}@example.com")),
                Role::User,
            );
            repo.create_user(&user).await.unwrap();

            let membership = create_test_group_membership("group-1", &format!("user-{i}"));
            repo.create_group_membership(&membership).await.unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_group_members("group-1", &pagination)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 5);

        // Verify joined user data is present
        assert!(result.items.iter().all(|m| m.user.email.is_some()));
    }

    #[tokio::test]
    async fn test_list_group_members_pagination() {
        let repo = setup_test_db().await;

        // Create group
        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        // Create users and add them to the group with delays
        for i in 1..=5 {
            let user = create_test_user(
                &format!("user-{i}"),
                UserType::Machine,
                Some(&format!("user{i}@example.com")),
                Role::User,
            );
            repo.create_user(&user).await.unwrap();

            let membership = create_test_group_membership("group-1", &format!("user-{i}"));
            repo.create_group_membership(&membership).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo
            .list_group_members("group-1", &pagination)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo
            .list_group_members("group-1", &pagination)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());
    }

    #[tokio::test]
    async fn test_list_user_groups() {
        let repo = setup_test_db().await;

        // Create user
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        // Create groups and add user to them
        for i in 1..=5 {
            let group = create_test_group(&format!("group-{i}"), &format!("Group {i}"));
            repo.create_group(&group).await.unwrap();

            let membership = create_test_group_membership(&format!("group-{i}"), "user-1");
            repo.create_group_membership(&membership).await.unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_user_groups("user-1", &pagination).await.unwrap();
        assert_eq!(result.items.len(), 5);

        // Verify joined group data is present
        assert!(result.items.iter().all(|m| !m.group.name.is_empty()));
    }

    #[tokio::test]
    async fn test_list_user_groups_pagination() {
        let repo = setup_test_db().await;

        // Create user
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        // Create groups and add user to them with delays
        for i in 1..=5 {
            let group = create_test_group(&format!("group-{i}"), &format!("Group {i}"));
            repo.create_group(&group).await.unwrap();

            let membership = create_test_group_membership(&format!("group-{i}"), "user-1");
            repo.create_group_membership(&membership).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo.list_user_groups("user-1", &pagination).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_user_groups("user-1", &pagination).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());
    }

    #[tokio::test]
    async fn test_delete_group_memberships_by_group_id() {
        let repo = setup_test_db().await;

        // Create groups
        let group1 = create_test_group("group-1", "Group 1");
        let group2 = create_test_group("group-2", "Group 2");
        repo.create_group(&group1).await.unwrap();
        repo.create_group(&group2).await.unwrap();

        // Create users
        for i in 1..=3 {
            let user = create_test_user(
                &format!("user-{i}"),
                UserType::Machine,
                Some(&format!("user{i}@example.com")),
                Role::User,
            );
            repo.create_user(&user).await.unwrap();

            // Add to both groups
            let m1 = create_test_group_membership("group-1", &format!("user-{i}"));
            let m2 = create_test_group_membership("group-2", &format!("user-{i}"));
            repo.create_group_membership(&m1).await.unwrap();
            repo.create_group_membership(&m2).await.unwrap();
        }

        // Delete memberships for group-1
        repo.delete_group_memberships_by_group_id("group-1")
            .await
            .unwrap();

        // Verify group-1 has no members
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_group_members("group-1", &pagination)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 0);

        // Verify group-2 still has members
        let result = repo
            .list_group_members("group-2", &pagination)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_group_memberships_by_user_id() {
        let repo = setup_test_db().await;

        // Create groups
        for i in 1..=3 {
            let group = create_test_group(&format!("group-{i}"), &format!("Group {i}"));
            repo.create_group(&group).await.unwrap();
        }

        // Create users
        let user1 = create_test_user(
            "user-1",
            UserType::Machine,
            Some("user1@example.com"),
            Role::User,
        );
        let user2 = create_test_user(
            "user-2",
            UserType::Human,
            Some("user2@example.com"),
            Role::User,
        );
        repo.create_user(&user1).await.unwrap();
        repo.create_user(&user2).await.unwrap();

        // Add user-1 to all groups
        for i in 1..=3 {
            let m = create_test_group_membership(&format!("group-{i}"), "user-1");
            repo.create_group_membership(&m).await.unwrap();
        }

        // Add user-2 to all groups
        for i in 1..=3 {
            let m = create_test_group_membership(&format!("group-{i}"), "user-2");
            repo.create_group_membership(&m).await.unwrap();
        }

        // Delete memberships for user-1
        repo.delete_group_memberships_by_user_id("user-1")
            .await
            .unwrap();

        // Verify user-1 has no groups
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_user_groups("user-1", &pagination).await.unwrap();
        assert_eq!(result.items.len(), 0);

        // Verify user-2 still has groups
        let result = repo.list_user_groups("user-2", &pagination).await.unwrap();
        assert_eq!(result.items.len(), 3);
    }

    #[tokio::test]
    async fn test_cascade_delete_memberships_on_group_delete() {
        let repo = setup_test_db().await;

        // Create group
        let group = create_test_group("group-1", "Test Group");
        repo.create_group(&group).await.unwrap();

        // Create users and add to group
        for i in 1..=3 {
            let user = create_test_user(
                &format!("user-{i}"),
                UserType::Machine,
                Some(&format!("user{i}@example.com")),
                Role::User,
            );
            repo.create_user(&user).await.unwrap();

            let m = create_test_group_membership("group-1", &format!("user-{i}"));
            repo.create_group_membership(&m).await.unwrap();
        }

        // Delete group (should cascade delete memberships due to foreign key)
        repo.delete_group("group-1").await.unwrap();

        // Verify memberships are gone
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        // Check users are no longer in the group
        for i in 1..=3 {
            let result = repo
                .list_user_groups(&format!("user-{i}"), &pagination)
                .await
                .unwrap();
            assert_eq!(result.items.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_cascade_delete_memberships_on_user_delete() {
        let repo = setup_test_db().await;

        // Create groups
        for i in 1..=3 {
            let group = create_test_group(&format!("group-{i}"), &format!("Group {i}"));
            repo.create_group(&group).await.unwrap();
        }

        // Create user and add to all groups
        let user = create_test_user(
            "user-1",
            UserType::Machine,
            Some("test@example.com"),
            Role::Admin,
        );
        repo.create_user(&user).await.unwrap();

        for i in 1..=3 {
            let m = create_test_group_membership(&format!("group-{i}"), "user-1");
            repo.create_group_membership(&m).await.unwrap();
        }

        // Delete user (should cascade delete memberships due to foreign key)
        repo.delete_user("user-1").await.unwrap();

        // Verify memberships are gone - check groups have no members
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        for i in 1..=3 {
            let result = repo
                .list_group_members(&format!("group-{i}"), &pagination)
                .await
                .unwrap();
            assert_eq!(result.items.len(), 0);
        }
    }

    // ============================================
    // JWT signing key tests
    // ============================================

    #[tokio::test]
    async fn test_create_and_get_jwt_signing_key() {
        let repo = setup_test_db().await;

        let key = create_test_jwt_signing_key(
            "kid-1",
            "encrypted-private-key-1",
            "public-key-1",
            "dek-alias-1",
        );
        repo.create_jwt_signing_key(&key).await.unwrap();

        let fetched = repo.get_jwt_signing_key_by_kid("kid-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.kid, "kid-1");
        assert_eq!(fetched.encrypted_private_key, "encrypted-private-key-1");
        assert_eq!(fetched.public_key, "public-key-1");
        assert_eq!(fetched.dek_alias, "dek-alias-1");
    }

    #[tokio::test]
    async fn test_get_jwt_signing_key_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo
            .get_jwt_signing_key_by_kid("nonexistent")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_jwt_signing_key() {
        let repo = setup_test_db().await;

        let key = create_test_jwt_signing_key(
            "kid-1",
            "encrypted-private-key-1",
            "public-key-1",
            "dek-alias-1",
        );
        repo.create_jwt_signing_key(&key).await.unwrap();

        // Verify key is not invalidated initially
        let fetched = repo
            .get_jwt_signing_key_by_kid("kid-1")
            .await
            .unwrap()
            .unwrap();
        assert!(!fetched.invalidated);

        // Invalidate the key
        repo.invalidate_jwt_signing_key("kid-1").await.unwrap();

        // Verify key is now invalidated
        let fetched = repo
            .get_jwt_signing_key_by_kid("kid-1")
            .await
            .unwrap()
            .unwrap();
        assert!(fetched.invalidated);
    }

    #[tokio::test]
    async fn test_list_jwt_signing_keys() {
        let repo = setup_test_db().await;

        // Create multiple keys
        for i in 1..=5 {
            let key = create_test_jwt_signing_key(
                &format!("kid-{i}"),
                &format!("encrypted-private-key-{i}"),
                &format!("public-key-{i}"),
                &format!("dek-alias-{i}"),
            );
            repo.create_jwt_signing_key(&key).await.unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_jwt_signing_keys(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 5);
    }

    #[tokio::test]
    async fn test_list_jwt_signing_keys_pagination() {
        let repo = setup_test_db().await;

        // Create 5 keys with delays
        for i in 1..=5 {
            let key = create_test_jwt_signing_key(
                &format!("kid-{i}"),
                &format!("encrypted-private-key-{i}"),
                &format!("public-key-{i}"),
                &format!("dek-alias-{i}"),
            );
            repo.create_jwt_signing_key(&key).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo.list_jwt_signing_keys(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_jwt_signing_keys(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get third page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_jwt_signing_keys(&pagination).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());
    }

    // ============================================
    // STS configuration tests
    // ============================================

    use crate::logic::sts::config::{DevModeConfig, JwtTemplateModeConfig, StsTokenConfig};
    use crate::logic::token_mapping::template::{
        JwtTokenMappingConfig, JwtTokenTemplateConfig, JwtTokenTemplateValidationConfig,
        MappingSource, TokenLocation,
    };

    fn create_test_sts_dev_mode_config(id: &str) -> StsConfigurationDb {
        let now = WrappedChronoDateTime::now();
        StsConfigurationDb {
            config: StsTokenConfig::DevMode(DevModeConfig { id: id.to_string() }),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_sts_jwt_template_config(id: &str) -> StsConfigurationDb {
        let now = WrappedChronoDateTime::now();
        StsConfigurationDb {
            config: StsTokenConfig::JwtTemplate(JwtTemplateModeConfig {
                id: id.to_string(),
                mapping_template: JwtTokenTemplateConfig {
                    jwks_uri: "https://example.com/.well-known/jwks.json".to_string(),
                    userinfo_url: None,
                    introspect_url: None,
                    access_token_location: Some(TokenLocation::Header("Authorization".to_string())),
                    id_token_location: None,
                    mapping_template: JwtTokenMappingConfig {
                        issuer_field: MappingSource::AccessToken("iss".to_string()),
                        audience_field: MappingSource::AccessToken("aud".to_string()),
                        scopes_field: Some(MappingSource::AccessToken("scope".to_string())),
                        sub_field: MappingSource::AccessToken("sub".to_string()),
                        email_field: Some(MappingSource::AccessToken("email".to_string())),
                        groups_field: Some(MappingSource::AccessToken("groups".to_string())),
                        group_to_role_mappings: vec![],
                        scope_to_role_mappings: vec![],
                        scope_to_group_mappings: vec![],
                    },
                },
                validation_template: JwtTokenTemplateValidationConfig {
                    issuer: Some("https://example.com".to_string()),
                    valid_audiences: Some(vec!["test-audience".to_string()]),
                    required_scopes: None,
                    required_groups: None,
                },
            }),
            created_at: now,
            updated_at: now,
        }
    }

    fn get_sts_config_id(config: &StsTokenConfig) -> &str {
        match config {
            StsTokenConfig::DevMode(c) => &c.id,
            StsTokenConfig::JwtTemplate(c) => &c.id,
        }
    }

    fn get_sts_config_type(config: &StsTokenConfig) -> &str {
        match config {
            StsTokenConfig::DevMode(_) => "dev",
            StsTokenConfig::JwtTemplate(_) => "jwt_template",
        }
    }

    #[tokio::test]
    async fn test_create_and_get_sts_configuration_jwt_template() {
        let repo = setup_test_db().await;

        let config = create_test_sts_jwt_template_config("config-1");
        repo.create_sts_configuration(&config).await.unwrap();

        let fetched = repo.get_sts_configuration_by_id("config-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(get_sts_config_id(&fetched.config), "config-1");
        assert_eq!(get_sts_config_type(&fetched.config), "jwt_template");
    }

    #[tokio::test]
    async fn test_create_and_get_sts_configuration_dev_mode() {
        let repo = setup_test_db().await;

        let config = create_test_sts_dev_mode_config("config-1");
        repo.create_sts_configuration(&config).await.unwrap();

        let fetched = repo.get_sts_configuration_by_id("config-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(get_sts_config_id(&fetched.config), "config-1");
        assert_eq!(get_sts_config_type(&fetched.config), "dev");
    }

    #[tokio::test]
    async fn test_get_sts_configuration_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo
            .get_sts_configuration_by_id("nonexistent")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_sts_configuration() {
        let repo = setup_test_db().await;

        let config = create_test_sts_jwt_template_config("config-1");
        repo.create_sts_configuration(&config).await.unwrap();

        // Verify it exists
        let fetched = repo.get_sts_configuration_by_id("config-1").await.unwrap();
        assert!(fetched.is_some());

        // Delete it
        repo.delete_sts_configuration("config-1").await.unwrap();

        // Verify it's gone
        let fetched = repo.get_sts_configuration_by_id("config-1").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_sts_configurations() {
        let repo = setup_test_db().await;

        // Create multiple configurations
        for i in 1..=5 {
            let config = if i % 2 == 0 {
                create_test_sts_dev_mode_config(&format!("config-{i}"))
            } else {
                create_test_sts_jwt_template_config(&format!("config-{i}"))
            };
            repo.create_sts_configuration(&config).await.unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_sts_configurations(&pagination, None)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 5);
    }

    #[tokio::test]
    async fn test_list_sts_configurations_filter_by_type() {
        let repo = setup_test_db().await;

        // Create mixed configurations
        for i in 1..=6 {
            let config = if i % 2 == 0 {
                create_test_sts_dev_mode_config(&format!("config-{i}"))
            } else {
                create_test_sts_jwt_template_config(&format!("config-{i}"))
            };
            repo.create_sts_configuration(&config).await.unwrap();
        }

        // Filter by jwt_template
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_sts_configurations(&pagination, Some(StsTokenConfigType::JwtTemplate))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);
        assert!(
            result
                .items
                .iter()
                .all(|c| get_sts_config_type(&c.config) == "jwt_template")
        );

        // Filter by dev_mode
        let result = repo
            .list_sts_configurations(&pagination, Some(StsTokenConfigType::DevMode))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);
        assert!(
            result
                .items
                .iter()
                .all(|c| get_sts_config_type(&c.config) == "dev")
        );
    }

    #[tokio::test]
    async fn test_list_sts_configurations_pagination() {
        let repo = setup_test_db().await;

        // Create 5 configurations with delays
        for i in 1..=5 {
            let config = create_test_sts_jwt_template_config(&format!("config-{i}"));
            repo.create_sts_configuration(&config).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo
            .list_sts_configurations(&pagination, None)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo
            .list_sts_configurations(&pagination, None)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Get third page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo
            .list_sts_configurations(&pagination, None)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_list_sts_configurations_pagination_with_filter() {
        let repo = setup_test_db().await;

        // Create mixed configurations with delays
        for i in 1..=6 {
            let config = if i % 2 == 0 {
                create_test_sts_dev_mode_config(&format!("config-{i}"))
            } else {
                create_test_sts_jwt_template_config(&format!("config-{i}"))
            };
            repo.create_sts_configuration(&config).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page of jwt_template only
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo
            .list_sts_configurations(&pagination, Some(StsTokenConfigType::JwtTemplate))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(
            result
                .items
                .iter()
                .all(|c| get_sts_config_type(&c.config) == "jwt_template")
        );
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo
            .list_sts_configurations(&pagination, Some(StsTokenConfigType::JwtTemplate))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(
            result
                .items
                .iter()
                .all(|c| get_sts_config_type(&c.config) == "jwt_template")
        );
        assert!(result.next_page_token.is_none());
    }

    // ============================================
    // OAuth State tests
    // ============================================

    use crate::logic::token_mapping::TokenMapping;
    use crate::logic::user_auth_flow::config::{
        EncryptedOauthConfig, EncryptedOidcConfig, EncryptedUserAuthFlowConfig,
    };
    use encryption::logic::crypto_services::EncryptedString;

    fn create_test_oauth_state(state: &str, config_id: &str) -> OAuthState {
        let now = WrappedChronoDateTime::now();
        let expires_at = *now.get_inner() + chrono::Duration::seconds(300);
        OAuthState {
            state: state.to_string(),
            config_id: config_id.to_string(),
            code_verifier: Some("verifier123".to_string()),
            nonce: Some("nonce123".to_string()),
            redirect_uri: Some("/dashboard".to_string()),
            created_at: now,
            expires_at: WrappedChronoDateTime::new(expires_at),
        }
    }

    async fn setup_test_user_auth_flow_config(repo: &Repository, id: &str) {
        let now = WrappedChronoDateTime::now();
        let config = UserAuthFlowConfigDb {
            id: id.to_string(),
            config: EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(EncryptedOidcConfig {
                id: id.to_string(),
                base_config: EncryptedOauthConfig {
                    id: id.to_string(),
                    authorization_endpoint: "https://example.com/authorize".to_string(),
                    token_endpoint: "https://example.com/token".to_string(),
                    jwks_endpoint: "https://example.com/.well-known/jwks.json".to_string(),
                    client_id: "test".to_string(),
                    encrypted_client_secret: EncryptedString("encrypted_secret".to_string()),
                    dek_alias: "default".to_string(),
                    scopes: vec!["openid".to_string()],
                    introspect_url: None,
                    mapping: TokenMapping::JwtTemplate(JwtTokenMappingConfig {
                        issuer_field: MappingSource::AccessToken("iss".to_string()),
                        audience_field: MappingSource::AccessToken("aud".to_string()),
                        scopes_field: None,
                        sub_field: MappingSource::AccessToken("sub".to_string()),
                        email_field: None,
                        groups_field: None,
                        group_to_role_mappings: vec![],
                        scope_to_role_mappings: vec![],
                        scope_to_group_mappings: vec![],
                    }),
                },
                discovery_endpoint: Some(
                    "https://example.com/.well-known/openid-configuration".to_string(),
                ),
                userinfo_endpoint: None,
                introspect_url: None,
                mapping: TokenMapping::JwtTemplate(JwtTokenMappingConfig {
                    issuer_field: MappingSource::AccessToken("iss".to_string()),
                    audience_field: MappingSource::AccessToken("aud".to_string()),
                    scopes_field: None,
                    sub_field: MappingSource::AccessToken("sub".to_string()),
                    email_field: None,
                    groups_field: None,
                    group_to_role_mappings: vec![],
                    scope_to_role_mappings: vec![],
                    scope_to_group_mappings: vec![],
                }),
            }),
            created_at: now,
            updated_at: now,
        };
        repo.create_user_auth_flow_config(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_and_get_oauth_state() {
        let repo = setup_test_db().await;

        // Create required IdP configuration first
        setup_test_user_auth_flow_config(&repo, "config-1").await;

        let oauth_state = create_test_oauth_state("state123", "config-1");
        repo.create_oauth_state(&oauth_state).await.unwrap();

        let fetched = repo.get_oauth_state_by_state("state123").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.state, "state123");
        assert_eq!(fetched.config_id, "config-1");
        assert_eq!(fetched.code_verifier, Some("verifier123".to_string()));
        assert_eq!(fetched.nonce, Some("nonce123".to_string()));
        assert_eq!(fetched.redirect_uri, Some("/dashboard".to_string()));
    }

    #[tokio::test]
    async fn test_get_oauth_state_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo.get_oauth_state_by_state("nonexistent").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_oauth_state() {
        let repo = setup_test_db().await;

        // Create required IdP configuration first
        setup_test_user_auth_flow_config(&repo, "config-1").await;

        let oauth_state = create_test_oauth_state("state123", "config-1");
        repo.create_oauth_state(&oauth_state).await.unwrap();

        // Verify it exists
        let fetched = repo.get_oauth_state_by_state("state123").await.unwrap();
        assert!(fetched.is_some());

        // Delete it
        repo.delete_oauth_state("state123").await.unwrap();

        // Verify it's gone
        let fetched = repo.get_oauth_state_by_state("state123").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_expired_oauth_states() {
        let repo = setup_test_db().await;

        // Create required IdP configuration first
        setup_test_user_auth_flow_config(&repo, "config-1").await;

        // Create expired state
        let now = WrappedChronoDateTime::now();
        let past = *now.get_inner() - chrono::Duration::seconds(100);
        let expired_state = OAuthState {
            state: "expired-state".to_string(),
            config_id: "config-1".to_string(),
            code_verifier: None,
            nonce: None,
            redirect_uri: None,
            created_at: WrappedChronoDateTime::new(past),
            expires_at: WrappedChronoDateTime::new(past),
        };
        repo.create_oauth_state(&expired_state).await.unwrap();

        // Create valid state
        let valid_state = create_test_oauth_state("valid-state", "config-1");
        repo.create_oauth_state(&valid_state).await.unwrap();

        // Delete expired states
        let deleted = repo.delete_expired_oauth_states().await.unwrap();
        assert_eq!(deleted, 1);

        // Verify expired is gone
        let fetched = repo
            .get_oauth_state_by_state("expired-state")
            .await
            .unwrap();
        assert!(fetched.is_none());

        // Verify valid still exists
        let fetched = repo.get_oauth_state_by_state("valid-state").await.unwrap();
        assert!(fetched.is_some());
    }

    #[tokio::test]
    async fn test_oauth_state_without_optional_fields() {
        let repo = setup_test_db().await;

        // Create required IdP configuration first
        setup_test_user_auth_flow_config(&repo, "config-1").await;

        let now = WrappedChronoDateTime::now();
        let expires_at = *now.get_inner() + chrono::Duration::seconds(300);
        let oauth_state = OAuthState {
            state: "state-minimal".to_string(),
            config_id: "config-1".to_string(),
            code_verifier: None,
            nonce: None,
            redirect_uri: None,
            created_at: now,
            expires_at: WrappedChronoDateTime::new(expires_at),
        };
        repo.create_oauth_state(&oauth_state).await.unwrap();

        let fetched = repo
            .get_oauth_state_by_state("state-minimal")
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.state, "state-minimal");
        assert_eq!(fetched.code_verifier, None);
        assert_eq!(fetched.nonce, None);
        assert_eq!(fetched.redirect_uri, None);
    }
}
