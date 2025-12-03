#![allow(non_camel_case_types)]
#![allow(dead_code)]
mod raw_impl;

#[allow(clippy::all)]
pub mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::repository::{
    ApiKey, ApiKeyWithUser, CreateApiKey, CreateGroup, CreateGroupMembership,
    CreateIdpConfiguration, CreateJwtSigningKey, CreateOAuthState, CreateStsConfiguration,
    CreateUser, Group, GroupMemberWithUser, GroupMembership, IdpConfiguration, JwtSigningKey,
    OAuthState, StsConfiguration, UpdateIdpConfiguration, UpdateStsConfiguration, UpdateUser, User,
    UserGroupWithGroup, UserRepositoryLike,
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
    async fn create_user(&self, params: &CreateUser) -> Result<(), CommonError> {
        let sqlc_params = create_user_params {
            id: &params.id,
            user_type: &params.user_type,
            email: &params.email,
            role: &params.role,
            description: &params.description,
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

        let email = params.email.clone().or(existing.email);
        let role = params.role.clone().unwrap_or(existing.role);
        let description = params.description.clone().or(existing.description);

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
        user_type: Option<&str>,
        role: Option<&str>,
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
        let user_type_owned = user_type.map(|s| s.to_string());
        let role_owned = role.map(|s| s.to_string());

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

    async fn create_api_key(&self, params: &CreateApiKey) -> Result<(), CommonError> {
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
    ) -> Result<Option<ApiKeyWithUser>, CommonError> {
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

    async fn get_api_key_by_id(&self, id: &str) -> Result<Option<ApiKeyWithUser>, CommonError> {
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
    ) -> Result<PaginatedResponse<ApiKey>, CommonError> {
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

        let items: Vec<ApiKey> = rows.into_iter().map(|row| row.into()).collect();

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
    async fn create_group(&self, params: &CreateGroup) -> Result<(), CommonError> {
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
    async fn create_group_membership(
        &self,
        params: &CreateGroupMembership,
    ) -> Result<(), CommonError> {
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
    async fn create_jwt_signing_key(
        &self,
        params: &CreateJwtSigningKey,
    ) -> Result<(), CommonError> {
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
        params: &CreateStsConfiguration,
    ) -> Result<(), CommonError> {
        let query = r#"
            INSERT INTO sts_configuration (id, type, value, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
        "#;

        self.conn
            .execute(
                query,
                vec![
                    params.id.clone().into(),
                    params.config_type.clone().into(),
                    params.value.clone().map(|v| v.into()).unwrap_or(libsql::Value::Null),
                    params.created_at.to_string().into(),
                    params.updated_at.to_string().into(),
                ],
            )
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
    ) -> Result<Option<StsConfiguration>, CommonError> {
        let query = r#"
            SELECT id, type, value, created_at, updated_at
            FROM sts_configuration
            WHERE id = ?
        "#;

        let mut rows = self
            .conn
            .query(query, vec![libsql::Value::from(id.to_string())])
            .await
            .context("Failed to get STS configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        if let Some(row) = rows.next().await.map_err(|e| CommonError::Repository {
            msg: e.to_string(),
            source: Some(e.into()),
        })? {
            Ok(Some(StsConfiguration {
                id: row.get::<String>(0).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config_type: row.get::<String>(1).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                value: row.get::<Option<String>>(2).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                created_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(3)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                updated_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(4)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_sts_configuration(
        &self,
        id: &str,
        params: &UpdateStsConfiguration,
    ) -> Result<(), CommonError> {
        let mut updates = vec![];
        let mut values: Vec<libsql::Value> = vec![];

        if let Some(ref config_type) = params.config_type {
            updates.push("type = ?");
            values.push(config_type.clone().into());
        }

        if let Some(ref value) = params.value {
            updates.push("value = ?");
            values.push(value.clone().into());
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        values.push(WrappedChronoDateTime::now().to_string().into());
        values.push(id.into());

        let query = format!(
            "UPDATE sts_configuration SET {} WHERE id = ?",
            updates.join(", ")
        );

        self.conn
            .execute(&query, values)
            .await
            .context("Failed to update STS configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }

    async fn delete_sts_configuration(&self, id: &str) -> Result<(), CommonError> {
        let query = "DELETE FROM sts_configuration WHERE id = ?";

        self.conn
            .execute(query, vec![libsql::Value::from(id.to_string())])
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
        config_type: Option<&str>,
    ) -> Result<PaginatedResponse<StsConfiguration>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(
                    |e| CommonError::Repository {
                        msg: format!("Invalid datetime in pagination token: {e}"),
                        source: Some(e.into()),
                    },
                )?)
            }
        } else {
            None
        };

        let (query, values): (String, Vec<libsql::Value>) = match (config_type, &cursor_datetime) {
            (Some(ct), Some(cursor)) => (
                r#"
                    SELECT id, type, value, created_at, updated_at
                    FROM sts_configuration
                    WHERE type = ? AND created_at < ?
                    ORDER BY created_at DESC
                    LIMIT ?
                "#.to_string(),
                vec![
                    ct.into(),
                    cursor.to_string().into(),
                    (pagination.page_size + 1).into(),
                ],
            ),
            (Some(ct), None) => (
                r#"
                    SELECT id, type, value, created_at, updated_at
                    FROM sts_configuration
                    WHERE type = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                "#.to_string(),
                vec![ct.into(), (pagination.page_size + 1).into()],
            ),
            (None, Some(cursor)) => (
                r#"
                    SELECT id, type, value, created_at, updated_at
                    FROM sts_configuration
                    WHERE created_at < ?
                    ORDER BY created_at DESC
                    LIMIT ?
                "#.to_string(),
                vec![
                    cursor.to_string().into(),
                    (pagination.page_size + 1).into(),
                ],
            ),
            (None, None) => (
                r#"
                    SELECT id, type, value, created_at, updated_at
                    FROM sts_configuration
                    ORDER BY created_at DESC
                    LIMIT ?
                "#.to_string(),
                vec![(pagination.page_size + 1).into()],
            ),
        };

        let mut rows = self
            .conn
            .query(&query, values)
            .await
            .context("Failed to list STS configurations")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let mut items = vec![];
        while let Some(row) = rows.next().await.map_err(|e| CommonError::Repository {
            msg: e.to_string(),
            source: Some(e.into()),
        })? {
            items.push(StsConfiguration {
                id: row.get::<String>(0).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config_type: row.get::<String>(1).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                value: row.get::<Option<String>>(2).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                created_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(3)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                updated_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(4)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
            });
        }

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    // IdP configuration methods
    async fn create_idp_configuration(
        &self,
        params: &CreateIdpConfiguration,
    ) -> Result<(), CommonError> {
        let query = r#"
            INSERT INTO idp_configuration (id, type, config, encrypted_client_secret, dek_alias, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        self.conn
            .execute(
                query,
                vec![
                    params.id.clone().into(),
                    params.config_type.clone().into(),
                    params.config.clone().into(),
                    params
                        .encrypted_client_secret
                        .clone()
                        .map(|v| v.into())
                        .unwrap_or(libsql::Value::Null),
                    params
                        .dek_alias
                        .clone()
                        .map(|v| v.into())
                        .unwrap_or(libsql::Value::Null),
                    params.created_at.to_string().into(),
                    params.updated_at.to_string().into(),
                ],
            )
            .await
            .context("Failed to create IdP configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }

    async fn get_idp_configuration_by_id(
        &self,
        id: &str,
    ) -> Result<Option<IdpConfiguration>, CommonError> {
        let query = r#"
            SELECT id, type, config, encrypted_client_secret, dek_alias, created_at, updated_at
            FROM idp_configuration
            WHERE id = ?
        "#;

        let mut rows = self
            .conn
            .query(query, vec![libsql::Value::from(id.to_string())])
            .await
            .context("Failed to get IdP configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        if let Some(row) = rows.next().await.map_err(|e| CommonError::Repository {
            msg: e.to_string(),
            source: Some(e.into()),
        })? {
            Ok(Some(IdpConfiguration {
                id: row.get::<String>(0).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config_type: row.get::<String>(1).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config: row.get::<String>(2).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                encrypted_client_secret: row
                    .get::<Option<String>>(3)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                dek_alias: row
                    .get::<Option<String>>(4)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                created_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(5)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                updated_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(6)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_idp_configuration(
        &self,
        id: &str,
        params: &UpdateIdpConfiguration,
    ) -> Result<(), CommonError> {
        let mut updates = vec![];
        let mut values: Vec<libsql::Value> = vec![];

        if let Some(ref config_type) = params.config_type {
            updates.push("type = ?");
            values.push(config_type.clone().into());
        }

        if let Some(ref config) = params.config {
            updates.push("config = ?");
            values.push(config.clone().into());
        }

        if let Some(ref encrypted_client_secret) = params.encrypted_client_secret {
            updates.push("encrypted_client_secret = ?");
            values.push(encrypted_client_secret.clone().into());
        }

        if let Some(ref dek_alias) = params.dek_alias {
            updates.push("dek_alias = ?");
            values.push(dek_alias.clone().into());
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        values.push(WrappedChronoDateTime::now().to_string().into());
        values.push(id.into());

        let query = format!(
            "UPDATE idp_configuration SET {} WHERE id = ?",
            updates.join(", ")
        );

        self.conn
            .execute(&query, values)
            .await
            .context("Failed to update IdP configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }

    async fn delete_idp_configuration(&self, id: &str) -> Result<(), CommonError> {
        let query = "DELETE FROM idp_configuration WHERE id = ?";

        self.conn
            .execute(query, vec![libsql::Value::from(id.to_string())])
            .await
            .context("Failed to delete IdP configuration")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }

    async fn list_idp_configurations(
        &self,
        pagination: &PaginationRequest,
        config_type: Option<&str>,
    ) -> Result<PaginatedResponse<IdpConfiguration>, CommonError> {
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(WrappedChronoDateTime::try_from(decoded_parts[0].as_str()).map_err(
                    |e| CommonError::Repository {
                        msg: format!("Invalid datetime in pagination token: {e}"),
                        source: Some(e.into()),
                    },
                )?)
            }
        } else {
            None
        };

        let (query, values): (String, Vec<libsql::Value>) = match (config_type, &cursor_datetime) {
            (Some(ct), Some(cursor)) => (
                r#"
                    SELECT id, type, config, encrypted_client_secret, dek_alias, created_at, updated_at
                    FROM idp_configuration
                    WHERE type = ? AND created_at < ?
                    ORDER BY created_at DESC
                    LIMIT ?
                "#
                .to_string(),
                vec![
                    ct.into(),
                    cursor.to_string().into(),
                    (pagination.page_size + 1).into(),
                ],
            ),
            (Some(ct), None) => (
                r#"
                    SELECT id, type, config, encrypted_client_secret, dek_alias, created_at, updated_at
                    FROM idp_configuration
                    WHERE type = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                "#
                .to_string(),
                vec![ct.into(), (pagination.page_size + 1).into()],
            ),
            (None, Some(cursor)) => (
                r#"
                    SELECT id, type, config, encrypted_client_secret, dek_alias, created_at, updated_at
                    FROM idp_configuration
                    WHERE created_at < ?
                    ORDER BY created_at DESC
                    LIMIT ?
                "#
                .to_string(),
                vec![
                    cursor.to_string().into(),
                    (pagination.page_size + 1).into(),
                ],
            ),
            (None, None) => (
                r#"
                    SELECT id, type, config, encrypted_client_secret, dek_alias, created_at, updated_at
                    FROM idp_configuration
                    ORDER BY created_at DESC
                    LIMIT ?
                "#
                .to_string(),
                vec![(pagination.page_size + 1).into()],
            ),
        };

        let mut rows = self
            .conn
            .query(&query, values)
            .await
            .context("Failed to list IdP configurations")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let mut items = vec![];
        while let Some(row) = rows.next().await.map_err(|e| CommonError::Repository {
            msg: e.to_string(),
            source: Some(e.into()),
        })? {
            items.push(IdpConfiguration {
                id: row.get::<String>(0).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config_type: row.get::<String>(1).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config: row.get::<String>(2).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                encrypted_client_secret: row
                    .get::<Option<String>>(3)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                dek_alias: row
                    .get::<Option<String>>(4)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                created_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(5)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                updated_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(6)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
            });
        }

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    // OAuth state methods
    async fn create_oauth_state(&self, params: &CreateOAuthState) -> Result<(), CommonError> {
        let query = r#"
            INSERT INTO oauth_state (state, config_id, code_verifier, nonce, redirect_uri, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        self.conn
            .execute(
                query,
                vec![
                    params.state.clone().into(),
                    params.config_id.clone().into(),
                    params
                        .code_verifier
                        .clone()
                        .map(|v| v.into())
                        .unwrap_or(libsql::Value::Null),
                    params
                        .nonce
                        .clone()
                        .map(|v| v.into())
                        .unwrap_or(libsql::Value::Null),
                    params
                        .redirect_uri
                        .clone()
                        .map(|v| v.into())
                        .unwrap_or(libsql::Value::Null),
                    params.created_at.to_string().into(),
                    params.expires_at.to_string().into(),
                ],
            )
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
        let query = r#"
            SELECT state, config_id, code_verifier, nonce, redirect_uri, created_at, expires_at
            FROM oauth_state
            WHERE state = ?
        "#;

        let mut rows = self
            .conn
            .query(query, vec![libsql::Value::from(state.to_string())])
            .await
            .context("Failed to get OAuth state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        if let Some(row) = rows.next().await.map_err(|e| CommonError::Repository {
            msg: e.to_string(),
            source: Some(e.into()),
        })? {
            Ok(Some(OAuthState {
                state: row.get::<String>(0).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                config_id: row.get::<String>(1).map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                code_verifier: row
                    .get::<Option<String>>(2)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                nonce: row
                    .get::<Option<String>>(3)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                redirect_uri: row
                    .get::<Option<String>>(4)
                    .map_err(|e| CommonError::Repository {
                        msg: e.to_string(),
                        source: Some(e.into()),
                    })?,
                created_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(5)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
                expires_at: WrappedChronoDateTime::try_from(
                    row.get::<String>(6)
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e.into()),
                        })?
                        .as_str(),
                )
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e.into()),
                })?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_oauth_state(&self, state: &str) -> Result<(), CommonError> {
        let query = "DELETE FROM oauth_state WHERE state = ?";

        self.conn
            .execute(query, vec![libsql::Value::from(state.to_string())])
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
        let query = "DELETE FROM oauth_state WHERE expires_at < ?";

        let rows_affected = self
            .conn
            .execute(query, vec![libsql::Value::from(now.to_string())])
            .await
            .context("Failed to delete expired OAuth states")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::CreateGroup;
    use crate::repository::CreateGroupMembership;
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

    fn create_test_user(id: &str, user_type: &str, email: Option<&str>, role: &str) -> CreateUser {
        let now = WrappedChronoDateTime::now();
        CreateUser {
            id: id.to_string(),
            user_type: user_type.to_string(),
            email: email.map(|s| s.to_string()),
            role: role.to_string(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_api_key(id: &str, hashed_value: &str, user_id: &str) -> CreateApiKey {
        let now = WrappedChronoDateTime::now();
        CreateApiKey {
            id: id.to_string(),
            hashed_value: hashed_value.to_string(),
            description: Some(format!("Test API key {}", id)),
            user_id: user_id.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_group(id: &str, name: &str) -> CreateGroup {
        let now = WrappedChronoDateTime::now();
        CreateGroup {
            id: id.to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_group_membership(group_id: &str, user_id: &str) -> CreateGroupMembership {
        let now = WrappedChronoDateTime::now();
        CreateGroupMembership {
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
    ) -> CreateJwtSigningKey {
        let now = WrappedChronoDateTime::now();
        let expires_at = *now.get_inner() + chrono::Duration::days(30);
        CreateJwtSigningKey {
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
            "machine",
            Some("test@example.com"),
            "admin",
        );
        repo.create_user(&user).await.unwrap();

        let fetched = repo.get_user_by_id("user-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, "user-1");
        assert_eq!(fetched.user_type, "machine");
        assert_eq!(fetched.email, Some("test@example.com".to_string()));
        assert_eq!(fetched.role, "admin");
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
            "machine",
            Some("old@example.com"),
            "user",
        );
        repo.create_user(&user).await.unwrap();

        let update = UpdateUser {
            email: Some("new@example.com".to_string()),
            role: Some("admin".to_string()),
            description: None,
        };
        repo.update_user("user-1", &update).await.unwrap();

        let fetched = repo.get_user_by_id("user-1").await.unwrap().unwrap();
        assert_eq!(fetched.email, Some("new@example.com".to_string()));
        assert_eq!(fetched.role, "admin");
    }

    #[tokio::test]
    async fn test_update_user_partial() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            "machine",
            Some("old@example.com"),
            "user",
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
        assert_eq!(fetched.role, "user"); // Should be unchanged
    }

    #[tokio::test]
    async fn test_delete_user() {
        let repo = setup_test_db().await;

        let user = create_test_user(
            "user-1",
            "machine",
            Some("test@example.com"),
            "admin",
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
                    "human"
                } else {
                    "machine"
                },
                Some(&format!("user{i}@example.com")),
                if i % 2 == 0 { "admin" } else { "user" },
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
            .list_users(&pagination, Some("machine"), None)
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);

        // Filter by role
        let result = repo
            .list_users(&pagination, None, Some("admin"))
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
                "machine",
                Some(&format!("user{i}@example.com")),
                "user",
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
            "machine",
            Some("test@example.com"),
            "admin",
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
            "machine",
            Some("test@example.com"),
            "admin",
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
            "machine",
            Some("user1@example.com"),
            "admin",
        );
        let user2 = create_test_user(
            "user-2",
            "human",
            Some("user2@example.com"),
            "user",
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
            "machine",
            Some("user1@example.com"),
            "admin",
        );
        let user2 = create_test_user(
            "user-2",
            "human",
            Some("user2@example.com"),
            "user",
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
            "machine",
            Some("test@example.com"),
            "admin",
        );
        repo.create_user(&user).await.unwrap();

        // Create API keys
        for i in 1..=3 {
            let api_key = create_test_api_key(
                &format!("api-key-{i}"),
                &format!("hash-{i}"),
                "user-1",
            );
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
            "machine",
            Some("test@example.com"),
            "admin",
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
            "machine",
            Some("test@example.com"),
            "admin",
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
                "machine",
                Some(&format!("user{i}@example.com")),
                "user",
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
                "machine",
                Some(&format!("user{i}@example.com")),
                "user",
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
            "machine",
            Some("test@example.com"),
            "admin",
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
            "machine",
            Some("test@example.com"),
            "admin",
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
                "machine",
                Some(&format!("user{i}@example.com")),
                "user",
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
            "machine",
            Some("user1@example.com"),
            "user",
        );
        let user2 = create_test_user(
            "user-2",
            "human",
            Some("user2@example.com"),
            "user",
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
                "machine",
                Some(&format!("user{i}@example.com")),
                "user",
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
            "machine",
            Some("test@example.com"),
            "admin",
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

    fn create_test_sts_configuration(
        id: &str,
        config_type: &str,
        value: Option<&str>,
    ) -> CreateStsConfiguration {
        let now = WrappedChronoDateTime::now();
        CreateStsConfiguration {
            id: id.to_string(),
            config_type: config_type.to_string(),
            value: value.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_sts_configuration() {
        let repo = setup_test_db().await;

        let config = create_test_sts_configuration(
            "config-1",
            "jwt_template",
            Some(r#"{"issuer":"test"}"#),
        );
        repo.create_sts_configuration(&config).await.unwrap();

        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, "config-1");
        assert_eq!(fetched.config_type, "jwt_template");
        assert_eq!(fetched.value, Some(r#"{"issuer":"test"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_create_sts_configuration_with_null_value() {
        let repo = setup_test_db().await;

        let config = create_test_sts_configuration("config-1", "dev", None);
        repo.create_sts_configuration(&config).await.unwrap();

        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, "config-1");
        assert_eq!(fetched.config_type, "dev");
        assert!(fetched.value.is_none());
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
    async fn test_update_sts_configuration() {
        let repo = setup_test_db().await;

        let config = create_test_sts_configuration(
            "config-1",
            "jwt_template",
            Some(r#"{"issuer":"original"}"#),
        );
        repo.create_sts_configuration(&config).await.unwrap();

        // Update the value
        let update = UpdateStsConfiguration {
            config_type: None,
            value: Some(r#"{"issuer":"updated"}"#.to_string()),
        };
        repo.update_sts_configuration("config-1", &update)
            .await
            .unwrap();

        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.value, Some(r#"{"issuer":"updated"}"#.to_string()));
        assert_eq!(fetched.config_type, "jwt_template"); // Unchanged
    }

    #[tokio::test]
    async fn test_update_sts_configuration_type() {
        let repo = setup_test_db().await;

        let config = create_test_sts_configuration("config-1", "jwt_template", None);
        repo.create_sts_configuration(&config).await.unwrap();

        // Update the type
        let update = UpdateStsConfiguration {
            config_type: Some("dev".to_string()),
            value: None,
        };
        repo.update_sts_configuration("config-1", &update)
            .await
            .unwrap();

        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.config_type, "dev");
    }

    #[tokio::test]
    async fn test_update_sts_configuration_empty_update() {
        let repo = setup_test_db().await;

        let config = create_test_sts_configuration(
            "config-1",
            "jwt_template",
            Some(r#"{"issuer":"test"}"#),
        );
        repo.create_sts_configuration(&config).await.unwrap();

        // Empty update should succeed without changing anything
        let update = UpdateStsConfiguration {
            config_type: None,
            value: None,
        };
        repo.update_sts_configuration("config-1", &update)
            .await
            .unwrap();

        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.config_type, "jwt_template");
        assert_eq!(fetched.value, Some(r#"{"issuer":"test"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_delete_sts_configuration() {
        let repo = setup_test_db().await;

        let config = create_test_sts_configuration(
            "config-1",
            "jwt_template",
            Some(r#"{"issuer":"test"}"#),
        );
        repo.create_sts_configuration(&config).await.unwrap();

        // Verify it exists
        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap();
        assert!(fetched.is_some());

        // Delete it
        repo.delete_sts_configuration("config-1").await.unwrap();

        // Verify it's gone
        let fetched = repo
            .get_sts_configuration_by_id("config-1")
            .await
            .unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_sts_configurations() {
        let repo = setup_test_db().await;

        // Create multiple configurations
        for i in 1..=5 {
            let config = create_test_sts_configuration(
                &format!("config-{i}"),
                if i % 2 == 0 { "dev" } else { "jwt_template" },
                Some(&format!(r#"{{"issuer":"test-{i}"}}"#)),
            );
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
            let config = create_test_sts_configuration(
                &format!("config-{i}"),
                if i % 2 == 0 { "dev" } else { "jwt_template" },
                Some(&format!(r#"{{"issuer":"test-{i}"}}"#)),
            );
            repo.create_sts_configuration(&config).await.unwrap();
        }

        // Filter by jwt_template
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo
            .list_sts_configurations(&pagination, Some("jwt_template"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);
        assert!(result.items.iter().all(|c| c.config_type == "jwt_template"));

        // Filter by dev
        let result = repo
            .list_sts_configurations(&pagination, Some("dev"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 3);
        assert!(result.items.iter().all(|c| c.config_type == "dev"));
    }

    #[tokio::test]
    async fn test_list_sts_configurations_pagination() {
        let repo = setup_test_db().await;

        // Create 5 configurations with delays
        for i in 1..=5 {
            let config = create_test_sts_configuration(
                &format!("config-{i}"),
                "jwt_template",
                Some(&format!(r#"{{"issuer":"test-{i}"}}"#)),
            );
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
            let config = create_test_sts_configuration(
                &format!("config-{i}"),
                if i % 2 == 0 { "dev" } else { "jwt_template" },
                Some(&format!(r#"{{"issuer":"test-{i}"}}"#)),
            );
            repo.create_sts_configuration(&config).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get first page of jwt_template only
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo
            .list_sts_configurations(&pagination, Some("jwt_template"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.items.iter().all(|c| c.config_type == "jwt_template"));
        assert!(result.next_page_token.is_some());

        // Get second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo
            .list_sts_configurations(&pagination, Some("jwt_template"))
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.items.iter().all(|c| c.config_type == "jwt_template"));
        assert!(result.next_page_token.is_none());
    }

    // ============================================
    // IdP Configuration tests
    // ============================================

    fn create_test_idp_configuration(
        id: &str,
        config_type: &str,
        config: &str,
    ) -> CreateIdpConfiguration {
        let now = WrappedChronoDateTime::now();
        CreateIdpConfiguration {
            id: id.to_string(),
            config_type: config_type.to_string(),
            config: config.to_string(),
            encrypted_client_secret: Some("encrypted_secret".to_string()),
            dek_alias: Some("default".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_idp_configuration() {
        let repo = setup_test_db().await;

        let config_json = r#"{"name":"Google","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://accounts.google.com"}"#;
        let config = create_test_idp_configuration("idp-1", "oidc_authorization_flow", config_json);
        repo.create_idp_configuration(&config).await.unwrap();

        let fetched = repo.get_idp_configuration_by_id("idp-1").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, "idp-1");
        assert_eq!(fetched.config_type, "oidc_authorization_flow");
        assert_eq!(fetched.encrypted_client_secret, Some("encrypted_secret".to_string()));
        assert_eq!(fetched.dek_alias, Some("default".to_string()));
    }

    #[tokio::test]
    async fn test_get_idp_configuration_not_found() {
        let repo = setup_test_db().await;

        let fetched = repo.get_idp_configuration_by_id("nonexistent").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_update_idp_configuration() {
        let repo = setup_test_db().await;

        let config_json = r#"{"name":"Google","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://accounts.google.com"}"#;
        let config = create_test_idp_configuration("idp-1", "oidc_authorization_flow", config_json);
        repo.create_idp_configuration(&config).await.unwrap();

        let new_config_json = r#"{"name":"Google Updated","client_id":"test-updated","redirect_uri":"http://localhost/callback","issuer_url":"https://accounts.google.com"}"#;
        let update = UpdateIdpConfiguration {
            config_type: None,
            config: Some(new_config_json.to_string()),
            encrypted_client_secret: Some("new_encrypted_secret".to_string()),
            dek_alias: Some("new_alias".to_string()),
        };
        repo.update_idp_configuration("idp-1", &update).await.unwrap();

        let fetched = repo.get_idp_configuration_by_id("idp-1").await.unwrap().unwrap();
        assert!(fetched.config.contains("Google Updated"));
        assert_eq!(fetched.encrypted_client_secret, Some("new_encrypted_secret".to_string()));
        assert_eq!(fetched.dek_alias, Some("new_alias".to_string()));
    }

    #[tokio::test]
    async fn test_update_idp_configuration_partial() {
        let repo = setup_test_db().await;

        let config_json = r#"{"name":"Google","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://accounts.google.com"}"#;
        let config = create_test_idp_configuration("idp-1", "oidc_authorization_flow", config_json);
        repo.create_idp_configuration(&config).await.unwrap();

        // Only update encrypted_client_secret
        let update = UpdateIdpConfiguration {
            config_type: None,
            config: None,
            encrypted_client_secret: Some("new_encrypted_secret".to_string()),
            dek_alias: None,
        };
        repo.update_idp_configuration("idp-1", &update).await.unwrap();

        let fetched = repo.get_idp_configuration_by_id("idp-1").await.unwrap().unwrap();
        assert!(fetched.config.contains("Google")); // Original config unchanged
        assert_eq!(fetched.encrypted_client_secret, Some("new_encrypted_secret".to_string()));
        assert_eq!(fetched.dek_alias, Some("default".to_string())); // Original dek_alias unchanged
    }

    #[tokio::test]
    async fn test_delete_idp_configuration() {
        let repo = setup_test_db().await;

        let config_json = r#"{"name":"Google","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://accounts.google.com"}"#;
        let config = create_test_idp_configuration("idp-1", "oidc_authorization_flow", config_json);
        repo.create_idp_configuration(&config).await.unwrap();

        repo.delete_idp_configuration("idp-1").await.unwrap();

        let fetched = repo.get_idp_configuration_by_id("idp-1").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_idp_configurations() {
        let repo = setup_test_db().await;

        for i in 1..=5 {
            let config_json = format!(r#"{{"name":"Provider {}","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://example{}.com"}}"#, i, i);
            let config = create_test_idp_configuration(
                &format!("idp-{}", i),
                "oidc_authorization_flow",
                &config_json,
            );
            repo.create_idp_configuration(&config).await.unwrap();
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_idp_configurations(&pagination, None).await.unwrap();
        assert_eq!(result.items.len(), 5);
    }

    #[tokio::test]
    async fn test_list_idp_configurations_by_type() {
        let repo = setup_test_db().await;

        // Create OIDC configs
        for i in 1..=3 {
            let config_json = format!(r#"{{"name":"OIDC Provider {}","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://oidc{}.com"}}"#, i, i);
            let config = create_test_idp_configuration(
                &format!("oidc-{}", i),
                "oidc_authorization_flow",
                &config_json,
            );
            repo.create_idp_configuration(&config).await.unwrap();
        }

        // Create OAuth configs
        for i in 1..=2 {
            let config_json = format!(r#"{{"name":"OAuth Provider {}","client_id":"test","redirect_uri":"http://localhost/callback","authorization_endpoint":"https://oauth{}.com/auth","token_endpoint":"https://oauth{}.com/token"}}"#, i, i, i);
            let config = create_test_idp_configuration(
                &format!("oauth-{}", i),
                "oauth_authorization_flow",
                &config_json,
            );
            repo.create_idp_configuration(&config).await.unwrap();
        }

        // List only OIDC
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_idp_configurations(&pagination, Some("oidc_authorization_flow")).await.unwrap();
        assert_eq!(result.items.len(), 3);
        assert!(result.items.iter().all(|c| c.config_type == "oidc_authorization_flow"));

        // List only OAuth
        let result = repo.list_idp_configurations(&pagination, Some("oauth_authorization_flow")).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.items.iter().all(|c| c.config_type == "oauth_authorization_flow"));
    }

    #[tokio::test]
    async fn test_list_idp_configurations_pagination() {
        let repo = setup_test_db().await;

        for i in 1..=5 {
            let config_json = format!(r#"{{"name":"Provider {}","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://example{}.com"}}"#, i, i);
            let config = create_test_idp_configuration(
                &format!("idp-{}", i),
                "oidc_authorization_flow",
                &config_json,
            );
            repo.create_idp_configuration(&config).await.unwrap();
        }

        // First page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo.list_idp_configurations(&pagination, None).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_idp_configurations(&pagination, None).await.unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.next_page_token.is_some());

        // Third page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token,
        };
        let result = repo.list_idp_configurations(&pagination, None).await.unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.next_page_token.is_none());
    }

    // ============================================
    // OAuth State tests
    // ============================================

    fn create_test_oauth_state(state: &str, config_id: &str) -> CreateOAuthState {
        let now = WrappedChronoDateTime::now();
        let expires_at = *now.get_inner() + chrono::Duration::seconds(300);
        CreateOAuthState {
            state: state.to_string(),
            config_id: config_id.to_string(),
            code_verifier: Some("verifier123".to_string()),
            nonce: Some("nonce123".to_string()),
            redirect_uri: Some("/dashboard".to_string()),
            created_at: now,
            expires_at: WrappedChronoDateTime::new(expires_at),
        }
    }

    async fn setup_test_idp_config(repo: &Repository, id: &str) {
        let config_json = r#"{"name":"Test Provider","client_id":"test","redirect_uri":"http://localhost/callback","issuer_url":"https://example.com"}"#;
        let config = create_test_idp_configuration(id, "oidc_authorization_flow", config_json);
        repo.create_idp_configuration(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_and_get_oauth_state() {
        let repo = setup_test_db().await;

        // Create required IdP configuration first
        setup_test_idp_config(&repo, "config-1").await;

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
        setup_test_idp_config(&repo, "config-1").await;

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
        setup_test_idp_config(&repo, "config-1").await;

        // Create expired state
        let now = WrappedChronoDateTime::now();
        let past = *now.get_inner() - chrono::Duration::seconds(100);
        let expired_state = CreateOAuthState {
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
        let fetched = repo.get_oauth_state_by_state("expired-state").await.unwrap();
        assert!(fetched.is_none());

        // Verify valid still exists
        let fetched = repo.get_oauth_state_by_state("valid-state").await.unwrap();
        assert!(fetched.is_some());
    }

    #[tokio::test]
    async fn test_oauth_state_without_optional_fields() {
        let repo = setup_test_db().await;

        // Create required IdP configuration first
        setup_test_idp_config(&repo, "config-1").await;

        let now = WrappedChronoDateTime::now();
        let expires_at = *now.get_inner() + chrono::Duration::seconds(300);
        let oauth_state = CreateOAuthState {
            state: "state-minimal".to_string(),
            config_id: "config-1".to_string(),
            code_verifier: None,
            nonce: None,
            redirect_uri: None,
            created_at: now,
            expires_at: WrappedChronoDateTime::new(expires_at),
        };
        repo.create_oauth_state(&oauth_state).await.unwrap();

        let fetched = repo.get_oauth_state_by_state("state-minimal").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.state, "state-minimal");
        assert_eq!(fetched.code_verifier, None);
        assert_eq!(fetched.nonce, None);
        assert_eq!(fetched.redirect_uri, None);
    }
}
