use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{future::Future, process::Stdio};

use clap::Parser;
use futures::{FutureExt, TryFutureExt, future};
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use rmcp::service::serve_directly_with_ct;
use shared::libsql::{
    establish_db_connection, inject_auth_token_to_db_url, merge_nested_migrations,
};
use shared::primitives::SqlMigrationLoader;
use tokio::process::Command;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::{debug, error, info, warn};

use crate::logic::ConnectionManager;
use crate::repository::Repository;
use crate::router;
use crate::router::RouterParams;
use crate::utils::restate::deploy::DeploymentRegistrationConfig;
use crate::utils::restate::invoke::RestateIngressClient;
use crate::utils::soma_agent_config::SomaConfig;
use crate::utils::{self, construct_src_dir_absolute, restate};
use crate::vite::{Assets, wait_for_vite_dev_server_shutdown};
use shared::command::run_child_process;
use shared::{error::CommonError, node::override_path_env};
use url::Url;

#[derive(Debug, Clone, Parser)]
pub struct StartParams {
    #[arg(long, default_value = "3000")]
    pub port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long)]
    pub src_dir: Option<PathBuf>,
    #[arg(long, default_value = "libsql:///var/lib/soma/local.db?mode=local")]
    pub db_conn_string: Url,
    #[arg(long)]
    pub db_auth_token: Option<String>,
    #[arg(long, default_value = "http://localhost:9070")]
    pub restate_admin_url: Url,
    #[arg(long, default_value = "http://localhost:8080")]
    pub restate_ingress_url: Url,
    #[arg(long)]
    pub restate_admin_token: Option<String>,
}

async fn setup_repository(
    conn_string: Url,
    auth_token: Option<String>,
) -> Result<(libsql::Database, shared::libsql::Connection, Repository), CommonError> {
    info!("starting metadata database");

    let migrations = Repository::load_sql_migrations();
    let migrations = merge_nested_migrations(vec![migrations]);
    let auth_conn_string = inject_auth_token_to_db_url(&conn_string, &auth_token)?;
    let (db, conn) = establish_db_connection(&auth_conn_string, Some(migrations)).await?;

    let repo = Repository::new(conn.clone());
    Ok((db, conn, repo))
}

fn find_free_port(start: u16, end: u16) -> std::io::Result<u16> {
    for port in start..=end {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if TcpListener::bind(addr).is_ok() {
            return Ok(port);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrNotAvailable,
        "No free ports found",
    ))
}

pub async fn cmd_start(subsys: &SubsystemHandle, params: StartParams) -> Result<(), CommonError> {
    let connection_manager = ConnectionManager::new();
    let (db, conn, repository) =
        setup_repository(params.db_conn_string.clone(), params.db_auth_token.clone()).await?;

    let (kill_restate_signal_trigger, kill_restate_signal_receiver) = oneshot::channel::<()>();
    let (shutdown_complete_restate_signal_trigger, shutdown_complete_restate_signal_receiver) =
        oneshot::channel::<()>();
    subsys.start(SubsystemBuilder::new(
        "restate",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("Shutting down restate");
                    let _ = kill_restate_signal_trigger.send(());
                    // Ignore channel errors - child may have already exited
                    let _ = shutdown_complete_restate_signal_receiver.await;
                },
                _ = start_restate_server(kill_restate_signal_receiver, shutdown_complete_restate_signal_trigger) => {
                    info!("Restate server stopped");
                    subsys.request_shutdown();
                }
            }
            Ok::<(), CommonError>(())
        },
    ));

    let runtime = determine_runtime(params.clone())?;
    let runtime_port = find_free_port(9080, 10080)?;

    let runtime = match runtime {
        Some(runtime) => runtime,
        None => return Err(CommonError::Unknown(anyhow::anyhow!("No runtime matched"))),
    };

    let (file_change_tx, mut _file_change_rx) = broadcast::channel::<FileChangeEvt>(10);
    let file_change_tx = Arc::new(file_change_tx);
    let src_dir = construct_src_dir_absolute(params.clone().src_dir)?;
    let src_dir_clone = src_dir.clone();
    let runtime_clone = runtime.clone();
    let file_change_tx_clone = file_change_tx.clone();
    subsys.start(SubsystemBuilder::new(
        "file-watcher",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    info!("File watcher stopped");
                }
                _ = start_dev_file_watcher(src_dir_clone, runtime_clone, file_change_tx_clone) => {
                    info!("File watcher stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    let file_change_tx_clone = file_change_tx.clone();
    subsys.start(SubsystemBuilder::new(
        "restartable-processes-on-config-change",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    subsys.wait_for_children().await;
                    info!("Restartable processes on config change stopped");
                }
                _ = start_dev_restartable_processes_on_config_change(&subsys, DevRestartableProcesses {
                    runtime,
                    runtime_port,
                    file_change_tx: file_change_tx_clone,
                    src_dir,
                    params,
                    connection_manager,
                    repository,
                }) => {
                    info!("Restartable processes on config change stopped unexpectedly");
                    subsys.wait_for_children().await;
                    info!("Restartable processes on config change stopped");
                }
            }
            
            Ok::<(), CommonError>(())
        },
    ));

    subsys.on_shutdown_requested().await;
    Ok(())
}

#[derive(Clone)]
struct DevRestartableProcesses {
    runtime: Runtime,
    runtime_port: u16,
    file_change_tx: Arc<FileChangeTx>,
    src_dir: PathBuf,
    params: StartParams,
    connection_manager: ConnectionManager,
    repository: Repository,
}

async fn start_dev_restartable_processes_on_config_change(
    subsys: &SubsystemHandle,
    params: DevRestartableProcesses,
) -> Result<(), CommonError> {
    loop {
        let params_clone = params.clone();
        let mut file_change_rx: broadcast::Receiver<FileChangeEvt> =
            params_clone.file_change_tx.subscribe();

        info!("🔁  starting system after config change");
        let soma_config = get_soma_config(params_clone.src_dir.clone())?;
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
                    _ = start_dev_restartable_processes(&subsys, params_clone.clone(), soma_config) => {
                        info!("Processes exitted unexpectedly, something went wrong.");
                        return Ok(());
                    }
                };

                Ok::<(), CommonError>(())
            },
        ));
        restart_rx.await?;
    }
}

async fn start_dev_restartable_processes(
    subsys: &SubsystemHandle,
    params: DevRestartableProcesses,
    soma_config: SomaConfig,
) -> Result<(), CommonError> {
    let DevRestartableProcesses {
        runtime,
        runtime_port,
        file_change_tx,
        src_dir,
        params,
        connection_manager,
        repository,
    } = params;

    let (kill_runtime_signal_trigger, kill_runtime_signal_receiver) = broadcast::channel::<()>(1);
    let (shutdown_runtime_complete_signal_trigger, shutdown_runtime_complete_signal_receiver) =
        oneshot::channel::<()>();
    let src_dir_clone = src_dir.clone();
    let runtime_clone = runtime.clone();
    let runtime_port_clone = runtime_port.clone();
    let mut file_change_rx = file_change_tx.subscribe();
    subsys.start(SubsystemBuilder::new(
        "runtime",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    // Ignore channel errors - child may have already exited
                    let _ = kill_runtime_signal_trigger.send(());
                    let _ = shutdown_runtime_complete_signal_receiver.await;
                    info!("Runtime shutdown complete");
                }
                _ = start_dev_runtime(src_dir_clone, runtime_clone, runtime_port_clone, &mut file_change_rx, kill_runtime_signal_receiver, shutdown_runtime_complete_signal_trigger) => {
                    info!("Runtime stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())

        },
    ));

    let params_clone = params.clone();
    let (mcp_transport_tx, mut mcp_transport_rx) = tokio::sync::mpsc::unbounded_channel();
    let soma_config_clone = soma_config.clone();
    let restate_ingress_client = RestateIngressClient::new(params.restate_ingress_url.to_string());
    let router_params = router::RouterParams::new(
        params.clone(),
        router::InitRouterParams {
            connection_manager,
            repository,
            mcp_transport_tx,
            soma_config: soma_config_clone,
            runtime_port: runtime_port,
            restate_ingress_client,
        },
    )?;

    let router_params_clone = router_params.clone();
    subsys.start(SubsystemBuilder::new(
        "axum-server",
        move |subsys: SubsystemHandle| async move {
            #[cfg(debug_assertions)]
            let _vite_scope_guard = {
                info!("Starting vite dev server");
                Assets::start_dev_server(false)
            };
            let (server_fut, handle) = start_axum_server(params_clone, router_params_clone)?;
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("Shutting down axum server");
                    #[cfg(debug_assertions)]
                    {
                        drop(_vite_scope_guard);
                        info!("Stopping vite dev server");
                        Assets::stop_dev_server();
                        wait_for_vite_dev_server_shutdown().await?;
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

    let mcp_service = router_params.mcp_service.clone();
    let mcp_ct = tokio_util::sync::CancellationToken::new();
    subsys.start(SubsystemBuilder::new(
        "mcp-server",
        move |subsys: SubsystemHandle| {
            let mcp_server_instance = mcp_service.clone();
            let mcp_ct = mcp_ct.clone();

            async move {
                loop {
                    tokio::select! {
                        _ = subsys.on_shutdown_requested() => {
                            tracing::info!("mcp-server subsystem shutdown requested.");
                            mcp_ct.cancel();
                            break;
                        }
                        maybe_transport = mcp_transport_rx.recv() => {
                            match maybe_transport {
                                Some(transport) => {
                                    let service = mcp_server_instance.clone();
                                    let ct = mcp_ct.child_token();

                                    tokio::spawn(async move {
                                        let server = serve_directly_with_ct(service, transport, None, ct);
                                        server.waiting().await?;
                                        tokio::io::Result::Ok(())
                                    });
                                }
                                None => {
                                    // Sender dropped; nothing left to serve.
                                    break;
                                }
                            }
                        }
                    }
                }

                // Ensure any in-flight sessions are asked to shut down.
                mcp_ct.cancel();

                Ok::<(), CommonError>(())
            }
        },
    ));

    info!("Starting Restate deployment");
    let runtime_port_clone = runtime_port.clone();
    let params_clone = params.clone();
    let soma_config_clone = soma_config.clone();
    subsys.start(SubsystemBuilder::new(
        "restate-deployment",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    info!("Restate deployment shutdown complete");
                }
                _ = start_restate_deployment(params_clone, runtime_port_clone, soma_config_clone) => {
                    info!("Restate deployment completed");
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    subsys.on_shutdown_requested().await;

    Ok(())
}

fn get_soma_config(src_dir: PathBuf) -> Result<SomaConfig, CommonError> {
    let soma_config = utils::soma_agent_config::SomaConfig::from_yaml(&fs::read_to_string(
        src_dir.join("soma.yaml"),
    )?)
    .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to get soma config: {:?}", e)))?;
    Ok(soma_config)
}

async fn on_soma_config_change(file_change_rx: &mut FileChangeRx) -> Result<bool, CommonError> {
    loop {
        let event = file_change_rx.recv().await?;
        if event
            .changes
            .iter()
            .any(|change| change.paths.iter().any(|path| path.ends_with("soma.yaml")))
        {
            return Ok(true);
        }
    }
}

// async fn start_dev_soma_config_watcher(src_dir: PathBuf, mut soma_config_watcher: broadcast::Receiver<()>, soma_config_clone: Arc<Mutex<SomaConfig>>) -> Result<(), CommonError> {
//     loop {
//         let event = soma_config_watcher.recv().await;
//         let mut soma_config = soma_config_clone.lock().await;
//         *soma_config = get_soma_config(src_dir.clone())?;
//     }
// }

async fn start_restate_deployment(
    params: StartParams,
    runtime_port: u16,
    soma_config: SomaConfig,
) -> Result<(), CommonError> {
    info!("Starting Restate deployment registration");

    // The HTTP service URI should point to the local Axum server, not the Restate admin
    let service_uri = format!("http://127.0.0.1:{}", runtime_port);

    info!(
        "Registering service at {} with Restate admin at {}",
        service_uri, params.restate_admin_url
    );
    restate::deploy::register_deployment(DeploymentRegistrationConfig {
        admin_url: params.restate_admin_url.to_string(),
        // TODO: this should be the service path from the soma.yaml file
        service_path: soma_config.project.clone(),
        deployment_type: restate::deploy::DeploymentType::Http {
            uri: service_uri,
            additional_headers: HashMap::new(),
        },
        bearer_token: params.restate_admin_token.clone(),
        private: false,
        insecure: true,
        force: true,
    })
    .await?;

    info!("Restate deployment registration complete");
    Ok(())
}

async fn start_restate_server(
    kill_signal: oneshot::Receiver<()>,
    shutdown_complete: oneshot::Sender<()>,
) -> Result<(), CommonError> {
    let mut cmd = Command::new("restate-server");
    cmd.arg("--log-filter")
        .arg("warn")
        .arg("--tracing-filter")
        .arg("warn");
    run_child_process(
        "restate-server",
        cmd,
        Some(kill_signal),
        Some(shutdown_complete),
        None,
    )
    .await?;
    Ok(())
}

fn start_axum_server(
    params: StartParams,
    router_params: RouterParams,
) -> Result<
    (
        impl Future<Output = Result<(), std::io::Error>>,
        axum_server::Handle,
    ),
    CommonError,
> {
    let addr: SocketAddr = format!("{}:{}", params.host, params.port)
        .parse()
        .map_err(|e| CommonError::AddrParseError { source: e })?;

    info!("Starting server on {}", addr);

    let router = router::initiate_routers(router_params)?;

    let handle = axum_server::Handle::new();
    let handle_clone = handle.clone();
    let server_fut = axum_server::bind(addr)
        .handle(handle)
        .serve(router.into_make_service());

    Ok((server_fut, handle_clone))
}

#[derive(Debug, Clone)]
enum Runtime {
    BunV1,
}

fn validate_runtime_bun_v1(src_dir: PathBuf) -> Result<bool, CommonError> {
    let files_to_check = vec![
        // "bun.lock",
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

async fn start_runtime_bun_v1(
    src_dir: PathBuf,
    runtime_port: u16,
    kill_signal: oneshot::Receiver<()>,
    shutdown_complete: oneshot::Sender<()>,
) -> Result<(), CommonError> {
    let mut cmd = Command::new("bun");
    cmd.arg("index.ts").current_dir(src_dir);
    run_child_process(
        "bun",
        cmd,
        Some(kill_signal),
        Some(shutdown_complete),
        Some(HashMap::from([(
            "PORT".to_string(),
            runtime_port.to_string(),
        )])),
    )
    .await?;

    Ok(())
}

async fn build_runtime_bun_v1(src_dir: PathBuf) -> Result<(), CommonError> {
    Ok(())
}

fn files_to_watch_bun_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    // Add the patterns you care about
    builder.add(Glob::new("**/*.ts")?);
    builder.add(Glob::new("package.json")?);
    builder.add(Glob::new("soma.yaml")?);

    Ok(builder.build()?)
}

fn files_to_ignore_bun_v1() -> Result<GlobSet, CommonError> {
    let mut builder = GlobSetBuilder::new();

    // Match node_modules anywhere in the path
    builder.add(Glob::new("**/node_modules/**")?);

    Ok(builder.build()?)
}

fn collect_paths_to_watch(
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

fn determine_runtime(params: StartParams) -> Result<Option<Runtime>, CommonError> {
    let src_dir = construct_src_dir_absolute(params.src_dir.clone())?;

    let possible_runtimes = vec![(Runtime::BunV1, validate_runtime_bun_v1)];

    let mut matched_runtimes = vec![];

    for (runtime, validate_fn) in possible_runtimes {
        let result = validate_fn(src_dir.clone());

        match result {
            Ok(true) => matched_runtimes.push(runtime),
            Ok(false) => (),
            Err(e) => return Err(e),
        };
    }

    match matched_runtimes.len() {
        0 => return Ok(None),
        1 => return Ok(Some(matched_runtimes[0].clone())),
        _ => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Multiple runtimes matched"
            )));
        }
    };
}

#[derive(Debug, Clone)]
pub struct FileChangeEvt {
    changes: Vec<notify::Event>,
}
pub type FileChangeTx = broadcast::Sender<FileChangeEvt>;
pub type FileChangeRx = broadcast::Receiver<FileChangeEvt>;
async fn start_dev_file_watcher(
    src_dir: PathBuf,
    runtime: Runtime,
    file_change_tx: Arc<FileChangeTx>,
) -> Result<(), CommonError> {
    let (files_to_watch, files_to_ignore) = match runtime {
        Runtime::BunV1 => (files_to_watch_bun_v1()?, files_to_ignore_bun_v1()?),
    };

    let (file_change_debounced_tx, mut file_change_debounced_rx) =
        mpsc::channel::<(Instant, Vec<notify::Event>)>(10);

    // Spawn a helper task that collapses bursts into one event
    tokio::spawn({
        let file_change_tx = file_change_tx.clone();
        async move {
            let mut last_trigger = Instant::now() - Duration::from_secs(10);
            let mut debounced_changes = Vec::new();
            while let Some((ts, changes)) = file_change_debounced_rx.recv().await {
                info!("🔁  file change received, {:?}", changes);
                debounced_changes.extend(changes);
                // If last trigger was within 1s, skip
                if ts.duration_since(last_trigger) < Duration::from_secs(1) {
                    continue;
                }
                last_trigger = ts;

                info!("🔁 Debounced file change fired");
                let _ = file_change_tx.send(FileChangeEvt {
                    changes: debounced_changes,
                });
                debounced_changes = Vec::new();
            }
        }
    });

    // Clone for use in the closure
    let files_to_ignore_clone = files_to_ignore.clone();
    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    // Filter out events from ignored paths
                    let mut any_relevant = vec![];

                    for event in events {
                        debug!("🔁  file change  received, {:?}", event);
                        // Filter out ignored paths
                        let ignored = event
                            .event
                            .paths
                            .iter()
                            .any(|p| files_to_ignore_clone.is_match(p));
                        if ignored {
                            continue;
                        }

                        // Only trigger on write / modify / rename / remove, not reads
                        match event.event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                                any_relevant.push(event.event);
                            }
                            _ => {}
                        }
                    }

                    if !any_relevant.is_empty() {
                        // Blockingly send a single event signal
                        let _ =
                            file_change_debounced_tx.blocking_send((Instant::now(), any_relevant));
                    }
                }
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            }
        },
    )?;

    let paths = collect_paths_to_watch(&src_dir, &files_to_watch, &files_to_ignore);

    for path in paths {
        // For directories, watch recursively; for files, non-recursive
        let mode = if path.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        debouncer.watch(&path, mode)?;
        tracing::debug!("Watching: {:?}", path);
    }

    future::pending::<()>().await;

    Ok(())
}

async fn start_dev_runtime(
    src_dir: PathBuf,
    runtime: Runtime,
    runtime_port: u16,
    file_change_signal: &mut FileChangeRx,
    mut kill_signal: broadcast::Receiver<()>,
    shutdown_complete_signal: oneshot::Sender<()>,
) -> Result<(), CommonError> {
    loop {
        let (dev_kill_signal_tx, dev_kill_signal_rx) = oneshot::channel::<()>();
        let (dev_shutdown_complete_tx, dev_shutdown_complete_rx) = oneshot::channel::<()>();
        // let runtime_fut = match runtime {
        //     Runtime::BunV1 => start_runtime_bun_v1(src_dir.clone(), dev_kill_signal_rx, dev_shutdown_complete_tx),
        // };

        let serve_fut = match runtime {
            Runtime::BunV1 => build_runtime_bun_v1(src_dir.clone()).and_then(|_| {
                start_runtime_bun_v1(
                    src_dir.clone(),
                    runtime_port,
                    dev_kill_signal_rx,
                    dev_shutdown_complete_tx,
                )
            }),
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
