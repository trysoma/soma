use std::path::PathBuf;
use std::sync::Arc;

use bridge::logic::OnConfigChangeEvt;
use encryption::logic::EncryptionKeyEvent;
use encryption::logic::envelope::EnvelopeEncryptionKey;
use serde_json::json;
use tracing::{info, warn};

use shared::error::CommonError;
use shared::soma_agent_definition::{EnvelopeKeyConfig, SecretConfig, SomaAgentDefinitionLike};
use soma_api_server::logic::on_change_pubsub::{SecretChangeEvt, SomaChangeEvt, SomaChangeRx};

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
                EnvelopeEncryptionKey::AwsKms { arn, region } => EnvelopeKeyConfig::AwsKms {
                    arn,
                    region,
                    deks: None,
                },
                EnvelopeEncryptionKey::Local { file_name } => EnvelopeKeyConfig::Local {
                    file_name,
                    deks: None,
                },
            };
            soma_definition.add_envelope_key(key_id, config).await?;
        }
        EncryptionKeyEvent::EnvelopeEncryptionKeyRemoved(eek_id) => {
            info!("Envelope encryption key removed: {:?}", eek_id);
            soma_definition.remove_envelope_key(eek_id).await?;
        }
        EncryptionKeyEvent::DataEncryptionKeyAdded(dek) => {
            // Store DEK temporarily by its ID - it will be renamed to alias when alias is added
            info!("Data encryption key added: {:?}", dek.id);
            let envelope_key_id = dek.envelope_encryption_key_id.id();
            soma_definition
                .add_dek(envelope_key_id, dek.id, dek.encrypted_data_encryption_key.0)
                .await?;
        }
        EncryptionKeyEvent::DataEncryptionKeyRemoved(dek_id) => {
            info!("Data encryption key removed: {:?}", dek_id);
            // Search all envelope keys to find and remove this DEK
            // DEKs might be stored by UUID or alias
            let definition = soma_definition.get_definition().await?;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    for (envelope_key_id, config) in envelope_keys {
                        if let Some(deks) = config.deks() {
                            // Check if stored as UUID
                            if deks.contains_key(&dek_id) {
                                soma_definition
                                    .remove_dek(envelope_key_id.clone(), dek_id.clone())
                                    .await?;
                                break;
                            }
                        }
                    }
                }
            }
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
        EncryptionKeyEvent::DataEncryptionKeyAliasAdded { alias, dek_id } => {
            // Rename the DEK from its UUID to its alias in the YAML
            // We need to find which envelope key contains this DEK
            info!(
                "DEK alias added: {:?} -> {:?} - renaming DEK key in yaml",
                alias, dek_id
            );
            let definition = soma_definition.get_definition().await?;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    for (envelope_key_id, config) in envelope_keys {
                        if let Some(deks) = config.deks() {
                            if deks.contains_key(&dek_id) {
                                soma_definition
                                    .rename_dek(
                                        envelope_key_id.clone(),
                                        dek_id.clone(),
                                        alias.clone(),
                                    )
                                    .await?;
                                break;
                            }
                        }
                    }
                }
            }
        }
        EncryptionKeyEvent::DataEncryptionKeyAliasRemoved { alias } => {
            info!("DEK alias removed: {:?}", alias);
            // When alias is removed, the DEK itself is typically also being removed
            // or migrated, so we don't need to do anything here
        }
        EncryptionKeyEvent::DataEncryptionKeyAliasUpdated { alias, dek_id } => {
            // Alias updated means the alias now points to a different DEK
            // We need to rename the new DEK to use this alias
            // Note: This event may fire before DataEncryptionKeyAdded during migration,
            // so if the DEK isn't found, we'll skip the rename and let DataEncryptionKeyMigrated handle it
            info!("DEK alias updated: {:?} -> {:?}", alias, dek_id);
            let definition = soma_definition.get_definition().await?;
            if let Some(encryption) = &definition.encryption {
                if let Some(envelope_keys) = &encryption.envelope_keys {
                    for (envelope_key_id, config) in envelope_keys {
                        if let Some(deks) = config.deks() {
                            // Check if DEK exists by UUID
                            if deks.contains_key(&dek_id) {
                                soma_definition
                                    .rename_dek(
                                        envelope_key_id.clone(),
                                        dek_id.clone(),
                                        alias.clone(),
                                    )
                                    .await?;
                                break;
                            }
                            // Also check if alias already exists (might have been renamed already)
                            if deks.contains_key(&alias) {
                                // Alias already exists - this is fine, skip
                                break;
                            }
                        }
                    }
                }
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
