#![allow(non_camel_case_types)]
mod raw_impl;

include!("raw.generated.rs");

use crate::logic::{
    BrokerState, DataEncryptionKey, FunctionInstanceSerialized, FunctionInstanceSerializedWithCredentials,
    ProviderInstanceSerialized, ResourceServerCredentialSerialized, UserCredentialSerialized,
};
use crate::repository::{
    CreateBrokerState, CreateDataEncryptionKey, CreateFunctionInstance, CreateProviderInstance,
    CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
};
use anyhow::Context;
use shared::{error::CommonError, primitives::{SqlMigrationLoader, WrappedUuidV4}};
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
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        let mut all_migrations = BTreeMap::new();
        let mut sqlite_migrations = BTreeMap::new();

        // 0_init migration
        sqlite_migrations.insert(
            "0_init.up.sql",
            include_str!("../../../migrations/0_init.up.sql"),
        );
        sqlite_migrations.insert(
            "0_init.down.sql",
            include_str!("../../../migrations/0_init.down.sql"),
        );

        all_migrations.insert("sqlite", sqlite_migrations);

        all_migrations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::{
        BrokerAction, BrokerState, Metadata, ResourceServerCredentialSerialized,
        UserCredentialSerialized, ProviderInstanceSerialized, FunctionInstanceSerialized,
    };
    use crate::repository::{
        CreateBrokerState, CreateFunctionInstance, CreateProviderInstance,
        CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
    };
    use shared::primitives::{
        SqlMigrationLoader, WrappedChronoDateTime, WrappedUuidV4, WrappedJsonValue,
    };
    use shared::test_utils::repository::setup_in_memory_database;

    #[tokio::test]
    async fn test_create_and_get_resource_server_credential() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
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

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
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
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        // Create provider instance
        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
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
        assert_eq!(retrieved.provider_controller_type_id, provider_instance.provider_controller_type_id);
    }

    #[tokio::test]
    async fn test_create_get_and_delete_function_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();

        // Setup credentials and provider instance
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
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
        };
        repo.create_user_credential(&CreateUserCredential::from(user_cred.clone()))
            .await
            .unwrap();

        let provider_instance = ProviderInstanceSerialized {
            id: uuid::Uuid::new_v4().to_string(),
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

        // Create resource server credential for broker state
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_oauth2_authorization_code_flow".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
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

        // Create resource server credential
        let resource_server_cred = ResourceServerCredentialSerialized {
            id: WrappedUuidV4::new(),
            type_id: "resource_server_no_auth".to_string(),
            metadata: Metadata::new(),
            value: WrappedJsonValue::new(serde_json::json!({})),
            created_at: now,
            updated_at: now,
            next_rotation_time: None,
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
            encryption_key: crate::logic::EncryptedDataKey("encrypted_key_data".to_string()),
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
        assert_eq!(retrieved.encryption_key.0, dek.encryption_key.0);
        match retrieved.envelope_encryption_key_id {
            crate::logic::EnvelopeEncryptionKeyId::AwsKms { arn } => {
                assert_eq!(arn, "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012");
            }
        }
    }
}
