use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bridge::logic::EnvelopeEncryptionKeyContents;
use bridge::logic::get_or_create_local_encryption_key;
use clap::Args;
use clap::Parser;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tracing::{error, info, warn};
use url::Url;

use shared::error::CommonError;
use shared::port::find_free_port;
use shared::soma_agent_definition::{SomaAgentDefinitionLike, YamlSomaAgentDefinition};

use soma_api_server::restate::{RestateServerRemoteParams, RestateServerLocalParams, RestateServerParams};
use soma_api_server::factory::{create_api_service, CreateApiServiceParams};
use crate::bridge::start_bridge_sync_to_yaml_subsystem;
use crate::utils::config::{CliConfig, get_config_file_path};
use crate::utils::construct_src_dir_absolute;
use crate::server::{StartAxumServerParams, start_axum_server};

#[derive(Args, Debug, Clone)]
#[group(multiple = false, required = false)]
pub struct RemoteRestateParams {
    #[arg(long = "restate-admin-url", requires = "ingress_url")]
    pub admin_url: Option<Url>,
    #[arg(long = "restate-ingress-url", requires = "admin_url")]
    pub ingress_url: Option<Url>,
    #[arg(
        long = "restate-admin-token",
        requires = "admin_url",
        requires = "ingress_url"
    )]
    pub admin_token: Option<String>,
}

impl TryFrom<RemoteRestateParams> for RestateServerParams {
    type Error = CommonError;
    fn try_from(params: RemoteRestateParams) -> Result<Self, Self::Error> {
        if params.admin_url.is_none() || params.ingress_url.is_none() {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Admin URL and ingress URL are required"
            )));
        }
        Ok(RestateServerParams::Remote(RestateServerRemoteParams {
            admin_address: params.admin_url.unwrap(),
            ingress_address: params.ingress_url.unwrap(),
            admin_token: params.admin_token,
        }))
    }
}

#[derive(Debug, Clone, Parser)]
pub struct DevParams {
    #[arg(long, default_value = "3000")]
    pub port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long)]
    pub src_dir: Option<PathBuf>,
    #[arg(long, default_value = "libsql://./.soma/local.db?mode=local")]
    pub db_conn_string: Url,
    #[arg(long)]
    pub db_auth_token: Option<String>,
    #[command(flatten)]
    pub remote_restate: Option<RemoteRestateParams>,

    #[arg(
        long,
        default_value = "local",
        help = "The type of key encryption key to use. Valid values are 'local' or a valid AWS KMS ARN (arn:aws:kms:region:account-id:key/key-id)."
    )]
    pub key_encryption_key: Option<String>,
    #[arg(
        long,
        help = "Delete the Restate data directory before starting (only applies to local Restate instances)"
    )]
    pub clean: bool,
}

/// Main entry point for the start command
pub async fn cmd_dev(params: DevParams, _cli_config: &mut CliConfig) -> Result<(), CommonError> {
    let (system_shutdown_signal_trigger, _system_shutdown_signal_receiver) =
        broadcast::channel::<()>(1);
    let project_dir = construct_src_dir_absolute(params.clone().src_dir)?;

    // Resolve relative db_conn_string paths relative to project_dir
    let db_conn_string = if params.db_conn_string.as_str().starts_with("libsql://./") {
        // Extract the path portion after libsql://./
        let url_str = params.db_conn_string.as_str();
        let path_with_query = url_str.strip_prefix("libsql://./").unwrap_or("");
        let (path_part, query_part) = path_with_query
            .split_once('?')
            .unwrap_or((path_with_query, ""));

        // Resolve relative path to absolute path relative to project_dir
        let absolute_path = project_dir.join(path_part);

        // Reconstruct the URL with absolute path
        let path_str = absolute_path.to_string_lossy();
        let new_url_str = if query_part.is_empty() {
            format!("libsql://{path_str}")
        } else {
            format!("libsql://{path_str}?{query_part}")
        };

        info!("Database path resolved to: {}", absolute_path.display());
        Url::parse(&new_url_str).unwrap_or_else(|_| params.db_conn_string.clone())
    } else {
        params.db_conn_string.clone()
    };

    // Find free port for SDK server
    // let sdk_port = find_free_port(9080, 10080)?;

    // Setup encryption key
    let envelope_encryption_key_contents = setup_encryption_key(params.key_encryption_key.clone())?;

    // Load soma definition
    let soma_definition: Arc<dyn SomaAgentDefinitionLike> = load_soma_definition(&project_dir)?;

    // Setup Restate parameters
    let restate_params = match params.remote_restate {
        Some(remote_restate) => remote_restate.try_into()?,
        None => RestateServerParams::Local(RestateServerLocalParams {
            project_dir: project_dir.clone(),
            ingress_port: 8080,
            admin_port: 9070,
            advertised_node_port: 5122,
            clean: params.clean,
        }),
    };

    // Find free port for SDK server
    let sdk_port = find_free_port(9080, 10080)?;

    // Start Restate server subsystem
    info!("Starting Restate server...");
    let restate_handle = crate::restate_server::start_restate_subsystem(
        restate_params.clone(),
        system_shutdown_signal_trigger.subscribe(),
    )?;

    // Start bridge config change listener subsystem
    info!("Starting bridge config change listener...");
    let (on_bridge_change_tx, bridge_sync_handle) = start_bridge_sync_to_yaml_subsystem(
        soma_definition.clone(),
        project_dir.clone(),
    )?;

    // Create API service and start all subsystems
    info!("Initializing API service and starting all subsystems...");
    let api_service_bundle = create_api_service(CreateApiServiceParams {
        project_dir: project_dir.clone(),
        host: params.host.clone(),
        port: params.port,
        sdk_port,
        db_conn_string: db_conn_string.to_string(),
        db_auth_token: params.db_auth_token.clone(),
        soma_definition: soma_definition.clone(),
        restate_params: restate_params.clone(),
        envelope_encryption_key_contents,
        system_shutdown_signal: system_shutdown_signal_trigger.clone(),
        on_bridge_config_change_tx: on_bridge_change_tx.clone(),
        restate_handle,
    })
    .await?;

    let api_service = api_service_bundle.api_service;
    let subsystems = api_service_bundle.subsystems;

    // Sync bridge from soma definition (now all providers should be available)
    info!("Syncing bridge from soma.yaml...");
    let api_base_url = format!("http://{}:{}", params.host, params.port);
    let api_config = soma_api_client::apis::configuration::Configuration {
        base_path: api_base_url,
        user_agent: Some("soma-cli".to_string()),
        client: reqwest::Client::new(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    };
    crate::bridge::sync_yaml_to_api_on_start::sync_bridge_db_from_soma_definition_on_start(
        &api_config,
        &soma_definition,
    )
    .await?;
    info!("Bridge sync completed");

    // Reload soma definition
    soma_definition.reload().await?;

    info!("API service initialized and all subsystems started");

    // Start Axum server subsystem
    // let (on_server_started_tx, on_server_started_rx) = oneshot::channel::<()>();
    let axum_system_shutdown_signal_rx = system_shutdown_signal_trigger.subscribe();
    let (axum_shutdown_complete_signal_trigger, axum_shutdown_complete_signal_receiver) =
        oneshot::channel::<()>();
    let api_service_clone = api_service.clone();
    let host_clone = params.host.clone();
    let port_clone = params.port;

    let (server_fut, _handle, _addr) = match start_axum_server(StartAxumServerParams {
        api_service: api_service_clone,
        host: host_clone,
        port: port_clone,
        system_shutdown_signal_rx: axum_system_shutdown_signal_rx,
    })
    .await
    {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to start Axum server: {:?}", e);
            let _ = axum_shutdown_complete_signal_trigger.send(());
            return Err(e);
        }
    };

    // let _ = on_server_started_tx.send(());

    tokio::spawn(async move {
        let res = server_fut.await;
        match res {
            Ok(()) => info!("Axum server stopped gracefully"),
            Err(e) => error!("Axum server stopped with error: {:?}", e),
        }
        let _ = axum_shutdown_complete_signal_trigger.send(());
    });

    // Shutdown monitoring thread - handles both unexpected exits and graceful shutdown
    let system_shutdown_signal_trigger_clone = system_shutdown_signal_trigger.clone();
    let mut shutdown_requested_rx = system_shutdown_signal_trigger.subscribe();

    // Convert subsystem handles to shutdown receivers
    let mut shutdown_receivers: Vec<(&str, oneshot::Receiver<()>)> = vec![
        ("axum_server", axum_shutdown_complete_signal_receiver),
    ];

    // Helper to convert SubsystemHandle to (name, receiver) pair
    let mut add_subsystem_handle = |name: &'static str, handle: Option<shared::subsystem::SubsystemHandle>| {
        if let Some(h) = handle {
            let (tx, rx) = oneshot::channel();
            let handle_name = name;
            tokio::spawn(async move {
                h.wait_for_shutdown().await;
                let _ = tx.send(());
            });
            shutdown_receivers.push((handle_name, rx));
        }
    };

    // Add all subsystems from the bundle
    add_subsystem_handle("restate", subsystems.restate);
    add_subsystem_handle("file_watcher", subsystems.file_watcher);
    add_subsystem_handle("bridge_sync_yaml", Some(bridge_sync_handle));
    add_subsystem_handle("sdk_server", subsystems.sdk_server);
    add_subsystem_handle("sdk_sync", subsystems.sdk_sync);
    add_subsystem_handle("mcp", subsystems.mcp);
    add_subsystem_handle("credential_rotation", subsystems.credential_rotation);
    add_subsystem_handle("bridge_codegen", subsystems.bridge_codegen);

    // Systems that can trigger shutdown (unexpected exits)
    let systems_that_can_trigger_shutdown: Vec<&str> = vec![
        "restate",
        "file_watcher",
        "bridge_sync_yaml",
        "axum_server",
        "mcp",
        "sdk_server",
        "sdk_sync",
        "credential_rotation",
    ];

    // Systems that require graceful shutdown (we wait for these after shutdown is triggered)
    let systems_requiring_graceful_shutdown: Vec<&str> = vec![
        "restate",
        "axum_server",
        "sdk_server",
        "mcp",
        "sdk_sync",
    ];

    // Track which systems have completed
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    let completed_systems: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // Spawn tasks to track each system's completion (these run independently)
    for (name, receiver) in shutdown_receivers {
        let name_str = name.to_string();
        let completed_systems_clone = completed_systems.clone();
        tokio::spawn(async move {
            let _ = receiver.await;
            let mut completed = completed_systems_clone.lock().unwrap();
            completed.insert(name_str);
        });
    }

    let shutdown_monitor_handle = tokio::spawn(async move {
        use futures::future::FutureExt;

        // Wait for either shutdown signal OR first system that can trigger shutdown to complete
        let shutdown_signal_fut = shutdown_requested_rx.recv().map(|_| None::<String>).boxed();

        // Create futures that complete when systems that can trigger shutdown complete
        let mut trigger_futures = Vec::new();
        for name in systems_that_can_trigger_shutdown {
            let name_str = name.to_string();
            let completed_systems_clone = completed_systems.clone();
            let fut = async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let completed = completed_systems_clone.lock().unwrap();
                    if completed.contains(&name_str) {
                        return Some(name_str);
                    }
                    drop(completed);
                }
            };
            trigger_futures.push(fut.boxed());
        }

        let mut all_waits = vec![shutdown_signal_fut];
        all_waits.extend(trigger_futures);

        let (result, _idx, _remaining) = futures::future::select_all(all_waits).await;

        let was_unexpected = result.is_some();
        if was_unexpected {
            if let Some(triggered_name) = result {
                info!(
                    "System shutdown gracefully, triggered by unexpected exit of {} system",
                    triggered_name
                );
                let _ = system_shutdown_signal_trigger_clone.send(());
            }
        } else {
            info!("Shutdown requested, waiting for all systems to complete");
        }

        // Now wait for all systems that require graceful shutdown to complete
        let systems_to_wait_for: Vec<String> = systems_requiring_graceful_shutdown
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Wait for remaining systems with timeout
        let timeout_fut = tokio::time::sleep(Duration::from_secs(30));
        let completed_systems_for_check = completed_systems.clone();
        let systems_to_wait_for_check = systems_to_wait_for.clone();
        let check_completion = async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let completed = completed_systems_for_check.lock().unwrap();
                let all_complete = systems_to_wait_for_check
                    .iter()
                    .all(|name| completed.contains(name));
                if all_complete {
                    break;
                }
                drop(completed);
            }
        };

        tokio::select! {
          _ = timeout_fut => {
            let completed = completed_systems.lock().unwrap();
            let still_waiting: Vec<String> = systems_to_wait_for
              .into_iter()
              .filter(|name| !completed.contains(name))
              .collect();
            error!("Failed to wait for graceful shutdown of {} systems (timeout after 30s)", still_waiting.join(", "));
          }
          _ = check_completion => {
            info!("All systems shut down gracefully");
          }
        }
    });

    // Wait for shutdown signal (Ctrl+C)
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received, triggering graceful shutdown");
    let _ = system_shutdown_signal_trigger.send(());

    // Wait for shutdown monitor to complete
    tokio::select! {
      _ = shutdown_monitor_handle => {
        info!("Shutdown monitoring completed");
      }
      _ = tokio::time::sleep(Duration::from_secs(35)) => {
        warn!("Shutdown monitoring timed out after 35s, proceeding anyway");
      }
    }

    Ok(())
}

/// Sets up the envelope encryption key
fn setup_encryption_key(
    key_encryption_key: Option<String>,
) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
    let local_key_path = get_config_file_path()?
        .parent()
        .ok_or(CommonError::Unknown(anyhow::anyhow!(
            "Failed to get config file path"
        )))?
        .join("local-key.bin");

    let envelope_encryption_key_contents = match key_encryption_key {
        Some(key_encryption_key) => match key_encryption_key.as_str() {
            "local" => get_or_create_local_encryption_key(&local_key_path)?,
            _ => {
                if is_valid_kms_arn(&key_encryption_key) {
                    EnvelopeEncryptionKeyContents::AwsKms {
                        arn: key_encryption_key,
                    }
                } else {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Invalid AWS KMS ARN: {key_encryption_key}"
                    )));
                }
            }
        },
        None => get_or_create_local_encryption_key(&local_key_path)?,
    };

    if matches!(
        envelope_encryption_key_contents,
        EnvelopeEncryptionKeyContents::Local { .. }
    ) {
        warn!(
            "Local key encryption key is for getting started quickly. Please use a valid AWS KMS ARN for production."
        );
    }

    Ok(envelope_encryption_key_contents)
}

/// Loads the soma definition from the source directory
fn load_soma_definition(src_dir: &Path) -> Result<Arc<dyn SomaAgentDefinitionLike>, CommonError> {
    let path_to_soma_definition = src_dir.join("soma.yaml");

    if !path_to_soma_definition.exists() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Soma definition not found at {}",
            path_to_soma_definition.display()
        )));
    }
    let soma_definition = YamlSomaAgentDefinition::load_from_file(path_to_soma_definition)?;
    Ok(Arc::new(soma_definition))
}

/// Validates if a string is a valid AWS KMS ARN (testable)
pub fn is_valid_kms_arn(arn: &str) -> bool {
    arn.starts_with("arn:aws:kms:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_kms_arn_with_valid_arn() {
        let arn = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012";
        assert!(is_valid_kms_arn(arn));
    }

    #[test]
    fn test_is_valid_kms_arn_with_invalid_arn() {
        let arn = "arn:aws:s3:::my-bucket";
        assert!(!is_valid_kms_arn(arn));
    }

    #[test]
    fn test_is_valid_kms_arn_with_random_string() {
        let arn = "not-an-arn";
        assert!(!is_valid_kms_arn(arn));
    }

    #[test]
    fn test_is_valid_kms_arn_empty_string() {
        assert!(!is_valid_kms_arn(""));
    }

    #[test]
    fn test_is_valid_kms_arn_local() {
        assert!(!is_valid_kms_arn("local"));
    }
}
