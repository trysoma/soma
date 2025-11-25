use bridge::logic::OnConfigChangeEvt as BridgeOnConfigChangeEvt;
use encryption::logic::EncryptionKeyEvent;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Re-export bridge events as BridgeEvt
pub type BridgeEvt = BridgeOnConfigChangeEvt;

/// Re-export encryption events as EncryptionEvt
pub type EncryptionEvt = EncryptionKeyEvent;

/// Unified change event for all Soma services
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SomaChangeEvt {
    Bridge(BridgeEvt),
    Encryption(EncryptionEvt),
}

// Type aliases for the broadcast channel
pub type SomaChangeTx = broadcast::Sender<SomaChangeEvt>;
pub type SomaChangeRx = broadcast::Receiver<SomaChangeEvt>;

/// Creates a new SomaChange broadcast channel and returns the sender
pub fn create_soma_change_channel(capacity: usize) -> (SomaChangeTx, SomaChangeRx) {
    broadcast::channel(capacity)
}

/// Starts the unified change pubsub system that forwards bridge and encryption events to the unified channel
pub async fn run_change_pubsub(
    soma_change_tx: SomaChangeTx,
    mut bridge_change_rx: bridge::logic::OnConfigChangeRx,
    mut encryption_change_rx: encryption::logic::EncryptionKeyEventReceiver,
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
