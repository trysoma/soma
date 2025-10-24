mod deployment_subsystem;
mod mcp_subsystem;
mod runtime_subsystem;
mod server_subsystem;

pub use deployment_subsystem::{start_deployment_subsystem, StartDeploymentSubsystemParams};
pub use mcp_subsystem::{start_mcp_transport_processor_subsystem, StartMcpTransportProcessorParams};
pub use runtime_subsystem::{start_runtime_subsystem, StartRuntimeSubsystemParams};
pub use server_subsystem::{start_axum_subsystem, StartAxumSubsystemParams};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bridge::logic::EnvelopeEncryptionKeyContents;
use bridge::logic::OnConfigChangeTx;
use tokio::sync::oneshot;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{error, info};

use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;

use crate::logic::ConnectionManager;
use crate::repository::Repository;
use crate::router;
use crate::utils::config::CliConfig;

use super::project_file_watcher::{FileChangeTx, FileChangeRx, on_soma_config_change};
use super::restate::RestateServerParams;

/// Parameters for managing restartable processes
#[derive(Clone)]
pub struct DevReloaderSubsystemParams {
    pub host: String,
    pub port: u16,
    pub runtime: super::runtime::Runtime,
    pub runtime_port: u16,
    pub prj_file_change_tx: Arc<FileChangeTx>,
    pub project_dir: PathBuf,
    pub connection_manager: ConnectionManager,
    pub repository: Repository,
    pub db_connection: shared::libsql::Connection,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    pub on_bridge_config_change_tx: OnConfigChangeTx,
    pub bridge_repository: bridge::repository::Repository,
    pub restate_client_params: RestateServerParams,
}

/// Starts restartable processes and restarts them when soma.yaml changes
pub async fn start_dev_reloader_subsystem(
    subsys: &SubsystemHandle,
    params: DevReloaderSubsystemParams,
) -> Result<(), CommonError> {
    loop {
        let params_clone = params.clone();
        let mut file_change_rx: FileChangeRx = params_clone.prj_file_change_tx.subscribe();

        info!("🔁  starting system after config change");
        let (restart_tx, restart_rx) = oneshot::channel::<()>();
        subsys.start(SubsystemBuilder::new(
            "restartable-processes",
            move |subsys: SubsystemHandle| async move {
                tokio::select! {
                    _ = on_soma_config_change(&mut file_change_rx) => {
                        info!("Soma config changed");
                        subsys.request_local_shutdown();
                        subsys.wait_for_children().await;
                        info!("Restartable processes on config change stopped");
                        let _ = restart_tx.send(());
                    }
                    result = reload_dev_processes(&subsys, params_clone) => {
                        if let Err(e) = result {
                            error!("Restartable processes stopped unexpectedly: {:?}", e);
                        }
                        info!("Processes exited unexpectedly, something went wrong.");
                        // Don't restart on unexpected exit - request global shutdown
                        subsys.request_shutdown();
                        return Ok(());
                    }
                };

                Ok::<(), CommonError>(())
            },
        ));
        match restart_rx.await {
            Ok(_) => {
                // Config changed, restart the processes
                // Wait for the previous subsystem to fully terminate before starting a new one
                // This prevents multiple instances from trying to bind to the same ports
                subsys.wait_for_children().await;
            }
            Err(_) => {
                // Sender was dropped without sending (subsystem exited unexpectedly)
                // This means the subsystem already requested shutdown, so we break the loop
                info!("Restart signal dropped, breaking restart loop");
                break;
            }
        }
    }
    Ok(())
}

/// Starts all restartable processes (server, runtime, MCP, restate deployment)
async fn reload_dev_processes(
    subsys: &SubsystemHandle,
    params: DevReloaderSubsystemParams,
) -> Result<(), CommonError> {
    let DevReloaderSubsystemParams {
        runtime,
        runtime_port,
        prj_file_change_tx,
        project_dir,
        connection_manager,
        repository,
        db_connection,
        soma_definition,
        envelope_encryption_key_contents,
        on_bridge_config_change_tx,
        bridge_repository,
        host,
        port,
        restate_client_params,
    } = params;

    soma_definition.reload().await?;

    let (mcp_transport_tx, mcp_transport_rx) = tokio::sync::mpsc::unbounded_channel();

    let routers = router::Routers::new(
        router::InitRouterParams {
            project_dir: project_dir.clone(),
            host: host.clone(),
            port,
            connection_manager: connection_manager.clone(),
            repository: repository.clone(),
            mcp_transport_tx,
            soma_definition: soma_definition.clone(),
            runtime_port,
            restate_ingress_client: restate_client_params.get_ingress_client()?,
            db_connection: db_connection.clone(),
            on_bridge_config_change_tx,
            envelope_encryption_key_contents: envelope_encryption_key_contents.clone(),
            bridge_repository: bridge_repository.clone(),
            mcp_sse_ping_interval: Duration::from_secs(10),
        },
    ).await?;

    let (on_server_started_tx, on_server_started_rx) = oneshot::channel::<()>();

    // Start Axum subsystem
    start_axum_subsystem(
        subsys,
        StartAxumSubsystemParams {
            routers: routers.clone(),
            project_dir: project_dir.clone(),
            host: host.clone(),
            port,
        },
        on_server_started_tx,
    );

    // Start MCP transport processor subsystem
    start_mcp_transport_processor_subsystem(
        subsys,
        StartMcpTransportProcessorParams {
            bridge_service: routers.bridge_service.clone(),
            mcp_transport_rx,
        },
    );

    info!("Waiting for server to start, before starting runtime");
    on_server_started_rx.await?;

    // Start runtime subsystem
    start_runtime_subsystem(
        subsys,
        StartRuntimeSubsystemParams {
            project_dir: &project_dir,
            runtime: &runtime,
            runtime_port,
            file_change_tx: &prj_file_change_tx,
        },
    );

    info!("Starting Restate deployment");

    // Determine deployment type and service path
    let service_uri = format!("http://{host}:{runtime_port}");
    let deployment_type = crate::utils::restate::deploy::DeploymentType::Http {
        uri: service_uri.clone(),
        additional_headers: std::collections::HashMap::new(),
    };
    let service_path = soma_definition.get_definition().await?.project.clone();

    // Start deployment subsystem
    start_deployment_subsystem(
        subsys,
        StartDeploymentSubsystemParams {
            restate_params: &restate_client_params,
            deployment_type,
            service_path,
        },
    );

    subsys.on_shutdown_requested().await;

    Ok(())
}
