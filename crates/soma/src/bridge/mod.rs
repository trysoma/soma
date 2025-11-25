// Bridge sync functionality for the Soma CLI.
// This module contains the logic for syncing between the database and soma.yaml file.

use std::path::PathBuf;
use std::sync::Arc;

use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;
use shared::subsystem::SubsystemHandle;
use soma_api_server::logic::on_change_pubsub::SomaChangeRx;
use tracing::error;

pub mod sync_to_yaml_on_bridge_change;
pub mod sync_yaml_to_api_on_start;

pub fn start_bridge_sync_to_yaml_subsystem(
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    project_dir: PathBuf,
    soma_change_rx: SomaChangeRx,
) -> Result<SubsystemHandle, CommonError> {
    use crate::bridge::sync_to_yaml_on_bridge_change::sync_on_soma_change;

    let (handle, signal) = SubsystemHandle::new("Bridge Sync");

    tokio::spawn(async move {
        match sync_on_soma_change(soma_change_rx, soma_definition, project_dir).await {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("Bridge config change listener stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}
