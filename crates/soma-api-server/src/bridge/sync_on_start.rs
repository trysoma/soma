use std::sync::Arc;

use bridge::logic::{
    CreateDataEncryptionKeyParams, CreateProviderInstanceParamsInner,
    CreateResourceServerCredentialParams, CreateResourceServerCredentialParamsInner,
    CreateUserCredentialParams, CreateUserCredentialParamsInner, DisableFunctionParamsInner,
    EnableFunctionParamsInner, EncryptedDataEncryptionKey, EnvelopeEncryptionKeyContents, Metadata,
    OnConfigChangeTx, PROVIDER_REGISTRY, ProviderInstanceSerializedWithFunctions,
    WithCredentialControllerTypeId, WithFunctionControllerTypeId, WithProviderControllerTypeId,
    WithProviderInstanceId, create_data_encryption_key, create_provider_instance,
    create_resource_server_credential, create_user_credential, delete_data_encryption_key,
    delete_provider_instance, disable_function, enable_function, list_data_encryption_keys,
    list_function_instances, register_all_bridge_providers,
};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedJsonValue},
    soma_agent_definition::SomaAgentDefinitionLike,
};
use tracing::info;
/// Synchronizes the bridge database with the soma.yaml definition.
///
/// This function performs a smart sync that:
/// 1. Syncs data encryption keys (adds missing, removes extra)
/// 2. For each provider in soma.yaml:
///    - Checks if provider instance exists in DB
///    - If not, creates it with all credentials
///    - If exists but credentials/config changed, recreates it
///    - If exists and unchanged, preserves it (including runtime fields like return_on_successful_brokering)
/// 3. Removes provider instances not in soma.yaml (only if status is "active")
/// 4. Syncs function instances for each provider
///
/// All operations are performed with `publish_on_change_evt: false` to prevent
/// circular updates back to the soma.yaml file during sync.
pub async fn sync_bridge_db_from_soma_definition_on_start<R>(
    key_encryption_key: &EnvelopeEncryptionKeyContents,
    on_config_change_tx: &OnConfigChangeTx,
    bridge_repo: &R,
    soma_repo: &crate::repository::Repository,
    soma_definition_provider: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError>
where
    R: bridge::repository::ProviderRepositoryLike + bridge::logic::DataEncryptionKeyRepositoryLike,
{
    let soma_definition = soma_definition_provider.get_definition().await?;
    use std::collections::{HashMap, HashSet};

    // Register all bridge providers before syncing
    // register_all_bridge_providers().await?;
    // PROVIDER_REGISTRY
    //     .write()
    //     .unwrap()
    //     .push(Arc::new(SomaProviderController::new(soma_repo.clone())));

    // 1. Sync data encryption keys
    // Get all existing keys
    let existing_keys = {
        let mut keys = HashMap::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = list_data_encryption_keys(bridge_repo, pagination).await?;
            for key in response.items {
                keys.insert(key.id.clone(), key);
            }
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }
        keys
    };

    // Get keys from soma definition
    let yaml_keys: HashMap<String, _> = soma_definition
        .bridge
        .as_ref()
        .map(|b| &b.encryption.0)
        .map(|enc| enc.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    // Create/update keys from yaml (but don't delete yet - wait until after providers are synced)
    for (key_id, encryption_config) in &yaml_keys {
        if !existing_keys.contains_key(key_id) {
            create_data_encryption_key(
                key_encryption_key,
                on_config_change_tx,
                bridge_repo,
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
    }

    // 2. Sync provider instances
    if let Some(bridge_config) = &soma_definition.bridge {
        if let Some(providers) = &bridge_config.providers {
            // Get all existing provider instances with credentials
            let existing_providers: HashMap<String, ProviderInstanceSerializedWithFunctions> = {
                let mut instances = HashMap::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let pagination = PaginationRequest {
                        page_size: 100,
                        next_page_token: next_page_token.clone(),
                    };
                    // Use repository directly to get instances with credentials
                    let response = bridge_repo
                        .list_provider_instances(&pagination, None, None)
                        .await?;
                    for item in response.items {
                        instances.insert(item.provider_instance.id.clone(), item);
                    }
                    if response.next_page_token.is_none() {
                        break;
                    }
                    next_page_token = response.next_page_token;
                }
                instances
            };

            let yaml_provider_ids: HashSet<String> = providers.keys().cloned().collect();

            // Process each provider from yaml
            for (provider_id, provider_config) in providers {
                let provider_controller_type_id = &provider_config.provider_controller_type_id;
                let credential_controller_type_id = &provider_config.credential_controller_type_id;

                tracing::info!(
                    "Syncing provider '{}' with controller type_id: '{}', credential type_id: '{}'",
                    provider_id,
                    provider_controller_type_id,
                    credential_controller_type_id
                );

                // Check if provider exists and if it needs updating
                let needs_recreate = if let Some(existing) = existing_providers.get(provider_id) {
                    // Check if key fields changed
                    let basic_fields_changed = existing
                        .provider_instance
                        .provider_controller_type_id
                        != *provider_controller_type_id
                        || existing.provider_instance.credential_controller_type_id
                            != *credential_controller_type_id
                        || existing.provider_instance.display_name != provider_config.display_name;

                    // Compare resource server credentials (encrypted values, no decryption needed)
                    let resource_cred_changed =
                        existing.resource_server_credential.value.get_inner()
                            != &provider_config.resource_server_credential.value
                            || existing.resource_server_credential.data_encryption_key_id
                                != provider_config
                                    .resource_server_credential
                                    .data_encryption_key_id
                            || serde_json::Value::Object(
                                existing.resource_server_credential.metadata.0.clone(),
                            ) != provider_config.resource_server_credential.metadata
                            || existing.resource_server_credential.type_id
                                != provider_config.resource_server_credential.type_id
                            || existing
                                .resource_server_credential
                                .next_rotation_time
                                .as_ref()
                                .map(|t| t.to_string())
                                != provider_config
                                    .resource_server_credential
                                    .next_rotation_time;

                    // Compare user credentials (if either exists)
                    let user_cred_changed =
                        match (&existing.user_credential, &provider_config.user_credential) {
                            (Some(existing_uc), Some(config_uc)) => {
                                // Both exist, compare them
                                existing_uc.value.get_inner() != &config_uc.value
                                    || existing_uc.data_encryption_key_id
                                        != config_uc.data_encryption_key_id
                                    || serde_json::Value::Object(existing_uc.metadata.0.clone())
                                        != config_uc.metadata
                                    || existing_uc.type_id != config_uc.type_id
                                    || existing_uc
                                        .next_rotation_time
                                        .as_ref()
                                        .map(|t| t.to_string())
                                        != config_uc.next_rotation_time
                            }
                            (None, None) => false, // Both don't exist, no change
                            _ => true,             // One exists but not the other, needs recreate
                        };

                    basic_fields_changed || resource_cred_changed || user_cred_changed
                } else {
                    // Provider doesn't exist, needs creation
                    true
                };

                if needs_recreate {
                    // Delete existing provider if it exists
                    if existing_providers.contains_key(provider_id) {
                        tracing::info!(
                            "Provider '{}' configuration changed, recreating",
                            provider_id
                        );
                        delete_provider_instance(
                            on_config_change_tx,
                            bridge_repo,
                            WithProviderInstanceId {
                                provider_instance_id: provider_id.clone(),
                                inner: (),
                            },
                            false,
                        )
                        .await?;
                    } else {
                        tracing::info!("Creating new provider '{}'", provider_id);
                    }

                    // Create resource server credential
                    let resource_server_credential = create_resource_server_credential(
                        bridge_repo,
                        CreateResourceServerCredentialParams {
                            provider_controller_type_id: provider_controller_type_id.clone(),
                            inner: WithCredentialControllerTypeId {
                                credential_controller_type_id: credential_controller_type_id
                                    .clone(),
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
                    let user_credential = if let Some(user_cred_config) =
                        &provider_config.user_credential
                    {
                        Some(
                            create_user_credential(
                                bridge_repo,
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
                        bridge_repo,
                        WithProviderControllerTypeId {
                            provider_controller_type_id: provider_controller_type_id.clone(),
                            inner: WithCredentialControllerTypeId {
                                credential_controller_type_id: credential_controller_type_id
                                    .clone(),
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
                } else {
                    tracing::info!("Provider '{}' unchanged, preserving", provider_id);
                }

                // Sync function instances for this provider
                let existing_functions = {
                    let mut instances = HashSet::new();
                    let mut next_page_token: Option<String> = None;
                    loop {
                        let pagination = PaginationRequest {
                            page_size: 100,
                            next_page_token: next_page_token.clone(),
                        };
                        let response = list_function_instances(
                            bridge_repo,
                            bridge::logic::ListFunctionInstancesParams {
                                pagination,
                                provider_instance_id: Some(provider_id.clone()),
                            },
                        )
                        .await?;
                        for item in response.items {
                            instances.insert(item.function_controller_type_id);
                        }
                        if response.next_page_token.is_none() {
                            break;
                        }
                        next_page_token = response.next_page_token;
                    }
                    instances
                };

                let yaml_functions: HashSet<String> = provider_config
                    .functions
                    .as_ref()
                    .map(|f| f.iter().cloned().collect())
                    .unwrap_or_default();

                // Disable functions not in yaml
                for function_id in existing_functions.iter() {
                    if !yaml_functions.contains(function_id) {
                        disable_function(
                            on_config_change_tx,
                            bridge_repo,
                            WithProviderInstanceId {
                                provider_instance_id: provider_id.clone(),
                                inner: WithFunctionControllerTypeId {
                                    function_controller_type_id: function_id.clone(),
                                    inner: DisableFunctionParamsInner {},
                                },
                            },
                            false,
                        )
                        .await?;
                    }
                }

                // Enable functions from yaml
                for function_id in yaml_functions.iter() {
                    if !existing_functions.contains(function_id) {
                        enable_function(
                            on_config_change_tx,
                            bridge_repo,
                            WithProviderInstanceId {
                                provider_instance_id: provider_id.clone(),
                                inner: WithFunctionControllerTypeId {
                                    function_controller_type_id: function_id.clone(),
                                    inner: EnableFunctionParamsInner {},
                                },
                            },
                            false,
                        )
                        .await?;
                    }
                }
            }

            // Delete provider instances not in yaml (only if status is "active")
            for (provider_id, existing) in existing_providers.iter() {
                if !yaml_provider_ids.contains(provider_id)
                    && existing.provider_instance.status == "active"
                {
                    tracing::info!(
                        "Deleting provider '{}' not in yaml (status: active)",
                        provider_id
                    );
                    delete_provider_instance(
                        on_config_change_tx,
                        bridge_repo,
                        WithProviderInstanceId {
                            provider_instance_id: provider_id.clone(),
                            inner: (),
                        },
                        false,
                    )
                    .await?;
                }
            }
        }
    }

    // 3. Delete unused data encryption keys (after providers are synced, so credentials are cleaned up)
    // Get all credentials to check which keys are still in use
    let keys_in_use: HashSet<String> = {
        let mut in_use = HashSet::new();

        // Check resource server credentials
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = bridge_repo
                .list_resource_server_credentials(&pagination)
                .await?;
            for cred in response.items {
                in_use.insert(cred.data_encryption_key_id);
            }
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }

        // Check user credentials
        let mut next_page_token: Option<String> = None;
        loop {
            let pagination = PaginationRequest {
                page_size: 100,
                next_page_token: next_page_token.clone(),
            };
            let response = bridge_repo.list_user_credentials(&pagination).await?;
            for cred in response.items {
                in_use.insert(cred.data_encryption_key_id);
            }
            if response.next_page_token.is_none() {
                break;
            }
            next_page_token = response.next_page_token;
        }

        in_use
    };

    // Delete keys not in yaml and not in use by any credentials
    for (key_id, _) in existing_keys.iter() {
        if !yaml_keys.contains_key(key_id) && !keys_in_use.contains(key_id) {
            delete_data_encryption_key(on_config_change_tx, bridge_repo, key_id.clone(), false)
                .await?;
        }
    }

    info!("Bridge synced from soma definition");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use shared::{
        primitives::SqlMigrationLoader,
        soma_agent_definition::{SomaAgentDefinition, YamlSomaAgentDefinition},
    };
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_sync_on_start_empty_database() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let bridge_repo = bridge::repository::Repository::new(conn.clone());
        let soma_repo = crate::repository::Repository::new(conn.clone());
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        // Create empty soma definition
        let soma_def = YamlSomaAgentDefinition {
            cached_definition: Arc::new(Mutex::new(SomaAgentDefinition {
                version: "1.0.0".to_string(),
                bridge: None,
            })),
            path: PathBuf::from("test.yaml"),
        };
        let soma_def_provider = Arc::new(soma_def) as Arc<dyn SomaAgentDefinitionLike>;

        let result = sync_bridge_db_from_soma_definition_on_start(
            &kek,
            &tx,
            &bridge_repo,
            &soma_repo,
            &soma_def_provider,
        )
        .await;
        assert!(result.is_ok());

        // Verify database is still empty
        let dek_list = list_data_encryption_keys(
            &bridge_repo,
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
    async fn test_sync_on_start_creates_encryption_keys() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let bridge_repo = bridge::repository::Repository::new(conn.clone());
        let soma_repo = crate::repository::Repository::new(conn.clone());
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
                envelope_encryption_key_id:
                    shared::soma_agent_definition::EnvelopeEncryptionKeyId::Local {
                        key_id: "test-key".to_string(),
                    },
            },
        );

        let soma_def_provider = YamlSomaAgentDefinition {
            cached_definition: Arc::new(Mutex::new(SomaAgentDefinition {
                version: "1.0.0".to_string(),
                bridge: Some(shared::soma_agent_definition::BridgeConfig {
                    encryption: shared::soma_agent_definition::BridgeEncryptionConfig(
                        encryption_map,
                    ),
                    providers: None,
                }),
            })),
            path: PathBuf::from("test.yaml"),
        };
        let soma_def_provider = Arc::new(soma_def_provider) as Arc<dyn SomaAgentDefinitionLike>;
        let result = sync_bridge_db_from_soma_definition_on_start(
            &kek,
            &tx,
            &bridge_repo,
            &soma_repo,
            &soma_def_provider,
        )
        .await;
        assert!(result.is_ok());

        // Verify encryption key was created
        let dek_list = list_data_encryption_keys(
            &bridge_repo,
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
    async fn test_sync_on_start_deletes_existing_keys() {
        shared::setup_test!();

        let (_db, conn) = shared::test_utils::repository::setup_in_memory_database(vec![
            bridge::repository::Repository::load_sql_migrations(),
        ])
        .await
        .unwrap();
        let bridge_repo = bridge::repository::Repository::new(conn.clone());
        let soma_repo = crate::repository::Repository::new(conn.clone());
        let (tx, _rx) = tokio::sync::mpsc::channel(10);

        let kek = EnvelopeEncryptionKeyContents::Local {
            key_id: "test-key".to_string(),
            key_bytes: vec![0u8; 32],
        };

        // Create some existing DEKs
        create_data_encryption_key(
            &kek,
            &tx,
            &bridge_repo,
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
            &bridge_repo,
            CreateDataEncryptionKeyParams {
                id: Some("old-key-2".to_string()),
                encrypted_data_envelope_key: None,
            },
            false,
        )
        .await
        .unwrap();

        // Sync with empty definition
        let soma_def_provider = YamlSomaAgentDefinition {
            cached_definition: Arc::new(Mutex::new(SomaAgentDefinition {
                version: "1.0.0".to_string(),
                bridge: Some(shared::soma_agent_definition::BridgeConfig {
                    encryption: shared::soma_agent_definition::BridgeEncryptionConfig(
                        std::collections::HashMap::new(),
                    ),
                    providers: None,
                }),
            })),
            path: PathBuf::from("test.yaml"),
        };
        let soma_def_provider = Arc::new(soma_def_provider) as Arc<dyn SomaAgentDefinitionLike>;
        let result = sync_bridge_db_from_soma_definition_on_start(
            &kek,
            &tx,
            &bridge_repo,
            &soma_repo,
            &soma_def_provider,
        )
        .await;
        assert!(result.is_ok());

        // Verify all old keys were deleted
        let dek_list = list_data_encryption_keys(
            &bridge_repo,
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
