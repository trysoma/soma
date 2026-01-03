//! SQLite repository implementation for environment crate

#![allow(non_camel_case_types)]
mod raw_impl;

#[allow(clippy::all, unused_mut)]
#[allow(dead_code)]
mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::logic::{secret::Secret, variable::Variable};
use crate::repository::{
    CreateSecret, CreateVariable, SecretRepositoryLike, UpdateSecret, UpdateVariable,
    VariableRepositoryLike,
};
use anyhow::Context;
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, SqlMigrationLoader, WrappedChronoDateTime,
        WrappedUuidV4, decode_pagination_token,
    },
};
use shared_macros::load_atlas_sql_migrations;
use std::collections::BTreeMap;

/// SQLite repository for environment data
#[derive(Clone)]
pub struct Repository {
    conn: shared::libsql::Connection,
}

impl Repository {
    /// Create a new repository instance
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }

    /// Get the underlying connection
    pub fn connection(&self) -> &shared::libsql::Connection {
        &self.conn
    }
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("dbs/environment/migrations")
    }
}

#[async_trait::async_trait]
impl SecretRepositoryLike for Repository {
    async fn create_secret(&self, params: &CreateSecret) -> Result<(), CommonError> {
        let sqlc_params = insert_secret_params {
            id: &params.id,
            key: &params.key,
            encrypted_secret: &params.encrypted_secret,
            dek_alias: &params.dek_alias,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_secret(&self.conn, sqlc_params)
            .await
            .context("Failed to create secret")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn update_secret(&self, params: &UpdateSecret) -> Result<(), CommonError> {
        let sqlc_params = update_secret_params {
            id: &params.id,
            encrypted_secret: &params.encrypted_secret,
            dek_alias: &params.dek_alias,
            updated_at: &params.updated_at,
        };

        update_secret(&self.conn, sqlc_params)
            .await
            .context("Failed to update secret")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_secret(&self, id: &WrappedUuidV4) -> Result<(), CommonError> {
        let sqlc_params = delete_secret_params { id };

        delete_secret(&self.conn, sqlc_params)
            .await
            .context("Failed to delete secret")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_secret_by_id(&self, id: &WrappedUuidV4) -> Result<Option<Secret>, CommonError> {
        let sqlc_params = get_secret_by_id_params { id };

        let result = get_secret_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get secret by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match result {
            Some(row) => Ok(Some(Secret::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn get_secret_by_key(&self, key: &str) -> Result<Option<Secret>, CommonError> {
        let key_string = key.to_string();
        let sqlc_params = get_secret_by_key_params { key: &key_string };

        let result = get_secret_by_key(&self.conn, sqlc_params)
            .await
            .context("Failed to get secret by key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match result {
            Some(row) => Ok(Some(Secret::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn get_secrets(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Secret>, CommonError> {
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

        let sqlc_params = get_secrets_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_secrets(&self.conn, sqlc_params)
            .await
            .context("Failed to get secrets")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Secret>, CommonError> =
            rows.into_iter().map(Secret::try_from).collect();
        let items = items?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |secret| vec![secret.created_at.get_inner().to_rfc3339()],
        ))
    }
}

#[async_trait::async_trait]
impl VariableRepositoryLike for Repository {
    async fn create_variable(&self, params: &CreateVariable) -> Result<(), CommonError> {
        let sqlc_params = insert_variable_params {
            id: &params.id,
            key: &params.key,
            value: &params.value,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        insert_variable(&self.conn, sqlc_params)
            .await
            .context("Failed to create variable")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn update_variable(&self, params: &UpdateVariable) -> Result<(), CommonError> {
        let sqlc_params = update_variable_params {
            id: &params.id,
            value: &params.value,
            updated_at: &params.updated_at,
        };

        update_variable(&self.conn, sqlc_params)
            .await
            .context("Failed to update variable")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_variable(&self, id: &WrappedUuidV4) -> Result<(), CommonError> {
        let sqlc_params = delete_variable_params { id };

        delete_variable(&self.conn, sqlc_params)
            .await
            .context("Failed to delete variable")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_variable_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<Variable>, CommonError> {
        let sqlc_params = get_variable_by_id_params { id };

        let result = get_variable_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get variable by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match result {
            Some(row) => Ok(Some(Variable::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn get_variable_by_key(&self, key: &str) -> Result<Option<Variable>, CommonError> {
        let key_string = key.to_string();
        let sqlc_params = get_variable_by_key_params { key: &key_string };

        let result = get_variable_by_key(&self.conn, sqlc_params)
            .await
            .context("Failed to get variable by key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match result {
            Some(row) => Ok(Some(Variable::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn get_variables(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<Variable>, CommonError> {
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

        let sqlc_params = get_variables_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_variables(&self.conn, sqlc_params)
            .await
            .context("Failed to get variables")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Result<Vec<Variable>, CommonError> =
            rows.into_iter().map(Variable::try_from).collect();
        let items = items?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |variable| vec![variable.created_at.get_inner().to_rfc3339()],
        ))
    }
}
