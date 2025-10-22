use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bridge::logic::DecryptionService;
use bridge::logic::FunctionControllerLike;
use bridge::logic::Metadata;
use bridge::logic::no_auth::NoAuthStaticCredentialConfiguration;
use bridge::logic::OnConfigChangeEvt;
use bridge::logic::OnConfigChangeRx;
use bridge::logic::ResourceServerCredentialSerialized;
use bridge::logic::StaticCredentialConfigurationLike;
use bridge::logic::StaticProviderCredentialControllerLike;
use bridge::logic::UserCredentialSerialized;
use bridge::logic::PROVIDER_REGISTRY;
use bridge::logic::ProviderControllerLike;
use bridge::logic::ProviderCredentialControllerLike;
use bridge::logic::get_or_create_local_encryption_key;
use bridge::logic::mcp::handle_mcp_transport;
use bridge::logic::no_auth::NoAuthController;
use bridge::logic::oauth::Oauth2JwtBearerAssertionFlowController;
use bridge::logic::register_all_bridge_providers;
use bridge::logic::{EnvelopeEncryptionKeyContents, EnvelopeEncryptionKeyId};
use clap::Parser;
use futures::{FutureExt, TryFutureExt, future};
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use rmcp::service::serve_directly_with_ct;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use shared::libsql::{
    establish_db_connection, inject_auth_token_to_db_url, merge_nested_migrations,
};
use shared::primitives::SqlMigrationLoader;
use shared::primitives::WrappedJsonValue;
use shared::primitives::WrappedSchema;
use tokio::process::Command;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::error;
use tracing::warn;
use tracing::{debug, info};
use utoipa::ToSchema;

use crate::logic::get_task_timeline_items;
use crate::logic::ConnectionManager;
use crate::logic::GetTaskTimelineItemsRequest;
use crate::logic::GetTaskTimelineItemsResponse;
use crate::logic::WithTaskId;
use crate::repository::Repository;
use crate::router;
use crate::router::RouterParams;
use crate::utils::config::CliConfig;
use crate::utils::config::get_config_file_path;
use crate::utils::restate::deploy::DeploymentRegistrationConfig;
use crate::utils::restate::invoke::RestateIngressClient;
use crate::utils::{self, construct_src_dir_absolute, restate};
use crate::vite::{Assets, wait_for_vite_dev_server_shutdown};
use shared::command::run_child_process;
use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinition;
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
    #[arg(
        long,
        default_value = "local",
        help = "The type of key encryption key to use. Valid values are 'local' or a valid AWS KMS ARN (arn:aws:kms:region:account-id:key/key-id)."
    )]
    pub key_encryption_key: Option<String>,
}

async fn setup_repository(
    conn_string: Url,
    auth_token: Option<String>,
) -> Result<(libsql::Database, shared::libsql::Connection, Repository), CommonError> {
    info!("starting metadata database");

    let migrations = Repository::load_sql_migrations();
    let migrations = merge_nested_migrations(vec![
        migrations,
        bridge::repository::Repository::load_sql_migrations(),
    ]);
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

pub async fn cmd_start(
    subsys: &SubsystemHandle,
    params: StartParams,
    cli_config: &mut CliConfig,
) -> Result<(), CommonError> {
    let key_encryption_key = params.key_encryption_key.clone();
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
                if key_encryption_key.starts_with("arn:aws:kms:") {
                    EnvelopeEncryptionKeyContents::AwsKms {
                        arn: key_encryption_key,
                    }
                } else {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Invalid AWS KMS ARN: {}",
                        key_encryption_key
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
    let src_dir = construct_src_dir_absolute(params.clone().src_dir)?;
    let path_to_soma_definition = src_dir.join("soma.yaml");

    if !path_to_soma_definition.exists() {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Soma definition not found at {}",
            path_to_soma_definition.display()
        )));
    }
    let soma_definition = shared::soma_agent_definition::YamlSomaAgentDefinition::load_from_file(
        path_to_soma_definition,
    )?;
    let soma_definition = Arc::new(soma_definition);

    let connection_manager = ConnectionManager::new();
    let (_db, _conn, repository) =
        setup_repository(params.db_conn_string.clone(), params.db_auth_token.clone()).await?;


    // Register all bridge providers before syncing
    bridge::logic::register_all_bridge_providers().await?;
    PROVIDER_REGISTRY
        .write()
        .unwrap()
        .push(Arc::new(SomaProviderController::new(repository.clone())));


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
                result = start_restate_server(kill_restate_signal_receiver, shutdown_complete_restate_signal_trigger) => {
                    if let Err(e) = result {
                        error!("Restate server stopped unexpectedly: {:?}", e);
                    }
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
                result = start_dev_file_watcher(src_dir_clone, runtime_clone, file_change_tx_clone) => {
                    if let Err(e) = result {
                        error!("File watcher stopped unexpectedly: {:?}", e);
                    }
                    info!("File watcher stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    let file_change_tx_clone = file_change_tx.clone();
    let cli_config_clone = cli_config.clone();
    let envelope_encryption_key_contents_clone = envelope_encryption_key_contents.clone();
    subsys.start(SubsystemBuilder::new(
        "restartable-processes-on-config-change",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    subsys.wait_for_children().await;
                    info!("Restartable processes on config change stopped");
                }
                result = start_dev_restartable_processes_on_config_change(&subsys, DevRestartableProcesses {
                    runtime,
                    runtime_port,
                    file_change_tx: file_change_tx_clone,
                    src_dir,
                    params,
                    connection_manager,
                    repository,
                    db_connection: _conn,
                    cli_config: cli_config_clone.clone(),
                    soma_definition: soma_definition.clone(),
                    envelope_encryption_key_contents: envelope_encryption_key_contents_clone,
                }) => {
                    if let Err(e) = result {
                        error!("Restartable processes on config change stopped unexpectedly: {:?}", e);
                    }
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
    db_connection: shared::libsql::Connection,
    cli_config: CliConfig,
    soma_definition: Arc<dyn shared::soma_agent_definition::SomaAgentDefinitionLike>,
    envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
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
                    result = start_dev_restartable_processes(&subsys, params_clone) => {
                        if let Err(e) = result {
                            error!("Restartable processes stopped unexpectedly: {:?}", e);
                        }
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
) -> Result<(), CommonError> {
    let DevRestartableProcesses {
        runtime,
        runtime_port,
        file_change_tx,
        src_dir,
        params,
        connection_manager,
        repository,
        db_connection,
        cli_config,
        soma_definition,
        envelope_encryption_key_contents,
    } = params;

    soma_definition.reload().await?;

    let params_clone = params.clone();
    let (mcp_transport_tx, mut mcp_transport_rx) = tokio::sync::mpsc::unbounded_channel();
    let restate_ingress_client = RestateIngressClient::new(params.restate_ingress_url.to_string());
    let (on_bridge_config_change_tx, on_bridge_config_change_rx) =
        mpsc::channel::<OnConfigChangeEvt>(10);

    // Sync bridge from soma definition
    let bridge_repository = bridge::repository::Repository::new(db_connection.clone());
    let soma_def = soma_definition.get_definition().await?;
    crate::bridge_sync::sync_bridge(
        &envelope_encryption_key_contents,
        &on_bridge_config_change_tx,
        &bridge_repository,
        &soma_def,
    )
    .await?;
    info!("Bridge synced from soma definition");

    let router_params = router::RouterParams::new(
        params.clone(),
        router::InitRouterParams {
            connection_manager: connection_manager.clone(),
            repository: repository.clone(),
            mcp_transport_tx,
            soma_definition: soma_definition.clone(),
            runtime_port,
            restate_ingress_client,
            db_connection: db_connection.clone(),
            on_bridge_config_change_tx,
            envelope_encryption_key_contents: envelope_encryption_key_contents.clone(),
            mcp_sse_ping_interval: Duration::from_secs(10),
        },
    )
    .await?;

    let router_params_clone = router_params.clone();
    let mut cli_config_clone = cli_config.clone();
    let (on_server_started_tx, on_server_started_rx) = oneshot::channel::<()>();
    subsys.start(SubsystemBuilder::new(
        "axum-server",
        move |subsys: SubsystemHandle| async move {
            #[cfg(debug_assertions)]
            let _vite_scope_guard = {
                info!("Starting vite dev server");
                Assets::start_dev_server(false)
            };
            let (server_fut, handle) =
                start_axum_server(params_clone, router_params_clone, &mut cli_config_clone).await?;
            let _ = on_server_started_tx.send(());
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

    let bridge_service = router_params.bridge_service.clone();
    let mcp_ct = tokio_util::sync::CancellationToken::new();
    subsys.start(SubsystemBuilder::new(
        "mcp-transport-processor",
        move |subsys: SubsystemHandle| {
            async move {
                loop {
                    tokio::select! {
                        _ = subsys.on_shutdown_requested() => {
                            tracing::info!("mcp-server subsystem shutdown requested.");
                            mcp_ct.cancel();
                            break;
                        }
                        maybe_transport = mcp_transport_rx.recv() => {
                            handle_mcp_transport(maybe_transport, &bridge_service, &mcp_ct).await?;
                        }
                    }
                }

                // Ensure any in-flight sessions are asked to shut down.
                mcp_ct.cancel();

                Ok::<(), CommonError>(())
            }
        },
    ));

    let soma_definition_clone = soma_definition.clone();
    subsys.start(SubsystemBuilder::new(
        "bridge-config-change-listener",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    info!("Bridge config change listener stopped");
                }
                _ = watch_for_bridge_config_change(on_bridge_config_change_rx, soma_definition_clone) => {
                    info!("Bridge config change watcher stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    info!("Waiting for server to start, before starting runtime");
    on_server_started_rx.await?;

    let (kill_runtime_signal_trigger, kill_runtime_signal_receiver) = broadcast::channel::<()>(1);
    let (shutdown_runtime_complete_signal_trigger, shutdown_runtime_complete_signal_receiver) =
        oneshot::channel::<()>();
    let src_dir_clone = src_dir.clone();
    let runtime_clone = runtime.clone();
    let runtime_port_clone = runtime_port;
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
                result = start_dev_runtime(src_dir_clone, runtime_clone, runtime_port_clone, &mut file_change_rx, kill_runtime_signal_receiver, shutdown_runtime_complete_signal_trigger) => {
                    if let Err(e) = result {
                        error!("Runtime stopped unexpectedly: {:?}", e);
                    }
                    info!("Runtime stopped");
                    subsys.request_shutdown();
                }
            }

            Ok::<(), CommonError>(())

        },
    ));

    info!("Starting Restate deployment");
    let runtime_port_clone = runtime_port;
    let params_clone = params.clone();
    let soma_definition_clone = soma_definition.clone();
    subsys.start(SubsystemBuilder::new(
        "restate-deployment",
        move |subsys: SubsystemHandle| async move {
            tokio::select! {
                _ = subsys.on_shutdown_requested() => {
                    info!("system shutdown requested");
                    info!("Restate deployment shutdown complete");
                }
                _ = start_restate_deployment(params_clone, runtime_port_clone, soma_definition_clone) => {
                    info!("Restate deployment completed");
                }
            }

            Ok::<(), CommonError>(())
        },
    ));

    subsys.on_shutdown_requested().await;

    Ok(())
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

async fn watch_for_bridge_config_change(
    mut on_bridge_config_change_rx: OnConfigChangeRx,
    soma_definition: Arc<dyn shared::soma_agent_definition::SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    loop {
        let event = match on_bridge_config_change_rx.recv().await {
            Some(event) => event,
            None => {
                info!("Bridge config change receiver dropped");
                return Ok(());
            }
        };

        match event {
            OnConfigChangeEvt::ProviderInstanceAdded(provider_instance) => {
                info!(
                    "Provider instance added: {:?}",
                    provider_instance.provider_instance.id
                );

                // Only write to soma.yaml if the provider instance status is "active"
                if provider_instance.provider_instance.status == "active" {
                    let user_credential = provider_instance.user_credential.as_ref().map(|uc| {
                        shared::soma_agent_definition::CredentialConfig {
                            id: uc.id.to_string(),
                            type_id: uc.type_id.clone(),
                            metadata: json!(uc.metadata.0.clone()),
                            value: uc.value.get_inner().clone(),
                            next_rotation_time: uc.next_rotation_time.map(|t| t.to_string()),
                            data_encryption_key_id: uc.data_encryption_key_id.clone(),
                        }
                    });

                    soma_definition
                        .add_provider(
                            provider_instance.provider_instance.id.clone(),
                            shared::soma_agent_definition::ProviderConfig {
                                provider_controller_type_id: provider_instance
                                    .provider_instance
                                    .provider_controller_type_id
                                    .clone(),
                                credential_controller_type_id: provider_instance
                                    .provider_instance
                                    .credential_controller_type_id
                                    .clone(),
                                display_name: provider_instance
                                    .provider_instance
                                    .display_name
                                    .clone(),
                                resource_server_credential:
                                    shared::soma_agent_definition::CredentialConfig {
                                        id: provider_instance
                                            .resource_server_credential
                                            .id
                                            .to_string(),
                                        type_id: provider_instance
                                            .resource_server_credential
                                            .type_id
                                            .clone(),
                                        metadata: json!(
                                            provider_instance
                                                .resource_server_credential
                                                .metadata
                                                .0
                                                .clone()
                                        ),
                                        value: provider_instance
                                            .resource_server_credential
                                            .value
                                            .get_inner()
                                            .clone(),
                                        next_rotation_time: provider_instance
                                            .resource_server_credential
                                            .next_rotation_time
                                            .map(|t| t.to_string()),
                                        data_encryption_key_id: provider_instance
                                            .resource_server_credential
                                            .data_encryption_key_id
                                            .clone(),
                                    },
                                user_credential,
                                functions: None,
                            },
                        )
                        .await?;
                }
            }
            OnConfigChangeEvt::ProviderInstanceRemoved(provider_instance_id) => {
                soma_definition
                    .remove_provider(provider_instance_id)
                    .await?;
            }
            OnConfigChangeEvt::DataEncryptionKeyAdded(data_encryption_key) => {
                info!("Data encryption key added: {:?}", data_encryption_key.id);
                soma_definition
                    .add_data_encryption_key(
                        data_encryption_key.id,
                        data_encryption_key.encrypted_data_encryption_key.0,
                        match data_encryption_key.envelope_encryption_key_id {
                            bridge::logic::EnvelopeEncryptionKeyId::AwsKms { arn } => {
                                shared::soma_agent_definition::EnvelopeEncryptionKeyId::AwsKms {
                                    arn,
                                }
                            }
                            bridge::logic::EnvelopeEncryptionKeyId::Local { key_id } => {
                                shared::soma_agent_definition::EnvelopeEncryptionKeyId::Local {
                                    key_id,
                                }
                            }
                        },
                    )
                    .await?;
            }
            OnConfigChangeEvt::DataEncryptionKeyRemoved(data_encryption_key_id) => {
                soma_definition
                    .remove_data_encryption_key(data_encryption_key_id)
                    .await?;
            }
            OnConfigChangeEvt::FunctionInstanceAdded(function_instance_serialized) => {
                info!(
                    "Function instance added: {:?}",
                    function_instance_serialized.function_controller_type_id
                );
                soma_definition
                    .add_function_instance(
                        function_instance_serialized.provider_instance_id.clone(),
                        function_instance_serialized
                            .function_controller_type_id
                            .clone(),
                        function_instance_serialized.provider_instance_id.clone(),
                    )
                    .await?;
            }
            OnConfigChangeEvt::FunctionInstanceRemoved(
                function_controller_type_id,
                provider_controller_type_id,
                provider_instance_id,
            ) => {
                info!(
                    "Function instance removed: function_controller_type_id={:?}, provider_instance_id={:?}",
                    function_controller_type_id, provider_instance_id
                );
                // Remove the function instance from the provider
                soma_definition
                    .remove_function_instance(
                        provider_controller_type_id,
                        function_controller_type_id,
                        provider_instance_id,
                    )
                    .await?;
            }
        }
    }

    Ok(())
}

async fn start_restate_deployment(
    params: StartParams,
    runtime_port: u16,
    soma_definition: Arc<dyn shared::soma_agent_definition::SomaAgentDefinitionLike>,
) -> Result<(), CommonError> {
    info!("Starting Restate deployment registration");

    // The HTTP service URI should point to the local Axum server, not the Restate admin
    let service_uri = format!("http://127.0.0.1:{runtime_port}");

    info!(
        "Registering service at {} with Restate admin at {}",
        service_uri, params.restate_admin_url
    );
    let definition = soma_definition.get_definition().await?;
    restate::deploy::register_deployment(DeploymentRegistrationConfig {
        admin_url: params.restate_admin_url.to_string(),
        // TODO: this should be the service path from the soma.yaml file
        service_path: definition.project.clone(),
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

async fn start_axum_server(
    params: StartParams,
    router_params: RouterParams,
    cli_config: &mut CliConfig,
) -> Result<
    (
        impl Future<Output = Result<(), std::io::Error>>,
        axum_server::Handle,
    ),
    CommonError,
> {
    let port = find_free_port(params.port, params.port + 100)?;
    let addr: SocketAddr = format!("{}:{}", params.host, port)
        .parse()
        .map_err(|e| CommonError::AddrParseError { source: e })?;

    info!("Starting server on {}", addr);

    let router = router::initiate_routers(router_params)?;
    info!("Router initiated");
    let handle = axum_server::Handle::new();
    let handle_clone = handle.clone();
    let server_fut = axum_server::bind(addr)
        .handle(handle)
        .serve(router.into_make_service());
    info!("Server bound");
    cli_config.update_dev_server_url(addr.to_string()).await?;
    info!("Server URL updated");
    info!("Server started");

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
        0 => Ok(None),
        1 => Ok(Some(matched_runtimes[0].clone())),
        _ => Err(CommonError::Unknown(anyhow::anyhow!(
            "Multiple runtimes matched"
        ))),
    }
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

pub struct SomaProviderController {
    repository: Repository,
}

impl SomaProviderController {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ProviderControllerLike for SomaProviderController {
    fn type_id(&self) -> &'static str {
        "soma"
    }

    fn documentation(&self) -> &'static str {
        ""
    }

    fn name(&self) -> &'static str {
        "Soma"
    }

    fn categories(&self) -> Vec<&'static str> {
        vec![]
    }

    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>> {
        vec![Arc::new(GetTaskTimelineItemsFunctionController {
            repository: self.repository.clone(),
        })]
    }

    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>> {
        vec![Arc::new(NoAuthController {
            static_credentials: NoAuthStaticCredentialConfiguration {
                metadata: Metadata::new(),
            },
        })]
    }
}

struct GetTaskTimelineItemsFunctionController {
    repository: Repository,
}

impl GetTaskTimelineItemsFunctionController {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

// #[derive(Serialize, Deserialize, ToSchema, Clone, JsonSchema)]
// struct GetTaskTimelineItemsFunctionParameters {
//     task_id: WrappedUuidV4,
//     pagination: PaginationRequest,
// }

// #[derive(Serialize, Deserialize, ToSchema, Clone, JsonSchema)]
// struct GetTaskTimelineItemsFunctionOutput {
//     message_id: String,
// }

#[async_trait]
impl FunctionControllerLike for GetTaskTimelineItemsFunctionController {
    fn type_id(&self) -> &'static str {
        "soma_get_task_timeline_items"
    }
    fn name(&self) -> &'static str {
        "Get task timeline items"
    }
    fn documentation(&self) -> &'static str {
        ""
    }
    fn parameters(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(GetTaskTimelineItemsRequest).into())
    }
    fn output(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(GetTaskTimelineItemsResponse).into())
    }
    fn categories(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn invoke(
        &self,
        crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        _static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        _resource_server_credential: &ResourceServerCredentialSerialized,
        user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<WrappedJsonValue, CommonError> {
        // Parse the function parameters
        let params: GetTaskTimelineItemsRequest = serde_json::from_value(params.into())
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid parameters: {}", e)))?;

        // Downcast to OAuth controller and decrypt credentials
        let cred_controller_type_id = credential_controller.type_id();

        if cred_controller_type_id == NoAuthController::static_type_id() {
            let _controller = credential_controller
                .as_any()
                .downcast_ref::<NoAuthController>()
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to downcast to NoAuthController"
                    ))
                })?;
            
        }  else {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Unsupported credential controller type: {}",
                cred_controller_type_id
            )));
        };

        let res = get_task_timeline_items(
            &self.repository,
            params,
        )
        .await;
        
        Ok(WrappedJsonValue::new(serde_json::json!(res)))
    }
}
