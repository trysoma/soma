#![allow(non_camel_case_types)]
mod raw_impl;

include!("raw.generated.rs");

use crate::repository::{
    CreateCredentialExchangeState, CreateFunctionInstance, CreateProviderInstance,
    CreateResourceServerCredential, CreateUserCredential, CredentialExchangeState,
    ProviderRepositoryLike,
};
use anyhow::Context;
use shared::{error::CommonError, primitives::SqlMigrationLoader};
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
            credential_type: &params.credential_type,
            credential_data: &params.credential_data,
            metadata: &params.metadata,
            run_refresh_before: &params.run_refresh_before,
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

    async fn create_user_credential(
        &self,
        params: &CreateUserCredential,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_user_credential_params {
            id: &params.id,
            credential_type: &params.credential_type,
            credential_data: &params.credential_data,
            metadata: &params.metadata,
            run_refresh_before: &params.run_refresh_before,
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

    async fn create_provider_instance(
        &self,
        params: &CreateProviderInstance,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_provider_instance_params {
            id: &params.id,
            provider_id: &params.provider_id,
            resource_server_credential_id: &params.resource_server_credential_id,
            user_credential_id: &params.user_credential_id,
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

    async fn create_function_instance(
        &self,
        params: &CreateFunctionInstance,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_function_instance_params {
            id: &params.id,
            function_id: &params.function_id,
            provider_instance_id: &params.provider_instance_id,
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

    async fn create_credential_exchange_state(
        &self,
        params: &CreateCredentialExchangeState,
    ) -> Result<(), CommonError> {
        let sqlc_params = create_credential_exchange_state_params {
            id: &params.id,
            state: &params.state,
        };

        create_credential_exchange_state(&self.conn, sqlc_params)
            .await
            .context("Failed to create credential exchange state")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_credential_exchange_state_by_id(
        &self,
        id: &str,
    ) -> Result<Option<CredentialExchangeState>, CommonError> {
        let sqlc_params = get_credential_exchange_state_by_id_params { id: &id.to_string() };

        let result = get_credential_exchange_state_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get credential exchange state by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| CredentialExchangeState {
            id: row.id,
            state: row.state,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
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
        Metadata, NoAuthResourceServerCredential, NoAuthUserCredential,
        Oauth2AuthorizationCodeFlowResourceServerCredential,
        Oauth2AuthorizationCodeFlowUserCredential, ResourceServerCredential,
        ResourceServerCredentialVariant, UserCredential, UserCredentialVariant,
    };
    use crate::repository::{
        CreateCredentialExchangeState, CreateFunctionInstance, CreateProviderInstance,
        CreateResourceServerCredential, CreateUserCredential, ProviderRepositoryLike,
    };
    use shared::primitives::{
        SqlMigrationLoader, WrappedChronoDateTime, WrappedUuidV4,
    };
    use shared::test_utils::repository::setup_in_memory_database;

    #[tokio::test]
    async fn test_create_resource_server_credential_no_auth() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let credential_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let inner = ResourceServerCredentialVariant::NoAuth(NoAuthResourceServerCredential {
            metadata: metadata.clone(),
        });

        let credential = ResourceServerCredential {
            id: credential_id.clone(),
            inner: inner.clone(),
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_params = CreateResourceServerCredential::try_from(credential).unwrap();
        repo.create_resource_server_credential(&create_params)
            .await
            .unwrap();

        // Verify it was created (would need a get query to fully verify)
        // For now, just ensuring no error is thrown
    }

    #[tokio::test]
    async fn test_create_resource_server_credential_oauth2() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let credential_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let inner = ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(
            Oauth2AuthorizationCodeFlowResourceServerCredential {
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
                redirect_uri: "https://example.com/callback".to_string(),
                metadata: metadata.clone(),
            },
        );

        let credential = ResourceServerCredential {
            id: credential_id.clone(),
            inner: inner.clone(),
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_params = CreateResourceServerCredential::try_from(credential).unwrap();
        repo.create_resource_server_credential(&create_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_user_credential_no_auth() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let credential_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let inner = UserCredentialVariant::NoAuth(NoAuthUserCredential {
            metadata: metadata.clone(),
        });

        let credential = UserCredential {
            id: credential_id.clone(),
            inner: inner.clone(),
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_params = CreateUserCredential::try_from(credential).unwrap();
        repo.create_user_credential(&create_params).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_user_credential_oauth2() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let credential_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let inner = UserCredentialVariant::Oauth2AuthorizationCodeFlow(
            Oauth2AuthorizationCodeFlowUserCredential {
                code: "test_code".to_string(),
                access_token: "test_access_token".to_string(),
                refresh_token: "test_refresh_token".to_string(),
                expiry_time: WrappedChronoDateTime::now(),
                sub: "test_sub".to_string(),
                metadata: metadata.clone(),
            },
        );

        let credential = UserCredential {
            id: credential_id.clone(),
            inner: inner.clone(),
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_params = CreateUserCredential::try_from(credential).unwrap();
        repo.create_user_credential(&create_params).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_provider_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // First create resource server credential
        let resource_server_cred_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let resource_server_inner =
            ResourceServerCredentialVariant::NoAuth(NoAuthResourceServerCredential {
                metadata: metadata.clone(),
            });

        let resource_server_credential = ResourceServerCredential {
            id: resource_server_cred_id.clone(),
            inner: resource_server_inner,
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_rs_params =
            CreateResourceServerCredential::try_from(resource_server_credential).unwrap();
        repo.create_resource_server_credential(&create_rs_params)
            .await
            .unwrap();

        // Create user credential
        let user_cred_id = WrappedUuidV4::new();
        let user_inner = UserCredentialVariant::NoAuth(NoAuthUserCredential {
            metadata: metadata.clone(),
        });

        let user_credential = UserCredential {
            id: user_cred_id.clone(),
            inner: user_inner,
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_user_params = CreateUserCredential::try_from(user_credential).unwrap();
        repo.create_user_credential(&create_user_params)
            .await
            .unwrap();

        // Create provider instance
        let provider_instance_id = uuid::Uuid::new_v4().to_string();
        let provider_id = "google_mail".to_string();

        let create_provider_params = CreateProviderInstance {
            id: provider_instance_id.clone(),
            provider_id: provider_id.clone(),
            resource_server_credential_id: resource_server_cred_id.clone(),
            user_credential_id: user_cred_id.clone(),
        };

        repo.create_provider_instance(&create_provider_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_multiple_credentials() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Create multiple resource server credentials
        for _i in 0..3 {
            let credential_id = WrappedUuidV4::new();
            let metadata = Metadata::new();
            let inner = ResourceServerCredentialVariant::NoAuth(NoAuthResourceServerCredential {
                metadata: metadata.clone(),
            });

            let credential = ResourceServerCredential {
                id: credential_id,
                inner,
                metadata,
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
                run_refresh_before: None,
            };

            let create_params = CreateResourceServerCredential::try_from(credential).unwrap();
            repo.create_resource_server_credential(&create_params)
                .await
                .unwrap();
        }

        // Create multiple user credentials
        for _i in 0..3 {
            let credential_id = WrappedUuidV4::new();
            let metadata = Metadata::new();
            let inner = UserCredentialVariant::NoAuth(NoAuthUserCredential {
                metadata: metadata.clone(),
            });

            let credential = UserCredential {
                id: credential_id,
                inner,
                metadata,
                created_at: WrappedChronoDateTime::now(),
                updated_at: WrappedChronoDateTime::now(),
                run_refresh_before: None,
            };

            let create_params = CreateUserCredential::try_from(credential).unwrap();
            repo.create_user_credential(&create_params).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_json_serialization_in_credentials() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Test that complex nested structures serialize correctly
        let credential_id = WrappedUuidV4::new();
        let mut metadata = Metadata::new();
        metadata
            .0
            .insert("key1".to_string(), serde_json::json!("value1"));
        metadata.0.insert(
            "nested".to_string(),
            serde_json::json!({"inner_key": "inner_value"}),
        );

        let inner = ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(
            Oauth2AuthorizationCodeFlowResourceServerCredential {
                client_id: "complex_client_id".to_string(),
                client_secret: "complex_secret".to_string(),
                redirect_uri: "https://example.com/redirect".to_string(),
                metadata: metadata.clone(),
            },
        );

        let credential = ResourceServerCredential {
            id: credential_id,
            inner,
            metadata,
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_params = CreateResourceServerCredential::try_from(credential).unwrap();
        repo.create_resource_server_credential(&create_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_credentials_with_run_refresh_before() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Test resource server credential with run_refresh_before
        let credential_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let refresh_time = WrappedChronoDateTime::now();
        let inner = ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(
            Oauth2AuthorizationCodeFlowResourceServerCredential {
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
                redirect_uri: "https://example.com/callback".to_string(),
                metadata: metadata.clone(),
            },
        );

        let credential = ResourceServerCredential {
            id: credential_id.clone(),
            inner,
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: Some(refresh_time),
        };

        let create_params = CreateResourceServerCredential::try_from(credential).unwrap();
        repo.create_resource_server_credential(&create_params)
            .await
            .unwrap();

        // Test user credential with run_refresh_before
        let user_credential_id = WrappedUuidV4::new();
        let user_refresh_time = WrappedChronoDateTime::now();
        let user_inner = UserCredentialVariant::Oauth2AuthorizationCodeFlow(
            Oauth2AuthorizationCodeFlowUserCredential {
                code: "test_code".to_string(),
                access_token: "test_access_token".to_string(),
                refresh_token: "test_refresh_token".to_string(),
                expiry_time: WrappedChronoDateTime::now(),
                sub: "test_sub".to_string(),
                metadata: metadata.clone(),
            },
        );

        let user_credential = UserCredential {
            id: user_credential_id,
            inner: user_inner,
            metadata,
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: Some(user_refresh_time),
        };

        let create_user_params = CreateUserCredential::try_from(user_credential).unwrap();
        repo.create_user_credential(&create_user_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_function_instance() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // First create resource server credential
        let resource_server_cred_id = WrappedUuidV4::new();
        let metadata = Metadata::new();
        let resource_server_inner =
            ResourceServerCredentialVariant::NoAuth(NoAuthResourceServerCredential {
                metadata: metadata.clone(),
            });

        let resource_server_credential = ResourceServerCredential {
            id: resource_server_cred_id.clone(),
            inner: resource_server_inner,
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_rs_params =
            CreateResourceServerCredential::try_from(resource_server_credential).unwrap();
        repo.create_resource_server_credential(&create_rs_params)
            .await
            .unwrap();

        // Create user credential
        let user_cred_id = WrappedUuidV4::new();
        let user_inner = UserCredentialVariant::NoAuth(NoAuthUserCredential {
            metadata: metadata.clone(),
        });

        let user_credential = UserCredential {
            id: user_cred_id.clone(),
            inner: user_inner,
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        let create_user_params = CreateUserCredential::try_from(user_credential).unwrap();
        repo.create_user_credential(&create_user_params)
            .await
            .unwrap();

        // Create provider instance
        let provider_instance_id = uuid::Uuid::new_v4().to_string();
        let provider_id = "google_mail".to_string();

        let create_provider_params = CreateProviderInstance {
            id: provider_instance_id.clone(),
            provider_id: provider_id.clone(),
            resource_server_credential_id: resource_server_cred_id.clone(),
            user_credential_id: user_cred_id.clone(),
        };

        repo.create_provider_instance(&create_provider_params)
            .await
            .unwrap();

        // Create function instance
        let function_instance_id = uuid::Uuid::new_v4().to_string();
        let function_id = "send_email".to_string();

        let create_function_params = CreateFunctionInstance {
            id: function_instance_id.clone(),
            function_id: function_id.clone(),
            provider_instance_id: provider_instance_id.clone(),
        };

        repo.create_function_instance(&create_function_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_multiple_function_instances() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // Setup: Create credentials and provider instance
        let resource_server_cred_id = WrappedUuidV4::new();
        let user_cred_id = WrappedUuidV4::new();
        let metadata = Metadata::new();

        let resource_server_credential = ResourceServerCredential {
            id: resource_server_cred_id.clone(),
            inner: ResourceServerCredentialVariant::NoAuth(NoAuthResourceServerCredential {
                metadata: metadata.clone(),
            }),
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        repo.create_resource_server_credential(
            &CreateResourceServerCredential::try_from(resource_server_credential).unwrap(),
        )
        .await
        .unwrap();

        let user_credential = UserCredential {
            id: user_cred_id.clone(),
            inner: UserCredentialVariant::NoAuth(NoAuthUserCredential {
                metadata: metadata.clone(),
            }),
            metadata: metadata.clone(),
            created_at: WrappedChronoDateTime::now(),
            updated_at: WrappedChronoDateTime::now(),
            run_refresh_before: None,
        };

        repo.create_user_credential(&CreateUserCredential::try_from(user_credential).unwrap())
            .await
            .unwrap();

        let provider_instance_id = uuid::Uuid::new_v4().to_string();
        repo.create_provider_instance(&CreateProviderInstance {
            id: provider_instance_id.clone(),
            provider_id: "google_mail".to_string(),
            resource_server_credential_id: resource_server_cred_id,
            user_credential_id: user_cred_id,
        })
        .await
        .unwrap();

        // Create multiple function instances
        let function_ids = vec!["send_email", "read_email", "delete_email"];
        for function_id in function_ids {
            let function_instance_id = uuid::Uuid::new_v4().to_string();
            repo.create_function_instance(&CreateFunctionInstance {
                id: function_instance_id,
                function_id: function_id.to_string(),
                provider_instance_id: provider_instance_id.clone(),
            })
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn test_create_credential_exchange_state() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let state_id = uuid::Uuid::new_v4().to_string();
        let mut state = Metadata::new();
        state
            .0
            .insert("provider_id".to_string(), serde_json::json!("google_mail"));
        state
            .0
            .insert("redirect_uri".to_string(), serde_json::json!("https://example.com/callback"));

        let create_params = CreateCredentialExchangeState {
            id: state_id.clone(),
            state: state.clone(),
        };

        repo.create_credential_exchange_state(&create_params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_credential_exchange_state_by_id() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        // First create a credential exchange state
        let state_id = uuid::Uuid::new_v4().to_string();
        let mut state = Metadata::new();
        state
            .0
            .insert("provider_id".to_string(), serde_json::json!("google_mail"));
        state
            .0
            .insert("user_email".to_string(), serde_json::json!("test@example.com"));
        state
            .0
            .insert("nonce".to_string(), serde_json::json!("random_nonce_value"));

        let create_params = CreateCredentialExchangeState {
            id: state_id.clone(),
            state: state.clone(),
        };

        repo.create_credential_exchange_state(&create_params)
            .await
            .unwrap();

        // Now retrieve it
        let retrieved = repo
            .get_credential_exchange_state_by_id(&state_id)
            .await
            .unwrap();

        assert!(retrieved.is_some());
        let retrieved_state = retrieved.unwrap();
        assert_eq!(retrieved_state.id, state_id);
        assert_eq!(
            retrieved_state.state.0.get("provider_id"),
            Some(&serde_json::json!("google_mail"))
        );
        assert_eq!(
            retrieved_state.state.0.get("user_email"),
            Some(&serde_json::json!("test@example.com"))
        );
        assert_eq!(
            retrieved_state.state.0.get("nonce"),
            Some(&serde_json::json!("random_nonce_value"))
        );
    }

    #[tokio::test]
    async fn test_get_nonexistent_credential_exchange_state() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let nonexistent_id = uuid::Uuid::new_v4().to_string();
        let result = repo
            .get_credential_exchange_state_by_id(&nonexistent_id)
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
