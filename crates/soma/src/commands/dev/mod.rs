mod bridge_util;
mod project_file_watcher;
mod repository;
mod restate;
pub mod runtime;
mod server;
mod mcp;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bridge::logic::get_or_create_local_encryption_key;
use bridge::logic::{EnvelopeEncryptionKeyContents};
use clap::Args;
use clap::Parser;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tracing::{error, info, warn};
use url::Url;

use shared::error::CommonError;
use shared::soma_agent_definition::{SomaAgentDefinitionLike, YamlSomaAgentDefinition};

use crate::commands::dev::mcp::{StartMcpConnectionManagerParams, start_mcp_connection_manager};
use crate::commands::dev::repository::setup_repository;
use crate::commands::dev::restate::{RestateServerLocalParams, RestateServerParams};
use crate::commands::dev::restate::RestateServerRemoteParams;
use crate::commands::dev::runtime::SyncDevRuntimeChangesFromSdkServerParams;
use crate::logic::ConnectionManager;
use crate::utils::config::{CliConfig, get_config_file_path};
use crate::utils::construct_src_dir_absolute;

use self::bridge_util::{start_sync_on_bridge_change, sync_bridge_db_from_soma_definition_on_start};
use self::project_file_watcher::start_project_file_watcher;
use self::restate::{start_restate_server};
use self::runtime::{determine_runtime, start_dev_runtime, sync_dev_runtime_changes_from_sdk_server, StartDevRuntimeParams, DEFAULT_SOMA_SERVER_SOCK};
use self::runtime::grpc_client::{establish_connection_with_retry};
use self::runtime::sdk_provider_sync::sync_providers_from_metadata;
use crate::commands::dev::runtime::grpc_client::create_unix_socket_client;
use self::server::{find_free_port, start_axum_server, StartAxumServerParams};
use crate::router;


#[derive(Args, Debug, Clone)]
#[group(multiple = false, required = false)]
pub struct RemoteRestateParams {
  #[arg(long = "restate-admin-url", requires = "ingress_url")]
  pub admin_url: Option<Url>,
  #[arg(long = "restate-ingress-url", requires = "admin_url")]
  pub ingress_url: Option<Url>,
  #[arg(long = "restate-admin-token", requires = "admin_url", requires = "ingress_url")]
  pub admin_token: Option<String>,
}

impl TryFrom<RemoteRestateParams> for RestateServerParams {
  type Error = CommonError;
  fn try_from(params: RemoteRestateParams) -> Result<Self, Self::Error> {
    if params.admin_url.is_none() || params.ingress_url.is_none() {
      return Err(CommonError::Unknown(anyhow::anyhow!("Admin URL and ingress URL are required")));
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
pub async fn cmd_dev(
  params: DevParams,
  _cli_config: &mut CliConfig,
) -> Result<(), CommonError> {

  let (system_shutdown_signal_trigger, _system_shutdown_signal_receiver) = broadcast::channel::<()>(1);
  let project_dir = construct_src_dir_absolute(params.clone().src_dir)?;

  // Resolve relative db_conn_string paths relative to project_dir
  let db_conn_string = if params.db_conn_string.as_str().starts_with("libsql://./") {
    // Extract the path portion after libsql://./
    let url_str = params.db_conn_string.as_str();
    let path_with_query = url_str.strip_prefix("libsql://./").unwrap_or("");
    let (path_part, _query_part) = path_with_query.split_once('?').unwrap_or((path_with_query, ""));
    
    // Resolve relative path to absolute path relative to project_dir
    let absolute_path = project_dir.join(path_part);
    
    // Reconstruct the URL with absolute path
    let mut resolved_url = params.db_conn_string.clone();
    // Remove the leading ./ from the path and set the absolute path
    // URL paths typically start with /, so we need to ensure the format is correct
    let path_str = absolute_path.to_string_lossy();
    // For libsql URLs, we can set the path directly
    resolved_url.set_path(&path_str);
    resolved_url
  } else {
    params.db_conn_string.clone()
  };

  // Determine runtime and find free port
  let runtime = match determine_runtime(&params) {
    Ok(Some(runtime)) => runtime,
    Ok(None) => return Err(CommonError::Unknown(anyhow::anyhow!("No runtime matched"))),
    Err(e) => return Err(e),
  };
  let runtime_port = find_free_port(9080, 10080)?;


  // Setup encryption key
  let envelope_encryption_key_contents = setup_encryption_key(params.key_encryption_key.clone())?;
  // Load soma definition
  let soma_definition: Arc<dyn SomaAgentDefinitionLike> = load_soma_definition(&project_dir)?;

  // Setup database and repositories
  let connection_manager = ConnectionManager::new();
  
  let (
    _db,
    conn,
    repository,
    bridge_repo,
  ) = setup_repository(&db_conn_string,  &params.db_auth_token).await?;

 
  // Start Restate server subsystem
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
  let restate_kill_signal_rx = system_shutdown_signal_trigger.subscribe();
  let (restate_shutdown_complete_signal_trigger, restate_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  let restate_params_clone = restate_params.clone();
  tokio::spawn(async move {
    match start_restate_server(restate_params_clone, restate_kill_signal_rx).await {
      Ok(()) => {
        info!("Restate server stopped gracefully");
      }
      Err(e) => {
        error!("Restate server stopped with non-zero exit code: {:?}", e);
      }
    }
    let _ = restate_shutdown_complete_signal_trigger.send(());
  });

  
  // Start file watcher subsystem
  let (prj_file_change_tx, _prj_file_change_rx, prj_file_watcher_fut) = start_project_file_watcher(
    project_dir.clone(),
  )?;
  let (prj_shutdown_complete_signal_trigger, prj_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  tokio::spawn(async move {
    match prj_file_watcher_fut.await {
      Ok(()) => {
        info!("File watcher stopped gracefully");
      }
      Err(e) => {
        error!("File watcher stopped with non-zero exit code: {:?}", e);
      }
    };

    let _ = prj_shutdown_complete_signal_trigger.send(());
  });

  // Start bridge config change listener subsystem
  let soma_definition_clone = soma_definition.clone();
  let (on_bridge_change_evt_tx, on_bridge_change_evt_fut) = start_sync_on_bridge_change(soma_definition_clone)?;
  let (on_bridge_shutdown_complete_signal_trigger, on_bridge_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  tokio::spawn(async move {
    match on_bridge_change_evt_fut.await {
      Ok(()) => {
        info!("Bridge config change listener stopped gracefully");
      }
      Err(e) => {
        error!("Bridge config change listener stopped with non-zero exit code: {:?}", e);
      }
    }
    let _ = on_bridge_shutdown_complete_signal_trigger.send(());
  });

  // Start runtime subsystem first (needed for SDK provider sync)
  let runtime_kill_signal_rx = system_shutdown_signal_trigger.subscribe();
  let (runtime_shutdown_complete_signal_trigger, runtime_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  let runtime_clone = runtime.clone();
  let project_dir_runtime = project_dir.clone();
  let prj_file_change_tx_runtime = prj_file_change_tx.clone();
  tokio::spawn(async move {
    match start_dev_runtime(StartDevRuntimeParams {
      project_dir: project_dir_runtime,
      runtime: runtime_clone,
      runtime_port,
      file_change_tx: prj_file_change_tx_runtime,
      kill_signal_rx: runtime_kill_signal_rx,
    }).await {
      Ok(()) => {
        info!("Runtime stopped gracefully");
      }
      Err(e) => {
        error!("Runtime stopped unexpectedly: {:?}", e);
      }
    }
    let _ = runtime_shutdown_complete_signal_trigger.send(());
  });

  // Wait for SDK server to be ready and sync providers before bridge sync
  let socket_path = DEFAULT_SOMA_SERVER_SOCK.to_string();
  info!("Waiting for SDK server to be ready...");
  match tokio::time::timeout(
    Duration::from_secs(30),
    establish_connection_with_retry(&socket_path)
  ).await {
    Ok(Ok(_)) => {
      info!("SDK server is ready, syncing providers...");
      // Fetch metadata and sync providers
      let mut client = create_unix_socket_client(&socket_path).await?;
      let request = tonic::Request::new(());
      let response = client
        .metadata(request)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to get SDK metadata: {e}")))?;
      let metadata = response.into_inner();
      sync_providers_from_metadata(&metadata)?;
      info!("SDK providers synced successfully");
    }
    Ok(Err(e)) => {
      warn!("Failed to connect to SDK server: {:?}. Bridge sync will proceed without SDK providers.", e);
    }
    Err(_) => {
      warn!("Timeout waiting for SDK server. Bridge sync will proceed without SDK providers.");
    }
  }

  // Sync bridge from soma definition (now SDK providers should be available)
  sync_bridge_db_from_soma_definition_on_start(
    &envelope_encryption_key_contents,
    &on_bridge_change_evt_tx,
    &bridge_repo,
    &repository,
    &soma_definition,
  )
  .await?;


  // Reload soma definition
  soma_definition.reload().await?;

  // Wait for Restate server to be ready before creating AdminClient
  info!("Waiting for Restate server to be ready...");
  let restate_admin_client = loop {
    match restate_params.get_admin_client().await {
      Ok(client) => {
        info!("Restate server is ready");
        break client;
      }
      Err(e) => {
        warn!("Restate server not ready yet: {:?}. Retrying...", e);
        tokio::time::sleep(Duration::from_millis(500)).await;
      }
    }
  };

  // Create MCP transport channel
  let (mcp_transport_tx, mcp_transport_rx) = tokio::sync::mpsc::unbounded_channel();

  // Initialize routers
  let routers = router::Routers::new(
    router::InitRouterParams {
      project_dir: project_dir.clone(),
      host: params.host.clone(),
      port: params.port,
      connection_manager: connection_manager.clone(),
      repository: repository.clone(),
      mcp_transport_tx,
      soma_definition: soma_definition.clone(),
      runtime_port,
      restate_ingress_client: restate_params.get_ingress_client()?,
      restate_admin_client,
      db_connection: conn.clone(),
      on_bridge_config_change_tx: on_bridge_change_evt_tx.clone(),
      envelope_encryption_key_contents: envelope_encryption_key_contents.clone(),
      bridge_repository: bridge_repo.clone(),
      mcp_sse_ping_interval: Duration::from_secs(10),
    },
  ).await?;

  // Start Axum server subsystem
  // let (on_server_started_tx, on_server_started_rx) = oneshot::channel::<()>();
  let axum_system_shutdown_signal_rx = system_shutdown_signal_trigger.subscribe();
  let (axum_shutdown_complete_signal_trigger, axum_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  let routers_clone = routers.clone();
  let project_dir_clone = project_dir.clone();
  let host_clone = params.host.clone();
  let port_clone = params.port;


  let (server_fut, _handle, _addr) = match start_axum_server(StartAxumServerParams {
    routers: routers_clone,
    project_dir: project_dir_clone,
    host: host_clone,
    port: port_clone,
    system_shutdown_signal_rx: axum_system_shutdown_signal_rx,
  }).await {
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


  // Start MCP transport processor subsystem
  let mcp_system_shutdown_signal_rx = system_shutdown_signal_trigger.subscribe();
  let (mcp_shutdown_complete_signal_trigger, mcp_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  let bridge_service_clone = routers.bridge_service.clone();
  tokio::spawn(async move {
    let res = start_mcp_connection_manager(StartMcpConnectionManagerParams {
      bridge_service: bridge_service_clone,
      mcp_transport_rx,
      system_shutdown_signal_rx: mcp_system_shutdown_signal_rx,
    }).await;
    match res {
      Ok(()) => info!("MCP transport processor stopped gracefully"),
      Err(e) => error!("MCP transport processor stopped with error: {:?}", e),
    }
    let _ = mcp_shutdown_complete_signal_trigger.send(());
  });


  // Start SDK reload watcher subsystem
  // let mut sdk_watcher_kill_signal_rx = system_shutdown_signal_trigger.subscribe();
  let (sdk_watcher_shutdown_complete_signal_trigger, sdk_watcher_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  let restate_params_sdk = restate_params.clone();
  let socket_path = DEFAULT_SOMA_SERVER_SOCK.to_string();
  let sync_dev_runtime_changes_from_sdk_server_system_shutdown_signal_rx = system_shutdown_signal_trigger.subscribe();
  tokio::spawn(async move {
    // tokio::select! {
    //   _ = sdk_watcher_kill_signal_rx.recv() => {
    //     info!("SDK reload watcher shutdown requested");
    //   }
    //   result = sync_dev_runtime_changes_from_sdk_server(&socket_path, &restate_params_sdk, runtime_port) => {
    //     if let Err(e) = result {
    //       error!("SDK reload watcher stopped: {:?}", e);
    //     }
    //   }
    // }
    let res = sync_dev_runtime_changes_from_sdk_server(SyncDevRuntimeChangesFromSdkServerParams {
      socket_path: socket_path.clone(),
      restate_params: restate_params_sdk,
      runtime_port,
      system_shutdown_signal_rx: sync_dev_runtime_changes_from_sdk_server_system_shutdown_signal_rx,
    }).await;
    match res {
      Ok(()) => info!("sync_dev_runtime_changes_from_sdk_server stopped gracefully"),
      Err(e) => error!("sync_dev_runtime_changes_from_sdk_server stopped with error: {:?}", e),
    }
    let _ = sdk_watcher_shutdown_complete_signal_trigger.send(());
  });

  // Start Restate deployment subsystem
  // This will move to the sync_dev_runtime_changes_from_sdk_server
  // info!("Starting Restate deployment");
  // let service_uri = format!("http://{}:{}", params.host, runtime_port);
  // let deployment_type = crate::utils::restate::deploy::DeploymentType::Http {
  //   uri: service_uri.clone(),
  //   additional_headers: std::collections::HashMap::new(),
  // };
  // let service_path = soma_definition.get_definition().await?.project.clone();
  // let mut deployment_kill_signal_rx = system_shutdown_signal_trigger.subscribe();
  // let (deployment_shutdown_complete_signal_trigger, deployment_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  // let restate_params_deployment = restate_params.clone();
  // let deployment_type_clone = deployment_type.clone();
  // let service_path_clone = service_path.clone();
  // tokio::spawn(async move {
  //   tokio::select! {
  //     _ = deployment_kill_signal_rx.recv() => {
  //       info!("Restate deployment shutdown requested");
  //     }
  //     result = start_restate_deployment(&restate_params_deployment, deployment_type_clone, service_path_clone) => {
  //       if let Err(e) = result {
  //         error!("Restate deployment stopped unexpectedly: {:?}", e);
  //       } else {
  //         info!("Restate deployment completed");
  //       }
  //     }
  //   }
  //   let _ = deployment_shutdown_complete_signal_trigger.send(());
  // });

  // Start credential rotation subsystem
  let (credential_rotation_shutdown_complete_signal_trigger, credential_rotation_shutdown_complete_signal_receiver) = oneshot::channel::<()>();
  let on_bridge_change_evt_tx_clone = on_bridge_change_evt_tx.clone();
  let bridge_repo_clone = bridge_repo.clone();
  let envelope_encryption_key_contents_clone = envelope_encryption_key_contents.clone();
  tokio::spawn(async move {
    bridge::logic::credential_rotation_task(bridge_repo_clone, envelope_encryption_key_contents_clone, on_bridge_change_evt_tx_clone).await;
    info!("Credential rotation stopped gracefully");
    let _ = credential_rotation_shutdown_complete_signal_trigger.send(());
  });

  // Shutdown monitoring thread - handles both unexpected exits and graceful shutdown
  let system_shutdown_signal_trigger_clone = system_shutdown_signal_trigger.clone();
  let mut shutdown_requested_rx = system_shutdown_signal_trigger.subscribe();
  let shutdown_receivers: Vec<(&str, oneshot::Receiver<()>)> = vec![
    ("restate", restate_shutdown_complete_signal_receiver),
    ("file_watcher", prj_shutdown_complete_signal_receiver),
    ("bridge_config_change", on_bridge_shutdown_complete_signal_receiver),
    ("axum_server", axum_shutdown_complete_signal_receiver),
    ("mcp_transport", mcp_shutdown_complete_signal_receiver),
    ("runtime", runtime_shutdown_complete_signal_receiver),
    ("sdk_watcher", sdk_watcher_shutdown_complete_signal_receiver),
    ("credential_rotation", credential_rotation_shutdown_complete_signal_receiver),
  ];
  
  // Systems that can trigger shutdown (unexpected exits)
  let systems_that_can_trigger_shutdown: Vec<&str> = vec![
    "restate",
    "file_watcher",
    "bridge_config_change",
    "axum_server",
    "mcp_transport",
    "runtime",
    "sdk_watcher",
    "credential_rotation",
  ];
  
  // Systems that require graceful shutdown (we wait for these after shutdown is triggered)
  let systems_requiring_graceful_shutdown: Vec<&str> = vec![
    "restate",
    "axum_server",
    "runtime",
    "mcp_transport",
    "sdk_watcher",
  ];
  
  // Track which systems have completed
  use std::sync::{Arc, Mutex};
  use std::collections::HashSet;
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
        info!("System shutdown gracefully, triggered by unexpected exit of {} system", triggered_name);
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
        let all_complete = systems_to_wait_for_check.iter().all(|name| completed.contains(name));
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
fn setup_encryption_key(key_encryption_key: Option<String>) -> Result<EnvelopeEncryptionKeyContents, CommonError> {
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
fn load_soma_definition(
  src_dir: &PathBuf,
) -> Result<Arc<dyn SomaAgentDefinitionLike>, CommonError> {
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
