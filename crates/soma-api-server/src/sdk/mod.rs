mod interface;
pub mod sdk_provider_sync;
mod typescript;

use std::path::{Path, PathBuf};
use std::time::Duration;

use shared::restate;
use shared::subsystem::SubsystemHandle;
use shared::uds::{
    DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client, establish_connection_with_retry,
    monitor_connection_health,
};
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

use shared::error::CommonError;

use crate::logic::environment_variable_sync::fetch_all_environment_variables;
use crate::logic::secret_sync::fetch_and_decrypt_all_secrets;
use crate::restate::RestateServerParams;
use encryption::logic::crypto_services::CryptoCache;
use interface::{ClientCtx, SdkClient};
use typescript::Typescript;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdkRuntime {
    PnpmV1,
}

/// Determines which SDK runtime to use from a directory path
pub fn determine_sdk_runtime(project_dir: &Path) -> Result<Option<SdkRuntime>, CommonError> {
    let possible_runtimes = vec![(SdkRuntime::PnpmV1, validate_sdk_runtime_pnpm_v1)];

    let mut matched_runtimes = vec![];

    for (runtime, validate_fn) in possible_runtimes {
        let result = validate_fn(project_dir.to_path_buf())?;
        if result {
            matched_runtimes.push(runtime);
        }
    }

    match matched_runtimes.len() {
        0 => Ok(None),
        1 => Ok(Some(matched_runtimes[0].clone())),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Multiple SDK runtimes matched"
        ))),
    }
}

fn validate_sdk_runtime_pnpm_v1(project_dir: PathBuf) -> Result<bool, CommonError> {
    let files_to_check = vec!["package.json", "vite.config.ts"];
    for file in files_to_check {
        let file_path = project_dir.join(file);
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

pub struct StartDevSdkParams {
    pub project_dir: PathBuf,
    pub sdk_runtime: SdkRuntime,
    pub sdk_port: u16,
    pub kill_signal_rx: broadcast::Receiver<()>,
    pub repository: std::sync::Arc<crate::repository::Repository>,
    pub crypto_cache: CryptoCache,
}

/// Starts the development SDK server with hot reloading on file changes
pub async fn start_dev_sdk(params: StartDevSdkParams) -> Result<(), CommonError> {
    let StartDevSdkParams {
        project_dir,
        sdk_runtime: _sdk_runtime,
        sdk_port,
        kill_signal_rx,
        repository,
        crypto_cache,
    } = params;

    // Fetch all secrets from the database
    info!("Fetching initial secrets from database...");
    let decrypted_secrets = fetch_and_decrypt_all_secrets(&repository, &crypto_cache).await?;
    let initial_secrets: std::collections::HashMap<String, String> = decrypted_secrets
        .into_iter()
        .map(|s| (s.key, s.value))
        .collect();
    info!("Fetched {} initial secrets", initial_secrets.len());

    // Fetch all environment variables from the database
    info!("Fetching initial environment variables from database...");
    let env_vars = fetch_all_environment_variables(&repository).await?;
    let initial_environment_variables: std::collections::HashMap<String, String> =
        env_vars.into_iter().map(|e| (e.key, e.value)).collect();
    info!(
        "Fetched {} initial environment variables",
        initial_environment_variables.len()
    );

    let typescript_client = Typescript::new();
    let ctx = ClientCtx {
        project_dir: project_dir.clone(),
        socket_path: DEFAULT_SOMA_SERVER_SOCK.to_string(),
        restate_runtime_port: sdk_port,
        kill_signal_rx: kill_signal_rx.resubscribe(),
        initial_secrets,
        initial_environment_variables,
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

pub fn start_sdk_server_subsystem(
    project_dir: PathBuf,
    sdk_runtime: SdkRuntime,
    sdk_port: u16,
    shutdown_rx: broadcast::Receiver<()>,
    repository: crate::repository::Repository,
    crypto_cache: CryptoCache,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("SDK Server");
    let repository = std::sync::Arc::new(repository);

    tokio::spawn(async move {
        match start_dev_sdk(StartDevSdkParams {
            project_dir,
            sdk_runtime,
            sdk_port,
            kill_signal_rx: shutdown_rx,
            repository,
            crypto_cache,
        })
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("SDK server stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}

pub fn start_sdk_sync_subsystem(
    socket_path: String,
    restate_params: RestateServerParams,
    sdk_port: u16,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("SDK Sync");

    tokio::spawn(async move {
        match sync_sdk_changes(SyncSdkChangesParams {
            socket_path,
            restate_params,
            sdk_port,
            system_shutdown_signal_rx: shutdown_rx,
        })
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("SDK sync watcher stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}

/// Fetch metadata and sync providers to the bridge registry
/// Returns the list of agents from the metadata response
async fn fetch_and_sync_providers(socket_path: &str) -> Result<Vec<sdk_proto::Agent>, CommonError> {
    let mut client = create_soma_unix_socket_client(socket_path).await?;

    let request = tonic::Request::new(());
    let response = client
        .metadata(request)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("gRPC call failed: {e}")))?;

    let metadata = response.into_inner();

    info!("=== SDK Metadata ===");
    info!("Provider count: {}", metadata.bridge_providers.len());

    for (i, provider) in metadata.bridge_providers.iter().enumerate() {
        info!(
            "Provider {}: type_id={}, name={}",
            i + 1,
            provider.type_id,
            provider.name
        );
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
    restate_params: &crate::restate::RestateServerParams,
    sdk_port: u16,
) -> Result<(), CommonError> {
    use std::collections::HashMap;

    info!(
        "Registering {} agent deployment(s) with Restate",
        agents.len()
    );

    for agent in agents {
        let service_uri = format!("http://127.0.0.1:{sdk_port}");
        let deployment_type = restate::deploy::DeploymentType::Http {
            uri: service_uri.clone(),
            additional_headers: HashMap::new(),
        };

        // Use the project_id.agent_id format as the service path (matches Restate service name)
        let service_path = format!("{}.{}", agent.project_id, agent.id);

        info!("Registering agent '{}' at {}", agent.name, service_uri);

        let admin_url = restate_params.get_admin_address()?;
        let config = restate::deploy::DeploymentRegistrationConfig {
            admin_url: admin_url.to_string(),
            service_path: service_path.clone(),
            deployment_type,
            bearer_token: restate_params.get_admin_token(),
            private: restate_params.get_private(),
            insecure: restate_params.get_insecure(),
            force: restate_params.get_force(),
        };

        match restate::deploy::register_deployment(config).await {
            Ok(metadata) => {
                info!(
                    "Successfully registered agent '{}' (service: {})",
                    agent.name, metadata.name
                );
            }
            Err(e) => {
                error!("Failed to register agent '{}': {:?}", agent.name, e);
                // Continue with other agents even if one fails
            }
        }
    }

    Ok(())
}

pub struct SyncSdkChangesParams {
    pub socket_path: String,
    pub restate_params: crate::restate::RestateServerParams,
    pub sdk_port: u16,
    pub system_shutdown_signal_rx: broadcast::Receiver<()>,
}

/// Watch for dev SDK server reloads by monitoring the gRPC connection
/// This function runs indefinitely, reconnecting when the server restarts
/// and syncing providers on each reconnection
#[allow(clippy::needless_return)]
pub async fn sync_sdk_changes(params: SyncSdkChangesParams) -> Result<(), CommonError> {
    let SyncSdkChangesParams {
        socket_path,
        restate_params,
        sdk_port,
        mut system_shutdown_signal_rx,
    } = params;

    let (sync_shutdown_complete_tx, sync_shutdown_complete_rx) = oneshot::channel::<CommonError>();
    let (system_shutdown_tx, system_shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        tokio::select! {
            _ = system_shutdown_rx => {
            }
            _ = internal_sync_sdk_changes_loop(socket_path, restate_params, sdk_port, sync_shutdown_complete_tx) => {
            }
        }
    });

    tokio::select! {
        _ = system_shutdown_signal_rx.recv() => {
            info!("SDK sync watcher shutdown requested");
            let _ = system_shutdown_tx.send(());

            return Ok(());
        }
        result = sync_shutdown_complete_rx => {
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

async fn internal_sync_sdk_changes_loop(
    socket_path: String,
    restate_params: crate::restate::RestateServerParams,
    sdk_port: u16,
    sync_shutdown_complete_tx: oneshot::Sender<CommonError>,
) {
    info!(
        "Starting dev SDK reload watcher for socket: {}",
        socket_path
    );
    let mut ticker = tokio::time::interval(Duration::from_millis(500));

    loop {
        // Try to establish connection with timeout
        let connection_result = tokio::time::timeout(
            Duration::from_secs(10),
            establish_connection_with_retry(&socket_path),
        )
        .await;

        match connection_result {
            Ok(Ok(_)) => {
                info!("Connected to SDK server, fetching metadata and syncing providers...");

                // Fetch metadata and sync providers to bridge
                match fetch_and_sync_providers(&socket_path).await {
                    Ok(agents) => {
                        // Register Restate deployments for each agent
                        if !agents.is_empty() {
                            if let Err(e) =
                                register_agent_deployments(agents, &restate_params, sdk_port).await
                            {
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
                let err =
                    CommonError::Unknown(anyhow::anyhow!("Failed to establish connection: {e:?}"));
                let _ = sync_shutdown_complete_tx.send(err);
                return;
            }
            Err(_) => {
                error!("Connection timeout after 10 seconds");
                let err = CommonError::Unknown(anyhow::anyhow!(
                    "Failed to connect to SDK server within 10 seconds"
                ));
                let _ = sync_shutdown_complete_tx.send(err);
                return;
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
    fn test_validate_sdk_runtime_pnpm_v1_with_valid_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create required files
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}").unwrap();

        let result = validate_sdk_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(result, "Should validate as PnpmV1 SDK runtime");
    }

    #[test]
    fn test_validate_sdk_runtime_pnpm_v1_missing_package_json() {
        let temp_dir = TempDir::new().unwrap();

        // Only create vite.config.ts
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}").unwrap();

        let result = validate_sdk_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without package.json");
    }

    #[test]
    fn test_validate_sdk_runtime_pnpm_v1_missing_vite_config() {
        let temp_dir = TempDir::new().unwrap();

        // Only create package.json
        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

        let result = validate_sdk_runtime_pnpm_v1(temp_dir.path().to_path_buf()).unwrap();
        assert!(!result, "Should not validate without vite.config.ts");
    }

    #[test]
    fn test_determine_sdk_runtime_pnpm_v1() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}").unwrap();

        let runtime = determine_sdk_runtime(temp_dir.path()).unwrap();
        assert_eq!(runtime, Some(SdkRuntime::PnpmV1));
    }

    #[test]
    fn test_determine_sdk_runtime_no_match() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory
        let runtime = determine_sdk_runtime(temp_dir.path()).unwrap();
        assert_eq!(runtime, None);
    }
}
