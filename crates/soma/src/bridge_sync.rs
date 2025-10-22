use bridge::logic::{
    create_data_encryption_key, create_provider_instance, create_resource_server_credential,
    create_user_credential, delete_data_encryption_key, delete_provider_instance,
    disable_function, enable_function, list_data_encryption_keys, list_function_instances,
    list_provider_instances, CreateDataEncryptionKeyParams, CreateProviderInstanceParamsInner,
    CreateResourceServerCredentialParams, CreateResourceServerCredentialParamsInner,
    CreateUserCredentialParams, CreateUserCredentialParamsInner, DisableFunctionParamsInner,
    EnableFunctionParamsInner, EncryptedDataEncryptionKey, EnvelopeEncryptionKeyContents,
    Metadata, OnConfigChangeTx, WithCredentialControllerTypeId, WithFunctionControllerTypeId,
    WithProviderControllerTypeId, WithProviderInstanceId,
};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedJsonValue},
    soma_agent_definition::SomaAgentDefinition,
};

/// Synchronizes the bridge database with the soma.yaml definition.
///
/// This function:
/// 1. Deletes all existing function instances
/// 2. Deletes all existing provider instances
/// 3. Deletes all existing user credentials
/// 4. Deletes all existing resource server credentials
/// 5. Deletes all existing data encryption keys
/// 6. Creates data encryption keys from the soma definition
/// 7. Creates provider instances from the soma definition
/// 8. Creates function instances for each provider
///
/// All operations are performed with `publish_on_change_evt: false` to prevent
/// circular updates back to the soma.yaml file during sync.
pub async fn sync_bridge(
    key_encryption_key: &EnvelopeEncryptionKeyContents,
    on_config_change_tx: &OnConfigChangeTx,
    repo: &impl bridge::repository::ProviderRepositoryLike,
    soma_definition: &SomaAgentDefinition,
) -> Result<(), CommonError> {
    // 1. Delete all function instances
    let function_instances_to_delete = {
        let mut instances = Vec::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = list_function_instances(
                repo,
                bridge::logic::ListFunctionInstancesParams {
                    pagination,
                    provider_instance_id: None,
                },
            )
            .await?;
            instances.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }
        instances
    };

    for item in function_instances_to_delete {
        disable_function(
            on_config_change_tx,
            repo,
            WithProviderInstanceId {
                provider_instance_id: item.provider_instance_id.clone(),
                inner: WithFunctionControllerTypeId {
                    function_controller_type_id: item.function_controller_type_id.clone(),
                    inner: DisableFunctionParamsInner {},
                },
            },
            false,
        )
        .await?;
    }

    // 2. Delete all provider instances
    let provider_instances_to_delete = {
        let mut instances = Vec::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = list_provider_instances(
                repo,
                bridge::logic::ListProviderInstancesParams {
                    pagination,
                    status: None,
                    provider_controller_type_id: None
                },
            )
            .await?;
            instances.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }
        instances
    };

    for item in provider_instances_to_delete {
        delete_provider_instance(
            on_config_change_tx,
            repo,
            WithProviderInstanceId {
                provider_instance_id: item.provider_instance.id.clone(),
                inner: (),
            },
            false,
        )
        .await?;
    }

    // 3. Delete all user credentials
    let user_credentials_to_delete = {
        let mut credentials = Vec::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = repo.list_user_credentials(&pagination).await?;
            credentials.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }
        credentials
    };

    for item in user_credentials_to_delete {
        repo.delete_user_credential(&item.id).await?;
    }

    // 4. Delete all resource server credentials
    let resource_server_credentials_to_delete = {
        let mut credentials = Vec::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = repo.list_resource_server_credentials(&pagination).await?;
            credentials.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }
        credentials
    };

    for item in resource_server_credentials_to_delete {
        repo.delete_resource_server_credential(&item.id).await?;
    }

    // 5. Delete all data encryption keys
    let data_encryption_keys_to_delete = {
        let mut keys = Vec::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = list_data_encryption_keys(repo, pagination).await?;
            keys.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }
        keys
    };

    for item in data_encryption_keys_to_delete {
        delete_data_encryption_key(on_config_change_tx, repo, item.id.clone(), false).await?;
    }

    // 6. Create encryption keys from soma definition
    if let Some(bridge_config) = &soma_definition.bridge {
        for (key_id, encryption_config) in &bridge_config.encryption.0 {
            create_data_encryption_key(
                key_encryption_key,
                on_config_change_tx,
                repo,
                CreateDataEncryptionKeyParams {
                    id: Some(key_id.clone()),
                    encrypted_data_envelope_key: Some(EncryptedDataEncryptionKey(
                        encryption_config.encrypted_data_encryption_key.clone(),
                    )),
                },
                false,
            )
            .await?;
        }

        // 7. Create provider instances from soma definition
        if let Some(providers) = &bridge_config.providers {
            for (provider_id, provider_config) in providers {
                let provider_controller_type_id = &provider_config.provider_controller_type_id;
                let credential_controller_type_id = &provider_config.credential_controller_type_id;

                tracing::info!(
                    "Syncing provider '{}' with controller type_id: '{}', credential type_id: '{}'",
                    provider_id,
                    provider_controller_type_id,
                    credential_controller_type_id
                );

                // Create resource server credential
                let resource_server_credential = create_resource_server_credential(
                    repo,
                    CreateResourceServerCredentialParams {
                        provider_controller_type_id: provider_controller_type_id.clone(),
                        inner: WithCredentialControllerTypeId {
                            credential_controller_type_id: credential_controller_type_id.clone(),
                            inner: CreateResourceServerCredentialParamsInner {
                                data_encryption_key_id: provider_config
                                    .resource_server_credential
                                    .data_encryption_key_id
                                    .clone(),
                                resource_server_configuration: WrappedJsonValue::new(
                                    provider_config.resource_server_credential.value.clone(),
                                ),
                                metadata: provider_config
                                    .resource_server_credential
                                    .metadata
                                    .as_object()
                                    .map(|m| Metadata(m.clone())),
                            },
                        },
                    },
                )
                .await?;

                // Create user credential if provided
                let user_credential =
                    if let Some(user_cred_config) = &provider_config.user_credential {
                        Some(
                            create_user_credential(
                                repo,
                                CreateUserCredentialParams {
                                    provider_controller_type_id: provider_controller_type_id
                                        .clone(),
                                    inner: WithCredentialControllerTypeId {
                                        credential_controller_type_id:
                                            credential_controller_type_id.clone(),
                                        inner: CreateUserCredentialParamsInner {
                                            data_encryption_key_id: user_cred_config
                                                .data_encryption_key_id
                                                .clone(),
                                            user_credential_configuration: WrappedJsonValue::new(
                                                user_cred_config.value.clone(),
                                            ),
                                            metadata: user_cred_config
                                                .metadata
                                                .as_object()
                                                .map(|m| Metadata(m.clone())),
                                        },
                                    },
                                },
                            )
                            .await?,
                        )
                    } else {
                        None
                    };

                // Create provider instance
                create_provider_instance(
                    on_config_change_tx,
                    repo,
                    WithProviderControllerTypeId {
                        provider_controller_type_id: provider_controller_type_id.clone(),
                        inner: WithCredentialControllerTypeId {
                            credential_controller_type_id: credential_controller_type_id.clone(),
                            inner: CreateProviderInstanceParamsInner {
                                provider_instance_id: Some(provider_id.clone()),
                                display_name: provider_config.display_name.clone(),
                                resource_server_credential_id: resource_server_credential.id,
                                user_credential_id: user_credential
                                    .as_ref()
                                    .map(|uc| uc.id.clone()),
                                return_on_successful_brokering: None,
                            },
                        },
                    },
                    false,
                )
                .await?;

                // 8. Create function instances for each provider
                if let Some(functions) = &provider_config.functions {
                    for function_controller_type_id in functions.iter() {
                        enable_function(
                            on_config_change_tx,
                            repo,
                            WithProviderInstanceId {
                                provider_instance_id: provider_id.clone(),
                                inner: WithFunctionControllerTypeId {
                                    function_controller_type_id: function_controller_type_id.clone(),
                                    inner: EnableFunctionParamsInner {},
                                },
                            },
                            false,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge::logic::EnvelopeEncryptionKeyId;
    use shared::primitives::SqlMigrationLoader;

    #[tokio::test]
    async fn test_sync_bridge_empty_database() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let repo = bridge::repository::Repository::new(conn);
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        // Create empty soma definition
        let soma_def = SomaAgentDefinition {
            project: "test-project".to_string(),
            agent: "test-agent".to_string(),
            description: "Test agent".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bridge: None,
        };

        let result = sync_bridge(&kek, &tx, &repo, &soma_def).await;
        assert!(result.is_ok());

        // Verify database is still empty
        let dek_list = list_data_encryption_keys(
            &repo,
            PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(dek_list.items.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_bridge_creates_encryption_keys() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let repo = bridge::repository::Repository::new(conn);
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        // Create soma definition with encryption keys
        let mut encryption_map = std::collections::HashMap::new();
        encryption_map.insert(
            "key1".to_string(),
            shared::soma_agent_definition::EncryptionConfiguration {
                encrypted_data_encryption_key: "encrypted-key-1".to_string(),
                envelope_encryption_key_id: shared::soma_agent_definition::EnvelopeEncryptionKeyId::Local {
                    key_id: "test-key".to_string(),
                },
            },
        );

        let soma_def = SomaAgentDefinition {
            project: "test-project".to_string(),
            agent: "test-agent".to_string(),
            description: "Test agent".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bridge: Some(shared::soma_agent_definition::BridgeConfig {
                encryption: shared::soma_agent_definition::BridgeEncryptionConfig(encryption_map),
                providers: None,
            }),
        };

        let result = sync_bridge(&kek, &tx, &repo, &soma_def).await;
        assert!(result.is_ok());

        // Verify encryption key was created
        let dek_list = list_data_encryption_keys(
            &repo,
            PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(dek_list.items.len(), 1);
        assert_eq!(dek_list.items[0].id, "key1");
    }

    #[tokio::test]
    async fn test_sync_bridge_deletes_existing_keys() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let repo = bridge::repository::Repository::new(conn);
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        // Create some existing DEKs
        create_data_encryption_key(
            &kek,
            &tx,
            &repo,
            CreateDataEncryptionKeyParams {
                id: Some("old-key-1".to_string()),
                encrypted_data_envelope_key: None,
            },
            false,
        )
        .await
        .unwrap();

        create_data_encryption_key(
            &kek,
            &tx,
            &repo,
            CreateDataEncryptionKeyParams {
                id: Some("old-key-2".to_string()),
                encrypted_data_envelope_key: None,
            },
            false,
        )
        .await
        .unwrap();

        // Sync with empty definition
        let soma_def = SomaAgentDefinition {
            project: "test-project".to_string(),
            agent: "test-agent".to_string(),
            description: "Test agent".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bridge: Some(shared::soma_agent_definition::BridgeConfig {
                encryption: shared::soma_agent_definition::BridgeEncryptionConfig(
                    std::collections::HashMap::new(),
                ),
                providers: None,
            }),
        };

        let result = sync_bridge(&kek, &tx, &repo, &soma_def).await;
        assert!(result.is_ok());

        // Verify all old keys were deleted
        let dek_list = list_data_encryption_keys(
            &repo,
            PaginationRequest {
                page_size: 10,
                next_page_token: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(dek_list.items.len(), 0);
    }
}
