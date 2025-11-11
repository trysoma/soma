mod interface;
mod typescript;
pub mod grpc_client;
pub mod sdk_provider_sync;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::TryFutureExt;
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

use shared::error::CommonError;

use crate::commands::dev::DevParams;
use crate::commands::dev::runtime::grpc_client::{create_unix_socket_client, establish_connection_with_retry, monitor_connection_health};
use crate::utils::construct_src_dir_absolute;

use super::project_file_watcher::FileChangeTx;
use interface::{ClientCtx, SdkClient};
use typescript::Typescript;

/// Default Unix socket path for the SDK gRPC server
pub const DEFAULT_SOMA_SERVER_SOCK: &str = "/tmp/soma-sdk.sock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    PnpmV1,
}

/// Determines which runtime to use based on the project structure
pub fn determine_runtime(params: &DevParams) -> Result<Option<Runtime>, CommonError> {
    let src_dir = construct_src_dir_absolute(params.src_dir.clone())?;
    determine_runtime_from_dir(&src_dir)
}

/// Determines runtime from a directory path (testable version)
pub fn determine_runtime_from_dir(src_dir: &Path) -> Result<Option<Runtime>, CommonError> {
    let possible_runtimes = vec![(Runtime::PnpmV1, validate_runtime_pnpm_v1)];

    let mut matched_runtimes = vec![];

    for (runtime, validate_fn) in possible_runtimes {
        let result = validate_fn(src_dir.to_path_buf())?;
        if result {
            matched_runtimes.push(runtime);
        }
    }

    match matched_runtimes.len() {
        0 => Ok(None),
        1 => Ok(Some(matched_runtimes[0].clone())),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Multiple runtimes matched"
        ))),
    }
}

fn validate_runtime_pnpm_v1(src_dir: PathBuf) -> Result<bool, CommonError> {
    let files_to_check = vec![
        "package.json",
        "vite.config.ts",
    ];
    for file in files_to_check {
        let file_path = src_dir.join(file);
        if !file_path.exists() {
            return Ok(false);
        }
    }
    Ok(true)
}


/// Check if the project uses Vite by looking for vite.config.ts
fn is_vite_project(src_dir: &Path) -> bool {
    src_dir.join("vite.config.ts").exists()
}


pub struct StartDevRuntimeParams {
    pub project_dir: PathBuf,
    pub runtime: Runtime,
    pub runtime_port: u16,
    pub file_change_tx: Arc<FileChangeTx>,
    pub kill_signal_rx: broadcast::Receiver<()>,
}

/// Starts the development runtime with hot reloading on file changes
pub async fn start_dev_runtime(
    params: StartDevRuntimeParams,
) -> Result<(), CommonError> {
    let StartDevRuntimeParams {
        project_dir,
        runtime: _runtime,
        runtime_port,
        file_change_tx,
        kill_signal_rx,
    } = params;

    let typescript_client = Typescript::new();
    let ctx = ClientCtx {
        project_dir: project_dir.clone(),
        socket_path: DEFAULT_SOMA_SERVER_SOCK.to_string(),
        restate_runtime_port: runtime_port,
        file_change_tx: file_change_tx.clone(),
        kill_signal_rx: kill_signal_rx.resubscribe(),
    };

    if !is_vite_project(&project_dir) {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Invalid runtime. Must use Vite"
        )));
    }

    info!("Detected Vite project, starting dev server...");
    typescript_client.start_dev_server(ctx).await?;

    Ok(())
}

/// Fetch metadata and sync providers to the bridge registry
/// Returns the list of agents from the metadata response
async fn fetch_and_sync_providers(
    socket_path: &str,
) -> Result<Vec<sdk_proto::Agent>, CommonError> {
    let mut client = create_unix_socket_client(socket_path).await?;

    let request = tonic::Request::new(());
    let response = client
        .metadata(request)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("gRPC call failed: {e}")))?;

    let metadata = response.into_inner();

    info!("=== SDK Metadata ===");
    info!("Provider count: {}", metadata.bridge_providers.len());

    for (i, provider) in metadata.bridge_providers.iter().enumerate() {
        info!("Provider {}: type_id={}, name={}", i + 1, provider.type_id, provider.name);
        info!("  Function count: {}", provider.functions.len());

        for (j, func) in provider.functions.iter().enumerate() {
            info!("    Function {}: {}", j + 1, func.name);
        }
    }

    info!("Agent count: {}", metadata.agents.len());
    for (i, agent) in metadata.agents.iter().enumerate() {
        info!("Agent {}: id={}, name={}", i + 1, agent.id, agent.name);
    }

    info!("=== End SDK Metadata ===");

    // Sync providers to bridge registry
    sdk_provider_sync::sync_providers_from_metadata(&metadata)?;

    Ok(metadata.agents)
}

/// Register Restate deployments for all agents
async fn register_agent_deployments(
    agents: Vec<sdk_proto::Agent>,
    restate_params: &super::restate::RestateServerParams,
    runtime_port: u16,
) -> Result<(), CommonError> {
    use std::collections::HashMap;

    info!("Registering {} agent deployment(s) with Restate", agents.len());

    for agent in agents {
        let service_uri = format!("http://127.0.0.1:{runtime_port}");
        let deployment_type = crate::utils::restate::deploy::DeploymentType::Http {
            uri: service_uri.clone(),
            additional_headers: HashMap::new(),
        };

        // Use the project_id.agent_id format as the service path (matches Restate service name)
        let service_path = format!("{}.{}", agent.project_id, agent.id);

        info!("Registering agent '{}' at {}", agent.name, service_uri);

        let admin_url = restate_params.get_admin_address()?;
        let config = crate::utils::restate::deploy::DeploymentRegistrationConfig {
            admin_url: admin_url.to_string(),
            service_path: service_path.clone(),
            deployment_type,
            bearer_token: restate_params.get_admin_token(),
            private: restate_params.get_private(),
            insecure: restate_params.get_insecure(),
            force: restate_params.get_force(),
        };

        match crate::utils::restate::deploy::register_deployment(config).await {
            Ok(metadata) => {
                info!("✓ Successfully registered agent '{}' (service: {})", agent.name, metadata.name);
            }
            Err(e) => {
                error!("✗ Failed to register agent '{}': {:?}", agent.name, e);
                // Continue with other agents even if one fails
            }
        }
    }

    Ok(())
}

pub struct SyncDevRuntimeChangesFromSdkServerParams {
    pub socket_path: String,
    pub restate_params: super::restate::RestateServerParams,
    pub runtime_port: u16,
    pub system_shutdown_signal_rx: broadcast::Receiver<()>,
}

/// Watch for dev runtime reloads by monitoring the gRPC connection
/// This function runs indefinitely, reconnecting when the server restarts
/// and syncing providers on each reconnection
pub async fn sync_dev_runtime_changes_from_sdk_server(
    params: SyncDevRuntimeChangesFromSdkServerParams,
) -> Result<(), CommonError> {
    let SyncDevRuntimeChangesFromSdkServerParams {
        socket_path,
        restate_params,
        runtime_port,
        mut system_shutdown_signal_rx,
    } = params;


    let (sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_trigger, sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_receiver) = oneshot::channel::<CommonError>();
    let (system_shutdown_signal_tx_clone, system_shutdown_signal_rx_clone_receiver) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        tokio::select! {
            _ = system_shutdown_signal_rx_clone_receiver => {
            }
            _ = internal_sync_dev_runtime_changes_from_sdk_server_loop(socket_path, restate_params, runtime_port, sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_trigger) => {
            }
        }
    });

    tokio::select! {
        _ = system_shutdown_signal_rx.recv() => {
            info!("SDK reload watcher shutdown requested");
            let _ = system_shutdown_signal_tx_clone.send(());

            return Ok(());
        }
        result = sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_receiver => {
            match result {
                Ok(err) => {
                    Err(err)
                }
                Err(e) => {
                    error!("SDK channel closed unexpectedly: {:?}", e);
                    Err(CommonError::Unknown(anyhow::anyhow!("SDK channel closed unexpectedly: {e:?}")))
                }
            }
        }
    }
}

pub async fn internal_sync_dev_runtime_changes_from_sdk_server_loop(
    socket_path: String,
    restate_params: super::restate::RestateServerParams,
    runtime_port: u16,
    sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_trigger: oneshot::Sender<CommonError>,
) {
    info!("Starting dev runtime reload watcher for socket: {}", socket_path);
    let mut ticker = tokio::time::interval(Duration::from_millis(500));
    
    loop {
        // Try to establish connection with timeout
        let connection_result = tokio::time::timeout(
            Duration::from_secs(10),
            establish_connection_with_retry(&socket_path)
        ).await;

        match connection_result {
            Ok(Ok(_)) => {
                info!("📡 Connected to SDK server, fetching metadata and syncing providers...");

                // Fetch metadata and sync providers to bridge
                match fetch_and_sync_providers(&socket_path).await {
                    Ok(agents) => {
                        // Register Restate deployments for each agent
                        if !agents.is_empty() {
                            if let Err(e) = register_agent_deployments(agents, &restate_params, runtime_port).await {
                                error!("Failed to register agent deployments: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch and sync providers: {:?}", e);
                    }
                }

                // Monitor connection health - when it breaks, we'll reconnect
                monitor_connection_health(&socket_path).await;
                info!("Connection lost, will reconnect...");
            }
            Ok(Err(e)) => {
                error!("Failed to establish connection: {:?}", e);
                let err = CommonError::Unknown(anyhow::anyhow!("Failed to establish connection: {e:?}"));
                let _ = sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_trigger.send(err);
                return
            }
            Err(_) => {
                error!("Connection timeout after 10 seconds");
                let err = CommonError::Unknown(anyhow::anyhow!("Failed to connect to SDK server within 10 seconds"));
                let _ = sync_dev_runtime_changes_from_sdk_server_shutdown_complete_signal_trigger.send(err);
                return
            }
        }

        // Brief pause before reconnecting
        ticker.tick().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_runtime_bun_v1_with_valid_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create required files
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let result = validate_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(result, "Should validate as BunV1 runtime");
    }

    #[test]
    fn test_validate_runtime_pnpm_v1_missing_package_json() {
        let temp_dir = TempDir::new().unwrap();

        // Only create index.ts
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let result = validate_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without package.json");
    }

    #[test]
    fn test_validate_runtime_pnpm_v1_missing_index_ts() {
        let temp_dir = TempDir::new().unwrap();

        // Only create package.json
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

        let result = validate_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without index.ts");
    }

    #[test]
    fn test_determine_runtime_from_dir_pnpm_v1() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let runtime = determine_runtime_from_dir(temp_dir.path()).unwrap();
        assert_eq!(runtime, Some(Runtime::PnpmV1));
    }

    #[test]
    fn test_determine_runtime_from_dir_no_match() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory
        let runtime = determine_runtime_from_dir(temp_dir.path()).unwrap();
        assert_eq!(runtime, None);
    }

}
