use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bridge::logic::{EnvelopeEncryptionKeyContents, OnConfigChangeTx, register_all_bridge_providers};
use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;
use shared::subsystem::SubsystemHandle;
use shared::uds::{create_soma_unix_socket_client, establish_connection_with_retry, DEFAULT_SOMA_SERVER_SOCK};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::logic::ConnectionManager;
use crate::repository::setup_repository;
use crate::restate::RestateServerParams;
use crate::{ApiService, InitRouterParams};
use crate::sdk::{SdkRuntime, determine_sdk_runtime, sdk_provider_sync};
use crate::subsystems::Subsystems;

pub struct CreateApiServiceParams {
    pub project_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub sdk_port: u16,
    pub db_conn_string: String,
    pub db_auth_token: Option<String>,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub restate_params: RestateServerParams,
    pub envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    pub system_shutdown_signal: broadcast::Sender<()>,
    pub on_bridge_config_change_tx: OnConfigChangeTx,
}

pub struct ApiServiceBundle {
    pub api_service: ApiService,
    pub subsystems: Subsystems,
}

/// Creates the API service and starts all subsystems
/// Returns the API service and subsystem handles for the caller to manage
pub async fn create_api_service(
    params: CreateApiServiceParams,
) -> Result<ApiServiceBundle, CommonError> {
    let CreateApiServiceParams {
        project_dir,
        host,
        port,
        sdk_port,
        db_conn_string,
        db_auth_token,
        soma_definition,
        restate_params,
        envelope_encryption_key_contents,
        system_shutdown_signal,
        on_bridge_config_change_tx,
    } = params;

    // Determine SDK runtime
    let sdk_runtime = match determine_sdk_runtime(&project_dir)? {
        Some(runtime) => runtime,
        None => return Err(CommonError::Unknown(anyhow::anyhow!("No SDK runtime matched"))),
    };

    // Setup database and repositories
    info!("Setting up database and repositories...");
    let connection_manager = ConnectionManager::new();
    let db_url = url::Url::parse(&db_conn_string)?;
    let (_db, _conn, repository, bridge_repo) =
        setup_repository(&db_url, &db_auth_token).await?;

    // Restate server is started by caller (soma crate)
    // We just use the passed-in handle

    // Wait for Restate to be ready
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

    // Start SDK server subsystem
    info!("Starting SDK server...");
    let sdk_server_handle = start_sdk_server_subsystem(
        project_dir.clone(),
        sdk_runtime,
        sdk_port,
        system_shutdown_signal.subscribe(),
    )?;

    // Wait for SDK server and sync providers
    let socket_path = DEFAULT_SOMA_SERVER_SOCK.to_string();
    info!("Waiting for SDK server to be ready...");
    let sdk_client = match tokio::time::timeout(
        Duration::from_secs(30),
        establish_connection_with_retry(&socket_path),
    )
    .await
    {
        Ok(Ok(_)) => {
            info!("SDK server is ready, syncing providers...");
            let mut client = create_soma_unix_socket_client(&socket_path).await?;
            let request = tonic::Request::new(());
            let response = client.metadata(request).await.map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to get SDK metadata: {e}"))
            })?;
            let metadata = response.into_inner();
            sdk_provider_sync::sync_providers_from_metadata(&metadata)?;
            info!("SDK providers synced successfully");
            // Store client for reuse
            Arc::new(tokio::sync::Mutex::new(Some(client)))
        }
        Ok(Err(e)) => {
            warn!(
                "Failed to connect to SDK server: {:?}. Continuing without SDK providers.",
                e
            );
            Arc::new(tokio::sync::Mutex::new(None))
        }
        Err(_) => {
            warn!("Timeout waiting for SDK server. Continuing without SDK providers.");
            Arc::new(tokio::sync::Mutex::new(None))
        }
    };

    // Create MCP transport channel
    let (mcp_transport_tx, mcp_transport_rx) = tokio::sync::mpsc::unbounded_channel();

    // Register built-in bridge providers (google_mail, stripe, etc.) BEFORE creating API service
    info!("Registering built-in bridge providers...");
    register_all_bridge_providers().await?;
    info!("Built-in providers registered");

    // Initialize API service
    info!("Initializing API service...");
    let api_service = ApiService::new(InitRouterParams {
        host: host.clone(),
        port,
        connection_manager: connection_manager.clone(),
        repository: repository.clone(),
        mcp_transport_tx: mcp_transport_tx.clone(),
        soma_definition: soma_definition.clone(),
        restate_ingress_client: restate_params.get_ingress_client()?,
        restate_admin_client: restate_admin_client.clone(),
        on_bridge_config_change_tx: on_bridge_config_change_tx.clone(),
        envelope_encryption_key_contents: envelope_encryption_key_contents.clone(),
        bridge_repository: bridge_repo.clone(),
        mcp_sse_ping_interval: Duration::from_secs(10),
        sdk_client: sdk_client.clone(),
    })
    .await?;
    info!("API service initialized");

    // Start MCP connection manager
    info!("Starting MCP connection manager...");
    let mcp_handle = start_mcp_subsystem(
        api_service.bridge_service.clone(),
        mcp_transport_rx,
        system_shutdown_signal.subscribe(),
    )?;

    // Start SDK sync watcher
    info!("Starting SDK sync watcher...");
    let sdk_sync_handle = start_sdk_sync_subsystem(
        socket_path,
        restate_params,
        sdk_port,
        system_shutdown_signal.subscribe(),
    )?;

    // Start credential rotation
    info!("Starting credential rotation...");
    let credential_rotation_handle = start_credential_rotation_subsystem(
        bridge_repo,
        envelope_encryption_key_contents,
        on_bridge_config_change_tx,
    )?;

    Ok(ApiServiceBundle {
        api_service,
        subsystems: Subsystems {
            sdk_server: Some(sdk_server_handle),
            sdk_sync: Some(sdk_sync_handle),
            mcp: Some(mcp_handle),
            credential_rotation: Some(credential_rotation_handle),
        },
    })
}

fn start_sdk_server_subsystem(
    project_dir: PathBuf,
    sdk_runtime: SdkRuntime,
    sdk_port: u16,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    use crate::sdk::{start_dev_sdk, StartDevSdkParams};

    let (handle, signal) = SubsystemHandle::new("SDK Server");

    tokio::spawn(async move {
        match start_dev_sdk(StartDevSdkParams {
            project_dir,
            sdk_runtime,
            sdk_port,
            kill_signal_rx: shutdown_rx,
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

fn start_mcp_subsystem(
    bridge_service: bridge::router::bridge::BridgeService,
    mcp_transport_rx: tokio::sync::mpsc::UnboundedReceiver<rmcp::transport::sse_server::SseServerTransport>,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    use crate::bridge::connection_manager::{start_mcp_connection_manager, StartMcpConnectionManagerParams};

    let (handle, signal) = SubsystemHandle::new("MCP");

    tokio::spawn(async move {
        match start_mcp_connection_manager(StartMcpConnectionManagerParams {
            bridge_service,
            mcp_transport_rx,
            system_shutdown_signal_rx: shutdown_rx,
        })
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("MCP connection manager stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}

fn start_sdk_sync_subsystem(
    socket_path: String,
    restate_params: RestateServerParams,
    sdk_port: u16,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    use crate::sdk::{sync_sdk_changes, SyncSdkChangesParams};

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

fn start_credential_rotation_subsystem(
    bridge_repo: bridge::repository::Repository,
    envelope_encryption_key_contents: EnvelopeEncryptionKeyContents,
    on_bridge_change_tx: OnConfigChangeTx,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("Credential Rotation");

    tokio::spawn(async move {
        bridge::logic::credential_rotation_task(
            bridge_repo,
            envelope_encryption_key_contents,
            on_bridge_change_tx,
        )
        .await;
        signal.signal_with_message("stopped gracefully");
    });

    Ok(handle)
}
