use bridge::logic::OnConfigChangeEvt as BridgeOnConfigChangeEvt;
use encryption::logic::EncryptionKeyEvent;
use identity::logic::OnConfigChangeEvt as IdentityOnConfigChangeEvt;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::logic::environment_variable::EnvironmentVariable;
use crate::logic::secret::Secret;

/// Re-export bridge events as BridgeEvt
pub type BridgeEvt = BridgeOnConfigChangeEvt;

/// Re-export encryption events as EncryptionEvt
pub type EncryptionEvt = EncryptionKeyEvent;

/// Re-export identity events as IdentityEvt
pub type IdentityEvt = IdentityOnConfigChangeEvt;

/// Secret change events
#[derive(Clone, Debug)]
pub enum SecretChangeEvt {
    Created(Secret),
    Updated(Secret),
    Deleted { id: String, key: String },
}

/// Type aliases for the secret event broadcast channel
pub type SecretChangeTx = broadcast::Sender<SecretChangeEvt>;
pub type SecretChangeRx = broadcast::Receiver<SecretChangeEvt>;

/// Creates a new SecretChange broadcast channel
pub fn create_secret_change_channel(capacity: usize) -> (SecretChangeTx, SecretChangeRx) {
    broadcast::channel(capacity)
}

/// Environment variable change events
#[derive(Clone, Debug)]
pub enum EnvironmentVariableChangeEvt {
    Created(EnvironmentVariable),
    Updated(EnvironmentVariable),
    Deleted { id: String, key: String },
}

/// Type aliases for the environment variable event broadcast channel
pub type EnvironmentVariableChangeTx = broadcast::Sender<EnvironmentVariableChangeEvt>;
pub type EnvironmentVariableChangeRx = broadcast::Receiver<EnvironmentVariableChangeEvt>;

/// Creates a new EnvironmentVariableChange broadcast channel
pub fn create_environment_variable_change_channel(
    capacity: usize,
) -> (EnvironmentVariableChangeTx, EnvironmentVariableChangeRx) {
    broadcast::channel(capacity)
}

/// Unified change event for all Soma services
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SomaChangeEvt {
    Bridge(BridgeEvt),
    Encryption(EncryptionEvt),
    Secret(SecretChangeEvt),
    EnvironmentVariable(EnvironmentVariableChangeEvt),
    Identity(IdentityEvt),
}

// Type aliases for the broadcast channel
pub type SomaChangeTx = broadcast::Sender<SomaChangeEvt>;
pub type SomaChangeRx = broadcast::Receiver<SomaChangeEvt>;

/// Creates a new SomaChange broadcast channel and returns the sender
pub fn create_soma_change_channel(capacity: usize) -> (SomaChangeTx, SomaChangeRx) {
    broadcast::channel(capacity)
}

/// Starts the unified change pubsub system that forwards bridge, encryption, secret, environment variable, and identity events to the unified channel
pub async fn run_change_pubsub(
    soma_change_tx: SomaChangeTx,
    mut bridge_change_rx: bridge::logic::OnConfigChangeRx,
    mut encryption_change_rx: encryption::logic::EncryptionKeyEventReceiver,
    mut secret_change_rx: SecretChangeRx,
    mut environment_variable_change_rx: EnvironmentVariableChangeRx,
    mut identity_change_rx: identity::logic::OnConfigChangeRx,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    info!("Starting unified change pubsub system");

    loop {
        tokio::select! {
            // Forward bridge events
            event = bridge_change_rx.recv() => {
                match event {
                    Ok(bridge_evt) => {
                        let soma_evt = SomaChangeEvt::Bridge(bridge_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::debug!("No receivers for bridge SomaChangeEvt: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Bridge change channel closed, stopping change pubsub");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Bridge change channel lagged, skipped {} messages", skipped);
                    }
                }
            }
            // Forward encryption events
            event = encryption_change_rx.recv() => {
                match event {
                    Ok(encryption_evt) => {
                        let soma_evt = SomaChangeEvt::Encryption(encryption_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::debug!("No receivers for encryption SomaChangeEvt: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Encryption change channel closed, stopping change pubsub");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Encryption change channel lagged, skipped {} messages", skipped);
                    }
                }
            }
            // Forward secret events
            event = secret_change_rx.recv() => {
                match event {
                    Ok(secret_evt) => {
                        let soma_evt = SomaChangeEvt::Secret(secret_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::debug!("No receivers for secret SomaChangeEvt: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Secret change channel closed, stopping change pubsub");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Secret change channel lagged, skipped {} messages", skipped);
                    }
                }
            }
            // Forward environment variable events
            event = environment_variable_change_rx.recv() => {
                match event {
                    Ok(env_var_evt) => {
                        let soma_evt = SomaChangeEvt::EnvironmentVariable(env_var_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::debug!("No receivers for environment variable SomaChangeEvt: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Environment variable change channel closed, stopping change pubsub");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Environment variable change channel lagged, skipped {} messages", skipped);
                    }
                }
            }
            // Forward identity events
            event = identity_change_rx.recv() => {
                match event {
                    Ok(identity_evt) => {
                        let soma_evt = SomaChangeEvt::Identity(identity_evt);
                        if let Err(e) = soma_change_tx.send(soma_evt) {
                            tracing::debug!("No receivers for identity SomaChangeEvt: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Identity change channel closed, stopping change pubsub");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Identity change channel lagged, skipped {} messages", skipped);
                    }
                }
            }
            // Handle shutdown
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, stopping change pubsub");
                break;
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

/// Helper to broadcast environment variable events through the unified channel
pub fn broadcast_environment_variable_event(
    tx: &SomaChangeTx,
    event: EnvironmentVariableChangeEvt,
) {
    let soma_evt = SomaChangeEvt::EnvironmentVariable(event);
    if let Err(e) = tx.send(soma_evt) {
        tracing::debug!("No receivers for environment variable event: {:?}", e);
    }
}

/// Helper to broadcast identity events through the unified channel
pub fn broadcast_identity_event(tx: &SomaChangeTx, event: IdentityEvt) {
    let soma_evt = SomaChangeEvt::Identity(event);
    if let Err(e) = tx.send(soma_evt) {
        tracing::debug!("No receivers for identity event: {:?}", e);
    }
}
