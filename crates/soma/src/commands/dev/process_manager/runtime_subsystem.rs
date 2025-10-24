use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{broadcast, oneshot};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{error, info};

use shared::error::CommonError;

use crate::commands::dev::project_file_watcher::{FileChangeTx, FileChangeRx};
use crate::commands::dev::runtime::{Runtime, start_dev_runtime, StartDevRuntimeParams};

pub struct StartRuntimeSubsystemParams<'a> {
    pub project_dir: &'a PathBuf,
    pub runtime: &'a Runtime,
    pub runtime_port: u16,
    pub file_change_tx: &'a Arc<FileChangeTx>,
}

/// Starts the runtime subsystem with hot reload support
pub fn start_runtime_subsystem(
    subsys: &SubsystemHandle,
    params: StartRuntimeSubsystemParams,
) {
    let project_dir = params.project_dir.clone();
    let runtime = params.runtime.clone();
    let runtime_port = params.runtime_port;
    let mut file_change_rx = params.file_change_tx.subscribe();

    let (kill_runtime_signal_trigger, kill_runtime_signal_receiver) = broadcast::channel::<()>(1);
    let (shutdown_runtime_complete_signal_trigger, shutdown_runtime_complete_signal_receiver) =
        oneshot::channel::<()>();

    subsys.start(SubsystemBuilder::new(
        "runtime",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    // Ignore channel errors - child may have already exited
                    let _ = kill_runtime_signal_trigger.send(());
                    let _ = shutdown_runtime_complete_signal_receiver.await;
                    info!("Runtime shutdown complete");
                }
                result = start_dev_runtime(
                    StartDevRuntimeParams {
                        project_dir,
                        runtime,
                        runtime_port,
                        file_change_signal: &mut file_change_rx,
                        kill_signal: kill_runtime_signal_receiver,
                        shutdown_complete_signal: shutdown_runtime_complete_signal_trigger,
                    }
                ) => {
                    if let Err(e) = result {
                        error!("Runtime stopped unexpectedly: {:?}", e);
                    }
                    info!("Runtime stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));
}
