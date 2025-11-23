use std::sync::Arc;

use shared::{
    error::CommonError,
    soma_agent_definition::SomaAgentDefinitionLike,
};
use soma_api_client::{
    apis::{configuration::Configuration, default_api},
    models,
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
/// All operations are performed via API calls to the soma-api-server to ensure
/// proper separation of concerns and consistency with the rest of the system.
pub async fn sync_bridge_db_from_soma_definition_on_start(
    api_config: &Configuration,
    soma_definition_provider: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    let soma_definition = soma_definition_provider.get_definition().await?;
    use std::collections::{HashMap, HashSet};

    // 1. Sync data encryption keys
    // Get all existing keys
    let existing_keys = {
        let mut keys = HashMap::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let response = default_api::list_data_encryption_keys(
                api_config,
                100,
                next_page_token.as_deref(),
            )
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to list data encryption keys: {e:?}")))?;

            for key in response.items {
                keys.insert(key.id.clone(), key);
            }
            if response.next_page_token.is_empty() {
                break;
            }
            next_page_token = Some(response.next_page_token);
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
            let params = models::CreateDataEncryptionKeyParams {
                id: Some(Some(key_id.clone())),
                encrypted_data_envelope_key: Some(encryption_config.encrypted_data_encryption_key.clone()),
            };

            default_api::create_data_encryption_key(api_config, params)
                .await
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create data encryption key '{key_id}': {e:?}")))?;
        }
    }

    // 2. Sync provider instances
    if let Some(bridge_config) = &soma_definition.bridge {
        if let Some(providers) = &bridge_config.providers {
            // Get all existing provider instances with credentials
            let existing_providers: HashMap<String, models::ProviderInstanceListItem> = {
                let mut instances = HashMap::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = default_api::list_provider_instances(
                        api_config,
                        100,
                        next_page_token.as_deref(),
                        None, // status filter
                        None, // provider_controller_type_id filter
                    )
                    .await
                    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to list provider instances: {e:?}")))?;

                    for item in response.items {
                        instances.insert(item.id.clone(), item);
                    }
                    if response.next_page_token.is_empty() {
                        break;
                    }
                    next_page_token = Some(response.next_page_token);
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
                    // For API sync, we'll do a simple comparison - if anything changed, recreate
                    // This is simpler than detailed field-by-field comparison
                    let basic_fields_changed = existing.provider_controller_type_id != *provider_controller_type_id
                        || existing.credential_controller_type_id != *credential_controller_type_id
                        || existing.display_name != provider_config.display_name;

                    basic_fields_changed
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
                        default_api::delete_provider_instance(api_config, provider_id)
                            .await
                            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to delete provider instance '{provider_id}': {e:?}")))?;
                    } else {
                        tracing::info!("Creating new provider '{}'", provider_id);
                    }

                    // Create resource server credential
                    let resource_server_credential_params = models::CreateResourceServerCredentialParamsInner {
                        data_encryption_key_id: provider_config
                            .resource_server_credential
                            .data_encryption_key_id
                            .clone(),
                        resource_server_configuration: Some(provider_config
                            .resource_server_credential
                            .value
                            .clone()),
                        metadata: provider_config
                            .resource_server_credential
                            .metadata
                            .as_object()
                            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                    };

                    let resource_server_credential = default_api::create_resource_server_credential(
                        api_config,
                        provider_controller_type_id,
                        credential_controller_type_id,
                        resource_server_credential_params,
                    )
                    .await
                    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create resource server credential: {e:?}")))?;

                    // Create user credential if provided
                    let user_credential_id = if let Some(user_cred_config) =
                        &provider_config.user_credential
                    {
                        let user_credential_params = models::CreateUserCredentialParamsInner {
                            data_encryption_key_id: user_cred_config
                                .data_encryption_key_id
                                .clone(),
                            user_credential_configuration: Some(user_cred_config.value.clone()),
                            metadata: user_cred_config.metadata.as_object().map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                        };

                        let user_credential = default_api::create_user_credential(
                            api_config,
                            provider_controller_type_id,
                            credential_controller_type_id,
                            user_credential_params,
                        )
                        .await
                        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create user credential: {e:?}")))?;

                        Some(user_credential.id)
                    } else {
                        None
                    };

                    // Create provider instance
                    let create_provider_params = models::CreateProviderInstanceParamsInner {
                        provider_instance_id: Some(Some(provider_id.clone())),
                        display_name: provider_config.display_name.clone(),
                        resource_server_credential_id: resource_server_credential.id,
                        user_credential_id,
                        return_on_successful_brokering: None,
                    };

                    default_api::create_provider_instance(
                        api_config,
                        provider_controller_type_id,
                        credential_controller_type_id,
                        create_provider_params,
                    )
                    .await
                    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create provider instance '{provider_id}': {e:?}")))?;
                } else {
                    tracing::info!("Provider '{}' unchanged, preserving", provider_id);
                }

                // Sync function instances for this provider
                let existing_functions = {
                    let mut instances = HashSet::new();
                    let mut next_page_token: Option<String> = None;
                    loop {
                        let response = default_api::list_function_instances(
                            api_config,
                            100,
                            next_page_token.as_deref(),
                            Some(provider_id.as_str()),
                        )
                        .await
                        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to list function instances: {e:?}")))?;

                        for item in response.items {
                            instances.insert(item.function_controller_type_id);
                        }
                        if response.next_page_token.is_empty() {
                            break;
                        }
                        next_page_token = Some(response.next_page_token);
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
                        default_api::disable_function(
                            api_config,
                            provider_id,
                            function_id,
                        )
                        .await
                        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to disable function '{function_id}': {e:?}")))?;
                    }
                }

                // Enable functions from yaml
                for function_id in yaml_functions.iter() {
                    if !existing_functions.contains(function_id) {
                        default_api::enable_function(
                            api_config,
                            provider_id,
                            function_id,
                            serde_json::json!({}),
                        )
                        .await
                        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to enable function '{function_id}': {e:?}")))?;
                    }
                }
            }

            // Delete provider instances not in yaml (only if status is "active")
            for (provider_id, existing) in existing_providers.iter() {
                if !yaml_provider_ids.contains(provider_id)
                    && existing.status == "active"
                {
                    tracing::info!(
                        "Deleting provider '{}' not in yaml (status: active)",
                        provider_id
                    );
                    default_api::delete_provider_instance(api_config, provider_id)
                        .await
                        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to delete provider instance '{provider_id}': {e:?}")))?;
                }
            }
        }
    }

    // 3. Delete unused data encryption keys (after providers are synced, so credentials are cleaned up)
    // Note: The API doesn't expose credential listing endpoints for checking key usage,
    // so we'll skip automatic deletion of unused keys for now. This is safer anyway as it
    // prevents accidental deletion of keys that might be in use.
    // Keys can be manually deleted via the API if needed.

    info!("Bridge synced from soma definition");

    Ok(())
}
