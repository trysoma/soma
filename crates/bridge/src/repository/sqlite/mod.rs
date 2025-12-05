#![allow(non_camel_case_types)]
#![allow(dead_code)]
mod raw_impl;

#[allow(clippy::all)]
pub mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::logic::credential::{
    BrokerState, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::logic::instance::{
    FunctionInstanceSerialized, FunctionInstanceSerializedWithCredentials,
    ProviderInstanceSerializedWithCredentials, ProviderInstanceSerializedWithFunctions,
};
use crate::repository::{
    CreateBrokerState, CreateFunctionInstance, CreateProviderInstance,
    CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
};
use anyhow::Context;
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue};
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, SqlMigrationLoader, WrappedUuidV4,
        decode_pagination_token,
    },
};
use shared_macros::load_atlas_sql_migrations;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Repository {
    conn: shared::libsql::Connection,
}

impl Repository {
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }
}

impl ProviderRepositoryLike for Repository {
    async fn create_resource_server_credential(
        &self,
        params: &CreateResourceServerCredential,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_resource_server_credential_params {
            id: &params.id,
            type_id: &params.type_id,
            metadata: &params.metadata,
            value: &params.value,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            next_rotation_time: &params.next_rotation_time,
            dek_alias: &params.dek_alias,
        };

        create_resource_server_credential(&self.conn, sqlc_params)
            .await
            .context("Failed to create resource server credential")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_resource_server_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<ResourceServerCredentialSerialized>, CommonError> {
        let sqlc_params = get_resource_server_credential_by_id_params { id };

        let result = get_resource_server_credential_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get resource server credential by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn create_user_credential(
        &self,
        params: &CreateUserCredential,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_user_credential_params {
            id: &params.id,
            type_id: &params.type_id,
            metadata: &params.metadata,
            value: &params.value,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            next_rotation_time: &params.next_rotation_time,
            dek_alias: &params.dek_alias,
        };

        create_user_credential(&self.conn, sqlc_params)
            .await
            .context("Failed to create user credential")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_user_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<UserCredentialSerialized>, CommonError> {
        let sqlc_params = get_user_credential_by_id_params { id };

        let result = get_user_credential_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get user credential by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_user_credential(&self, id: &WrappedUuidV4) -> Result<(), CommonError> {
        let sqlc_params = delete_user_credential_params { id };

        delete_user_credential(&self.conn, sqlc_params)
            .await
            .context("Failed to delete user credential")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_resource_server_credential(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_resource_server_credential_params { id };

        delete_resource_server_credential(&self.conn, sqlc_params)
            .await
            .context("Failed to delete resource server credential")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_user_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UserCredentialSerialized>, CommonError> {
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
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_user_credentials_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_user_credentials(&self.conn, sqlc_params)
            .await
            .context("Failed to list user credentials")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<UserCredentialSerialized> = rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn list_resource_server_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<ResourceServerCredentialSerialized>, CommonError> {
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
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_resource_server_credentials_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_resource_server_credentials(&self.conn, sqlc_params)
            .await
            .context("Failed to list resource server credentials")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<ResourceServerCredentialSerialized> = rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn create_provider_instance(
        &self,
        params: &CreateProviderInstance,
    ) -> Result<(), CommonError> {
        // let return_on_successful_brokering_json = params.return_on_successful_brokering.as_ref()
        //     .map(|r| serde_json::to_value(r).ok())
        //     .flatten()
        //     .map(|v| WrappedJsonValue::new(v));

        let sqlc_params = create_provider_instance_params {
            id: &params.id,
            display_name: &params.display_name,
            resource_server_credential_id: &params.resource_server_credential_id,
            user_credential_id: &params.user_credential_id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            provider_controller_type_id: &params.provider_controller_type_id,
            credential_controller_type_id: &params.credential_controller_type_id,
            status: &params.status,
            return_on_successful_brokering: &params
                .return_on_successful_brokering
                .as_ref()
                .map(|v| WrappedJsonValue::new(serde_json::to_value(v).ok().unwrap_or_default())),
        };

        create_provider_instance(&self.conn, sqlc_params)
            .await
            .context("Failed to create provider instance")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_provider_instance_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ProviderInstanceSerializedWithFunctions>, CommonError> {
        let sqlc_params = get_provider_instance_by_id_params {
            id: &id.to_string(),
        };

        let result = get_provider_instance_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get provider instance by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn update_provider_instance(
        &self,
        id: &str,
        display_name: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = update_provider_instance_params {
            display_name: &display_name.to_string(),
            id: &id.to_string(),
        };

        update_provider_instance(&self.conn, sqlc_params)
            .await
            .context("Failed to update provider instance")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn update_provider_instance_after_brokering(
        &self,
        id: &str,
        user_credential_id: &WrappedUuidV4,
    ) -> Result<(), CommonError> {
        let sqlc_params = update_provider_instance_after_brokering_params {
            user_credential_id: &Some(user_credential_id.clone()),
            id: &id.to_string(),
        };

        update_provider_instance_after_brokering(&self.conn, sqlc_params)
            .await
            .context("Failed to update provider instance after brokering")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn delete_provider_instance(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_provider_instance_params {
            id: &id.to_string(),
        };

        delete_provider_instance(&self.conn, sqlc_params)
            .await
            .context("Failed to delete provider instance")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn create_function_instance(
        &self,
        params: &CreateFunctionInstance,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_function_instance_params {
            function_controller_type_id: &params.function_controller_type_id,
            provider_controller_type_id: &params.provider_controller_type_id,
            provider_instance_id: &params.provider_instance_id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_function_instance(&self.conn, sqlc_params)
            .await
            .context("Failed to create function instance")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_function_instance_by_id(
        &self,
        function_controller_type_id: &str,
        provider_controller_type_id: &str,
        provider_instance_id: &str,
    ) -> Result<Option<FunctionInstanceSerialized>, CommonError> {
        let sqlc_params = get_function_instance_by_id_params {
            function_controller_type_id: &function_controller_type_id.to_string(),
            provider_controller_type_id: &provider_controller_type_id.to_string(),
            provider_instance_id: &provider_instance_id.to_string(),
        };

        let result = get_function_instance_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get function instance by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_function_instance(
        &self,
        function_controller_type_id: &str,
        provider_controller_type_id: &str,
        provider_instance_id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_function_instance_params {
            function_controller_type_id: &function_controller_type_id.to_string(),
            provider_controller_type_id: &provider_controller_type_id.to_string(),
            provider_instance_id: &provider_instance_id.to_string(),
        };

        delete_function_instance(&self.conn, sqlc_params)
            .await
            .context("Failed to delete function instance")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_function_instance_with_credentials(
        &self,
        function_controller_type_id: &str,
        provider_controller_type_id: &str,
        provider_instance_id: &str,
    ) -> Result<Option<FunctionInstanceSerializedWithCredentials>, CommonError> {
        let sqlc_params = get_function_instance_with_credentials_params {
            function_controller_type_id: &function_controller_type_id.to_string(),
            provider_controller_type_id: &provider_controller_type_id.to_string(),
            provider_instance_id: &provider_instance_id.to_string(),
        };

        let result = get_function_instance_with_credentials(&self.conn, sqlc_params)
            .await
            .context("Failed to get function instance with credentials")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn create_broker_state(&self, params: &CreateBrokerState) -> Result<(), CommonError> {
        let sqlc_params = create_broker_state_params {
            id: &params.id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            provider_instance_id: &params.provider_instance_id,
            provider_controller_type_id: &params.provider_controller_type_id,
            credential_controller_type_id: &params.credential_controller_type_id,
            metadata: &params.metadata,
            action: &params.action,
        };

        create_broker_state(&self.conn, sqlc_params)
            .await
            .context("Failed to create broker state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_broker_state_by_id(&self, id: &str) -> Result<Option<BrokerState>, CommonError> {
        let sqlc_params = get_broker_state_by_id_params {
            id: &id.to_string(),
        };

        let result = get_broker_state_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get broker state by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_broker_state(&self, id: &str) -> Result<(), CommonError> {
        let sqlc_params = delete_broker_state_params {
            id: &id.to_string(),
        };

        delete_broker_state(&self.conn, sqlc_params)
            .await
            .context("Failed to delete broker state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_provider_instances(
        &self,
        pagination: &PaginationRequest,
        status: Option<&str>,
        provider_controller_type_id: Option<&str>,
    ) -> Result<PaginatedResponse<ProviderInstanceSerializedWithFunctions>, CommonError> {
        // Decode the base64 token to get the datetime cursor
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
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_provider_instances_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
            status: &status.map(|status| status.to_string()),
            provider_controller_type_id: &provider_controller_type_id.map(|s| s.to_string()),
        };

        let rows = get_provider_instances(&self.conn, sqlc_params)
            .await
            .context("Failed to get provider instances")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<ProviderInstanceSerializedWithFunctions> = rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.provider_instance.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn list_function_instances(
        &self,
        pagination: &PaginationRequest,
        provider_instance_id: Option<&str>,
    ) -> Result<PaginatedResponse<FunctionInstanceSerialized>, CommonError> {
        // Decode the base64 token to get the datetime cursor
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
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let sqlc_params = get_function_instances_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
            provider_instance_id: &provider_instance_id.map(|id| id.to_string()),
        };

        let rows = get_function_instances(&self.conn, sqlc_params)
            .await
            .context("Failed to get function instances")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<FunctionInstanceSerialized> = rows
            .into_iter()
            .map(|row| FunctionInstanceSerialized {
                function_controller_type_id: row.function_controller_type_id,
                provider_controller_type_id: row.provider_controller_type_id,
                provider_instance_id: row.provider_instance_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }

    async fn get_provider_instances_grouped_by_function_controller_type_id(
        &self,
        function_controller_type_ids: &[String],
    ) -> Result<
        Vec<crate::repository::ProviderInstancesGroupedByFunctionControllerTypeId>,
        CommonError,
    > {
        // Convert the slice of strings to JSON array format for SQLite IN clause
        let ids_json = function_controller_type_ids.join(", ");
        tracing::info!("ids_json: {}", ids_json);

        let sqlc_params = ManualGetProviderInstancesGroupedByFunctionControllerTypeIdParams {
            function_controller_type_ids: &Some(function_controller_type_ids.to_vec()),
        };

        let rows = manual_get_provider_instances_grouped_by_function_controller_type_id(
            &self.conn,
            sqlc_params,
        )
        .await
        .context("Failed to get provider instances grouped by function controller type id")
        .map_err(|e| CommonError::Repository {
            msg: e.to_string(),
            source: Some(e),
        })?;

        let items: Vec<crate::repository::ProviderInstancesGroupedByFunctionControllerTypeId> =
            rows.into_iter()
                .map(|row| row.try_into())
                .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    async fn update_resource_server_credential(
        &self,
        id: &WrappedUuidV4,
        value: Option<&WrappedJsonValue>,
        metadata: Option<&crate::logic::Metadata>,
        next_rotation_time: Option<&WrappedChronoDateTime>,
        updated_at: Option<&WrappedChronoDateTime>,
    ) -> Result<(), CommonError> {
        let value_owned = value.cloned();
        let metadata_owned =
            metadata.map(|m| WrappedJsonValue::new(serde_json::Value::Object(m.0.clone())));
        let next_rotation_owned = next_rotation_time.cloned();
        let updated_at_owned = updated_at.cloned();

        let params = update_resource_server_credential_params {
            id,
            value: &value_owned,
            metadata: &metadata_owned,
            next_rotation_time: &next_rotation_owned,
            updated_at: &updated_at_owned,
        };

        update_resource_server_credential(&self.conn, params)
            .await
            .context("Failed to update resource server credential")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }

    async fn update_user_credential(
        &self,
        id: &WrappedUuidV4,
        value: Option<&WrappedJsonValue>,
        metadata: Option<&crate::logic::Metadata>,
        next_rotation_time: Option<&WrappedChronoDateTime>,
        updated_at: Option<&WrappedChronoDateTime>,
    ) -> Result<(), CommonError> {
        let value_owned = value.cloned();
        let metadata_owned =
            metadata.map(|m| WrappedJsonValue::new(serde_json::Value::Object(m.0.clone())));
        let next_rotation_owned = next_rotation_time.cloned();
        let updated_at_owned = updated_at.cloned();

        let params = update_user_credential_params {
            id,
            value: &value_owned,
            metadata: &metadata_owned,
            next_rotation_time: &next_rotation_owned,
            updated_at: &updated_at_owned,
        };

        update_user_credential(&self.conn, params)
            .await
            .context("Failed to update user credential")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }

    async fn list_provider_instances_with_credentials(
        &self,
        pagination: &PaginationRequest,
        status: Option<&str>,
        rotation_window_end: Option<&WrappedChronoDateTime>,
    ) -> Result<PaginatedResponse<ProviderInstanceSerializedWithCredentials>, CommonError> {
        // Decode the cursor from the pagination token
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
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        let params: get_provider_instances_with_credentials_params<'_> =
            get_provider_instances_with_credentials_params {
                cursor: &cursor_datetime,
                status: &status.map(|s| s.to_string()),
                rotation_window_end: &rotation_window_end.copied(),
                page_size: &pagination.page_size,
            };

        let rows = get_provider_instances_with_credentials(&self.conn, params)
            .await
            .context("Failed to get provider instances with credentials")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<ProviderInstanceSerializedWithCredentials> = rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        // Check if we have more items than requested
        let has_more = items.len() > pagination.page_size as usize;
        let items = if has_more {
            items[..pagination.page_size as usize].to_vec()
        } else {
            items
        };

        let next_page_token = if has_more {
            items
                .last()
                .map(|item| item.provider_instance.created_at.to_string())
        } else {
            None
        };

        Ok(PaginatedResponse {
            items,
            next_page_token,
        })
    }
}

struct ManualGetProviderInstancesGroupedByFunctionControllerTypeIdParams<'a> {
    function_controller_type_ids: &'a Option<Vec<String>>,
}

async fn manual_get_provider_instances_grouped_by_function_controller_type_id(
    conn: &shared::libsql::Connection,
    params: ManualGetProviderInstancesGroupedByFunctionControllerTypeIdParams<'_>,
) -> Result<Vec<Row_get_provider_instances_grouped_by_function_controller_type_id>, libsql::Error> {
    let where_clause = match params.function_controller_type_ids {
        Some(ids) => {
            format!(
                "WHERE fi.function_controller_type_id IN ({})",
                ids.iter()
                    .map(|id| format!("'{id}'"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
        None => "".to_string(),
    };
    let stmt = conn
        .prepare(
            format!(
                r#"SELECT 
    fi.function_controller_type_id,
    CAST(
        JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'id', pi.id,
                'display_name', pi.display_name,
                'provider_controller_type_id', pi.provider_controller_type_id,
                'credential_controller_type_id', pi.credential_controller_type_id,
                'status', pi.status,
                'return_on_successful_brokering', pi.return_on_successful_brokering,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', pi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', pi.updated_at),

                -- resource server credential
                'resource_server_credential', COALESCE((
                    SELECT JSON_OBJECT(
                        'id', rsc.id,
                        'type_id', rsc.type_id,
                        'metadata', JSON(rsc.metadata),
                        'value', JSON(rsc.value),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
                        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
                        'next_rotation_time', CASE
                            WHEN rsc.next_rotation_time IS NOT NULL
                            THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time)
                            ELSE NULL END,
                        'dek_alias', rsc.dek_alias
                    )
                    FROM resource_server_credential rsc
                    WHERE rsc.id = pi.resource_server_credential_id
                ), JSON('null')),

                -- user credential
                'user_credential', COALESCE((
                    SELECT JSON_OBJECT(
                        'id', uc.id,
                        'type_id', uc.type_id,
                        'metadata', JSON(uc.metadata),
                        'value', JSON(uc.value),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
                        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
                        'next_rotation_time', CASE
                            WHEN uc.next_rotation_time IS NOT NULL
                            THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time)
                            ELSE NULL END,
                        'dek_alias', uc.dek_alias
                    )
                    FROM user_credential uc
                    WHERE uc.id = pi.user_credential_id
                ), JSON('null')),

                -- include function_instance metadata
                'function_instance', JSON_OBJECT(
                    'provider_controller_type_id', fi.provider_controller_type_id,
                    'provider_instance_id', fi.provider_instance_id,
                    'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                    'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
                )
            )
        ) AS TEXT
    ) AS provider_instances
FROM function_instance fi
JOIN provider_instance pi ON fi.provider_instance_id = pi.id
{where_clause}
GROUP BY fi.function_controller_type_id
ORDER BY fi.function_controller_type_id ASC"#,
            )
            .as_str(),
        )
        .await?;

    let mut rows = stmt.query(libsql::params![]).await?;
    let mut mapped = vec![];

    while let Some(row) = rows.next().await? {
        mapped.push(
            Row_get_provider_instances_grouped_by_function_controller_type_id {
                function_controller_type_id: row.get(0)?,
                provider_instances: row.get(1)?,
            },
        );
    }

    Ok(mapped)
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("dbs/bridge/migrations")
    }
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::{
        Metadata, ProviderInstanceSerialized,
        credential::{BrokerAction, BrokerActionRedirect},
        instance::FunctionInstanceSerialized,
    };
    use crate::repository::{
        BrokerState, CreateBrokerState, CreateFunctionInstance, CreateProviderInstance,
        CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
        ResourceServerCredentialSerialized, UserCredentialSerialized,
    };
    use shared::primitives::{
        SqlMigrationLoader, WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4,
    };
    use shared::test_utils::repository::setup_in_memory_database;

    /// Helper to create a test DEK alias for tests.
    /// Since bridge repository no longer manages DEKs, we just return a test alias string.
    fn create_test_dek_alias() -> String {
        format!("test-dek-{}", uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_create_and_get_resource_server_credential() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        let credential = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({
                "client_id": "test_client",
                "client_secret": "test_secret",
                "redirect_uri": "https://example.com/callback"
            })),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias,
        };

        let create_params = CreateResourceServerCredential::from(credential.clone());
        repo.create_resource_server_credential(&create_params)
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_resource_server_credential_by_id(&credential.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, credential.id);
        assert_eq!(retrieved.type_id, credential.type_id);
    }

    #[tokio::test]
    async fn test_create_and_get_user_credential() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        let credential = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({
                "code": "test_code",
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token",
                "expiry_time": now.to_string(),
                "sub": "test_sub"
            })),
            created_at: now,
            updated_at: now,
            next_rotation_time: Some(now),
            dek_alias,
        };

        let create_params = CreateUserCredential::from(credential.clone());
        repo.create_user_credential(&create_params).await.unwrap();

        // Verify it was created
        let retrieved = repo
            .get_user_credential_by_id(&credential.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, credential.id);
        assert_eq!(retrieved.type_id, credential.type_id);
    }

    #[tokio::test]
    async fn test_create_and_get_provider_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        // Create user credential
        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: Some(user_cred.id.clone()),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };

        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.provider_instance.id, provider_instance.id);
        assert_eq!(
            retrieved.provider_instance.display_name,
            provider_instance.display_name
        );
        assert_eq!(
            retrieved.provider_instance.provider_controller_type_id,
            provider_instance.provider_controller_type_id
        );
    }

    #[tokio::test]
    async fn test_update_provider_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        // Create user credential
        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Original Name".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: Some(user_cred.id.clone()),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };

        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Verify it was created with original name
        let retrieved = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.provider_instance.display_name, "Original Name");

        // Store the original updated_at timestamp
        let original_updated_at = retrieved.provider_instance.updated_at;

        // Sleep 1 second to ensure different timestamp (SQLite CURRENT_TIMESTAMP has second precision)
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Update the display name
        repo.update_provider_instance(&provider_instance.id, "Updated Name")
            .await
            .unwrap();

        // Verify it was updated
        let updated = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.provider_instance.id, provider_instance.id);
        assert_eq!(updated.provider_instance.display_name, "Updated Name");
        // Verify updated_at was changed (should be greater than the original)
        assert!(updated.provider_instance.updated_at.get_inner() > original_updated_at.get_inner());
    }

    #[tokio::test]
    async fn test_delete_provider_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        // Create user credential
        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Delete".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: Some(user_cred.id.clone()),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };

        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap();
        assert!(retrieved.is_some());

        // Delete the provider instance
        repo.delete_provider_instance(&provider_instance.id)
            .await
            .unwrap();

        // Verify it was deleted
        let deleted = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap();

        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_delete_provider_instance_with_cascade() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        // Create user credential
        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Cascade".to_string(),
            resource_server_credential_id: resource_server_cred.id,
            user_credential_id: Some(user_cred.id),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Create a function instance that depends on the provider instance
        let function_instance = FunctionInstanceSerialized {
            function_controller_type_id: "send_email".to_string(),
            provider_controller_type_id: provider_instance.provider_controller_type_id.clone(),
            provider_instance_id: provider_instance.id.clone(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(function_instance.clone()))
            .await
            .unwrap();

        // Verify function instance was created
        let retrieved_function = repo
            .get_function_instance_by_id(
                &function_instance.function_controller_type_id,
                &function_instance.provider_controller_type_id,
                &function_instance.provider_instance_id,
            )
            .await
            .unwrap();
        assert!(retrieved_function.is_some());

        // Delete the provider instance - should cascade delete function instances
        repo.delete_provider_instance(&provider_instance.id)
            .await
            .unwrap();

        // Verify provider instance was deleted
        let deleted_provider = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap();
        assert!(deleted_provider.is_none());

        // Verify function instance was also cascade deleted
        let deleted_function = repo
            .get_function_instance_by_id(
                &function_instance.function_controller_type_id,
                &function_instance.provider_controller_type_id,
                &function_instance.provider_instance_id,
            )
            .await
            .unwrap();
        assert!(deleted_function.is_none());
    }

    #[tokio::test]
    async fn test_create_get_and_delete_function_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Setup credentials and provider instance
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Function".to_string(),
            resource_server_credential_id: resource_server_cred.id,
            user_credential_id: Some(user_cred.id),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Create function instance
        let function_instance = FunctionInstanceSerialized {
            function_controller_type_id: "send_email".to_string(),
            provider_controller_type_id: provider_instance.provider_controller_type_id.clone(),
            provider_instance_id: provider_instance.id.clone(),
            created_at: now,
            updated_at: now,
        };

        repo.create_function_instance(&CreateFunctionInstance::from(function_instance.clone()))
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_function_instance_by_id(
                &function_instance.function_controller_type_id,
                &function_instance.provider_controller_type_id,
                &function_instance.provider_instance_id,
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            retrieved.function_controller_type_id,
            function_instance.function_controller_type_id
        );
        assert_eq!(
            retrieved.provider_controller_type_id,
            function_instance.provider_controller_type_id
        );
        assert_eq!(
            retrieved.provider_instance_id,
            function_instance.provider_instance_id
        );

        // Delete the function instance
        repo.delete_function_instance(
            &function_instance.function_controller_type_id,
            &function_instance.provider_controller_type_id,
            &function_instance.provider_instance_id,
        )
        .await
        .unwrap();

        // Verify it was deleted
        let deleted = repo
            .get_function_instance_by_id(
                &function_instance.function_controller_type_id,
                &function_instance.provider_controller_type_id,
                &function_instance.provider_instance_id,
            )
            .await
            .unwrap();

        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_create_and_get_broker_state() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential for broker state
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias,
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Function".to_string(),
            resource_server_credential_id: resource_server_cred.id,
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        let broker_state = BrokerState {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            provider_instance_id: provider_instance.id,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            action: BrokerAction::Redirect(BrokerActionRedirect {
                url: "https://example.com/oauth/authorize".to_string(),
            }),
        };

        repo.create_broker_state(&CreateBrokerState::from(broker_state.clone()))
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_broker_state_by_id(&broker_state.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, broker_state.id);
        assert_eq!(
            retrieved.provider_controller_type_id,
            broker_state.provider_controller_type_id
        );
        match retrieved.action {
            BrokerAction::Redirect(redirect) => {
                assert_eq!(redirect.url, "https://example.com/oauth/authorize")
            }
            _ => panic!("Expected Redirect action"),
        }
    }

    #[tokio::test]
    async fn test_delete_broker_state() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias,
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Function".to_string(),
            resource_server_credential_id: resource_server_cred.id,
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        let broker_state = BrokerState {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            provider_instance_id: provider_instance.id,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            action: BrokerAction::None,
        };

        repo.create_broker_state(&CreateBrokerState::from(broker_state.clone()))
            .await
            .unwrap();

        // Delete the broker state
        repo.delete_broker_state(&broker_state.id).await.unwrap();

        // Verify it was deleted
        let deleted = repo.get_broker_state_by_id(&broker_state.id).await.unwrap();

        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_records() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Test getting nonexistent resource server credential
        let result = repo
            .get_resource_server_credential_by_id(&WrappedUuidV4::new())
            .await
            .unwrap();
        assert!(result.is_none());

        // Test getting nonexistent user credential
        let result = repo
            .get_user_credential_by_id(&WrappedUuidV4::new())
            .await
            .unwrap();
        assert!(result.is_none());

        // Test getting nonexistent provider instance
        let result = repo
            .get_provider_instance_by_id(&uuid::Uuid::new_v4().to_string())
            .await
            .unwrap();
        assert!(result.is_none());

        // Test getting nonexistent function instance
        let result = repo
            .get_function_instance_by_id(
                "nonexistent_function",
                "nonexistent_provider",
                &uuid::Uuid::new_v4().to_string(),
            )
            .await
            .unwrap();
        assert!(result.is_none());

        // Test getting nonexistent broker state
        let result = repo
            .get_broker_state_by_id(&uuid::Uuid::new_v4().to_string())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_provider_instances_json_deserialization() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential with JSON fields
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_oauth2".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({
                "client_id": "test_client",
                "client_secret": "test_secret"
            })),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        // Create user credential with JSON fields
        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2_token".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({
                "access_token": "test_token",
                "refresh_token": "test_refresh"
            })),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider JSON".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: Some(user_cred.id.clone()),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "oauth2".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Create a function instance
        let function_instance = FunctionInstanceSerialized {
            function_controller_type_id: "send_email".to_string(),
            provider_controller_type_id: provider_instance.provider_controller_type_id.clone(),
            provider_instance_id: provider_instance.id.clone(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(function_instance.clone()))
            .await
            .unwrap();

        // List provider instances - this will test JSON deserialization
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = repo
            .list_provider_instances(&pagination, None, None)
            .await
            .unwrap();

        assert_eq!(result.items.len(), 1);
        let item = &result.items[0];

        // Verify provider instance
        assert_eq!(item.provider_instance.id, provider_instance.id);
        assert_eq!(
            item.provider_instance.display_name,
            provider_instance.display_name
        );

        // Verify resource server credential was deserialized correctly
        assert_eq!(item.resource_server_credential.id, resource_server_cred.id);
        assert_eq!(
            item.resource_server_credential.type_id,
            resource_server_cred.type_id
        );
        // Verify the JSON value was properly deserialized (not double-encoded)
        let rsc_value = item.resource_server_credential.value.get_inner();
        assert_eq!(rsc_value.get("client_id").unwrap(), "test_client");
        assert_eq!(rsc_value.get("client_secret").unwrap(), "test_secret");

        // Verify user credential was deserialized correctly
        assert!(item.user_credential.is_some());
        let uc = item.user_credential.as_ref().unwrap();
        assert_eq!(uc.id, user_cred.id);
        assert_eq!(uc.type_id, user_cred.type_id);
        // Verify the JSON value was properly deserialized (not double-encoded)
        let uc_value = uc.value.get_inner();
        assert_eq!(uc_value.get("access_token").unwrap(), "test_token");
        assert_eq!(uc_value.get("refresh_token").unwrap(), "test_refresh");

        // Verify functions were deserialized correctly
        assert_eq!(item.functions.len(), 1);
        assert_eq!(
            item.functions[0].function_controller_type_id,
            function_instance.function_controller_type_id
        );
        assert_eq!(
            item.functions[0].provider_controller_type_id,
            function_instance.provider_controller_type_id
        );
        assert_eq!(
            item.functions[0].provider_instance_id,
            function_instance.provider_instance_id
        );
    }

    #[tokio::test]
    async fn test_get_provider_instance_by_id_json_deserialization() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credential with JSON fields
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_oauth2".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({
                "client_id": "test_client_123",
                "client_secret": "test_secret_456"
            })),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(
            resource_server_cred.clone(),
        ))
        .await
        .unwrap();

        // Create user credential with JSON fields
        let user_cred = UserCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2_token".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({
                "access_token": "test_access_token_789",
                "refresh_token": "test_refresh_token_000"
            })),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider By ID".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: Some(user_cred.id.clone()),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "github".to_string(),
            credential_controller_type_id: "oauth2".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Create multiple function instances
        let function_instance_1 = FunctionInstanceSerialized {
            function_controller_type_id: "create_repo".to_string(),
            provider_controller_type_id: provider_instance.provider_controller_type_id.clone(),
            provider_instance_id: provider_instance.id.clone(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(function_instance_1.clone()))
            .await
            .unwrap();

        let function_instance_2 = FunctionInstanceSerialized {
            function_controller_type_id: "create_issue".to_string(),
            provider_controller_type_id: provider_instance.provider_controller_type_id.clone(),
            provider_instance_id: provider_instance.id.clone(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(function_instance_2.clone()))
            .await
            .unwrap();

        // Get provider instance by ID - this will test JSON deserialization
        let result = repo
            .get_provider_instance_by_id(&provider_instance.id)
            .await
            .unwrap()
            .unwrap();

        // Verify provider instance
        assert_eq!(result.provider_instance.id, provider_instance.id);
        assert_eq!(result.provider_instance.display_name, "Test Provider By ID");
        assert_eq!(
            result.provider_instance.provider_controller_type_id,
            "github"
        );

        // Verify resource server credential was deserialized correctly
        assert_eq!(
            result.resource_server_credential.id,
            resource_server_cred.id
        );
        assert_eq!(
            result.resource_server_credential.type_id,
            resource_server_cred.type_id
        );
        // Verify the JSON value was properly deserialized (not double-encoded)
        let rsc_value = result.resource_server_credential.value.get_inner();
        assert_eq!(rsc_value.get("client_id").unwrap(), "test_client_123");
        assert_eq!(rsc_value.get("client_secret").unwrap(), "test_secret_456");

        // Verify user credential was deserialized correctly
        assert!(result.user_credential.is_some());
        let uc = result.user_credential.as_ref().unwrap();
        assert_eq!(uc.id, user_cred.id);
        assert_eq!(uc.type_id, user_cred.type_id);
        // Verify the JSON value was properly deserialized (not double-encoded)
        let uc_value = uc.value.get_inner();
        assert_eq!(
            uc_value.get("access_token").unwrap(),
            "test_access_token_789"
        );
        assert_eq!(
            uc_value.get("refresh_token").unwrap(),
            "test_refresh_token_000"
        );

        // Verify functions were deserialized correctly
        assert_eq!(result.functions.len(), 2);
        // Functions are ordered, so verify both are present
        let func_types: Vec<String> = result
            .functions
            .iter()
            .map(|f| f.function_controller_type_id.clone())
            .collect();
        assert!(func_types.contains(&"create_repo".to_string()));
        assert!(func_types.contains(&"create_issue".to_string()));

        // Verify all functions have the correct provider_controller_type_id and provider_instance_id
        for func in &result.functions {
            assert_eq!(
                func.provider_controller_type_id,
                provider_instance.provider_controller_type_id
            );
            assert_eq!(func.provider_instance_id, provider_instance.id);
        }
    }

    #[tokio::test]
    async fn test_list_provider_instances_filter_by_status() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let repo = Repository::new(conn);

        // No need to create DEK - bridge repository doesn't manage encryption keys
        let now = shared::primitives::WrappedChronoDateTime::now();

        // Create resource server credentials
        let rsc_id_1 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_1 = CreateResourceServerCredential {
            id: rsc_id_1.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({"test": "value"})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_1)
            .await
            .unwrap();

        let rsc_id_2 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_2 = CreateResourceServerCredential {
            id: rsc_id_2.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({"test": "value"})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_2)
            .await
            .unwrap();

        // Create provider instances with different statuses
        let pi_params_1 = CreateProviderInstance {
            id: "pi-active".to_string(),
            display_name: "Active Provider".to_string(),
            resource_server_credential_id: rsc_id_1,
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "test_provider".to_string(),
            credential_controller_type_id: "test_credential".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_1).await.unwrap();

        let pi_params_2 = CreateProviderInstance {
            id: "pi-disabled".to_string(),
            display_name: "Disabled Provider".to_string(),
            resource_server_credential_id: rsc_id_2,
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "test_provider".to_string(),
            credential_controller_type_id: "test_credential".to_string(),
            status: "disabled".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_2).await.unwrap();

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        // Test with status=None (should return all)
        let result_all = repo
            .list_provider_instances(&pagination, None, None)
            .await
            .unwrap();
        assert_eq!(result_all.items.len(), 2);

        // Test with status="active" (should return only active)
        let result_active = repo
            .list_provider_instances(&pagination, Some("active"), None)
            .await
            .unwrap();
        assert_eq!(result_active.items.len(), 1);
        assert_eq!(result_active.items[0].provider_instance.status, "active");

        // Test with status="disabled" (should return only disabled)
        let result_disabled = repo
            .list_provider_instances(&pagination, Some("disabled"), None)
            .await
            .unwrap();
        assert_eq!(result_disabled.items.len(), 1);
        assert_eq!(
            result_disabled.items[0].provider_instance.status,
            "disabled"
        );
    }

    #[tokio::test]
    async fn test_list_function_instances_filter_by_provider_instance() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let repo = Repository::new(conn);

        // No need to create DEK - bridge repository doesn't manage encryption keys
        let now = shared::primitives::WrappedChronoDateTime::now();

        // Create resource server credentials
        let rsc_id_1 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_1 = CreateResourceServerCredential {
            id: rsc_id_1.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({"test": "value"})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_1)
            .await
            .unwrap();

        let rsc_id_2 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_2 = CreateResourceServerCredential {
            id: rsc_id_2.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({"test": "value"})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_2)
            .await
            .unwrap();

        // Create provider instances
        let pi_params_1 = CreateProviderInstance {
            id: "pi-1".to_string(),
            display_name: "Provider 1".to_string(),
            resource_server_credential_id: rsc_id_1,
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "test_provider".to_string(),
            credential_controller_type_id: "test_credential".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_1).await.unwrap();

        let pi_params_2 = CreateProviderInstance {
            id: "pi-2".to_string(),
            display_name: "Provider 2".to_string(),
            resource_server_credential_id: rsc_id_2,
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "test_provider".to_string(),
            credential_controller_type_id: "test_credential".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_2).await.unwrap();

        // Create function instances for different provider instances
        let fi_params_1 = CreateFunctionInstance {
            function_controller_type_id: "test_function_1".to_string(),
            provider_controller_type_id: "test_provider".to_string(),
            provider_instance_id: "pi-1".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&fi_params_1).await.unwrap();

        let fi_params_2 = CreateFunctionInstance {
            function_controller_type_id: "test_function_2".to_string(),
            provider_controller_type_id: "test_provider".to_string(),
            provider_instance_id: "pi-1".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&fi_params_2).await.unwrap();

        let fi_params_3 = CreateFunctionInstance {
            function_controller_type_id: "test_function_3".to_string(),
            provider_controller_type_id: "test_provider".to_string(),
            provider_instance_id: "pi-2".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&fi_params_3).await.unwrap();

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        // Test with provider_instance_id=None (should return all)
        let result_all = repo
            .list_function_instances(&pagination, None)
            .await
            .unwrap();
        assert_eq!(result_all.items.len(), 3);

        // Test with provider_instance_id="pi-1" (should return only pi-1 functions)
        let result_pi1 = repo
            .list_function_instances(&pagination, Some("pi-1"))
            .await
            .unwrap();
        assert_eq!(result_pi1.items.len(), 2);
        assert!(
            result_pi1
                .items
                .iter()
                .all(|item| item.provider_instance_id == "pi-1")
        );

        // Test with provider_instance_id="pi-2" (should return only pi-2 functions)
        let result_pi2 = repo
            .list_function_instances(&pagination, Some("pi-2"))
            .await
            .unwrap();
        assert_eq!(result_pi2.items.len(), 1);
        assert_eq!(result_pi2.items[0].provider_instance_id, "pi-2");
    }

    #[tokio::test]
    async fn test_list_provider_instances_filter_by_provider_controller_type_id() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let repo = Repository::new(conn);

        // No need to create DEK - bridge repository doesn't manage encryption keys
        let now = shared::primitives::WrappedChronoDateTime::now();

        // Create resource server credentials for provider instances
        let rsc_id_1 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_1 = CreateResourceServerCredential {
            id: rsc_id_1.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_1)
            .await
            .unwrap();

        let rsc_id_2 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_2 = CreateResourceServerCredential {
            id: rsc_id_2.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_2)
            .await
            .unwrap();

        let rsc_id_3 = shared::primitives::WrappedUuidV4::new();
        let rsc_params_3 = CreateResourceServerCredential {
            id: rsc_id_3.clone(),
            type_id: "test_type".to_string(),
            metadata: crate::logic::Metadata::new(),
            value: shared::primitives::WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: "test-dek".to_string(),
        };
        repo.create_resource_server_credential(&rsc_params_3)
            .await
            .unwrap();

        // Create three provider instances with different provider_controller_type_ids
        let pi_params_1 = CreateProviderInstance {
            id: "pi-1".to_string(),
            display_name: "Provider 1".to_string(),
            resource_server_credential_id: rsc_id_1.clone(),
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "github".to_string(),
            credential_controller_type_id: "test_cred".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_1).await.unwrap();

        let pi_params_2 = CreateProviderInstance {
            id: "pi-2".to_string(),
            display_name: "Provider 2".to_string(),
            resource_server_credential_id: rsc_id_2.clone(),
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "gitlab".to_string(),
            credential_controller_type_id: "test_cred".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_2).await.unwrap();

        let pi_params_3 = CreateProviderInstance {
            id: "pi-3".to_string(),
            display_name: "Provider 3".to_string(),
            resource_server_credential_id: rsc_id_3.clone(),
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "github".to_string(),
            credential_controller_type_id: "test_cred".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&pi_params_3).await.unwrap();

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        // Test with provider_controller_type_id=None (should return all 3)
        let result_all = repo
            .list_provider_instances(&pagination, None, None)
            .await
            .unwrap();
        assert_eq!(result_all.items.len(), 3);

        // Test with provider_controller_type_id="github" (should return 2)
        let result_github = repo
            .list_provider_instances(&pagination, None, Some("github"))
            .await
            .unwrap();
        assert_eq!(result_github.items.len(), 2);
        assert!(
            result_github
                .items
                .iter()
                .all(|item| item.provider_instance.provider_controller_type_id == "github")
        );

        // Test with provider_controller_type_id="gitlab" (should return 1)
        let result_gitlab = repo
            .list_provider_instances(&pagination, None, Some("gitlab"))
            .await
            .unwrap();
        assert_eq!(result_gitlab.items.len(), 1);
        assert_eq!(
            result_gitlab.items[0]
                .provider_instance
                .provider_controller_type_id,
            "gitlab"
        );

        // Test with combined filters: status="active" AND provider_controller_type_id="github" (should return 2)
        let result_combined = repo
            .list_provider_instances(&pagination, Some("active"), Some("github"))
            .await
            .unwrap();
        assert_eq!(result_combined.items.len(), 2);
        assert!(
            result_combined
                .items
                .iter()
                .all(|item| item.provider_instance.status == "active"
                    && item.provider_instance.provider_controller_type_id == "github")
        );
    }

    #[tokio::test]
    async fn test_get_provider_instances_grouped_by_function_controller_type_id() {
        shared::setup_test!();

        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create resource server credentials
        let rsc1 = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({"client_id": "test1"})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(rsc1.clone()))
            .await
            .unwrap();

        let rsc2 = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "oauth2".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({"client_id": "test2"})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(rsc2.clone()))
            .await
            .unwrap();

        // Create provider instances
        let pi1 = ProviderInstanceSerialized {
            id: "provider-1".to_string(),
            display_name: "Provider 1".to_string(),
            resource_server_credential_id: rsc1.id.clone(),
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "oauth2".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(pi1.clone()))
            .await
            .unwrap();

        let pi2 = ProviderInstanceSerialized {
            id: "provider-2".to_string(),
            display_name: "Provider 2".to_string(),
            resource_server_credential_id: rsc2.id.clone(),
            user_credential_id: None,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "oauth2".to_string(),
            status: "active".to_string(),
            return_on_successful_brokering: None,
        };
        repo.create_provider_instance(&CreateProviderInstance::from(pi2.clone()))
            .await
            .unwrap();

        // Create function instances
        let fi1 = FunctionInstanceSerialized {
            function_controller_type_id: "send_email".to_string(),
            provider_controller_type_id: "google_mail".to_string(),
            provider_instance_id: "provider-1".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(fi1.clone()))
            .await
            .unwrap();

        let fi2 = FunctionInstanceSerialized {
            function_controller_type_id: "send_email".to_string(),
            provider_controller_type_id: "google_mail".to_string(),
            provider_instance_id: "provider-2".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(fi2.clone()))
            .await
            .unwrap();

        let fi3 = FunctionInstanceSerialized {
            function_controller_type_id: "read_email".to_string(),
            provider_controller_type_id: "google_mail".to_string(),
            provider_instance_id: "provider-1".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.create_function_instance(&CreateFunctionInstance::from(fi3.clone()))
            .await
            .unwrap();

        // Test: Get grouped provider instances for specific function types
        let function_ids = vec!["send_email".to_string(), "read_email".to_string()];
        let result = repo
            .get_provider_instances_grouped_by_function_controller_type_id(&function_ids)
            .await
            .unwrap();

        // Should have 2 groups (one for send_email, one for read_email)
        assert_eq!(result.len(), 2);

        // Find the send_email group
        let send_email_group = result
            .iter()
            .find(|g| g.function_controller_type_id == "send_email")
            .expect("send_email group not found");

        // Should have 2 provider instances for send_email
        assert_eq!(send_email_group.provider_instances.len(), 2);

        // Verify provider instances have credentials
        for pi in &send_email_group.provider_instances {
            assert!(!pi.provider_instance.id.is_empty());
            assert!(!pi.resource_server_credential.id.to_string().is_empty());
        }

        // Find the read_email group
        let read_email_group = result
            .iter()
            .find(|g| g.function_controller_type_id == "read_email")
            .expect("read_email group not found");

        // Should have 1 provider instance for read_email
        assert_eq!(read_email_group.provider_instances.len(), 1);
        assert_eq!(
            read_email_group.provider_instances[0].provider_instance.id,
            "provider-1"
        );

        // Test with empty function_ids
        let result_empty = repo
            .get_provider_instances_grouped_by_function_controller_type_id(&[])
            .await
            .unwrap();
        assert_eq!(result_empty.len(), 0);
    }

    #[tokio::test]
    async fn test_update_resource_server_credential() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create initial resource server credential
        let initial_value = WrappedJsonValue::new(serde_json::json!({
            "client_id": "initial-client-id",
            "client_secret": "initial-secret"
        }));
        let initial_metadata = Metadata::new();
        let initial_rotation_time = WrappedChronoDateTime::now();

        let rsc_id = WrappedUuidV4::new();
        let rsc = ResourceServerCredentialSerialized {
            id: rsc_id.clone(),
            type_id: "oauth2_client_credentials".to_string(),
            metadata: initial_metadata.clone(),
            value: initial_value.clone(),
            created_at: now,
            updated_at: now,
            next_rotation_time: Some(initial_rotation_time),
            dek_alias: dek_alias.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(rsc.clone()))
            .await
            .unwrap();

        // Test 1: Update only value, other fields should remain unchanged
        let new_value = WrappedJsonValue::new(serde_json::json!({
            "client_id": "updated-client-id",
            "client_secret": "updated-secret"
        }));
        repo.update_resource_server_credential(&rsc_id, Some(&new_value), None, None, None)
            .await
            .unwrap();

        let updated = repo
            .get_resource_server_credential_by_id(&rsc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.value, new_value, "Value should be updated");
        assert_eq!(
            updated.metadata.0, initial_metadata.0,
            "Metadata should remain unchanged when None is passed"
        );
        assert_eq!(
            updated.next_rotation_time,
            Some(initial_rotation_time),
            "Rotation time should remain unchanged when None is passed"
        );

        // Test 2: Update only metadata, other fields should remain unchanged
        let mut new_metadata_map = serde_json::Map::new();
        new_metadata_map.insert("key".to_string(), serde_json::json!("value"));
        let new_metadata = Metadata(new_metadata_map);

        repo.update_resource_server_credential(&rsc_id, None, Some(&new_metadata), None, None)
            .await
            .unwrap();

        let updated = repo
            .get_resource_server_credential_by_id(&rsc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated.value, new_value,
            "Value should remain unchanged when None is passed"
        );
        assert_eq!(
            updated.metadata.0, new_metadata.0,
            "Metadata should be updated"
        );
        assert_eq!(
            updated.next_rotation_time,
            Some(initial_rotation_time),
            "Rotation time should remain unchanged when None is passed"
        );

        // Test 3: Update only next_rotation_time, other fields should remain unchanged
        let new_rotation_time = WrappedChronoDateTime::now();
        repo.update_resource_server_credential(&rsc_id, None, None, Some(&new_rotation_time), None)
            .await
            .unwrap();

        let updated = repo
            .get_resource_server_credential_by_id(&rsc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated.value, new_value,
            "Value should remain unchanged when None is passed"
        );
        assert_eq!(
            updated.metadata.0, new_metadata.0,
            "Metadata should remain unchanged when None is passed"
        );
        assert!(
            updated.next_rotation_time.is_some(),
            "Rotation time should be updated"
        );

        // Test 4: Pass None for all optional fields, all should remain unchanged
        let before_none_update = repo
            .get_resource_server_credential_by_id(&rsc_id)
            .await
            .unwrap()
            .unwrap();

        repo.update_resource_server_credential(&rsc_id, None, None, None, None)
            .await
            .unwrap();

        let after_none_update = repo
            .get_resource_server_credential_by_id(&rsc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            after_none_update.value, before_none_update.value,
            "Value should remain unchanged when None is passed"
        );
        assert_eq!(
            after_none_update.metadata.0, before_none_update.metadata.0,
            "Metadata should remain unchanged when None is passed"
        );
        assert_eq!(
            after_none_update.next_rotation_time, before_none_update.next_rotation_time,
            "Rotation time should remain unchanged when None is passed"
        );

        // Test 5: Update all fields at once
        let final_value = WrappedJsonValue::new(serde_json::json!({
            "client_id": "final-client-id",
            "client_secret": "final-secret"
        }));
        let mut final_metadata_map = serde_json::Map::new();
        final_metadata_map.insert("final".to_string(), serde_json::json!(true));
        let final_metadata = Metadata(final_metadata_map);
        let final_rotation_time = WrappedChronoDateTime::now();

        repo.update_resource_server_credential(
            &rsc_id,
            Some(&final_value),
            Some(&final_metadata),
            Some(&final_rotation_time),
            None,
        )
        .await
        .unwrap();

        let updated = repo
            .get_resource_server_credential_by_id(&rsc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.value, final_value, "All fields should be updated");
        assert_eq!(
            updated.metadata.0, final_metadata.0,
            "All fields should be updated"
        );
        assert!(
            updated.next_rotation_time.is_some(),
            "All fields should be updated"
        );
    }

    #[tokio::test]
    async fn test_update_user_credential() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);
        let now = WrappedChronoDateTime::now();
        let dek_alias = create_test_dek_alias();

        // Create initial user credential
        let initial_value = WrappedJsonValue::new(serde_json::json!({
            "access_token": "initial-token",
            "refresh_token": "initial-refresh"
        }));
        let mut initial_metadata_map = serde_json::Map::new();
        initial_metadata_map.insert(
            "initial_key".to_string(),
            serde_json::json!("initial_value"),
        );
        let initial_metadata = Metadata(initial_metadata_map);
        let initial_rotation_time = WrappedChronoDateTime::now();

        let uc_id = WrappedUuidV4::new();
        let uc = UserCredentialSerialized {
            id: uc_id.clone(),
            type_id: "oauth2_authorization_code".to_string(),
            metadata: initial_metadata.clone(),
            value: initial_value.clone(),
            created_at: now,
            updated_at: now,
            next_rotation_time: Some(initial_rotation_time),
            dek_alias: dek_alias.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(uc.clone()))
            .await
            .unwrap();

        // Test 1: Update only value, other fields should remain unchanged
        let new_value = WrappedJsonValue::new(serde_json::json!({
            "access_token": "updated-token",
            "refresh_token": "updated-refresh"
        }));
        repo.update_user_credential(&uc_id, Some(&new_value), None, None, None)
            .await
            .unwrap();

        let updated = repo
            .get_user_credential_by_id(&uc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.value, new_value, "Value should be updated");
        assert_eq!(
            updated.metadata.0, initial_metadata.0,
            "Metadata should remain unchanged when None is passed"
        );
        assert_eq!(
            updated.next_rotation_time,
            Some(initial_rotation_time),
            "Rotation time should remain unchanged when None is passed"
        );

        // Test 2: Update only metadata, other fields should remain unchanged
        let mut new_metadata_map = serde_json::Map::new();
        new_metadata_map.insert("new_key".to_string(), serde_json::json!("new_value"));
        let new_metadata = Metadata(new_metadata_map);

        repo.update_user_credential(&uc_id, None, Some(&new_metadata), None, None)
            .await
            .unwrap();

        let updated = repo
            .get_user_credential_by_id(&uc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated.value, new_value,
            "Value should remain unchanged when None is passed"
        );
        assert_eq!(
            updated.metadata.0, new_metadata.0,
            "Metadata should be updated"
        );
        assert_eq!(
            updated.next_rotation_time,
            Some(initial_rotation_time),
            "Rotation time should remain unchanged when None is passed"
        );

        // Test 3: Update only next_rotation_time, other fields should remain unchanged
        let new_rotation_time = WrappedChronoDateTime::now();
        repo.update_user_credential(&uc_id, None, None, Some(&new_rotation_time), None)
            .await
            .unwrap();

        let updated = repo
            .get_user_credential_by_id(&uc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated.value, new_value,
            "Value should remain unchanged when None is passed"
        );
        assert_eq!(
            updated.metadata.0, new_metadata.0,
            "Metadata should remain unchanged when None is passed"
        );
        assert!(
            updated.next_rotation_time.is_some(),
            "Rotation time should be updated"
        );

        // Test 4: Update with None values (all should remain unchanged)
        let before_none_update = repo
            .get_user_credential_by_id(&uc_id)
            .await
            .unwrap()
            .unwrap();

        repo.update_user_credential(&uc_id, None, None, None, None)
            .await
            .unwrap();

        let after_none_update = repo
            .get_user_credential_by_id(&uc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            after_none_update.value, before_none_update.value,
            "Value should remain unchanged when None is passed"
        );
        assert_eq!(
            after_none_update.metadata.0, before_none_update.metadata.0,
            "Metadata should remain unchanged when None is passed"
        );
        assert_eq!(
            after_none_update.next_rotation_time, before_none_update.next_rotation_time,
            "Rotation time should remain unchanged when None is passed"
        );

        // Test 5: Update all fields at once
        let final_value = WrappedJsonValue::new(serde_json::json!({
            "access_token": "final-token",
            "refresh_token": "final-refresh"
        }));
        let mut final_metadata_map = serde_json::Map::new();
        final_metadata_map.insert("final_key".to_string(), serde_json::json!("final_value"));
        let final_metadata = Metadata(final_metadata_map);
        let final_rotation_time = WrappedChronoDateTime::now();

        repo.update_user_credential(
            &uc_id,
            Some(&final_value),
            Some(&final_metadata),
            Some(&final_rotation_time),
            None,
        )
        .await
        .unwrap();

        let updated = repo
            .get_user_credential_by_id(&uc_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.value, final_value, "All fields should be updated");
        assert_eq!(
            updated.metadata.0, final_metadata.0,
            "All fields should be updated"
        );
        assert!(
            updated.next_rotation_time.is_some(),
            "All fields should be updated"
        );
    }
}
