use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use bridge::logic::{OnConfigChangeEvt, OnConfigChangeRx, OnConfigChangeTx};
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{info, warn};

use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;

/// Watches for bridge configuration changes and updates soma.yaml accordingly
pub async fn sync_on_bridge_change(
    mut on_bridge_config_change_rx: OnConfigChangeRx,
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    // sdk_runtime: SdkRuntime,
    _project_dir: PathBuf,
    // bridge_repo: Arc<Repository>,
) -> Result<(), CommonError> {
    loop {
        let event = match on_bridge_config_change_rx.recv().await {
            Ok(event) => event,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                info!("Bridge config change receiver closed");
                return Ok(());
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(
                    "Bridge config change receiver lagged, skipped {} messages",
                    skipped
                );
                continue;
            }
        };

        match event {
            OnConfigChangeEvt::ProviderInstanceAdded(provider_instance) => {
                info!(
                    "Provider instance added: {:?}",
                    provider_instance.provider_instance.id
                );

                // Only write to soma.yaml if the provider instance status is "active"
                // TODO: we cant do the above because we need to save function isntances before oauth callback is received
                // if provider_instance.provider_instance.status == "active" {
                let user_credential = provider_instance.user_credential.as_ref().map(|uc| {
                    shared::soma_agent_definition::CredentialConfig {
                        id: uc.id.to_string(),
                        type_id: uc.type_id.clone(),
                        metadata: json!(uc.metadata.0.clone()),
                        value: uc.value.get_inner().clone(),
                        next_rotation_time: uc.next_rotation_time.map(|t| t.to_string()),
                        data_encryption_key_id: uc.data_encryption_key_id.clone(),
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
                                    data_encryption_key_id: provider_instance
                                        .resource_server_credential
                                        .data_encryption_key_id
                                        .clone(),
                                },
                            user_credential,
                            functions: None,
                        },
                    )
                    .await?;
                // }
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
                        data_encryption_key_id: uc.data_encryption_key_id.clone(),
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
                                    data_encryption_key_id: provider_instance
                                        .resource_server_credential
                                        .data_encryption_key_id
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
            OnConfigChangeEvt::DataEncryptionKeyAdded(data_encryption_key) => {
                info!("Data encryption key added: {:?}", data_encryption_key.id);
                soma_definition
                    .add_data_encryption_key(
                        data_encryption_key.id,
                        data_encryption_key.encrypted_data_encryption_key.0,
                        match data_encryption_key.envelope_encryption_key_id {
                            bridge::logic::EnvelopeEncryptionKey::AwsKms { arn, region } => {
                                shared::soma_agent_definition::EnvelopeEncryptionKey::AwsKms {
                                    arn,
                                    region,
                                }
                            }
                            bridge::logic::EnvelopeEncryptionKey::Local { location } => {
                                shared::soma_agent_definition::EnvelopeEncryptionKey::Local {
                                    location,
                                }
                            }
                        },
                    )
                    .await?;
            }
            OnConfigChangeEvt::DataEncryptionKeyRemoved(data_encryption_key_id) => {
                soma_definition
                    .remove_data_encryption_key(data_encryption_key_id)
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
                // Remove the function instance from the provider
                soma_definition
                    .remove_function_instance(
                        provider_controller_type_id,
                        function_controller_type_id,
                        provider_instance_id,
                    )
                    .await?;
            }
        }
    }
}

/// Starts the bridge config change listener subsystem
pub fn start_sync_on_bridge_change(
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    // sdk_runtime: SdkRuntime,
    project_dir: PathBuf,
    // bridge_repo: Arc<Repository>,
) -> Result<
    (
        OnConfigChangeTx,
        impl Future<Output = Result<(), CommonError>> + Send,
    ),
    CommonError,
> {
    let (on_bridge_config_change_tx, on_bridge_config_change_rx) = broadcast::channel(100);

    let sync_on_bridge_change_fut = sync_on_bridge_change(
        on_bridge_config_change_rx,
        soma_definition,
        // sdk_runtime,
        project_dir,
        // bridge_repo,
    );

    Ok((on_bridge_config_change_tx, sync_on_bridge_change_fut))
}
