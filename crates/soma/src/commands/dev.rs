use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use clap::Parser;
use futures::future;
use indicatif::ProgressBar;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tracing::debug;
use tracing::trace;
use tracing::error;
use url::Url;

use shared::error::CommonError;
use shared::port::find_free_port;
use shared::soma_agent_definition::{SomaAgentDefinitionLike, YamlSomaAgentDefinition};

use crate::bridge::start_bridge_sync_to_yaml_subsystem;
use shared::process_manager::CustomProcessManager;
use crate::server::{StartAxumServerParams, start_axum_server};
use crate::utils::wait_for_soma_api_health_check;
use crate::utils::{CliConfig, construct_cwd_absolute};
use soma_api_server::factory::{CreateApiServiceParams, create_api_service};
use soma_api_server::restate::{
    RestateServerLocalParams, RestateServerParams, RestateServerRemoteParams,
};

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
            admin_address: params.admin_url.clone().unwrap(),
            ingress_address: params.ingress_url.clone().unwrap(),
            admin_token: params.admin_token,
            // Default to using the ingress address for the soma restate service
            soma_restate_service_address: params.ingress_url.unwrap(),
            soma_restate_service_additional_headers: std::collections::HashMap::new(),
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
    pub cwd: Option<PathBuf>,
    #[arg(long, default_value = "libsql://./.soma/local.db?mode=local")]
    pub db_conn_string: Url,
    #[arg(long)]
    pub db_auth_token: Option<String>,
    #[command(flatten)]
    pub remote_restate: Option<RemoteRestateParams>,

    #[arg(
        long,
        help = "Delete the Restate data directory, local sqlite DB before starting (only applies to local Restate instances and local sqlite DB)"
    )]
    pub clean: bool,
}

/// Main entry point for the start command
pub async fn cmd_dev(params: DevParams, cli_config: &mut CliConfig) -> Result<(), CommonError> {
    // Create shutdown notification channel
    let (shutdown_notifier_tx, mut shutdown_notifier_rx) = oneshot::channel();
    
    // Create process manager with shutdown notifier
    let mut process_manager = shared::process_manager::CustomProcessManager::new_with_shutdown_notifier(Some(shutdown_notifier_tx)).await
        .inspect_err(|_e| {
            error!("Failed to start process manager");
        })?;
    
    // Spawn cmd_dev_inner
    let process_manager_arc = Arc::new(tokio::sync::Mutex::new(process_manager));
    let process_manager_arc_for_shutdown = process_manager_arc.clone();
    let params_clone = params.clone();
    let cli_config_clone = cli_config.clone();
    let mut cmd_dev_inner_handle = tokio::spawn(async move {
        cmd_dev_inner(params_clone, &cli_config_clone, process_manager_arc).await
    });
    
    // Wait for one of: Ctrl+C, cmd_dev_inner to complete/error, or shutdown notification
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            debug!("Shutdown signal received, triggering graceful shutdown");
        }
        result = &mut cmd_dev_inner_handle => {
            match result {
                Ok(Ok(())) => {
                    debug!("cmd_dev_inner completed successfully");
                }
                Ok(Err(e)) => {
                    debug!(error = ?e, "cmd_dev_inner returned error");
                    return Err(e);
                }
                Err(e) => {
                    debug!(error = ?e, "cmd_dev_inner panicked");
                    return Err(CommonError::Unknown(anyhow::anyhow!("{e:?}")));
                }
            }
        }
        _ = &mut shutdown_notifier_rx => {
            debug!("Process manager triggered shutdown");
        }
    }
    
    // Trigger process manager shutdown
    {
        let mut pm = process_manager_arc_for_shutdown.lock().await;
        pm.trigger_shutdown().await?;
        pm.on_shutdown_complete().await?;
    }
    
    debug!("Shutdown complete");
    Ok(())
}

/// Inner implementation of the dev command
async fn cmd_dev_inner(
    params: DevParams,
    _cli_config: &CliConfig,
    process_manager_arc: Arc<tokio::sync::Mutex<shared::process_manager::CustomProcessManager>>,
) -> Result<(), CommonError> {
    let project_dir = construct_cwd_absolute(params.clone().cwd)?;

    debug!("Starting dev server in project directory: {}", project_dir.display());
    trace!("Starting process manager");
    let mut process_manager = CustomProcessManager::new().await
        .inspect_err(|_e| {
            error!("Failed to start process manager");
        })?;
    trace!("Process manager started");

    trace!("setting up Libsql database");
    // Resolve relative db_conn_string paths relative to project_dir
    let db_conn_string = if params.db_conn_string.as_str().starts_with("libsql://./") {
        debug!("Libsql connection is a relative path, resolving to absolute path");
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

        debug!("Database path resolved to: {}", absolute_path.display());

        if params.clean && absolute_path.exists() {
            debug!("Libsql connection is a relative path and --clean flag is set, cleaning local sqlite DB");
            trace!("Deleting local sqlite DB file: {}", absolute_path.display());
            std::fs::remove_file(absolute_path)
            .inspect_err(|_e| {
                error!("Failed to clean local sqlite DB");
            })
            .map_err(|e| CommonError::from(e))?;
            trace!("Local sqlite DB file deleted successfully");
        }

        Url::parse(&new_url_str).unwrap_or_else(|_| params.db_conn_string.clone())
    } else {
        debug!("Libsql connection is a remote HTTP connection or an absolute file path, using as is");
        params.db_conn_string.clone()
    };

    trace!("Libsql database setup complete");

    // Load soma definition
    trace!("Loading soma definition");
    let soma_definition: Arc<dyn SomaAgentDefinitionLike> = load_soma_definition(&project_dir)
        .inspect_err(|_e| {
            error!("Failed to load soma definition");
        })?;
    debug!("soma definition: {:?}", soma_definition.get_definition().await?);
    trace!("Soma definition loaded");

    trace!("Configuring restate server");
    // Find free port for SDK server
    let soma_restate_service_port = find_free_port(9080, 10080)?;

    // Setup Restate parameters
    let restate_params = match params.remote_restate {
        Some(remote_restate) => {
            debug!("Configuring remote restate server parameters");
            debug!("restate admin url: {:?}", remote_restate.admin_url);
            debug!("restate ingress url: {:?}", remote_restate.ingress_url);
            debug!("restate admin token: **********");
            remote_restate.try_into()?
        },
        None => {
            let restate_server_data_dir = project_dir.join(".soma/restate-data");
            let ingress_port = 8080;
            let admin_port = 9070;
            debug!("Configuring local restate server parameters");
            debug!("restate server data directory: {:?}", restate_server_data_dir);
            debug!("restate ingress port (this is where requests to trigger a restate workflow are sent): {:?}", ingress_port);
            debug!("restate admin port (this is where the restate admin API is exposed): {:?}", admin_port);
            debug!("restate soma restate service port (this is where the Soma SDK restate service is exposed): {:?}", soma_restate_service_port);
            debug!("restate clean: {:?}", params.clean);
            RestateServerParams::Local(RestateServerLocalParams {
                restate_server_data_dir,
                ingress_port,
                admin_port,
                soma_restate_service_port,
                soma_restate_service_additional_headers: std::collections::HashMap::new(),
                clean: params.clean,
            })
        },
    };


    // Start Restate server subsystem
    let mut bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Waiting for Restate to start...");
    crate::restate_server::start_restate(
        &mut process_manager,
        restate_params.clone(),
    ).await?;
    bar.finish_and_clear();
    trace!("Restate server configured");

    // Create API service and start all subsystems
    trace!("Starting API server");
    bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Waiting for API server to start...");
    let process_manager_arc = Arc::new(tokio::sync::Mutex::new(process_manager));
    let api_service_bundle = create_api_service(CreateApiServiceParams {
        project_dir: project_dir.clone(),
        host: params.host.clone(),
        port: params.port,
        base_url: format!("http://{}:{}", params.host, params.port),
        soma_restate_service_port,
        db_conn_string: db_conn_string.to_string(),
        db_auth_token: params.db_auth_token.clone(),
        soma_definition: soma_definition.clone(),
        restate_params: restate_params.clone(),
        process_manager: process_manager_arc.clone(),
    })
    .await?;
    bar.finish_and_clear();
    trace!("API server started");

    // Start bridge config change listener subsystem (uses unified change channel from factory)
    trace!("Starting bridge config change listener...");
    let bridge_sync_handle = start_bridge_sync_to_yaml_subsystem(
        soma_definition.clone(),
        project_dir.clone(),
        api_service_bundle.soma_change_tx.subscribe(),
    )?;
    trace!("Bridge config change listener started");
    let api_service = api_service_bundle.api_service;
    let subsystems = api_service_bundle.subsystems;


    // Start Axum server subsystem
    let api_service_clone = api_service.clone();
    let host_clone = params.host.clone();
    let port_clone = params.port;

    let axum_server_result = match start_axum_server(StartAxumServerParams {
        api_service: api_service_clone,
        host: host_clone,
        port: port_clone,
    })
    .await
    {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to start Axum server: {:?}", e);
            return Err(e);
        }
    };

    // Register axum server with process manager
    let on_shutdown_triggered = axum_server_result.on_shutdown_triggered;
    let on_shutdown_complete = axum_server_result.on_shutdown_complete;
    
    // Start the server future in a separate task (not managed by process manager since it's a one-shot)
    let server_fut = axum_server_result.server_fut;
    tokio::spawn(async move {
        let res = server_fut.await;
        match res {
            Ok(()) => trace!("Axum server stopped"),
            Err(e) => error!(error = ?e, "Axum server stopped with error"),
        }
    });
    
    let process_manager_for_axum = process_manager_arc.clone();
    process_manager_for_axum.lock().await.start_thread("axum_server", shared::process_manager::ThreadConfig {
        spawn_fn: move || {
            // This thread just waits forever since the server is running in a separate task
            tokio::spawn(async move {
                futures::future::pending::<Result<(), CommonError>>().await
            })
        },
        health_check: None,
        on_terminal_stop: shared::process_manager::OnTerminalStop::TriggerShutdown,
        on_stop: shared::process_manager::OnStop::Nothing,
        shutdown_priority: 9,
        follow_logs: false,
        on_shutdown_triggered: Some(on_shutdown_triggered),
        on_shutdown_complete: Some(on_shutdown_complete),
    }).await
    .inspect_err(|e| error!(error = %e, "Failed to register axum server with process manager"))?;

    // Create API client configuration for the soma API server
    let api_base_url = format!("http://{}:{}", params.host, params.port);
    let api_config = crate::utils::create_api_client_config(&api_base_url);

    // Wait for API service to be ready
    bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Synchronizing soma.yaml on server start");
    trace!("Waiting for API service");
    wait_for_soma_api_health_check(&api_config, 30, 10).await?;
    trace!("API service ready");
    // Sync bridge from soma definition (now all providers should be available)
    trace!("Syncing bridge from soma.yaml");
    crate::bridge::sync_yaml_to_api_on_start::sync_bridge_db_from_soma_definition_on_start(
        &api_config,
        &soma_definition,
    )
    .await?;
    trace!("Bridge sync completed");

    // Enable dev mode STS config for development
    trace!("Enabling dev mode STS configuration");
    let dev_sts_result = enable_dev_mode_sts(&api_config).await;
    match dev_sts_result {
        Ok(()) => trace!("Dev mode STS configuration enabled"),
        Err(e) => debug!(error = ?e, "Failed to enable dev mode STS configuration, continuing"),
    }

    // Give SDK server time to fully initialize its gRPC handlers after bridge sync
    // This ensures that secrets/env vars created during bridge sync can be synced properly
    trace!("Waiting for SDK server after bridge sync");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Reload soma definition (with error handling to avoid crashes on race conditions)
    if let Err(e) = soma_definition.reload().await {
        error!(
            "Failed to reload soma definition after bridge sync: {:?}. Continuing with cached definition.",
            e
        );
        // Don't fail the entire process - the cached definition should still be valid
    }
    bar.finish_and_clear();

    // Wait indefinitely - shutdown will be handled by the outer cmd_dev function
    future::pending::<()>().await;
    Ok(())
}

/// Loads the soma definition from the source directory
fn load_soma_definition(
    project_dir: &Path,
) -> Result<Arc<dyn SomaAgentDefinitionLike>, CommonError> {
    let path_to_soma_definition = project_dir.join("soma.yaml");
    debug!("Loading soma definition from: {}", path_to_soma_definition.display());

    if !path_to_soma_definition.exists() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Soma definition not found at {}",
            path_to_soma_definition.display()
        )));
    }
    let soma_definition = YamlSomaAgentDefinition::load_from_file(path_to_soma_definition)?;
    Ok(Arc::new(soma_definition))
}

/// Enables the dev mode STS configuration for development
/// This allows unauthenticated access during development
async fn enable_dev_mode_sts(
    api_config: &soma_api_client::apis::configuration::Configuration,
) -> Result<(), CommonError> {
    use soma_api_client::apis::identity_api;
    use soma_api_client::models;

    const DEV_MODE_STS_ID: &str = "dev";

    // Check if dev mode STS config already exists
    let existing_configs = identity_api::route_list_sts_configs(api_config, 100, None)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to list STS configs: {e:?}")))?;

    let dev_mode_exists = existing_configs.items.iter().any(|config| match config {
        models::StsTokenConfig::StsTokenConfigOneOf1(c) => c.dev_mode.id == DEV_MODE_STS_ID,
        _ => false,
    });

    if dev_mode_exists {
        trace!("Dev mode STS configuration already exists");
        return Ok(());
    }

    // Create dev mode STS config
    let params = models::StsTokenConfig::StsTokenConfigOneOf1(models::StsTokenConfigOneOf1 {
        dev_mode: models::DevModeConfig {
            id: DEV_MODE_STS_ID.to_string(),
        },
    });

    identity_api::route_create_sts_config(api_config, params)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to create dev mode STS config: {e:?}"
            ))
        })?;

    Ok(())
}
