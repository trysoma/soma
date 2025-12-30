use std::sync::Arc;

use shared::{
    error::CommonError,
    soma_agent_definition::{EnvelopeKeyConfig, SomaAgentDefinitionLike},
};
use soma_api_client::{
    apis::{configuration::Configuration, encryption_api, environment_api, identity_api, mcp_api},
    models,
};
use tracing::debug;

/// Synchronizes encryption and MCP data from soma.yaml to the API on startup.
///
/// This function performs a smart sync that:
/// 1. Syncs envelope encryption keys (creates missing)
/// 2. Syncs data encryption keys (imports missing DEKs under each envelope key)
/// 3. Syncs DEK aliases (creates missing)
/// 4. Syncs provider instances with credentials (using dek_alias)
/// 5. Syncs function instances for each provider
///
/// All operations are performed via API calls to the soma-api-server.
pub async fn sync_mcp_db_from_soma_definition_on_start(
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
                    let response = encryption_api::list_envelope_encryption_keys(
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
                    encryption_api::create_envelope_encryption_key(api_config, envelope_key)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to create envelope encryption key '{key_id}': {e:?}"
                            ))
                        })?;
                    debug!("Created envelope encryption key: {}", key_id);
                }

                // 1b. Sync DEKs for this envelope key
                // DEKs in YAML are now keyed by their alias (e.g., "default")
                if let Some(deks) = envelope_key_config.deks() {
                    // Import missing DEKs - check by alias
                    for (alias, dek_config) in deks {
                        // Check if a DEK with this alias already exists
                        let alias_exists =
                            encryption_api::get_dek_by_alias_or_id(api_config, alias)
                                .await
                                .is_ok();

                        if !alias_exists {
                            // Import the DEK (will generate a new ID)
                            let import_params = models::ImportDataEncryptionKeyParamsRoute {
                                id: None, // Let the server generate the ID
                                encrypted_data_encryption_key: dek_config.encrypted_key.clone(),
                            };
                            let imported_dek = encryption_api::import_data_encryption_key(
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
                            debug!(
                                "Imported DEK with ID '{}' under envelope key '{}'",
                                imported_dek.id, key_id
                            );

                            // Create the alias for the imported DEK
                            let create_alias_req = models::CreateDekAliasRequest {
                                alias: alias.clone(),
                                dek_id: imported_dek.id.clone(),
                            };
                            encryption_api::create_dek_alias(api_config, create_alias_req)
                                .await
                                .map_err(|e| {
                                    CommonError::Unknown(anyhow::anyhow!(
                                        "Failed to create DEK alias '{alias}' -> '{}': {e:?}",
                                        imported_dek.id
                                    ))
                                })?;
                            debug!("Created DEK alias '{}' -> '{}'", alias, imported_dek.id);
                        }
                    }
                }
            }
        }
    }

    // 2. Sync provider instances
    if let Some(mcp_config) = &soma_definition.mcp {
        if let Some(providers) = &mcp_config.providers {
            // Get all existing provider instances with credentials
            let existing_providers: HashMap<String, models::ProviderInstanceListItem> = {
                let mut instances = HashMap::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = mcp_api::list_provider_instances(
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

                tracing::debug!(
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
                        tracing::debug!(
                            "Provider '{}' configuration changed, recreating",
                            provider_id
                        );
                        mcp_api::delete_provider_instance(api_config, provider_id)
                            .await
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to delete provider instance '{provider_id}': {e:?}"
                                ))
                            })?;
                    } else {
                        tracing::debug!("Creating new provider '{}'", provider_id);
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

                    let resource_server_credential = mcp_api::create_resource_server_credential(
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

                            let user_credential = mcp_api::create_user_credential(
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

                    mcp_api::create_provider_instance(
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
                    tracing::trace!("Provider '{}' unchanged, preserving", provider_id);
                }

                // Sync function instances for this provider
                let existing_functions = {
                    let mut instances = HashSet::new();
                    let mut next_page_token: Option<String> = None;
                    loop {
                        let response = mcp_api::list_function_instances(
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
                        mcp_api::disable_function(api_config, provider_id, function_id)
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
                        mcp_api::enable_function(
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
                    tracing::debug!(
                        "Deleting provider '{}' not in yaml (status: active)",
                        provider_id
                    );
                    mcp_api::delete_provider_instance(api_config, provider_id)
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

    debug!("MCP synced from soma definition");

    // 3. Sync secrets
    // NOTE: Secrets in soma.yaml are stored with their ENCRYPTED values
    // We use import_secret (not create_secret) to avoid double-encryption
    if let Some(secrets) = &soma_definition.secrets {
        use std::collections::HashSet;

        // Get existing secrets
        let existing_secrets: HashSet<String> = {
            let mut keys = HashSet::new();
            let mut next_page_token: Option<String> = None;
            loop {
                let response =
                    environment_api::list_secrets(api_config, 100, next_page_token.as_deref())
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

        // Import pre-encrypted secrets from yaml
        for (key, secret_config) in secrets {
            if !existing_secrets.contains(key) {
                // Use import_secret which stores the already-encrypted value as-is
                let import_req = models::ImportSecretRequest {
                    key: key.clone(),
                    encrypted_value: secret_config.value.clone(),
                    dek_alias: secret_config.dek_alias.clone(),
                };
                environment_api::import_secret(api_config, import_req)
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to import secret '{key}': {e:?}"
                        ))
                    })?;
                debug!("Imported secret '{}'", key);
            }
        }
    }

    debug!("Secrets synced from soma definition");

    // 4. Sync environment variables
    if let Some(env_vars) = &soma_definition.environment_variables {
        use std::collections::HashSet;

        // Get existing environment variables
        let existing_env_vars: HashSet<String> = {
            let mut keys = HashSet::new();
            let mut next_page_token: Option<String> = None;
            loop {
                let response =
                    environment_api::list_variables(api_config, 100, next_page_token.as_deref())
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to list environment variables: {e:?}"
                            ))
                        })?;

                for env_var in response.variables {
                    keys.insert(env_var.key);
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

        // Create or update environment variables from yaml
        for (key, value) in env_vars {
            if !existing_env_vars.contains(key) {
                let create_req = models::CreateVariableRequest {
                    key: key.clone(),
                    value: value.clone(),
                };
                environment_api::create_variable(api_config, create_req)
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to create environment variable '{key}': {e:?}"
                        ))
                    })?;
                debug!("Created environment variable '{}'", key);
            }
        }
    }

    debug!("Environment variables synced from soma definition");

    // 5. Sync MCP server instances
    if let Some(mcp_config) = &soma_definition.mcp {
        if let Some(mcp_servers) = &mcp_config.mcp_servers {
            use std::collections::HashSet;

            // Get existing MCP server instances
            let existing_mcp_servers: HashMap<
                String,
                models::McpServerInstanceSerializedWithFunctions,
            > = {
                let mut servers = HashMap::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = mcp_api::list_mcp_server_instances(
                        api_config,
                        100,
                        next_page_token.as_deref(),
                    )
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to list MCP server instances: {e:?}"
                        ))
                    })?;

                    for item in response.items {
                        servers.insert(item.id.clone(), item);
                    }
                    if response.next_page_token.is_none() {
                        break;
                    }
                    next_page_token = response.next_page_token;
                }
                servers
            };

            let yaml_mcp_server_ids: HashSet<String> = mcp_servers.keys().cloned().collect();

            // Create or update MCP servers from yaml
            for (mcp_server_id, mcp_server_config) in mcp_servers {
                let needs_create = !existing_mcp_servers.contains_key(mcp_server_id);
                let needs_update = existing_mcp_servers
                    .get(mcp_server_id)
                    .map(|existing| existing.name != mcp_server_config.name)
                    .unwrap_or(false);

                if needs_create {
                    // Create MCP server instance
                    let create_req = models::CreateMcpServerInstanceRequest {
                        id: mcp_server_id.clone(),
                        name: mcp_server_config.name.clone(),
                    };
                    mcp_api::create_mcp_server_instance(api_config, create_req)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to create MCP server instance '{mcp_server_id}': {e:?}"
                            ))
                        })?;
                    debug!("Created MCP server instance '{}'", mcp_server_id);
                } else if needs_update {
                    // Update MCP server instance name
                    let update_req = models::UpdateMcpServerInstanceRequest {
                        name: mcp_server_config.name.clone(),
                    };
                    mcp_api::update_mcp_server_instance(api_config, mcp_server_id, update_req)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to update MCP server instance '{mcp_server_id}': {e:?}"
                            ))
                        })?;
                    debug!("Updated MCP server instance '{}'", mcp_server_id);
                }

                // Sync functions for this MCP server
                let existing_functions: HashSet<(String, String, String)> = existing_mcp_servers
                    .get(mcp_server_id)
                    .map(|s| {
                        s.functions
                            .iter()
                            .map(|f| {
                                (
                                    f.function_controller_type_id.clone(),
                                    f.provider_controller_type_id.clone(),
                                    f.provider_instance_id.clone(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let yaml_functions: HashSet<(String, String, String)> = mcp_server_config
                    .functions
                    .as_ref()
                    .map(|funcs| {
                        funcs
                            .iter()
                            .map(|f| {
                                (
                                    f.function_controller_type_id.clone(),
                                    f.provider_controller_type_id.clone(),
                                    f.provider_instance_id.clone(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Remove functions not in yaml
                for (func_ctrl_id, prov_ctrl_id, prov_inst_id) in existing_functions.iter() {
                    if !yaml_functions.contains(&(
                        func_ctrl_id.clone(),
                        prov_ctrl_id.clone(),
                        prov_inst_id.clone(),
                    )) {
                        mcp_api::remove_mcp_server_instance_function(
                            api_config,
                            mcp_server_id,
                            func_ctrl_id,
                            prov_ctrl_id,
                            prov_inst_id,
                        )
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to remove function from MCP server '{mcp_server_id}': {e:?}"
                            ))
                        })?;
                        debug!(
                            "Removed function '{}/{}/{}' from MCP server '{}'",
                            func_ctrl_id, prov_ctrl_id, prov_inst_id, mcp_server_id
                        );
                    }
                }

                // Add or update functions from yaml
                if let Some(yaml_funcs) = &mcp_server_config.functions {
                    for func_config in yaml_funcs {
                        let func_key = (
                            func_config.function_controller_type_id.clone(),
                            func_config.provider_controller_type_id.clone(),
                            func_config.provider_instance_id.clone(),
                        );

                        if !existing_functions.contains(&func_key) {
                            // Add new function
                            let add_req = models::AddMcpServerInstanceFunctionRequest {
                                function_controller_type_id: func_config
                                    .function_controller_type_id
                                    .clone(),
                                provider_controller_type_id: func_config
                                    .provider_controller_type_id
                                    .clone(),
                                provider_instance_id: func_config.provider_instance_id.clone(),
                                function_name: func_config.function_name.clone(),
                                function_description: func_config
                                    .function_description
                                    .clone()
                                    .map(Some),
                            };
                            mcp_api::add_mcp_server_instance_function(
                                api_config,
                                mcp_server_id,
                                add_req,
                            )
                            .await
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to add function '{}' to MCP server '{mcp_server_id}': {e:?}",
                                    func_config.function_name
                                ))
                            })?;
                            debug!(
                                "Added function '{}' to MCP server '{}'",
                                func_config.function_name, mcp_server_id
                            );
                        } else {
                            // Check if function needs update (name or description changed)
                            let existing_func =
                                existing_mcp_servers.get(mcp_server_id).and_then(|s| {
                                    s.functions.iter().find(|f| {
                                        f.function_controller_type_id
                                            == func_config.function_controller_type_id
                                            && f.provider_controller_type_id
                                                == func_config.provider_controller_type_id
                                            && f.provider_instance_id
                                                == func_config.provider_instance_id
                                    })
                                });

                            if let Some(existing) = existing_func {
                                // Flatten the doubly-wrapped Option for comparison
                                let existing_desc = existing.function_description.clone().flatten();
                                if existing.function_name != func_config.function_name
                                    || existing_desc != func_config.function_description
                                {
                                    let update_req =
                                        models::UpdateMcpServerInstanceFunctionRequest {
                                            function_name: func_config.function_name.clone(),
                                            function_description: func_config
                                                .function_description
                                                .clone()
                                                .map(Some),
                                        };
                                    mcp_api::update_mcp_server_instance_function(
                                        api_config,
                                        mcp_server_id,
                                        &func_config.function_controller_type_id,
                                        &func_config.provider_controller_type_id,
                                        &func_config.provider_instance_id,
                                        update_req,
                                    )
                                    .await
                                    .map_err(|e| {
                                        CommonError::Unknown(anyhow::anyhow!(
                                            "Failed to update function '{}' in MCP server '{mcp_server_id}': {e:?}",
                                            func_config.function_name
                                        ))
                                    })?;
                                    debug!(
                                        "Updated function '{}' in MCP server '{}'",
                                        func_config.function_name, mcp_server_id
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // Delete MCP servers not in yaml
            for mcp_server_id in existing_mcp_servers.keys() {
                if !yaml_mcp_server_ids.contains(mcp_server_id) {
                    mcp_api::delete_mcp_server_instance(api_config, mcp_server_id)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to delete MCP server instance '{mcp_server_id}': {e:?}"
                            ))
                        })?;
                    debug!(
                        "Deleted MCP server instance '{}' not in yaml",
                        mcp_server_id
                    );
                }
            }
        }
    }

    debug!("MCP server instances synced from soma definition");

    // 6. Sync identity configuration (API keys and STS configs)
    if let Some(identity_config) = &soma_definition.identity {
        // 5a. Sync API keys
        if let Some(api_keys) = &identity_config.api_keys {
            use std::collections::HashSet;

            // Get existing API keys
            let existing_api_keys: HashSet<String> = {
                let mut ids = HashSet::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = identity_api::route_list_api_keys(
                        api_config,
                        100,
                        next_page_token.as_deref(),
                    )
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!("Failed to list API keys: {e:?}"))
                    })?;

                    for api_key in response.items {
                        ids.insert(api_key.id);
                    }
                    // Handle doubly wrapped Option<Option<String>> from generated API client
                    match response.next_page_token.flatten() {
                        Some(token) if !token.is_empty() => {
                            next_page_token = Some(token);
                        }
                        _ => break,
                    }
                }
                ids
            };

            // Import API keys from yaml (uses import endpoint which decrypts the stored value)
            for (id, api_key_config) in api_keys {
                if !existing_api_keys.contains(id) {
                    let role = parse_role_string(&api_key_config.role)?;
                    let import_req = models::EncryptedApiKeyConfig {
                        id: id.clone(),
                        encrypted_hashed_value: api_key_config.encrypted_hashed_value.clone(),
                        dek_alias: api_key_config.dek_alias.clone(),
                        description: Some(api_key_config.description.clone()),
                        role,
                        user_id: api_key_config.user_id.clone(),
                    };
                    identity_api::route_import_api_key(api_config, import_req)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to import API key '{id}': {e:?}"
                            ))
                        })?;
                    debug!("Imported API key '{}'", id);
                }
            }
        }

        // 5b. Sync STS configurations
        if let Some(sts_configs) = &identity_config.sts_configurations {
            use std::collections::HashSet;

            // Get existing STS configs
            let existing_sts_configs: HashSet<String> = {
                let mut ids = HashSet::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = identity_api::route_list_sts_configs(
                        api_config,
                        100,
                        next_page_token.as_deref(),
                    )
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!("Failed to list STS configs: {e:?}"))
                    })?;

                    for config in &response.items {
                        let config_id = get_sts_config_id(config);
                        ids.insert(config_id);
                    }
                    // Handle optional next_page_token
                    match response.next_page_token {
                        Some(token) if !token.is_empty() => {
                            next_page_token = Some(token);
                        }
                        _ => break,
                    }
                }
                ids
            };

            // Create STS configs from yaml (create is idempotent)
            for (id, sts_config) in sts_configs {
                if !existing_sts_configs.contains(id) {
                    use shared::soma_agent_definition::StsConfigYaml;
                    let create_req = match sts_config {
                        StsConfigYaml::Dev {} => models::StsTokenConfig::StsTokenConfigOneOf1(
                            models::StsTokenConfigOneOf1 {
                                dev_mode: models::DevModeConfig { id: id.clone() },
                            },
                        ),
                        StsConfigYaml::JwtTemplate(jwt_config) => {
                            let mapping_template = convert_jwt_template_to_api(jwt_config)?;
                            models::StsTokenConfig::StsTokenConfigOneOf(
                                models::StsTokenConfigOneOf {
                                    jwt_template: models::JwtTemplateModeConfig {
                                        id: id.clone(),
                                        mapping_template,
                                        validation_template:
                                            models::JwtTokenTemplateValidationConfig {
                                                issuer: jwt_config
                                                    .validation
                                                    .issuer
                                                    .clone()
                                                    .map(Some),
                                                valid_audiences: jwt_config
                                                    .validation
                                                    .valid_audiences
                                                    .clone()
                                                    .map(Some),
                                                required_groups: jwt_config
                                                    .validation
                                                    .required_groups
                                                    .clone()
                                                    .map(Some),
                                                required_scopes: jwt_config
                                                    .validation
                                                    .required_scopes
                                                    .clone()
                                                    .map(Some),
                                            },
                                    },
                                },
                            )
                        }
                    };
                    identity_api::route_create_sts_config(api_config, create_req)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to create STS config '{id}': {e:?}"
                            ))
                        })?;
                    debug!("Created STS config '{}'", id);
                }
            }
        }

        // 5c. Sync user auth flow configurations
        if let Some(user_auth_flows) = &identity_config.user_auth_flows {
            use std::collections::HashSet;

            // Get existing user auth flow configs
            let existing_user_auth_flows: HashSet<String> = {
                let mut ids = HashSet::new();
                let mut next_page_token: Option<String> = None;
                loop {
                    let response = identity_api::route_list_user_auth_flow_configs(
                        api_config,
                        Some(100),
                        next_page_token.as_deref(),
                        None, // config_type filter
                    )
                    .await
                    .map_err(|e| {
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to list user auth flow configs: {e:?}"
                        ))
                    })?;

                    for item in response.items {
                        // Extract ID from the config
                        let config_id = get_user_auth_flow_config_id(&item.config);
                        ids.insert(config_id);
                    }
                    // Handle doubly wrapped Option<Option<String>> from generated API client
                    match response.next_page_token.flatten() {
                        Some(token) if !token.is_empty() => {
                            next_page_token = Some(token);
                        }
                        _ => break,
                    }
                }
                ids
            };

            // Import user auth flow configs from yaml
            for (id, config) in user_auth_flows {
                if !existing_user_auth_flows.contains(id) {
                    let import_config = convert_yaml_to_api_user_auth_flow(id, config)?;
                    let import_req = models::ImportUserAuthFlowConfigParams {
                        config: import_config,
                    };
                    identity_api::route_import_user_auth_flow_config(api_config, import_req)
                        .await
                        .map_err(|e| {
                            CommonError::Unknown(anyhow::anyhow!(
                                "Failed to import user auth flow config '{id}': {e:?}"
                            ))
                        })?;
                    debug!("Imported user auth flow config '{}'", id);
                }
            }
        }

        debug!("Identity configuration synced from soma definition");
    }

    Ok(())
}

/// Extract the ID from an EncryptedUserAuthFlowConfig API model
fn get_user_auth_flow_config_id(config: &models::EncryptedUserAuthFlowConfig) -> String {
    match config {
        models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf(c) => {
            c.oidc_authorization_code_flow.id.clone()
        }
        models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf1(c) => {
            c.oauth_authorization_code_flow.id.clone()
        }
        models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf2(c) => {
            c.oidc_authorization_code_pkce_flow.id.clone()
        }
        models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf3(c) => {
            c.oauth_authorization_code_pkce_flow.id.clone()
        }
    }
}

/// Extract the ID from an StsTokenConfig API model
fn get_sts_config_id(config: &models::StsTokenConfig) -> String {
    match config {
        models::StsTokenConfig::StsTokenConfigOneOf(c) => c.jwt_template.id.clone(),
        models::StsTokenConfig::StsTokenConfigOneOf1(c) => c.dev_mode.id.clone(),
    }
}

/// Parse a role string to the models::Role enum
fn parse_role_string(role: &str) -> Result<models::Role, CommonError> {
    match role.to_lowercase().as_str() {
        "admin" => Ok(models::Role::Admin),
        "maintainer" => Ok(models::Role::Maintainer),
        "agent" => Ok(models::Role::Agent),
        "user" => Ok(models::Role::User),
        _ => Err(CommonError::InvalidRequest {
            msg: format!("Invalid role: {role}"),
            source: None,
        }),
    }
}

/// Convert a YAML JWT template config to the API model
fn convert_jwt_template_to_api(
    jwt_config: &shared::soma_agent_definition::JwtTemplateConfigYaml,
) -> Result<models::JwtTokenTemplateConfig, CommonError> {
    // Convert the config using serde_json (the types should be compatible)
    let json_value = serde_json::to_value(jwt_config).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Failed to serialize JWT template config: {e}"
        ))
    })?;
    serde_json::from_value(json_value).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!(
            "Failed to convert JWT template config: {e}"
        ))
    })
}

/// Convert YAML user auth flow config to API model for import
fn convert_yaml_to_api_user_auth_flow(
    id: &str,
    config: &shared::soma_agent_definition::UserAuthFlowYamlConfig,
) -> Result<models::EncryptedUserAuthFlowConfig, CommonError> {
    use shared::soma_agent_definition::{
        EncryptedOauthYamlConfig, EncryptedOidcYamlConfig, UserAuthFlowYamlConfig,
    };

    fn convert_oauth_yaml_to_api(
        id: &str,
        oauth: &EncryptedOauthYamlConfig,
    ) -> Result<models::EncryptedOauthConfig, CommonError> {
        Ok(models::EncryptedOauthConfig {
            id: id.to_string(),
            authorization_endpoint: oauth.authorization_endpoint.clone(),
            token_endpoint: oauth.token_endpoint.clone(),
            jwks_endpoint: oauth.jwks_endpoint.clone(),
            client_id: oauth.client_id.clone(),
            encrypted_client_secret: oauth.encrypted_client_secret.clone(),
            dek_alias: oauth.dek_alias.clone(),
            scopes: oauth.scopes.clone(),
            introspect_url: oauth.introspect_url.clone().map(Some),
            mapping: serde_json::from_value(oauth.oauth_mapping_config.clone()).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to parse token mapping: {e}"))
            })?,
        })
    }

    fn convert_oidc_yaml_to_api(
        id: &str,
        oidc: &EncryptedOidcYamlConfig,
    ) -> Result<models::EncryptedOidcConfig, CommonError> {
        let base_config = convert_oauth_yaml_to_api(id, &oidc.base_config)?;
        Ok(models::EncryptedOidcConfig {
            id: id.to_string(),
            base_config,
            discovery_endpoint: oidc.discovery_endpoint.clone().map(Some),
            userinfo_endpoint: oidc.userinfo_endpoint.clone().map(Some),
            introspect_url: oidc.introspect_url.clone().map(Some),
            mapping: serde_json::from_value(oidc.oidc_mapping_config.clone()).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to parse token mapping: {e}"))
            })?,
        })
    }

    match config {
        UserAuthFlowYamlConfig::OidcAuthorizationCodeFlow(oidc) => Ok(
            models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf(
                models::EncryptedUserAuthFlowConfigOneOf {
                    oidc_authorization_code_flow: convert_oidc_yaml_to_api(id, oidc)?,
                },
            ),
        ),
        UserAuthFlowYamlConfig::OauthAuthorizationCodeFlow(oauth) => Ok(
            models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf1(
                models::EncryptedUserAuthFlowConfigOneOf1 {
                    oauth_authorization_code_flow: convert_oauth_yaml_to_api(id, oauth)?,
                },
            ),
        ),
        UserAuthFlowYamlConfig::OidcAuthorizationCodePkceFlow(oidc) => Ok(
            models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf2(
                models::EncryptedUserAuthFlowConfigOneOf2 {
                    oidc_authorization_code_pkce_flow: convert_oidc_yaml_to_api(id, oidc)?,
                },
            ),
        ),
        UserAuthFlowYamlConfig::OauthAuthorizationCodePkceFlow(oauth) => Ok(
            models::EncryptedUserAuthFlowConfig::EncryptedUserAuthFlowConfigOneOf3(
                models::EncryptedUserAuthFlowConfigOneOf3 {
                    oauth_authorization_code_pkce_flow: convert_oauth_yaml_to_api(id, oauth)?,
                },
            ),
        ),
    }
}

/// Extract the key ID from an EnvelopeEncryptionKey API model
fn get_envelope_key_id(key: &models::EnvelopeEncryptionKey) -> String {
    match key {
        models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(aws_kms) => aws_kms.arn.clone(),
        models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(local) => {
            local.file_name.clone()
        }
    }
}

/// Convert EnvelopeKeyConfig from soma.yaml to API model
fn envelope_key_config_to_api_model(
    _key_id: &str,
    config: &EnvelopeKeyConfig,
) -> models::EnvelopeEncryptionKey {
    match config {
        EnvelopeKeyConfig::AwsKms(aws_kms) => {
            models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf(
                models::EnvelopeEncryptionKeyOneOf {
                    arn: aws_kms.arn.clone(),
                    region: aws_kms.region.clone(),
                    r#type: models::envelope_encryption_key_one_of::Type::AwsKms,
                },
            )
        }
        EnvelopeKeyConfig::Local(local) => {
            models::EnvelopeEncryptionKey::EnvelopeEncryptionKeyOneOf1(
                models::EnvelopeEncryptionKeyOneOf1 {
                    file_name: local.file_name.clone(),
                    r#type: models::envelope_encryption_key_one_of_1::Type::Local,
                },
            )
        }
    }
}
