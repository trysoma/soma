use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use futures::future;
use indicatif::ProgressBar;
use tokio::sync::oneshot;
use tracing::debug;
use tracing::error;
use tracing::trace;
use url::Url;

use shared::error::CommonError;
use shared::soma_agent_definition::{SomaAgentDefinitionLike, YamlSomaAgentDefinition};

use crate::mcp::run_mcp_sync_to_yaml_loop;
use crate::server::start_axum_server;
use crate::utils::{CliConfig, construct_cwd_absolute, create_and_wait_for_api_client};
use shared::process_manager::CustomProcessManager;
use soma_api_server::factory::{CreateApiServiceParams, create_api_service};

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

    #[arg(
        long,
        help = "Delete the local sqlite DB before starting (only applies to local sqlite DB)"
    )]
    pub clean: bool,
}

/// Main entry point for the start command
pub async fn cmd_dev(params: DevParams, cli_config: &mut CliConfig) -> Result<(), CommonError> {
    // Create shutdown notification channel
    let (shutdown_notifier_tx, mut shutdown_notifier_rx) = oneshot::channel();

    // Create process manager with shutdown notifier (uses interior mutability, no Mutex needed)
    let process_manager =
        shared::process_manager::CustomProcessManager::new_with_shutdown_notifier(Some(
            shutdown_notifier_tx,
        ))
        .await
        .inspect_err(|_e| {
            error!("Failed to start process manager");
        })?;

    // Wrap in Arc for sharing across tasks (no Mutex needed - interior mutability)
    let process_manager_arc = Arc::new(process_manager);
    let process_manager_arc_for_shutdown = process_manager_arc.clone();
    let params_clone = params.clone();
    let cli_config_clone = cli_config.clone();
    let mut cmd_dev_inner_handle = tokio::spawn(async move {
        cmd_dev_inner(params_clone, &cli_config_clone, process_manager_arc).await
    });

    // Wait for one of: Ctrl+C, cmd_dev_inner to complete/error, or shutdown notification
    let cmd_result: Result<(), CommonError>;

    let shutdown_reason = tokio::select! {
        biased;

        _ = tokio::signal::ctrl_c() => {
            debug!("Shutdown signal received (Ctrl+C)");
            cmd_result = Ok(());
            "ctrl_c"
        }

        _ = &mut shutdown_notifier_rx => {
            debug!("Process manager triggered shutdown");
            cmd_result = Ok(());
            "notifier"
        }

        result = &mut cmd_dev_inner_handle => {
            match result {
                Ok(Ok(())) => {
                    debug!("cmd_dev_inner completed successfully");
                    cmd_result = Ok(());
                    "cmd_completed"
                }
                Ok(Err(e)) => {
                    debug!(error = ?e, "cmd_dev_inner returned error");
                    cmd_result = Err(e);
                    "cmd_error"
                }
                Err(e) => {
                    debug!(error = ?e, "cmd_dev_inner panicked");
                    cmd_result = Err(CommonError::Unknown(anyhow::anyhow!("{e:?}")));
                    "cmd_panic"
                }
            }
        }
    };

    debug!(reason = shutdown_reason, "Initiating shutdown sequence");

    if !matches!(shutdown_reason, "cmd_completed" | "cmd_error" | "cmd_panic") {
        cmd_dev_inner_handle.abort();
        let _ = cmd_dev_inner_handle.await;
    }

    process_manager_arc_for_shutdown.trigger_shutdown().await?;
    process_manager_arc_for_shutdown
        .on_shutdown_complete()
        .await?;

    debug!("Shutdown complete");

    cmd_result
}

/// Inner implementation of the dev command
async fn cmd_dev_inner(
    params: DevParams,
    _cli_config: &CliConfig,
    process_manager: Arc<CustomProcessManager>,
) -> Result<(), CommonError> {
    let project_dir = construct_cwd_absolute(params.clone().cwd)?;

    debug!(
        "Starting dev server in project directory: {}",
        project_dir.display()
    );

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
            debug!(
                "Libsql connection is a relative path and --clean flag is set, cleaning local sqlite DB"
            );
            trace!("Deleting local sqlite DB file: {}", absolute_path.display());
            std::fs::remove_file(absolute_path)
                .inspect_err(|_e| {
                    error!("Failed to clean local sqlite DB");
                })
                .map_err(CommonError::from)?;
            trace!("Local sqlite DB file deleted successfully");
        }

        Url::parse(&new_url_str).unwrap_or_else(|_| params.db_conn_string.clone())
    } else {
        debug!(
            "Libsql connection is a remote HTTP connection or an absolute file path, using as is"
        );
        params.db_conn_string.clone()
    };

    trace!("Libsql database setup complete");

    // Load soma definition
    trace!("Loading soma definition");
    let soma_definition: Arc<dyn SomaAgentDefinitionLike> = load_soma_definition(&project_dir)
        .inspect_err(|_e| {
            error!("Failed to load soma definition");
        })?;
    debug!(
        "soma definition: {:?}",
        soma_definition.get_definition().await?
    );
    trace!("Soma definition loaded");

    // Create API service and start all subsystems
    trace!("Starting API server");
    let mut bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Waiting for API server to start...");
    let api_service_bundle = create_api_service(CreateApiServiceParams {
        base_url: format!("http://{}:{}", params.host, params.port),
        project_dir: project_dir.clone(),
        db_conn_string: db_conn_string.to_string(),
        db_auth_token: params.db_auth_token.clone(),
        process_manager: process_manager.clone(),
    })
    .await?;
    bar.finish_and_clear();
    trace!("API server started");

    // Start MCP config change listener (uses unified change channel from factory)
    trace!("Starting MCP config change listener...");
    let soma_definition_for_mcp = soma_definition.clone();
    let project_dir_for_mcp = project_dir.clone();
    let soma_change_rx = api_service_bundle.soma_change_tx.subscribe();
    process_manager
        .start_thread(
            "mcp_sync_to_yaml",
            shared::process_manager::ThreadConfig {
                spawn_fn: move || {
                    let soma_definition = soma_definition_for_mcp.clone();
                    let project_dir = project_dir_for_mcp.clone();
                    let soma_change_rx = soma_change_rx.resubscribe();
                    tokio::spawn(async move {
                        run_mcp_sync_to_yaml_loop(soma_definition, project_dir, soma_change_rx)
                            .await
                    })
                },
                health_check: None,
                on_terminal_stop: shared::process_manager::OnTerminalStop::Ignore,
                on_stop: shared::process_manager::OnStop::Nothing,
                shutdown_priority: 2,
                follow_logs: false,
                on_shutdown_triggered: None,
                on_shutdown_complete: None,
            },
        )
        .await
        .inspect_err(|e| error!(error = %e, "Failed to start MCP sync to YAML thread"))?;
    trace!("MCP config change listener started");
    let api_service = api_service_bundle.api_service;

    // Start Axum server subsystem
    start_axum_server(crate::server::StartAxumServerParams {
        api_service: api_service.clone(),
        host: params.host.clone(),
        port: params.port,
        process_manager: process_manager.clone(),
    })
    .await
    .inspect_err(|e| {
        error!("Failed to start Axum server: {:?}", e);
    })?;

    // Create API client configuration for the soma API server and exchange STS token
    let api_base_url = format!("http://{}:{}", params.host, params.port);
    bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Synchronizing soma.yaml on server start");
    trace!("Waiting for API service and exchanging STS token");
    let api_config = create_and_wait_for_api_client(
        &api_base_url,
        30,
        Some(api_service_bundle.bootstrap_api_key),
    )
    .await?;
    trace!("API service ready");

    // Enable dev mode STS config for development
    trace!("Enabling dev mode STS configuration");
    let dev_sts_result = enable_dev_mode_sts(&api_config).await;
    match dev_sts_result {
        Ok(()) => trace!("Dev mode STS configuration enabled"),
        Err(e) => debug!(error = ?e, "Failed to enable dev mode STS configuration, continuing"),
    }

    // Sync MCP from soma definition (now all providers should be available)
    trace!("Syncing MCP from soma.yaml");
    crate::mcp::sync_yaml_to_api_on_start::sync_mcp_db_from_soma_definition_on_start(
        &api_config,
        &soma_definition,
    )
    .await?;
    trace!("MCP sync completed");

    // Reload soma definition (with error handling to avoid crashes on race conditions)
    if let Err(e) = soma_definition.reload().await {
        error!(
            "Failed to reload soma definition after mcp sync: {:?}. Continuing with cached definition.",
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
    debug!(
        "Loading soma definition from: {}",
        path_to_soma_definition.display()
    );

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
