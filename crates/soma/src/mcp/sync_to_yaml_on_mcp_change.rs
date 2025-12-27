use std::path::PathBuf;
use std::sync::Arc;

use encryption::logic::EncryptionKeyEvent;
use encryption::logic::envelope::EnvelopeEncryptionKey;
use identity::logic::OnConfigChangeEvt as IdentityOnConfigChangeEvt;
use mcp::logic::OnConfigChangeEvt;
use serde_json::json;
use tracing::{debug, trace, warn};

use shared::error::CommonError;
use shared::soma_agent_definition::{
    ApiKeyYamlConfig, EncryptedOauthYamlConfig, EncryptedOidcYamlConfig, EnvelopeKeyConfig,
    EnvelopeKeyConfigAwsKms, EnvelopeKeyConfigLocal, McpServerConfig, McpServerFunctionConfig,
    SecretConfig, SomaAgentDefinitionLike, StsConfigYaml, UserAuthFlowYamlConfig,
};
use soma_api_server::logic::on_change_pubsub::{
    EnvironmentVariableChangeEvt, SecretChangeEvt, SomaChangeEvt, SomaChangeRx,
};

/// Watches for unified soma change events and updates soma.yaml accordingly
pub async fn sync_on_soma_change(
    mut soma_change_rx: SomaChangeRx,
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    _project_dir: PathBuf,
) -> Result<(), CommonError> {
    loop {
        let event = match soma_change_rx.recv().await {
            Ok(event) => event,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                debug!("Soma change receiver closed");
                return Ok(());
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!("Soma change receiver lagged, skipped {} messages", skipped);
                continue;
            }
        };

        match event {
            SomaChangeEvt::Mcp(mcp_evt) => {
                handle_mcp_event(mcp_evt, &soma_definition).await?;
            }
            SomaChangeEvt::Encryption(encryption_evt) => {
                handle_encryption_event(encryption_evt, &soma_definition).await?;
            }
            SomaChangeEvt::Secret(secret_evt) => {
                handle_secret_event(secret_evt, &soma_definition).await?;
            }
            SomaChangeEvt::EnvironmentVariable(env_var_evt) => {
                handle_environment_variable_event(env_var_evt, &soma_definition).await?;
            }
            SomaChangeEvt::Identity(identity_evt) => {
                handle_identity_event(identity_evt, &soma_definition).await?;
            }
        }
    }
}

async fn handle_mcp_event(
    event: OnConfigChangeEvt,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        OnConfigChangeEvt::ProviderInstanceAdded(provider_instance) => {
            debug!(
                "Provider instance added: {:?}",
                provider_instance.provider_instance.id
            );

            let user_credential = provider_instance.user_credential.as_ref().map(|uc| {
                shared::soma_agent_definition::CredentialConfig {
                    id: uc.id.to_string(),
                    type_id: uc.type_id.clone(),
                    metadata: json!(uc.metadata.0.clone()),
                    value: uc.value.get_inner().clone(),
                    next_rotation_time: uc.next_rotation_time.map(|t| t.to_string()),
                    dek_alias: uc.dek_alias.clone(),
                }
            });

            soma_definition
                .add_provider(
                    provider_instance.provider_instance.id.clone(),
                    shared::soma_agent_definition::ProviderConfig {
                        provider_controller_type_id: provider_instance
                            .provider_instance
                            .provider_controller_type_id
                            .clone(),
                        credential_controller_type_id: provider_instance
                            .provider_instance
                            .credential_controller_type_id
                            .clone(),
                        display_name: provider_instance.provider_instance.display_name.clone(),
                        resource_server_credential:
                            shared::soma_agent_definition::CredentialConfig {
                                id: provider_instance.resource_server_credential.id.to_string(),
                                type_id: provider_instance
                                    .resource_server_credential
                                    .type_id
                                    .clone(),
                                metadata: json!(
                                    provider_instance
                                        .resource_server_credential
                                        .metadata
                                        .0
                                        .clone()
                                ),
                                value: provider_instance
                                    .resource_server_credential
                                    .value
                                    .get_inner()
                                    .clone(),
                                next_rotation_time: provider_instance
                                    .resource_server_credential
                                    .next_rotation_time
                                    .map(|t| t.to_string()),
                                dek_alias: provider_instance
                                    .resource_server_credential
                                    .dek_alias
                                    .clone(),
                            },
                        user_credential,
                        functions: None,
                    },
                )
                .await?;
        }
        OnConfigChangeEvt::ProviderInstanceUpdated(provider_instance) => {
            debug!(
                "Provider instance updated: {:?}",
                provider_instance.provider_instance.id
            );

            let user_credential = provider_instance.user_credential.as_ref().map(|uc| {
                shared::soma_agent_definition::CredentialConfig {
                    id: uc.id.to_string(),
                    type_id: uc.type_id.clone(),
                    metadata: json!(uc.metadata.0.clone()),
                    value: uc.value.get_inner().clone(),
                    next_rotation_time: uc.next_rotation_time.map(|t| t.to_string()),
                    dek_alias: uc.dek_alias.clone(),
                }
            });

            soma_definition
                .update_provider(
                    provider_instance.provider_instance.id.clone(),
                    shared::soma_agent_definition::ProviderConfig {
                        provider_controller_type_id: provider_instance
                            .provider_instance
                            .provider_controller_type_id
                            .clone(),
                        credential_controller_type_id: provider_instance
                            .provider_instance
                            .credential_controller_type_id
                            .clone(),
                        display_name: provider_instance.provider_instance.display_name.clone(),
                        resource_server_credential:
                            shared::soma_agent_definition::CredentialConfig {
                                id: provider_instance.resource_server_credential.id.to_string(),
                                type_id: provider_instance
                                    .resource_server_credential
                                    .type_id
                                    .clone(),
                                metadata: json!(
                                    provider_instance
                                        .resource_server_credential
                                        .metadata
                                        .0
                                        .clone()
                                ),
                                value: provider_instance
                                    .resource_server_credential
                                    .value
                                    .get_inner()
                                    .clone(),
                                next_rotation_time: provider_instance
                                    .resource_server_credential
                                    .next_rotation_time
                                    .map(|t| t.to_string()),
                                dek_alias: provider_instance
                                    .resource_server_credential
                                    .dek_alias
                                    .clone(),
                            },
                        user_credential,
                        functions: None,
                    },
                )
                .await?;
        }
        OnConfigChangeEvt::ProviderInstanceRemoved(provider_instance_id) => {
            soma_definition
                .remove_provider(provider_instance_id)
                .await?;
        }
        OnConfigChangeEvt::FunctionInstanceAdded(function_instance_serialized) => {
            debug!(
                "Function instance added: {:?}",
                function_instance_serialized.function_controller_type_id
            );
            soma_definition
                .add_function_instance(
                    function_instance_serialized
                        .provider_controller_type_id
                        .clone(),
                    function_instance_serialized
                        .function_controller_type_id
                        .clone(),
                    function_instance_serialized.provider_instance_id.clone(),
                )
                .await?;
        }
        OnConfigChangeEvt::FunctionInstanceRemoved(
            function_controller_type_id,
            provider_controller_type_id,
            provider_instance_id,
        ) => {
            debug!(
                "Function instance removed: function_controller_type_id={:?}, provider_instance_id={:?}",
                function_controller_type_id, provider_instance_id
            );
            soma_definition
                .remove_function_instance(
                    provider_controller_type_id,
                    function_controller_type_id,
                    provider_instance_id,
                )
                .await?;
        }
        OnConfigChangeEvt::McpServerInstanceAdded(mcp_server) => {
            debug!("MCP server instance added: {:?}", mcp_server.id);

            let functions = mcp_server
                .functions
                .iter()
                .map(|f| McpServerFunctionConfig {
                    function_controller_type_id: f.function_controller_type_id.clone(),
                    provider_controller_type_id: f.provider_controller_type_id.clone(),
                    provider_instance_id: f.provider_instance_id.clone(),
                    function_name: f.function_name.clone(),
                    function_description: f.function_description.clone(),
                })
                .collect();

            soma_definition
                .add_mcp_server(
                    mcp_server.id.clone(),
                    McpServerConfig {
                        name: mcp_server.name.clone(),
                        functions: Some(functions),
                    },
                )
                .await?;
        }
        OnConfigChangeEvt::McpServerInstanceUpdated(mcp_server) => {
            debug!("MCP server instance updated: {:?}", mcp_server.id);

            let functions = mcp_server
                .functions
                .iter()
                .map(|f| McpServerFunctionConfig {
                    function_controller_type_id: f.function_controller_type_id.clone(),
                    provider_controller_type_id: f.provider_controller_type_id.clone(),
                    provider_instance_id: f.provider_instance_id.clone(),
                    function_name: f.function_name.clone(),
                    function_description: f.function_description.clone(),
                })
                .collect();

            soma_definition
                .update_mcp_server(
                    mcp_server.id.clone(),
                    McpServerConfig {
                        name: mcp_server.name.clone(),
                        functions: Some(functions),
                    },
                )
                .await?;
        }
        OnConfigChangeEvt::McpServerInstanceRemoved(mcp_server_id) => {
            debug!("MCP server instance removed: {:?}", mcp_server_id);
            soma_definition.remove_mcp_server(mcp_server_id).await?;
        }
        OnConfigChangeEvt::McpServerInstanceFunctionAdded(function) => {
            debug!(
                "MCP server function added: {:?} to {:?}",
                function.function_name, function.mcp_server_instance_id
            );
            soma_definition
                .add_mcp_server_function(
                    function.mcp_server_instance_id.clone(),
                    McpServerFunctionConfig {
                        function_controller_type_id: function.function_controller_type_id.clone(),
                        provider_controller_type_id: function.provider_controller_type_id.clone(),
                        provider_instance_id: function.provider_instance_id.clone(),
                        function_name: function.function_name.clone(),
                        function_description: function.function_description.clone(),
                    },
                )
                .await?;
        }
        OnConfigChangeEvt::McpServerInstanceFunctionUpdated(function) => {
            debug!(
                "MCP server function updated: {:?} in {:?}",
                function.function_name, function.mcp_server_instance_id
            );
            soma_definition
                .update_mcp_server_function(
                    function.mcp_server_instance_id.clone(),
                    McpServerFunctionConfig {
                        function_controller_type_id: function.function_controller_type_id.clone(),
                        provider_controller_type_id: function.provider_controller_type_id.clone(),
                        provider_instance_id: function.provider_instance_id.clone(),
                        function_name: function.function_name.clone(),
                        function_description: function.function_description.clone(),
                    },
                )
                .await?;
        }
        OnConfigChangeEvt::McpServerInstanceFunctionRemoved(
            mcp_server_instance_id,
            function_controller_type_id,
            provider_controller_type_id,
            provider_instance_id,
        ) => {
            debug!(
                "MCP server function removed from {:?}: {}/{}/{}",
                mcp_server_instance_id,
                function_controller_type_id,
                provider_controller_type_id,
                provider_instance_id
            );
            soma_definition
                .remove_mcp_server_function(
                    mcp_server_instance_id,
                    function_controller_type_id,
                    provider_controller_type_id,
                    provider_instance_id,
                )
                .await?;
        }
    }
    Ok(())
}

async fn handle_encryption_event(
    event: EncryptionKeyEvent,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        EncryptionKeyEvent::EnvelopeEncryptionKeyAdded(eek) => {
            debug!("Envelope encryption key added: {:?}", eek.id());
            let key_id = eek.id();
            let config = match eek {
                EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                    EnvelopeKeyConfig::AwsKms(EnvelopeKeyConfigAwsKms {
                        arn: aws_kms.arn.clone(),
                        region: aws_kms.region.clone(),
                        deks: None,
                    })
                }
                EnvelopeEncryptionKey::Local(local) => {
                    EnvelopeKeyConfig::Local(EnvelopeKeyConfigLocal {
                        file_name: local.file_name.clone(),
                        deks: None,
                    })
                }
            };
            soma_definition.add_envelope_key(key_id, config).await?;
        }
        EncryptionKeyEvent::EnvelopeEncryptionKeyRemoved(eek_id) => {
            debug!("Envelope encryption key removed: {:?}", eek_id);
            soma_definition.remove_envelope_key(eek_id).await?;
        }
        EncryptionKeyEvent::DataEncryptionKeyAdded(dek) => {
            // DEK creation events are ignored - we only sync DEKs when aliases are added/updated
            // This ensures we always have complete data (including alias) in YAML
            trace!(
                "Data encryption key added: {:?} (ignoring - will sync when alias is added)",
                dek.id
            );
        }
        EncryptionKeyEvent::DataEncryptionKeyRemoved(dek_id) => {
            // DEK removal events are ignored - we only sync DEKs when aliases are added/updated/removed
            // When a DEK is removed, its aliases are also removed, which will trigger alias removal events
            trace!(
                "Data encryption key removed: {:?} (ignoring - will sync when alias is removed)",
                dek_id
            );
        }
        EncryptionKeyEvent::DataEncryptionKeyMigrated {
            old_dek_id,
            new_dek_id,
            from_envelope_key,
            to_envelope_key,
            aliases,
        } => {
            debug!(
                "Data encryption key migrated: {:?} -> {:?} from {:?} to {:?} with aliases: {:?}",
                old_dek_id,
                new_dek_id,
                from_envelope_key.id(),
                to_envelope_key.id(),
                aliases
            );
            let definition = soma_definition.get_definition().await?;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    let from_envelope_key_id = from_envelope_key.id();
                    let to_envelope_key_id = to_envelope_key.id();

                    // Step 1: Remove the old DEK from the source envelope key
                    if let Some(envelope_key_config) = envelope_keys.get(&from_envelope_key_id) {
                        if let Some(deks) = envelope_key_config.deks() {
                            // First, try to remove by UUID (if stored as UUID before alias was added)
                            if deks.contains_key(&old_dek_id) {
                                soma_definition
                                    .remove_dek(from_envelope_key_id.clone(), old_dek_id.clone())
                                    .await?;
                            } else {
                                // DEK is stored by alias - remove by each alias that was migrated
                                for alias in &aliases {
                                    if deks.contains_key(alias) {
                                        soma_definition
                                            .remove_dek(from_envelope_key_id.clone(), alias.clone())
                                            .await?;
                                        break; // Only one alias should match
                                    }
                                }
                            }
                        }
                    }

                    // Step 2: Ensure the new DEK exists in target envelope key (it should have been added via DataEncryptionKeyAdded)
                    // Step 3: Rename the new DEK from UUID to alias in the target envelope key
                    if let Some(envelope_key_config) = envelope_keys.get(&to_envelope_key_id) {
                        if let Some(deks) = envelope_key_config.deks() {
                            // Check if new DEK exists by UUID
                            if deks.contains_key(&new_dek_id) {
                                // Rename it to the alias (use first alias if multiple)
                                if let Some(alias) = aliases.first() {
                                    soma_definition
                                        .rename_dek(
                                            to_envelope_key_id.clone(),
                                            new_dek_id.clone(),
                                            alias.clone(),
                                        )
                                        .await?;
                                }
                            } else {
                                // DEK might already be renamed, or we need to wait for DataEncryptionKeyAdded
                                // Check if any of the aliases already exist in target envelope key
                                for alias in &aliases {
                                    if deks.contains_key(alias) {
                                        // Alias already exists - this is fine, it means the rename already happened
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        EncryptionKeyEvent::DataEncryptionKeyAliasAdded { alias, dek } => {
            // Add or update the DEK in YAML with the alias as the key
            // The event includes full DEK data, so we can add it directly
            debug!(
                "DEK alias added: {:?} -> {:?} - adding DEK to yaml",
                alias, dek.id
            );
            let envelope_key_id = dek.envelope_encryption_key_id.id();

            // Check if DEK already exists in YAML (by UUID or alias)
            let definition = soma_definition.get_definition().await?;
            let mut needs_add = true;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    if let Some(config) = envelope_keys.get(&envelope_key_id) {
                        if let Some(deks) = config.deks() {
                            // Check if DEK exists by UUID or alias
                            if deks.contains_key(&dek.id) || deks.contains_key(&alias) {
                                // If it exists by UUID, rename to alias
                                if deks.contains_key(&dek.id) && !deks.contains_key(&alias) {
                                    soma_definition
                                        .rename_dek(
                                            envelope_key_id.clone(),
                                            dek.id.clone(),
                                            alias.clone(),
                                        )
                                        .await?;
                                }
                                needs_add = false;
                            }
                        }
                    }
                }
            }

            // Add the DEK if it doesn't exist
            if needs_add {
                soma_definition
                    .add_dek(
                        envelope_key_id,
                        alias.clone(),
                        dek.encrypted_data_encryption_key.0,
                    )
                    .await?;
            }
        }
        EncryptionKeyEvent::DataEncryptionKeyAliasRemoved { alias } => {
            debug!("DEK alias removed: {:?} - removing DEK from yaml", alias);
            // Remove the DEK from YAML by searching all envelope keys
            let definition = soma_definition.get_definition().await?;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    for (envelope_key_id, config) in envelope_keys {
                        if let Some(deks) = config.deks() {
                            if deks.contains_key(&alias) {
                                soma_definition
                                    .remove_dek(envelope_key_id.clone(), alias.clone())
                                    .await?;
                                break;
                            }
                        }
                    }
                }
            }
        }
        EncryptionKeyEvent::DataEncryptionKeyAliasUpdated { alias, dek } => {
            // Alias updated means the alias now points to a different DEK
            // Update the DEK in YAML with the new DEK data
            debug!("DEK alias updated: {:?} -> {:?}", alias, dek.id);
            let envelope_key_id = dek.envelope_encryption_key_id.id();

            // Check if alias already exists in YAML
            let definition = soma_definition.get_definition().await?;
            let mut needs_add = true;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    if let Some(config) = envelope_keys.get(&envelope_key_id) {
                        if let Some(deks) = config.deks() {
                            if deks.contains_key(&alias) {
                                // Alias exists - update it by removing and re-adding
                                soma_definition
                                    .remove_dek(envelope_key_id.clone(), alias.clone())
                                    .await?;
                            }
                            // Also check if DEK exists by UUID (might need to rename)
                            if deks.contains_key(&dek.id) && !deks.contains_key(&alias) {
                                soma_definition
                                    .rename_dek(
                                        envelope_key_id.clone(),
                                        dek.id.clone(),
                                        alias.clone(),
                                    )
                                    .await?;
                                needs_add = false;
                            }
                        }
                    }
                }
            }

            // Add the DEK with the alias if it doesn't exist
            if needs_add {
                soma_definition
                    .add_dek(
                        envelope_key_id,
                        alias.clone(),
                        dek.encrypted_data_encryption_key.0,
                    )
                    .await?;
            }
        }
    }
    Ok(())
}

async fn handle_secret_event(
    event: SecretChangeEvt,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        SecretChangeEvt::Created(secret) => {
            // Check if this secret already exists in YAML
            // If it does, skip writing to avoid overwriting with a re-encrypted value
            // (encryption produces different ciphertext each time due to random nonces)
            let definition = soma_definition.get_definition().await?;
            let secret_exists_in_yaml = definition
                .secrets
                .as_ref()
                .map(|secrets| secrets.contains_key(&secret.key))
                .unwrap_or(false);

            if secret_exists_in_yaml {
                trace!(
                    "Secret '{}' already exists in YAML, skipping write to preserve encrypted value",
                    secret.key
                );
            } else {
                debug!("Secret created: {:?}", secret.key);
                let config = SecretConfig {
                    value: secret.encrypted_secret,
                    dek_alias: secret.dek_alias,
                };
                soma_definition.add_secret(secret.key, config).await?;
            }
        }
        SecretChangeEvt::Updated(secret) => {
            debug!("Secret updated: {:?}", secret.key);
            let config = SecretConfig {
                value: secret.encrypted_secret,
                dek_alias: secret.dek_alias,
            };
            soma_definition.update_secret(secret.key, config).await?;
        }
        SecretChangeEvt::Deleted { id: _, key } => {
            debug!("Secret deleted: {:?}", key);
            soma_definition.remove_secret(key).await?;
        }
    }
    Ok(())
}

async fn handle_environment_variable_event(
    event: EnvironmentVariableChangeEvt,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        EnvironmentVariableChangeEvt::Created(env_var) => {
            debug!("Environment variable created: {:?}", env_var.key);
            soma_definition
                .add_environment_variable(env_var.key, env_var.value)
                .await?;
        }
        EnvironmentVariableChangeEvt::Updated(env_var) => {
            debug!("Environment variable updated: {:?}", env_var.key);
            soma_definition
                .update_environment_variable(env_var.key, env_var.value)
                .await?;
        }
        EnvironmentVariableChangeEvt::Deleted { id: _, key } => {
            debug!("Environment variable deleted: {:?}", key);
            soma_definition.remove_environment_variable(key).await?;
        }
    }
    Ok(())
}

async fn handle_identity_event(
    event: IdentityOnConfigChangeEvt,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        IdentityOnConfigChangeEvt::ApiKeyCreated(api_key_info) => {
            debug!("API key created: {:?}", api_key_info.id);

            // Check if this API key already exists in YAML
            // If it does, skip writing to avoid overwriting with a re-encrypted value
            let definition = soma_definition.get_definition().await?;
            let api_key_exists_in_yaml = definition
                .identity
                .as_ref()
                .and_then(|identity| identity.api_keys.as_ref())
                .map(|api_keys| api_keys.contains_key(&api_key_info.id))
                .unwrap_or(false);

            if api_key_exists_in_yaml {
                trace!(
                    "API key '{}' already exists in YAML, skipping write to preserve encrypted value",
                    api_key_info.id
                );
            } else {
                let config = ApiKeyYamlConfig {
                    description: api_key_info.description.clone(),
                    encrypted_hashed_value: api_key_info.encrypted_hashed_value,
                    dek_alias: api_key_info.dek_alias,
                    role: api_key_info.role.as_str().to_string(),
                    user_id: api_key_info.user_id,
                };
                soma_definition.add_api_key(api_key_info.id, config).await?;
            }
        }
        IdentityOnConfigChangeEvt::ApiKeyDeleted(id) => {
            debug!("API key deleted: {:?}", id);
            soma_definition.remove_api_key(id).await?;
        }
        IdentityOnConfigChangeEvt::StsConfigCreated(sts_config_info) => {
            use identity::logic::sts::config::StsTokenConfig as IdentityStsTokenConfig;

            // Skip syncing DevMode configs to YAML - they are ephemeral and only used in dev
            if matches!(&sts_config_info, IdentityStsTokenConfig::DevMode(_)) {
                trace!("Skipping DevMode STS config sync to YAML (dev mode configs are ephemeral)");
                return Ok(());
            }

            let config_id = match &sts_config_info {
                IdentityStsTokenConfig::JwtTemplate(config) => config.id.clone(),
                IdentityStsTokenConfig::DevMode(config) => config.id.clone(),
            };

            debug!("STS config created: {:?}", config_id);

            // Check if this STS config already exists in YAML
            let definition = soma_definition.get_definition().await?;
            let sts_config_exists_in_yaml = definition
                .identity
                .as_ref()
                .and_then(|identity| identity.sts_configurations.as_ref())
                .map(|sts_configs| sts_configs.contains_key(&config_id))
                .unwrap_or(false);

            if sts_config_exists_in_yaml {
                trace!(
                    "STS config '{}' already exists in YAML, skipping write",
                    config_id
                );
            } else {
                // Convert identity STS config to YAML config
                let yaml_config = match &sts_config_info {
                    IdentityStsTokenConfig::DevMode(_) => {
                        // This branch should never be reached due to early return above
                        unreachable!("DevMode configs should be skipped earlier")
                    }
                    IdentityStsTokenConfig::JwtTemplate(jwt_config) => {
                        // Convert the JwtTemplateModeConfig to JwtTemplateConfigYaml
                        let jwt_yaml = serde_json::to_value(jwt_config)
                            .and_then(serde_json::from_value)
                            .map_err(|e| {
                                CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to convert STS config to YAML format: {e}"
                                ))
                            })?;
                        StsConfigYaml::JwtTemplate(jwt_yaml)
                    }
                };

                soma_definition
                    .add_sts_config(config_id, yaml_config)
                    .await?;
            }
        }
        IdentityOnConfigChangeEvt::StsConfigDeleted(id) => {
            debug!("STS config deleted: {:?}", id);
            soma_definition.remove_sts_config(id).await?;
        }
        IdentityOnConfigChangeEvt::UserAuthFlowConfigCreated(config) => {
            let config_id = config.id().to_string();
            debug!("User auth flow config created: {:?}", config_id);

            // Check if this config already exists in YAML
            let definition = soma_definition.get_definition().await?;
            let config_exists_in_yaml = definition
                .identity
                .as_ref()
                .and_then(|identity| identity.user_auth_flows.as_ref())
                .map(|configs| configs.contains_key(&config_id))
                .unwrap_or(false);

            if config_exists_in_yaml {
                trace!(
                    "User auth flow config '{}' already exists in YAML, skipping write to preserve encrypted value",
                    config_id
                );
            } else {
                // Convert from identity crate type to YAML type
                let yaml_config = convert_user_auth_flow_to_yaml(&config)?;
                soma_definition
                    .add_user_auth_flow(config_id, yaml_config)
                    .await?;
            }
        }
        IdentityOnConfigChangeEvt::UserAuthFlowConfigDeleted(id) => {
            debug!("User auth flow config deleted: {:?}", id);
            soma_definition.remove_user_auth_flow(id).await?;
        }
    }
    Ok(())
}

/// Convert an EncryptedUserAuthFlowConfig from the identity crate to the YAML config type
fn convert_user_auth_flow_to_yaml(
    config: &identity::logic::user_auth_flow::EncryptedUserAuthFlowConfig,
) -> Result<UserAuthFlowYamlConfig, CommonError> {
    use identity::logic::user_auth_flow::{
        EncryptedOauthConfig, EncryptedOidcConfig, EncryptedUserAuthFlowConfig,
    };

    fn convert_oauth_config(
        oauth: &EncryptedOauthConfig,
    ) -> Result<EncryptedOauthYamlConfig, CommonError> {
        let mapping_json = serde_json::to_value(&oauth.mapping).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to serialize token mapping: {e}"))
        })?;

        Ok(EncryptedOauthYamlConfig {
            authorization_endpoint: oauth.authorization_endpoint.clone(),
            token_endpoint: oauth.token_endpoint.clone(),
            jwks_endpoint: oauth.jwks_endpoint.clone(),
            client_id: oauth.client_id.clone(),
            encrypted_client_secret: oauth.encrypted_client_secret.0.clone(),
            dek_alias: oauth.dek_alias.clone(),
            scopes: oauth.scopes.clone(),
            introspect_url: oauth.introspect_url.clone(),
            oauth_mapping_config: mapping_json,
        })
    }

    fn convert_oidc_config(
        oidc: &EncryptedOidcConfig,
    ) -> Result<EncryptedOidcYamlConfig, CommonError> {
        let base_config = convert_oauth_config(&oidc.base_config)?;
        let mapping_json = serde_json::to_value(&oidc.mapping).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to serialize token mapping: {e}"))
        })?;

        Ok(EncryptedOidcYamlConfig {
            base_config,
            discovery_endpoint: oidc.discovery_endpoint.clone(),
            userinfo_endpoint: oidc.userinfo_endpoint.clone(),
            introspect_url: oidc.introspect_url.clone(),
            oidc_mapping_config: mapping_json,
        })
    }

    match config {
        EncryptedUserAuthFlowConfig::OidcAuthorizationCodeFlow(oidc) => Ok(
            UserAuthFlowYamlConfig::OidcAuthorizationCodeFlow(convert_oidc_config(oidc)?),
        ),
        EncryptedUserAuthFlowConfig::OauthAuthorizationCodeFlow(oauth) => Ok(
            UserAuthFlowYamlConfig::OauthAuthorizationCodeFlow(convert_oauth_config(oauth)?),
        ),
        EncryptedUserAuthFlowConfig::OidcAuthorizationCodePkceFlow(oidc) => Ok(
            UserAuthFlowYamlConfig::OidcAuthorizationCodePkceFlow(convert_oidc_config(oidc)?),
        ),
        EncryptedUserAuthFlowConfig::OauthAuthorizationCodePkceFlow(oauth) => Ok(
            UserAuthFlowYamlConfig::OauthAuthorizationCodePkceFlow(convert_oauth_config(oauth)?),
        ),
    }
}
