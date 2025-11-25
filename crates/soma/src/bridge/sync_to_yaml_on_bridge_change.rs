use std::path::PathBuf;
use std::sync::Arc;

use bridge::logic::OnConfigChangeEvt;
use encryption::logic::EncryptionKeyEvent;
use encryption::logic::envelope::EnvelopeEncryptionKey;
use serde_json::json;
use tracing::{info, warn};

use shared::error::CommonError;
use shared::soma_agent_definition::{EnvelopeKeyConfig, SomaAgentDefinitionLike};
use soma_api_server::logic::on_change_pubsub::{SomaChangeEvt, SomaChangeRx};

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
                EnvelopeEncryptionKey::Local { location } => EnvelopeKeyConfig::Local {
                    location,
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
            info!("Data encryption key added: {:?}", dek.id);
            let envelope_key_id = dek.envelope_encryption_key_id.id();
            soma_definition
                .add_dek(envelope_key_id, dek.id, dek.encrypted_data_encryption_key.0)
                .await?;
        }
        EncryptionKeyEvent::DataEncryptionKeyRemoved(dek_id) => {
            info!("Data encryption key removed: {:?}", dek_id);
            // Note: We don't know which envelope key it belonged to without looking it up
            // For now, we'll need to search all envelope keys
            // TODO: Consider changing the event to include envelope_key_id
            warn!("DEK removed event doesn't include envelope_key_id - skipping yaml sync for now");
        }
        EncryptionKeyEvent::DataEncryptionKeyMigrated {
            old_dek_id,
            new_dek_id,
            from_envelope_key,
            to_envelope_key,
        } => {
            info!(
                "Data encryption key migrated: {:?} -> {:?}",
                old_dek_id, new_dek_id
            );
            // This is a complex operation - for now just log it
            // The new DEK should have been added via DataEncryptionKeyAdded event
        }
        EncryptionKeyEvent::DataEncryptionKeyAliasChanged => {
            info!("DEK alias changed");
            // This is a generic event - we'd need more info to update properly
            // Individual alias add/remove events would be more useful here
        }
    }
    Ok(())
}
