use encryption::logic::EncryptionKeyEvent;
use identity::logic::OnConfigChangeEvt as IdentityOnConfigChangeEvt;
use tool::logic::OnConfigChangeEvt as McpOnConfigChangeEvt;
use tokio::sync::broadcast;
use tracing::{debug, warn};

// Re-export environment crate types for change events
pub use environment::logic::secret::{
    SecretChangeEvt, SecretChangeRx, SecretChangeTx, create_secret_change_channel,
};
pub use environment::logic::variable::{
    VariableChangeEvt, VariableChangeRx, VariableChangeTx, create_variable_change_channel,
};

/// Re-export mcp events as McpEvt
pub type McpEvt = McpOnConfigChangeEvt;

/// Re-export encryption events as EncryptionEvt
pub type EncryptionEvt = EncryptionKeyEvent;

/// Re-export identity events as IdentityEvt
pub type IdentityEvt = IdentityOnConfigChangeEvt;

/// Unified change event for all Soma services
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SomaChangeEvt {
    Mcp(McpEvt),
    Encryption(EncryptionEvt),
    Secret(SecretChangeEvt),
    Variable(VariableChangeEvt),
    Identity(IdentityEvt),
}

// Type aliases for the broadcast channel
pub type SomaChangeTx = broadcast::Sender<SomaChangeEvt>;
pub type SomaChangeRx = broadcast::Receiver<SomaChangeEvt>;

/// Creates a new SomaChange broadcast channel and returns the sender
pub fn create_soma_change_channel(capacity: usize) -> (SomaChangeTx, SomaChangeRx) {
    broadcast::channel(capacity)
}

/// Starts the unified change pubsub system that forwards mcp, encryption, secret, variable, and identity events to the unified channel.
/// This function runs indefinitely until aborted by the process manager.
pub async fn run_change_pubsub(
    soma_change_tx: SomaChangeTx,
    mut mcp_change_rx: tool::logic::OnConfigChangeRx,
    mut encryption_change_rx: encryption::logic::EncryptionKeyEventReceiver,
    mut secret_change_rx: SecretChangeRx,
    mut variable_change_rx: VariableChangeRx,
    mut identity_change_rx: identity::logic::OnConfigChangeRx,
) {
    debug!("Change pubsub system started");

    loop {
        tokio::select! {
            // Forward mcp events
            event = mcp_change_rx.recv() => {
                match event {
                    Ok(mcp_evt) => {
                        let soma_evt = SomaChangeEvt::Mcp(mcp_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::trace!("No receivers for mcp event: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("MCP change channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "MCP change channel lagged");
                    }
                }
            }
            // Forward encryption events
            event = encryption_change_rx.recv() => {
                match event {
                    Ok(encryption_evt) => {
                        let soma_evt = SomaChangeEvt::Encryption(encryption_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::trace!("No receivers for encryption event: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Encryption change channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "Encryption change channel lagged");
                    }
                }
            }
            // Forward secret events
            event = secret_change_rx.recv() => {
                match event {
                    Ok(secret_evt) => {
                        let soma_evt = SomaChangeEvt::Secret(secret_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::trace!("No receivers for secret event: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Secret change channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "Secret change channel lagged");
                    }
                }
            }
            // Forward variable events
            event = variable_change_rx.recv() => {
                match event {
                    Ok(var_evt) => {
                        let soma_evt = SomaChangeEvt::Variable(var_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::trace!("No receivers for variable event: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Variable change channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "Variable change channel lagged");
                    }
                }
            }
            // Forward identity events
            event = identity_change_rx.recv() => {
                match event {
                    Ok(identity_evt) => {
                        let soma_evt = SomaChangeEvt::Identity(identity_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::trace!("No receivers for identity event: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Identity change channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "Identity change channel lagged");
                    }
                }
            }
        }
    }
}

/// Helper to broadcast encryption events through the unified channel
pub fn broadcast_encryption_event(tx: &SomaChangeTx, event: EncryptionEvt) {
    let soma_evt = SomaChangeEvt::Encryption(event);
    if let Err(e) = tx.send(soma_evt) {
        tracing::debug!("No receivers for encryption event: {:?}", e);
    }
}

/// Helper to broadcast secret events through the unified channel
pub fn broadcast_secret_event(tx: &SomaChangeTx, event: SecretChangeEvt) {
    let soma_evt = SomaChangeEvt::Secret(event);
    if let Err(e) = tx.send(soma_evt) {
        tracing::debug!("No receivers for secret event: {:?}", e);
    }
}

/// Helper to broadcast variable events through the unified channel
pub fn broadcast_variable_event(tx: &SomaChangeTx, event: VariableChangeEvt) {
    let soma_evt = SomaChangeEvt::Variable(event);
    if let Err(e) = tx.send(soma_evt) {
        tracing::debug!("No receivers for variable event: {:?}", e);
    }
}

/// Helper to broadcast identity events through the unified channel
pub fn broadcast_identity_event(tx: &SomaChangeTx, event: IdentityEvt) {
    let soma_evt = SomaChangeEvt::Identity(event);
    if let Err(e) = tx.send(soma_evt) {
        tracing::debug!("No receivers for identity event: {:?}", e);
    }
}
