// Bridge sync functionality for the Soma CLI.
// This module contains the logic for syncing between the database and soma.yaml file.

use std::path::PathBuf;
use std::sync::Arc;

use bridge::logic::OnConfigChangeTx;
use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;
use shared::subsystem::SubsystemHandle;
use tracing::error;

pub mod sync_to_yaml_on_bridge_change;
pub mod sync_yaml_to_api_on_start;

pub fn start_bridge_sync_to_yaml_subsystem(
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    project_dir: PathBuf,
) -> Result<(OnConfigChangeTx, SubsystemHandle), CommonError> {
    use crate::bridge::sync_to_yaml_on_bridge_change::start_sync_on_bridge_change;

    let (on_bridge_change_tx, on_bridge_change_fut) =
        start_sync_on_bridge_change(soma_definition, project_dir)?;

    let (handle, signal) = SubsystemHandle::new("Bridge Sync");

    tokio::spawn(async move {
        match on_bridge_change_fut.await {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("Bridge config change listener stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok((on_bridge_change_tx, handle))
}
