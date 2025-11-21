use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use tokio::sync::{broadcast, oneshot};
use tracing::{info, warn};
use url::Url;

use bridge::logic::{PROVIDER_REGISTRY, register_all_bridge_providers};
use shared::error::CommonError;

use crate::{
    codegen::{self},
    commands::dev::bridge_util::providers::soma::SomaProviderController,
    commands::dev::runtime::{
        StartDevRuntimeParams, start_dev_runtime,
        grpc_client::{establish_connection_with_retry, create_unix_socket_client},
        sdk_provider_sync::sync_providers_from_metadata,
        DEFAULT_SOMA_SERVER_SOCK,
    },
    repository::setup_repository,
    utils::{config::CliConfig, construct_src_dir_absolute},
};

#[derive(Args, Debug, Clone)]
pub struct CodegenParams {
    #[arg(long)]
    pub src_dir: Option<PathBuf>,
    #[arg(long, default_value = "libsql://./.soma/local.db?mode=local")]
    pub db_conn_string: Url,
    #[arg(long)]
    pub db_auth_token: Option<String>,
}

pub async fn cmd_codegen(params: CodegenParams, _config: &mut CliConfig) -> Result<(), CommonError> {
    let project_dir = construct_src_dir_absolute(params.src_dir)?;

    // Determine runtime based on project directory
    let runtime = match codegen::determine_runtime_from_dir(&project_dir)? {
        Some(runtime) => runtime,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Could not determine runtime from project directory: {}",
                project_dir.display()
            )))
        }
    };

    info!("Detected runtime: {:?}", runtime);

    // Resolve relative db_conn_string paths relative to project_dir
    let db_conn_string = resolve_db_connection_string(&params.db_conn_string, &project_dir)?;

    // Setup repository
    info!("Setting up repository...");
    let (_db, _conn, repository, bridge_repo) =
        setup_repository(&db_conn_string, &params.db_auth_token).await?;

    // Register all bridge providers before generating code
    info!("Registering bridge providers...");
    register_all_bridge_providers().await?;

    // Register Soma provider controller
    PROVIDER_REGISTRY
        .write()
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to lock provider registry: {}", e)))?
        .push(Arc::new(SomaProviderController::new(repository.clone())));

    // Start SDK dev runtime to load custom providers from project
    info!("Starting SDK dev runtime to load custom providers...");
    let (kill_signal_tx, kill_signal_rx) = broadcast::channel::<()>(1);
    let (file_change_tx, _file_change_rx) = broadcast::channel(10);
    let file_change_tx = Arc::new(file_change_tx);

    let runtime_clone = runtime.clone();
    let project_dir_clone = project_dir.clone();
    let (runtime_complete_tx, runtime_complete_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let _result = start_dev_runtime(StartDevRuntimeParams {
            project_dir: project_dir_clone,
            runtime: runtime_clone,
            runtime_port: 9080, // Default port for standalone mode
            file_change_tx,
            kill_signal_rx,
        })
        .await;
        let _ = runtime_complete_tx.send(());
    });

    // Wait for SDK server to be ready and fetch metadata
    let socket_path = DEFAULT_SOMA_SERVER_SOCK;
    info!("Waiting for SDK server to be ready...");
    match tokio::time::timeout(
        Duration::from_secs(30),
        establish_connection_with_retry(&socket_path.to_string()),
    )
    .await
    {
        Ok(Ok(_)) => {
            info!("SDK server is ready, fetching metadata...");
            let mut client = create_unix_socket_client(&socket_path.to_string()).await?;
            let request = tonic::Request::new(());
            let response = client.metadata(request).await.map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to get SDK metadata: {e}"))
            })?;
            let metadata = response.into_inner();

            info!("Syncing custom providers from project...");
            sync_providers_from_metadata(&metadata)?;
        }
        Ok(Err(e)) => {
            warn!(
                "Failed to connect to SDK server: {:?}. Continuing without custom providers.",
                e
            );
        }
        Err(_) => {
            warn!("Timeout waiting for SDK server. Continuing without custom providers.");
        }
    }

    // Generate bridge client
    info!("Generating bridge client...");
    codegen::regenerate_bridge_client(&runtime, &project_dir, &bridge_repo).await?;

    info!("Bridge client generation complete!");

    // Shut down SDK runtime
    info!("Shutting down SDK dev runtime...");
    let _ = kill_signal_tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(5), runtime_complete_rx).await;

    Ok(())
}

/// Resolves relative database connection string paths
fn resolve_db_connection_string(
    db_conn_string: &Url,
    project_dir: &PathBuf,
) -> Result<Url, CommonError> {
    if db_conn_string.as_str().starts_with("libsql://./") {
        // Extract the path portion after libsql://./
        let url_str = db_conn_string.as_str();
        let path_with_query = url_str.strip_prefix("libsql://./").unwrap_or("");
        let (path_part, query_part) = path_with_query
            .split_once('?')
            .unwrap_or((path_with_query, ""));

        // Resolve relative path to absolute path relative to project_dir
        let absolute_path = project_dir.join(path_part);

        // Reconstruct the URL with absolute path
        let path_str = absolute_path.to_string_lossy();
        let new_url_str = if query_part.is_empty() {
            format!("libsql://{}", path_str)
        } else {
            format!("libsql://{}?{}", path_str, query_part)
        };

        info!("Database path resolved to: {}", absolute_path.display());
        Url::parse(&new_url_str)
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse resolved URL: {}", e)))
    } else {
        Ok(db_conn_string.clone())
    }
}
