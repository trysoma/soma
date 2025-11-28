use std::path::PathBuf;
use std::sync::Arc;

use bridge::logic::OnConfigChangeEvt;
use encryption::logic::EncryptionKeyEvent;
use encryption::logic::envelope::EnvelopeEncryptionKey;
use serde_json::json;
use tracing::{info, warn};

use shared::error::CommonError;
use shared::soma_agent_definition::{
    EnvelopeKeyConfig, EnvelopeKeyConfigAwsKms, EnvelopeKeyConfigLocal, SecretConfig,
    SomaAgentDefinitionLike,
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
                info!("Soma change receiver closed");
                return Ok(());
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!("Soma change receiver lagged, skipped {} messages", skipped);
                continue;
            }
        };

        match event {
            SomaChangeEvt::Bridge(bridge_evt) => {
                handle_bridge_event(bridge_evt, &soma_definition).await?;
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
        }
    }
}

async fn handle_bridge_event(
    event: OnConfigChangeEvt,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        OnConfigChangeEvt::ProviderInstanceAdded(provider_instance) => {
            info!(
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
            info!(
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
            info!(
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
            info!(
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
    }
    Ok(())
}

async fn handle_encryption_event(
    event: EncryptionKeyEvent,
    soma_definition: &Arc<dyn SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    match event {
        EncryptionKeyEvent::EnvelopeEncryptionKeyAdded(eek) => {
            info!("Envelope encryption key added: {:?}", eek.id());
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
            info!("Envelope encryption key removed: {:?}", eek_id);
            soma_definition.remove_envelope_key(eek_id).await?;
        }
        EncryptionKeyEvent::DataEncryptionKeyAdded(dek) => {
            // DEK creation events are ignored - we only sync DEKs when aliases are added/updated
            // This ensures we always have complete data (including alias) in YAML
            info!(
                "Data encryption key added: {:?} (ignoring - will sync when alias is added)",
                dek.id
            );
        }
        EncryptionKeyEvent::DataEncryptionKeyRemoved(dek_id) => {
            // DEK removal events are ignored - we only sync DEKs when aliases are added/updated/removed
            // When a DEK is removed, its aliases are also removed, which will trigger alias removal events
            info!(
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
            info!(
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
            info!(
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
            info!("DEK alias removed: {:?} - removing DEK from yaml", alias);
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
            info!("DEK alias updated: {:?} -> {:?}", alias, dek.id);
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
            info!("Secret created: {:?}", secret.key);
            let config = SecretConfig {
                value: secret.encrypted_secret,
                dek_alias: secret.dek_alias,
            };
            soma_definition.add_secret(secret.key, config).await?;
        }
        SecretChangeEvt::Updated(secret) => {
            info!("Secret updated: {:?}", secret.key);
            let config = SecretConfig {
                value: secret.encrypted_secret,
                dek_alias: secret.dek_alias,
            };
            soma_definition.update_secret(secret.key, config).await?;
        }
        SecretChangeEvt::Deleted { id: _, key } => {
            info!("Secret deleted: {:?}", key);
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
            info!("Environment variable created: {:?}", env_var.key);
            soma_definition
                .add_environment_variable(env_var.key, env_var.value)
                .await?;
        }
        EnvironmentVariableChangeEvt::Updated(env_var) => {
            info!("Environment variable updated: {:?}", env_var.key);
            soma_definition
                .update_environment_variable(env_var.key, env_var.value)
                .await?;
        }
        EnvironmentVariableChangeEvt::Deleted { id: _, key } => {
            info!("Environment variable deleted: {:?}", key);
            soma_definition.remove_environment_variable(key).await?;
        }
    }
    Ok(())
}
