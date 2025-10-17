#![allow(non_camel_case_types)]
mod raw_impl;

include!("raw.generated.rs");

use crate::logic::{
    BrokerState, DataEncryptionKey, DataEncryptionKeyListItem, FunctionInstanceSerialized, FunctionInstanceSerializedWithCredentials,
    ProviderInstanceSerialized, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::repository::{
    CreateBrokerState, CreateDataEncryptionKey, CreateFunctionInstance, CreateProviderInstance,
    CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
};
use anyhow::Context;
use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, SqlMigrationLoader, WrappedUuidV4, decode_pagination_token},
};
use std::collections::BTreeMap;
use shared_macros::load_sql_migrations;

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
            data_encryption_key_id: &params.data_encryption_key_id,
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
            data_encryption_key_id: &params.data_encryption_key_id,
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

    async fn create_provider_instance(
        &self,
        params: &CreateProviderInstance,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_provider_instance_params {
            id: &params.id,
            display_name: &params.display_name,
            resource_server_credential_id: &params.resource_server_credential_id,
            user_credential_id: &params.user_credential_id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            provider_controller_type_id: &params.provider_controller_type_id,
            credential_controller_type_id: &params.credential_controller_type_id,
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
    ) -> Result<Option<ProviderInstanceSerialized>, CommonError> {
        let sqlc_params = get_provider_instance_by_id_params { id: &id.to_string() };

        let result = get_provider_instance_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get provider instance by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_provider_instance(
        &self,
        id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_provider_instance_params { id: &id.to_string() };

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
            id: &params.id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            provider_instance_id: &params.provider_instance_id,
            function_controller_type_id: &params.function_controller_type_id,
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
        id: &str,
    ) -> Result<Option<FunctionInstanceSerialized>, CommonError> {
        let sqlc_params = get_function_instance_by_id_params { id: &id.to_string() };

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
        id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_function_instance_params { id: &id.to_string() };

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
        id: &str,
    ) -> Result<Option<FunctionInstanceSerializedWithCredentials>, CommonError> {
        let sqlc_params = get_function_instance_with_credentials_params { id: &id.to_string() };

        let result = get_function_instance_with_credentials(&self.conn, sqlc_params)
            .await
            .context("Failed to get function instance with credentials")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn create_broker_state(
        &self,
        params: &CreateBrokerState,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_broker_state_params {
            id: &params.id,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
            resource_server_cred_id: &params.resource_server_cred_id,
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

    async fn get_broker_state_by_id(
        &self,
        id: &str,
    ) -> Result<Option<BrokerState>, CommonError> {
        let sqlc_params = get_broker_state_by_id_params { id: &id.to_string() };

        let result = get_broker_state_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get broker state by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_broker_state(
        &self,
        id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_broker_state_params { id: &id.to_string() };

        delete_broker_state(&self.conn, sqlc_params)
            .await
            .context("Failed to delete broker state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn create_data_encryption_key(
        &self,
        params: &CreateDataEncryptionKey,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_data_encryption_key_params {
            id: &params.id,
            envelope_encryption_key_id: &params.envelope_encryption_key_id,
            encryption_key: &params.encryption_key,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_data_encryption_key(&self.conn, sqlc_params)
            .await
            .context("Failed to create data encryption key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_data_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError> {
        let sqlc_params = get_data_encryption_key_by_id_params { id: &id.to_string() };

        let result = get_data_encryption_key_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get data encryption key by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn delete_data_encryption_key(
        &self,
        id: &str,
    ) -> Result<(), CommonError> {
        let sqlc_params = delete_data_encryption_key_params { id: &id.to_string() };

        delete_data_encryption_key(&self.conn, sqlc_params)
            .await
            .context("Failed to delete data encryption key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_data_encryption_keys(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<DataEncryptionKeyListItem>, CommonError> {
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

        let sqlc_params = get_data_encryption_keys_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_data_encryption_keys(&self.conn, sqlc_params)
            .await
            .context("Failed to get data encryption keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let items: Vec<DataEncryptionKeyListItem> = rows
            .into_iter()
            .map(|row| DataEncryptionKeyListItem {
                id: row.id,
                envelope_encryption_key_id: row.envelope_encryption_key_id,
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
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_sql_migrations!("migrations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::{
        BrokerAction, BrokerState, DataEncryptionKey, EncryptedDataEncryptionKey,
        EnvelopeEncryptionKeyId, Metadata, ResourceServerCredentialSerialized,
        UserCredentialSerialized, ProviderInstanceSerialized, FunctionInstanceSerialized,
    };
    use crate::repository::{
        CreateBrokerState, CreateDataEncryptionKey, CreateFunctionInstance, CreateProviderInstance,
        CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
    };
    use shared::primitives::{
        SqlMigrationLoader, WrappedChronoDateTime, WrappedUuidV4, WrappedJsonValue,
    };
    use shared::test_utils::repository::setup_in_memory_database;

    async fn create_test_dek(repo: &Repository, now: WrappedChronoDateTime) -> String {
        let dek_id = uuid::Uuid::new_v4().to_string();
        let dek = DataEncryptionKey {
            id: dek_id.clone(),
            envelope_encryption_key_id: EnvelopeEncryptionKeyId::AwsKms {
                arn: "arn:aws:kms:us-east-1:123456789012:key/test-key".to_string(),
            },
            encrypted_data_encryption_key: EncryptedDataEncryptionKey("test_encrypted_key".to_string()),
            created_at: now,
            updated_at: now,
        };
        repo.create_data_encryption_key(&CreateDataEncryptionKey::from(dek)).await.unwrap();
        dek_id
    }

    #[tokio::test]
    async fn test_create_and_get_resource_server_credential() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_id = create_test_dek(&repo, now).await;

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
            data_encryption_key_id: dek_id,
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
        let dek_id = create_test_dek(&repo, now).await;

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
            data_encryption_key_id: dek_id,
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
        let dek_id = create_test_dek(&repo, now).await;

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(resource_server_cred.clone()))
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
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: user_cred.id.clone(),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
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

        assert_eq!(retrieved.id, provider_instance.id);
        assert_eq!(retrieved.display_name, provider_instance.display_name);
        assert_eq!(retrieved.provider_controller_type_id, provider_instance.provider_controller_type_id);
    }

    #[tokio::test]
    async fn test_delete_provider_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_id = create_test_dek(&repo, now).await;

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(resource_server_cred.clone()))
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
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Delete".to_string(),
            resource_server_credential_id: resource_server_cred.id.clone(),
            user_credential_id: user_cred.id.clone(),
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
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
        let dek_id = create_test_dek(&repo, now).await;

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(resource_server_cred.clone()))
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
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Cascade".to_string(),
            resource_server_credential_id: resource_server_cred.id,
            user_credential_id: user_cred.id,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Create a function instance that depends on the provider instance
        let function_instance = FunctionInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            provider_instance_id: provider_instance.id.clone(),
            function_controller_type_id: "send_email".to_string(),
        };
        repo.create_function_instance(&CreateFunctionInstance::from(function_instance.clone()))
            .await
            .unwrap();

        // Verify function instance was created
        let retrieved_function = repo
            .get_function_instance_by_id(&function_instance.id)
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
            .get_function_instance_by_id(&function_instance.id)
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
        let dek_id = create_test_dek(&repo, now).await;

        // Setup credentials and provider instance
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(resource_server_cred.clone()))
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
            data_encryption_key_id: dek_id.clone(),
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test Provider Function".to_string(),
            resource_server_credential_id: resource_server_cred.id,
            user_credential_id: user_cred.id,
            created_at: now,
            updated_at: now,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
        };
        repo.create_provider_instance(&CreateProviderInstance::from(provider_instance.clone()))
            .await
            .unwrap();

        // Create function instance
        let function_instance = FunctionInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            provider_instance_id: provider_instance.id.clone(),
            function_controller_type_id: "send_email".to_string(),
        };

        repo.create_function_instance(&CreateFunctionInstance::from(function_instance.clone()))
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_function_instance_by_id(&function_instance.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, function_instance.id);
        assert_eq!(retrieved.function_controller_type_id, function_instance.function_controller_type_id);

        // Delete the function instance
        repo.delete_function_instance(&function_instance.id)
            .await
            .unwrap();

        // Verify it was deleted
        let deleted = repo
            .get_function_instance_by_id(&function_instance.id)
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
        let dek_id = create_test_dek(&repo, now).await;

        // Create resource server credential for broker state
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            data_encryption_key_id: dek_id,
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(resource_server_cred.clone()))
            .await
            .unwrap();

        let broker_state = BrokerState {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            resource_server_cred_id: resource_server_cred.id,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            action: BrokerAction::Redirect {
                url: "https://example.com/oauth/authorize".to_string(),
            },
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
        assert_eq!(retrieved.provider_controller_type_id, broker_state.provider_controller_type_id);
        match retrieved.action {
            BrokerAction::Redirect { url } => assert_eq!(url, "https://example.com/oauth/authorize"),
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
        let dek_id = create_test_dek(&repo, now).await;

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
            data_encryption_key_id: dek_id,
        };
        repo.create_resource_server_credential(&CreateResourceServerCredential::from(resource_server_cred.clone()))
            .await
            .unwrap();

        let broker_state = BrokerState {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            resource_server_cred_id: resource_server_cred.id,
            provider_controller_type_id: "google_mail".to_string(),
            credential_controller_type_id: "no_auth".to_string(),
            metadata: Metadata::new(),
            action: BrokerAction::None,
        };

        repo.create_broker_state(&CreateBrokerState::from(broker_state.clone()))
            .await
            .unwrap();

        // Delete the broker state
        repo.delete_broker_state(&broker_state.id)
            .await
            .unwrap();

        // Verify it was deleted
        let deleted = repo
            .get_broker_state_by_id(&broker_state.id)
            .await
            .unwrap();

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
            .get_function_instance_by_id(&uuid::Uuid::new_v4().to_string())
            .await
            .unwrap();
        assert!(result.is_none());

        // Test getting nonexistent broker state
        let result = repo
            .get_broker_state_by_id(&uuid::Uuid::new_v4().to_string())
            .await
            .unwrap();
        assert!(result.is_none());

        // Test getting nonexistent data encryption key
        let result = repo
            .get_data_encryption_key_by_id(&uuid::Uuid::new_v4().to_string())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_and_get_data_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: crate::logic::EnvelopeEncryptionKeyId::AwsKms {
                arn: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012".to_string(),
            },
            encrypted_data_encryption_key: crate::logic::EncryptedDataEncryptionKey("encrypted_key_data".to_string()),
            created_at: now,
            updated_at: now,
        };

        let create_params = CreateDataEncryptionKey::from(dek.clone());
        repo.create_data_encryption_key(&create_params)
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_data_encryption_key_by_id(&dek.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, dek.id);
        assert_eq!(retrieved.encrypted_data_encryption_key.0, dek.encrypted_data_encryption_key.0);
        match retrieved.envelope_encryption_key_id {
            crate::logic::EnvelopeEncryptionKeyId::AwsKms { arn } => {
                assert_eq!(arn, "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012");
            }
            crate::logic::EnvelopeEncryptionKeyId::Local { .. } => {
                panic!("Expected AwsKms variant");
            }
        }
    }

    #[tokio::test]
    async fn test_delete_data_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: crate::logic::EnvelopeEncryptionKeyId::AwsKms {
                arn: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012".to_string(),
            },
            encrypted_data_encryption_key: crate::logic::EncryptedDataEncryptionKey("encrypted_key_data".to_string()),
            created_at: now,
            updated_at: now,
        };

        let create_params = CreateDataEncryptionKey::from(dek.clone());
        repo.create_data_encryption_key(&create_params)
            .await
            .unwrap();

        // Verify it was created
        let retrieved = repo
            .get_data_encryption_key_by_id(&dek.id)
            .await
            .unwrap();
        assert!(retrieved.is_some());

        // Delete the data encryption key
        repo.delete_data_encryption_key(&dek.id)
            .await
            .unwrap();

        // Verify it was deleted
        let deleted = repo
            .get_data_encryption_key_by_id(&dek.id)
            .await
            .unwrap();

        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys_empty() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = repo.list_data_encryption_keys(&pagination)
            .await
            .unwrap();

        assert_eq!(result.items.len(), 0);
        assert!(result.next_page_token.is_none());
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys_single_page() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create 3 data encryption keys
        let now = WrappedChronoDateTime::now();
        let mut deks = vec![];

        for i in 0..3 {
            let dek = DataEncryptionKey {
                id: format!("dek-{}", i),
                envelope_encryption_key_id: crate::logic::EnvelopeEncryptionKeyId::AwsKms {
                    arn: format!("arn:aws:kms:us-east-1:123456789012:key/key-{}", i),
                },
                encrypted_data_encryption_key: crate::logic::EncryptedDataEncryptionKey(format!("encrypted_key_{}", i)),
                created_at: now,
                updated_at: now,
            };
            deks.push(dek.clone());
            repo.create_data_encryption_key(&CreateDataEncryptionKey::from(dek))
                .await
                .unwrap();

            // Sleep briefly to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = repo.list_data_encryption_keys(&pagination)
            .await
            .unwrap();

        assert_eq!(result.items.len(), 3);
        assert!(result.next_page_token.is_none());

        // Verify that the encrypted_data_encryption_key is not returned (check struct fields)
        for item in &result.items {
            assert!(item.id.starts_with("dek-"));
            // The DataEncryptionKeyListItem struct doesn't have encrypted_data_encryption_key field
        }
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create 5 data encryption keys with different timestamps
        let mut deks = vec![];

        for i in 0..5 {
            // Sleep briefly BEFORE each creation to ensure different timestamps
            if i > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            let now = WrappedChronoDateTime::now();
            let dek = DataEncryptionKey {
                id: format!("dek-{}", i),
                envelope_encryption_key_id: crate::logic::EnvelopeEncryptionKeyId::AwsKms {
                    arn: format!("arn:aws:kms:us-east-1:123456789012:key/key-{}", i),
                },
                encrypted_data_encryption_key: crate::logic::EncryptedDataEncryptionKey(format!("encrypted_key_{}", i)),
                created_at: now,
                updated_at: now,
            };
            deks.push(dek.clone());
            repo.create_data_encryption_key(&CreateDataEncryptionKey::from(dek))
                .await
                .unwrap();
        }

        // First page with page size of 2
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };

        let first_page = repo.list_data_encryption_keys(&pagination)
            .await
            .unwrap();

        assert_eq!(first_page.items.len(), 2);
        assert!(first_page.next_page_token.is_some());

        // Second page using the token from first page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: first_page.next_page_token.clone(),
        };

        let second_page = repo.list_data_encryption_keys(&pagination)
            .await
            .unwrap();

        assert_eq!(second_page.items.len(), 2);
        assert!(second_page.next_page_token.is_some());

        // Third page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: second_page.next_page_token.clone(),
        };

        let third_page = repo.list_data_encryption_keys(&pagination)
            .await
            .unwrap();

        assert_eq!(third_page.items.len(), 1);
        assert!(third_page.next_page_token.is_none());

        // Verify no duplicates across pages
        let mut all_ids = vec![];
        all_ids.extend(first_page.items.iter().map(|i| i.id.clone()));
        all_ids.extend(second_page.items.iter().map(|i| i.id.clone()));
        all_ids.extend(third_page.items.iter().map(|i| i.id.clone()));

        let mut unique_ids = all_ids.clone();
        unique_ids.sort();
        unique_ids.dedup();

        assert_eq!(all_ids.len(), unique_ids.len());
        assert_eq!(unique_ids.len(), 5);
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys_does_not_include_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek = DataEncryptionKey {
            id: uuid::Uuid::new_v4().to_string(),
            envelope_encryption_key_id: crate::logic::EnvelopeEncryptionKeyId::AwsKms {
                arn: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012".to_string(),
            },
            encrypted_data_encryption_key: crate::logic::EncryptedDataEncryptionKey("super_secret_encrypted_key_data".to_string()),
            created_at: now,
            updated_at: now,
        };

        let create_params = CreateDataEncryptionKey::from(dek.clone());
        repo.create_data_encryption_key(&create_params)
            .await
            .unwrap();

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };

        let result = repo.list_data_encryption_keys(&pagination)
            .await
            .unwrap();

        assert_eq!(result.items.len(), 1);
        let list_item = &result.items[0];

        // Verify the fields that ARE present
        assert_eq!(list_item.id, dek.id);
        assert_eq!(list_item.created_at, dek.created_at);
        assert_eq!(list_item.updated_at, dek.updated_at);

        // The DataEncryptionKeyListItem type doesn't have the encrypted_data_encryption_key field,
        // so we can't accidentally return it. This is verified at compile time.

        // To verify the sensitive data isn't there, we can check by getting the full object
        // and ensuring the list item doesn't contain the encrypted key
        let full_dek = repo.get_data_encryption_key_by_id(&dek.id)
            .await
            .unwrap()
            .unwrap();

        // The full DEK has the encrypted key
        assert_eq!(full_dek.encrypted_data_encryption_key.0, "super_secret_encrypted_key_data");

        // But the list item type doesn't have this field at all
        // (compile-time safety through type system)
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys_invalid_pagination_token() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: Some("invalid_token".to_string()),
        };

        let result = repo.list_data_encryption_keys(&pagination)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CommonError::Repository { msg, .. } => {
                assert!(msg.contains("Invalid pagination token"));
            }
            _ => panic!("Expected Repository error"),
        }
    }
}
