use std::sync::Arc;

use shared::{
    error::CommonError,
    soma_agent_definition::{EnvelopeKeyConfig, SomaAgentDefinitionLike},
};
use soma_api_client::{
    apis::{configuration::Configuration, default_api},
    models,
};
use tracing::info;

/// Synchronizes encryption and bridge data from soma.yaml to the API on startup.
///
/// This function performs a smart sync that:
/// 1. Syncs envelope encryption keys (creates missing)
/// 2. Syncs data encryption keys (imports missing DEKs under each envelope key)
/// 3. Syncs DEK aliases (creates missing)
/// 4. Syncs provider instances with credentials (using dek_alias)
/// 5. Syncs function instances for each provider
///
/// All operations are performed via API calls to the soma-api-server.
pub async fn sync_bridge_db_from_soma_definition_on_start(
    api_config: &Configuration,
    soma_definition_provider: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    let soma_definition = soma_definition_provider.get_definition().await?;
    use std::collections::{HashMap, HashSet};

    // 1. Sync encryption configuration
    if let Some(encryption_config) = &soma_definition.encryption {
        // 1a. Sync envelope encryption keys
        if let Some(envelope_keys) = &encryption_config.envelope_keys {
            // Get existing envelope keys
            let existing_envelope_keys: HashMap<String, models::EnvelopeEncryptionKey> = {
                let mut keys = HashMap::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = default_api::list_envelope_encryption_keys(
                        api_config,
                        100,
                        next_page_token.as_deref(),
                    )
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to list envelope encryption keys: {e:?}"
                        ))
                    })?;

                    for key in response.items {
                        let key_id = get_envelope_key_id(&key);
                        keys.insert(key_id, key);
                    }
                    if response.next_page_token.is_none() {
                        break;
                    }
                    next_page_token = response.next_page_token;
                }
                keys
            };

            // Create missing envelope keys and sync their DEKs
            for (key_id, envelope_key_config) in envelope_keys {
                // Create envelope key if it doesn't exist
                if !existing_envelope_keys.contains_key(key_id) {
                    let envelope_key =
                        envelope_key_config_to_api_model(key_id, envelope_key_config);
                    default_api::create_envelope_encryption_key(api_config, envelope_key)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to create envelope encryption key '{key_id}': {e:?}"
                            ))
                        })?;
                    info!("Created envelope encryption key: {}", key_id);
                }

                // 1b. Sync DEKs for this envelope key
                // DEKs in YAML are now keyed by their alias (e.g., "default")
                if let Some(deks) = envelope_key_config.deks() {
                    // Import missing DEKs - check by alias
                    for (alias, dek_config) in deks {
                        // Check if a DEK with this alias already exists
                        let alias_exists = default_api::get_dek_by_alias_or_id(api_config, alias)
                            .await
                            .is_ok();

                        if !alias_exists {
                            // Import the DEK (will generate a new ID)
                            let import_params = models::ImportDataEncryptionKeyParamsRoute {
                                id: None, // Let the server generate the ID
                                encrypted_data_encryption_key: dek_config.encrypted_key.clone(),
                            };
                            let imported_dek = default_api::import_data_encryption_key(
                                api_config,
                                key_id,
                                import_params,
                            )
                            .await
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to import DEK with alias '{alias}' under envelope key '{key_id}': {e:?}"
                                ))
                            })?;
                            info!(
                                "Imported DEK with ID '{}' under envelope key '{}'",
                                imported_dek.id, key_id
                            );

                            // Create the alias for the imported DEK
                            let create_alias_req = models::CreateDekAliasRequest {
                                alias: alias.clone(),
                                dek_id: imported_dek.id.clone(),
                            };
                            default_api::create_dek_alias(api_config, create_alias_req)
                                .await
                                .map_err(|e| {
                                    CommonError::Unknown(anyhow::anyhow!(
                                        "Failed to create DEK alias '{alias}' -> '{}': {e:?}",
                                        imported_dek.id
                                    ))
                                })?;
                            info!("Created DEK alias '{}' -> '{}'", alias, imported_dek.id);
                        }
                    }
                }
            }
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
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to list provider instances: {e:?}"
                        ))
                    })?;

                    for item in response.items {
                        instances.insert(item.id.clone(), item);
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
                    existing.provider_controller_type_id != *provider_controller_type_id
                        || existing.credential_controller_type_id != *credential_controller_type_id
                        || existing.display_name != provider_config.display_name
                } else {
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
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to delete provider instance '{provider_id}': {e:?}"
                                ))
                            })?;
                    } else {
                        tracing::info!("Creating new provider '{}'", provider_id);
                    }

                    // Create resource server credential
                    let resource_server_credential_params =
                        models::CreateResourceServerCredentialParamsInner {
                            dek_alias: provider_config.resource_server_credential.dek_alias.clone(),
                            resource_server_configuration: Some(
                                provider_config.resource_server_credential.value.clone(),
                            ),
                            metadata: provider_config
                                .resource_server_credential
                                .metadata
                                .as_object()
                                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                        };

                    let resource_server_credential =
                        default_api::create_resource_server_credential(
                            api_config,
                            provider_controller_type_id,
                            credential_controller_type_id,
                            resource_server_credential_params,
                        )
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to create resource server credential: {e:?}"
                            ))
                        })?;

                    // Create user credential if provided
                    let user_credential_id =
                        if let Some(user_cred_config) = &provider_config.user_credential {
                            let user_credential_params = models::CreateUserCredentialParamsInner {
                                dek_alias: user_cred_config.dek_alias.clone(),
                                user_credential_configuration: Some(user_cred_config.value.clone()),
                                metadata: user_cred_config.metadata.as_object().map(|m| {
                                    m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                                }),
                            };

                            let user_credential = default_api::create_user_credential(
                                api_config,
                                provider_controller_type_id,
                                credential_controller_type_id,
                                user_credential_params,
                            )
                            .await
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to create user credential: {e:?}"
                                ))
                            })?;

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
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to create provider instance '{provider_id}': {e:?}"
                        ))
                    })?;
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
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to list function instances: {e:?}"
                            ))
                        })?;

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
                        default_api::disable_function(api_config, provider_id, function_id)
                            .await
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to disable function '{function_id}': {e:?}"
                                ))
                            })?;
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
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to enable function '{function_id}': {e:?}"
                            ))
                        })?;
                    }
                }
            }

            // Delete provider instances not in yaml (only if status is "active")
            for (provider_id, existing) in existing_providers.iter() {
                if !yaml_provider_ids.contains(provider_id) && existing.status == "active" {
                    tracing::info!(
                        "Deleting provider '{}' not in yaml (status: active)",
                        provider_id
                    );
                    default_api::delete_provider_instance(api_config, provider_id)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to delete provider instance '{provider_id}': {e:?}"
                            ))
                        })?;
                }
            }
        }
    }

    info!("Bridge synced from soma definition");

    // 3. Sync secrets
    if let Some(secrets) = &soma_definition.secrets {
        use std::collections::HashSet;

        // Get existing secrets
        let existing_secrets: HashSet<String> = {
            let mut keys = HashSet::new();
            let mut next_page_token: Option<String> = None;
            loop {
                let response =
                    default_api::list_secrets(api_config, 100, next_page_token.as_deref())
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!("Failed to list secrets: {e:?}"))
                        })?;

                for secret in response.secrets {
                    keys.insert(secret.key);
                }
                // Handle doubly wrapped Option<Option<String>> from generated API client
                match response.next_page_token.flatten() {
                    Some(token) if !token.is_empty() => {
                        next_page_token = Some(token);
                    }
                    _ => break,
                }
            }
            keys
        };

        // Create or update secrets from yaml
        for (key, secret_config) in secrets {
            if !existing_secrets.contains(key) {
                let create_req = models::CreateSecretRequest {
                    key: key.clone(),
                    raw_value: secret_config.value.clone(),
                    dek_alias: secret_config.dek_alias.clone(),
                };
                default_api::create_secret(api_config, create_req)
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to create secret '{key}': {e:?}"
                        ))
                    })?;
                info!("Created secret '{}'", key);
            }
        }
    }

    info!("Secrets synced from soma definition");

    Ok(())
}

/// Extract the key ID from an EnvelopeEncryptionKey API model
fn get_envelope_key_id(key: &models::EnvelopeEncryptionKey) -> String {
    match key {
        models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(aws_kms) => aws_kms.arn.clone(),
        models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(local) => local.file_name.clone(),
    }
}

/// Convert EnvelopeKeyConfig from soma.yaml to API model
fn envelope_key_config_to_api_model(
    _key_id: &str,
    config: &EnvelopeKeyConfig,
) -> models::EnvelopeEncryptionKey {
    match config {
        EnvelopeKeyConfig::AwsKms { arn, region, .. } => {
            models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(Box::new(
                models::EnvelopeEncryptionKeyOneOf {
                    arn: arn.clone(),
                    region: region.clone(),
                    r#type: models::envelope_encryption_key_one_of::Type::AwsKms,
                },
            ))
        }
        EnvelopeKeyConfig::Local { file_name, .. } => {
            models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(Box::new(
                models::EnvelopeEncryptionKeyOneOf1 {
                    file_name: file_name.clone(),
                    r#type: models::envelope_encryption_key_one_of_1::Type::Local,
                },
            ))
        }
    }
}
