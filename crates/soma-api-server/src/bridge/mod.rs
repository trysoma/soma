use bridge::logic::{EnvelopeEncryptionKeyContents, OnConfigChangeTx};
use shared::{error::CommonError, subsystem::SubsystemHandle};
use tracing::error;

use crate::bridge::connection_manager::{StartMcpConnectionManagerParams, start_mcp_connection_manager};

pub mod connection_manager;
pub mod providers;



pub fn start_credential_rotation_subsystem(
    bridge_repo: bridge::repository::Repository,
    envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    on_bridge_change_tx: OnConfigChangeTx,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("Credential Rotation");

    tokio::spawn(async move {
        bridge::logic::credential_rotation_task(
            bridge_repo,
            envelope_encryption_key_contents,
            on_bridge_change_tx,
        )
        .await;
        signal.signal_with_message("stopped gracefully");
    });

    Ok(handle)
}


pub fn start_mcp_subsystem(
    bridge_service: bridge::router::bridge::BridgeService,
    mcp_transport_rx: tokio::sync::mpsc::UnboundedReceiver<rmcp::transport::sse_server::SseServerTransport>,
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {

    let (handle, signal) = SubsystemHandle::new("MCP");

    tokio::spawn(async move {
        match start_mcp_connection_manager(StartMcpConnectionManagerParams {
            bridge_service,
            mcp_transport_rx,
            system_shutdown_signal_rx: shutdown_rx,
        })
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("MCP connection manager stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}


