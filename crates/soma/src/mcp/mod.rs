//! MCP sync functionality for the Soma CLI.
//! This module contains the logic for syncing between the database and soma.yaml file.

use std::path::PathBuf;
use std::sync::Arc;

use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;
use soma_api_server::logic::on_change_pubsub::SomaChangeRx;
use tracing::debug;

pub mod sync_to_yaml_on_mcp_change;
pub mod sync_yaml_to_api_on_start;

/// Runs the MCP sync to YAML loop - listens for MCP config changes and syncs to soma.yaml
pub async fn run_mcp_sync_to_yaml_loop(
    soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    project_dir: PathBuf,
    soma_change_rx: SomaChangeRx,
) -> Result<(), CommonError> {
    use crate::mcp::sync_to_yaml_on_mcp_change::sync_on_soma_change;

    debug!("MCP sync to YAML loop started");
    sync_on_soma_change(soma_change_rx, soma_definition, project_dir).await
}
