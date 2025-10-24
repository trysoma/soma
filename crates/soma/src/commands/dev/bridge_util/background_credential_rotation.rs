use bridge::logic::{EnvelopeEncryptionKeyContents, OnConfigChangeTx};
use shared::error::CommonError;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::info;

/// Starts the credential rotation subsystem
pub fn start_bridge_background_credential_rotation_subsystem(
    subsys: &SubsystemHandle,
    bridge_repo: bridge::repository::Repository,
    envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    on_bridge_config_change_tx: OnConfigChangeTx,
) {
    subsys.start(SubsystemBuilder::new(
        "bridge-background-credential-rotation",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                },
                _ = bridge::logic::credential_rotation_task(bridge_repo, envelope_encryption_key_contents, on_bridge_config_change_tx) => {
                    info!("Bridge credential rotator stopped");
                    subsys.request_shutdown();
                }
            }
            Ok::<(), CommonError>(())
        },
    ));
}
