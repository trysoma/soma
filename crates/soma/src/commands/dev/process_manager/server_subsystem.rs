use std::path::PathBuf;

use tokio::sync::oneshot;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::info;

use shared::error::CommonError;

use crate::commands::dev::server::StartAxumServerParams;
use crate::router;

#[cfg(debug_assertions)]
use crate::commands::dev::server::{start_vite_dev_server, stop_vite_dev_server};
use crate::commands::dev::server::start_axum_server;

pub struct StartAxumSubsystemParams {
    pub routers: router::Routers,
    pub project_dir: PathBuf,
    pub host: String,
    pub port: u16,
}

/// Starts the Axum web server subsystem with optional Vite dev server
pub fn start_axum_subsystem(
    subsys: &SubsystemHandle,
    params: StartAxumSubsystemParams,
    on_server_started_tx: oneshot::Sender<()>,
) {
    let StartAxumSubsystemParams {
        routers,
        project_dir,
        host,
        port,
    } = params;

    subsys.start(SubsystemBuilder::new(
        "axum-server",
        move |subsys: SubsystemHandle| async move {
            #[cfg(debug_assertions)]
            let _vite_scope_guard = start_vite_dev_server();

            let (server_fut, handle, addr) =
                start_axum_server(StartAxumServerParams {
                    routers,
                    project_dir,
                    host,
                    port,
                }).await?;

            let _ = on_server_started_tx.send(());

            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("Shutting down axum server");
                    #[cfg(debug_assertions)]
                    {
                        drop(_vite_scope_guard);
                        stop_vite_dev_server().await?;
                    }
                    handle.shutdown();
                    info!("Axum server shut down");
                }
                _ = server_fut => {
                    info!("Axum server stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));
}
