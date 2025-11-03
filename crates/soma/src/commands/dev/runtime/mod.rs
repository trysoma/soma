mod interface;
mod typescript;
pub mod grpc_client;
pub mod sdk_provider_sync;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::{FutureExt, TryFutureExt, future};
use globset::{Glob, GlobSet, GlobSetBuilder};
use tokio::process::Command;
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

use shared::command::run_child_process;
use shared::error::CommonError;

use crate::commands::dev::DevParams;
use crate::commands::dev::runtime::grpc_client::{create_unix_socket_client, establish_connection_with_retry, monitor_connection_health};
use crate::utils::construct_src_dir_absolute;

use super::project_file_watcher::FileChangeRx;
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
    validate_runtime_pnpm_v1_internal(&src_dir)
}

/// Internal validation function (easier to test)
fn validate_runtime_pnpm_v1_internal(src_dir: &Path) -> Result<bool, CommonError> {
    let files_to_check = vec![
        "package.json",
        "index.ts",
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

pub fn files_to_watch_pnpm_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    builder.add(Glob::new("**/*.ts")?);
    builder.add(Glob::new("package.json")?);
    builder.add(Glob::new("soma.yaml")?);

    Ok(builder.build()?)
}

pub fn files_to_ignore_pnpm_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    // Match node_modules anywhere in the path
    builder.add(Glob::new("**/node_modules/**")?);

    // Ignore .soma build directory to prevent infinite restart loops
    builder.add(Glob::new("**/.soma/**")?);

    Ok(builder.build()?)
}

pub fn collect_paths_to_watch(
    root: &Path,
    watch_globs: &GlobSet,
    ignore_globs: &GlobSet,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        // Match against path relative to root for glob patterns
        let relative_path = path.strip_prefix(root).unwrap_or(&path);

        if ignore_globs.is_match(relative_path) {
            continue;
        }

        if path.is_dir() {
            // Push subdirs for recursive traversal
            if let Ok(read_dir) = fs::read_dir(&path) {
                for entry in read_dir.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if watch_globs.is_match(relative_path) {
            paths.push(path);
        }
    }

    paths
}

pub struct StartDevRuntimeParams<'a> {
    pub project_dir: PathBuf,
    pub runtime: Runtime,
    pub runtime_port: u16,
    pub file_change_signal: &'a mut FileChangeRx,
    pub kill_signal: broadcast::Receiver<()>,
    pub shutdown_complete_signal: oneshot::Sender<()>,
}

/// Starts the development runtime with hot reloading on file changes
pub async fn start_dev_runtime<'a>(
    params: StartDevRuntimeParams<'a>,
) -> Result<(), CommonError> {
    let StartDevRuntimeParams {
        project_dir,
        runtime,
        runtime_port,
        file_change_signal,
        mut kill_signal,
        shutdown_complete_signal,
    } = params;
    loop {
        let (dev_kill_signal_tx, dev_kill_signal_rx) = oneshot::channel::<()>();
        let (dev_shutdown_complete_tx, dev_shutdown_complete_rx) = oneshot::channel::<()>();

        let serve_fut = match runtime {
            Runtime::PnpmV1 => {
                // Check if this is a Vite-based project
                if is_vite_project(&project_dir) {
                    info!("Detected Vite project, using Vite dev server");

                    // Use the Typescript SdkClient to start the vite dev server
                    let typescript_client = Typescript::new();
                    let ctx = ClientCtx {
                        project_dir: project_dir.clone(),
                        socket_path: DEFAULT_SOMA_SERVER_SOCK.to_string(),
                        restate_runtime_port: runtime_port,
                    };

                    async move {
                        let dev_server_handle = typescript_client.start_dev_server(ctx).await?;

                        // Wait for kill signal or server completion
                        tokio::select! {
                            _ = dev_kill_signal_rx => {
                                info!("Kill signal received, shutting down vite dev server");
                                let _ = dev_server_handle.kill_signal_tx.send(());
                                let _ = dev_server_handle.shutdown_complete_rx.await;
                                let _ = dev_shutdown_complete_tx.send(());
                            }
                            result = dev_server_handle.dev_server_fut => {
                                result?;
                            }
                        }
                        Ok::<(), CommonError>(())
                    }.boxed()
                } else {
                    return Err(CommonError::Unknown(anyhow::anyhow!("Invalid runtime. must use vite")));
                }
            },
            _ => {
                return Err(CommonError::Unknown(anyhow::anyhow!("Invalid runtime")));
            }
        };

        let serve_fut = serve_fut.then(async |_| {
            info!("Runtime stopped, awaiting file change to restart or complete shutdown (CTRL+C)");
            future::pending::<()>().await;
            Ok::<(), CommonError>(())
        });

        tokio::select! {
            _ = file_change_signal.recv() => {
                info!("File change detected");
                let _ = dev_kill_signal_tx.send(());
                // Ignore channel errors during restart - process may have already exited
                let _ = dev_shutdown_complete_rx.await;
                continue;
            }
            _ = serve_fut => {}
            _ = kill_signal.recv() => {
                info!("System kill signal received");
                let _ = dev_kill_signal_tx.send(());
                // Ignore channel errors during shutdown - process may have already exited
                let _ = dev_shutdown_complete_rx.await;
                let _ = shutdown_complete_signal.send(());
                break;
            }
        }
    }

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
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("gRPC call failed: {}", e)))?;

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
        let service_uri = format!("http://127.0.0.1:{}", runtime_port);
        let deployment_type = crate::utils::restate::deploy::DeploymentType::Http {
            uri: service_uri.clone(),
            additional_headers: HashMap::new(),
        };

        // Use the agent ID as the service path
        let service_path = agent.id.clone();

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

/// Watch for dev runtime reloads by monitoring the gRPC connection
/// This function runs indefinitely, reconnecting when the server restarts
/// and syncing providers on each reconnection
pub async fn watch_for_dev_runtime_reload(
    socket_path: &str,
    restate_params: &super::restate::RestateServerParams,
    runtime_port: u16,
) -> Result<(), CommonError> {
    use tokio::time::Duration;

    info!("Starting dev runtime reload watcher for socket: {}", socket_path);
    let mut ticker = tokio::time::interval(Duration::from_millis(500));

    loop {
        // Try to establish connection with timeout
        let connection_result = tokio::time::timeout(
            Duration::from_secs(10),
            establish_connection_with_retry(socket_path)
        ).await;

        match connection_result {
            Ok(Ok(_)) => {
                info!("📡 Connected to SDK server, fetching metadata and syncing providers...");

                // Fetch metadata and sync providers to bridge
                match fetch_and_sync_providers(socket_path).await {
                    Ok(agents) => {
                        // Register Restate deployments for each agent
                        if !agents.is_empty() {
                            if let Err(e) = register_agent_deployments(agents, restate_params, runtime_port).await {
                                error!("Failed to register agent deployments: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch and sync providers: {:?}", e);
                    }
                }

                // Monitor connection health - when it breaks, we'll reconnect
                monitor_connection_health(socket_path).await;
                info!("Connection lost, will reconnect...");
            }
            Ok(Err(e)) => {
                error!("Failed to establish connection: {:?}", e);
                return Err(e);
            }
            Err(_) => {
                error!("Connection timeout after 10 seconds");
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Failed to connect to SDK server within 10 seconds"
                )));
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

        let result = validate_runtime_pnpm_v1_internal(temp_dir.path()).unwrap();
        assert!(result, "Should validate as BunV1 runtime");
    }

    #[test]
    fn test_validate_runtime_pnpm_v1_missing_package_json() {
        let temp_dir = TempDir::new().unwrap();

        // Only create index.ts
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();

        let result = validate_runtime_pnpm_v1_internal(temp_dir.path()).unwrap();
        assert!(!result, "Should not validate without package.json");
    }

    #[test]
    fn test_validate_runtime_pnpm_v1_missing_index_ts() {
        let temp_dir = TempDir::new().unwrap();

        // Only create package.json
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

        let result = validate_runtime_pnpm_v1_internal(temp_dir.path()).unwrap();
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
