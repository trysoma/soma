use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bridge::logic::{OnConfigChangeTx, register_all_bridge_providers};
use encryption::logic::crypto_services::CryptoCache;
use shared::error::CommonError;
use shared::soma_agent_definition::SomaAgentDefinitionLike;
use shared::subsystem::SubsystemHandle;
use shared::uds::{
    DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client, establish_connection_with_retry,
};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::logic::on_change_pubsub::{SomaChangeTx, create_soma_change_channel, run_change_pubsub};
use crate::logic::task::ConnectionManager;
use crate::repository::setup_repository;
use crate::restate::RestateServerParams;
use crate::sdk::{SdkRuntime, determine_sdk_runtime, sdk_provider_sync};
use crate::subsystems::Subsystems;
use crate::{ApiService, InitApiServiceParams};

pub struct CreateApiServiceParams {
    pub base_url: String,
    pub project_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub soma_restate_service_port: u16,
    pub db_conn_string: String,
    pub db_auth_token: Option<String>,
    pub soma_definition: Arc<dyn SomaAgentDefinitionLike>,
    pub restate_params: RestateServerParams,
    pub system_shutdown_signal: broadcast::Sender<()>,
}

pub struct ApiServiceBundle {
    pub api_service: ApiService,
    pub subsystems: Subsystems,
    /// Unified change channel for external listeners to subscribe to bridge and encryption events
    pub soma_change_tx: SomaChangeTx,
}

/// Creates the API service and starts all subsystems
/// Returns the API service and subsystem handles for the caller to manage
pub async fn create_api_service(
    params: CreateApiServiceParams,
) -> Result<ApiServiceBundle, CommonError> {
    let CreateApiServiceParams {
        base_url,
        project_dir,
        host,
        port,
        soma_restate_service_port,
        db_conn_string,
        db_auth_token,
        soma_definition,
        restate_params,
        system_shutdown_signal,
    } = params;

    // Determine SDK runtime
    let sdk_runtime = match determine_sdk_runtime(&project_dir)? {
        Some(runtime) => runtime,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "No SDK runtime matched"
            )));
        }
    };

    // Setup database and repositories
    info!("Setting up database and repositories...");
    let connection_manager = ConnectionManager::new();
    let db_url = url::Url::parse(&db_conn_string)?;
    let (_db, conn, repository, bridge_repo, encryption_repo) =
        setup_repository(&db_url, &db_auth_token).await?;

    // Create identity repository (uses same connection)
    let identity_repo = identity::repository::Repository::new(conn.clone());

    // Create the bridge config change channel
    let (on_bridge_config_change_tx, on_bridge_config_change_rx): (OnConfigChangeTx, _) =
        tokio::sync::broadcast::channel(100);

    // Create encryption event channel
    let (encryption_change_tx, encryption_change_rx): (
        encryption::logic::EncryptionKeyEventSender,
        _,
    ) = tokio::sync::broadcast::channel(100);

    // Create secret event channel
    let (secret_change_tx, secret_change_rx) =
        crate::logic::on_change_pubsub::create_secret_change_channel(100);

    // Create environment variable event channel
    let (environment_variable_change_tx, environment_variable_change_rx) =
        crate::logic::on_change_pubsub::create_environment_variable_change_channel(100);

    // Create the unified soma change channel
    let (soma_change_tx, _soma_change_rx) = create_soma_change_channel(100);

    // Initialize the crypto cache from the encryption repository
    info!("Initializing crypto cache...");
    let local_envelope_encryption_key_path = project_dir.join(".soma/envelope-encryption-keys");
    let crypto_cache = CryptoCache::new(
        encryption_repo.clone(),
        local_envelope_encryption_key_path.clone(),
    );
    encryption::logic::crypto_services::init_crypto_cache(&crypto_cache).await?;

    // Create JWKS cache (JWKs will be created when default DEK alias is available)
    let internal_jwks_cache = identity::logic::jwk::cache::JwksCache::new(identity_repo.clone());

    // Create JWK rotation state to track initialization
    let jwk_rotation_state = crate::logic::identity::JwkRotationState::new();

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
        soma_restate_service_port,
        system_shutdown_signal.subscribe(),
        repository.clone(),
        crypto_cache.clone(),
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

            // Wait for SDK server healthcheck to pass before triggering bridge client generation
            wait_for_sdk_healthcheck(&mut client).await?;

            // Perform initial secret sync to SDK (after SDK is fully ready)
            info!("Performing initial secret sync to SDK...");
            let repository_arc_for_initial_sync = std::sync::Arc::new(repository.clone());
            match crate::logic::secret_sync::fetch_and_decrypt_all_secrets(
                &repository_arc_for_initial_sync,
                &crypto_cache,
            )
            .await
            {
                Ok(secrets) => {
                    if !secrets.is_empty() {
                        info!("Found {} secrets to sync to SDK", secrets.len());
                        match crate::logic::secret_sync::sync_secrets_to_sdk(&mut client, secrets)
                            .await
                        {
                            Ok(()) => {
                                info!("Initial secret sync completed successfully");
                            }
                            Err(e) => {
                                warn!("Failed to perform initial secret sync: {:?}", e);
                                // Don't fail startup - secrets will be synced on next change
                            }
                        }
                    } else {
                        info!("No secrets to sync on startup");
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch secrets for initial sync: {:?}", e);
                    // Don't fail startup - secrets will be synced on next change
                }
            }

            // Perform initial environment variable sync to SDK (after SDK is fully ready)
            info!("Performing initial environment variable sync to SDK...");
            match crate::logic::environment_variable_sync::fetch_all_environment_variables(
                &repository_arc_for_initial_sync,
            )
            .await
            {
                Ok(env_vars) => {
                    if !env_vars.is_empty() {
                        info!(
                            "Found {} environment variables to sync to SDK",
                            env_vars.len()
                        );
                        match crate::logic::environment_variable_sync::sync_environment_variables_to_sdk(
                            &mut client,
                            env_vars,
                        )
                        .await
                        {
                            Ok(()) => {
                                info!("Initial environment variable sync completed successfully");
                            }
                            Err(e) => {
                                warn!("Failed to perform initial environment variable sync: {:?}", e);
                                // Don't fail startup - env vars will be synced on next change
                            }
                        }
                    } else {
                        info!("No environment variables to sync on startup");
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch environment variables for initial sync: {:?}",
                        e
                    );
                    // Don't fail startup - env vars will be synced on next change
                }
            }

            // Trigger initial bridge client generation on start
            info!("Triggering initial bridge client generation...");
            match crate::logic::bridge::codegen::trigger_bridge_client_generation(
                &mut client,
                &bridge_repo,
            )
            .await
            {
                Ok(()) => {
                    info!("Initial bridge client generation completed successfully");
                }
                Err(e) => {
                    warn!(
                        "Failed to trigger initial bridge client generation: {:?}",
                        e
                    );
                    // Don't fail startup if codegen fails - it will be retried on bridge changes
                }
            }

            // Store client for reuse
            Arc::new(tokio::sync::Mutex::new(Some(client)))
        }
        Ok(Err(_)) | Err(_) => {
            // SDK server not ready - create empty client
            Arc::new(tokio::sync::Mutex::new(None))
        }
    };

    // Subscribe to bridge config change events for bridge client generation listener
    // Do this AFTER SDK server is ready to avoid processing events before server is ready
    // Broadcast channels support multiple subscribers natively - no wrapper needed!
    let bridge_client_gen_rx = on_bridge_config_change_tx.subscribe();

    // Create MCP transport channel
    let (mcp_transport_tx, mcp_transport_rx) = tokio::sync::mpsc::unbounded_channel();

    // Register built-in bridge providers (google_mail, stripe, etc.) BEFORE creating API service
    info!("Registering built-in bridge providers...");
    register_all_bridge_providers().await?;
    info!("Built-in providers registered");

    // Initialize API service
    info!("Initializing API service...");
    let local_envelope_encryption_key_path = project_dir.join(".soma/envelope-encryption-keys");
    let api_service = ApiService::new(InitApiServiceParams {
        base_url: base_url.clone(),
        host: host.clone(),
        port,
        internal_jwks_cache: internal_jwks_cache.clone(),
        soma_restate_service_port,
        connection_manager: connection_manager.clone(),
        repository: repository.clone(),
        mcp_transport_tx: mcp_transport_tx.clone(),
        soma_definition: soma_definition.clone(),
        restate_ingress_client: restate_params.get_ingress_client()?,
        restate_admin_client: restate_admin_client.clone(),
        restate_params: restate_params.clone(),
        on_bridge_config_change_tx: on_bridge_config_change_tx.clone(),
        crypto_cache: crypto_cache.clone(),
        bridge_repository: bridge_repo.clone(),
        identity_repository: identity_repo.clone(),
        mcp_sse_ping_interval: Duration::from_secs(10),
        sdk_client: sdk_client.clone(),
        on_encryption_change_tx: encryption_change_tx.clone(),
        on_secret_change_tx: secret_change_tx.clone(),
        on_environment_variable_change_tx: environment_variable_change_tx.clone(),
        encryption_repository: encryption_repo.clone(),
        local_envelope_encryption_key_path,
    })
    .await?;
    info!("API service initialized");

    // Start the unified change pubsub forwarder (after api_service is created so we can subscribe to identity events)
    info!("Starting unified change pubsub...");
    let soma_change_tx_clone = soma_change_tx.clone();
    let pubsub_shutdown_rx = system_shutdown_signal.subscribe();
    let identity_change_rx = api_service.identity_service.on_config_change_tx.subscribe();
    tokio::spawn(async move {
        run_change_pubsub(
            soma_change_tx_clone,
            on_bridge_config_change_rx,
            encryption_change_rx,
            secret_change_rx,
            environment_variable_change_rx,
            identity_change_rx,
            pubsub_shutdown_rx,
        )
        .await;
    });

    // Start MCP connection manager
    info!("Starting MCP connection manager...");
    let mcp_handle = start_mcp_subsystem(
        api_service.bridge_service.clone(),
        mcp_transport_rx,
        system_shutdown_signal.subscribe(),
    )?;

    // Note: SDK sync is now SDK-initiated. When the SDK server starts (or restarts due to HMR),
    // it calls the /_internal/v1/resync_sdk endpoint to trigger sync of providers, agents,
    // secrets, and environment variables. This replaces the old connection-monitoring approach.

    // Start credential rotation
    info!("Starting credential rotation...");
    let credential_rotation_handle = start_credential_rotation_subsystem(
        bridge_repo.clone(),
        crypto_cache.clone(),
        on_bridge_config_change_tx.clone(),
        system_shutdown_signal.subscribe(),
    )?;

    // Start bridge client generation listener
    info!("Starting bridge client generation listener...");
    let bridge_client_gen_handle = crate::logic::bridge::start_bridge_client_generation_subsystem(
        bridge_repo.clone(),
        sdk_client.clone(),
        bridge_client_gen_rx,
        system_shutdown_signal.subscribe(),
    )?;

    // Start secret sync subsystem
    info!("Starting secret sync subsystem...");
    let secret_sync_rx = secret_change_tx.subscribe();
    let socket_path_clone = socket_path.clone();
    let secret_sync_handle = crate::logic::secret_sync::start_secret_sync_subsystem(
        repository.clone(),
        crypto_cache.clone(),
        socket_path_clone.clone(),
        secret_sync_rx,
        system_shutdown_signal.subscribe(),
    )?;

    // Start environment variable sync subsystem
    info!("Starting environment variable sync subsystem...");
    let env_var_sync_rx = environment_variable_change_tx.subscribe();
    let socket_path_for_env_sync = socket_path.clone();
    let env_var_sync_handle =
        crate::logic::environment_variable_sync::start_environment_variable_sync_subsystem(
            repository.clone(),
            socket_path_for_env_sync.clone(),
            env_var_sync_rx,
            system_shutdown_signal.subscribe(),
        )?;

    // Start JWK init listener (will start JWK rotation when default DEK is available)
    info!("Starting JWK init listener...");
    let encryption_change_rx_for_jwk = encryption_change_tx.subscribe();
    let jwk_init_handle = crate::logic::identity::start_jwk_init_on_dek_listener(
        identity_repo.clone(),
        crypto_cache.clone(),
        internal_jwks_cache.clone(),
        jwk_rotation_state.clone(),
        encryption_change_rx_for_jwk,
        system_shutdown_signal.clone(),
    )?;

    // Note: Initial sync of secrets and environment variables now happens AFTER SDK server
    // healthcheck passes (see above, around line 171). This ensures the SDK server's gRPC
    // handlers are fully registered before we try to sync.

    Ok(ApiServiceBundle {
        api_service,
        subsystems: Subsystems {
            sdk_server: Some(sdk_server_handle),
            mcp: Some(mcp_handle),
            credential_rotation: Some(credential_rotation_handle),
            bridge_client_generation: Some(bridge_client_gen_handle),
            secret_sync: Some(secret_sync_handle),
            environment_variable_sync: Some(env_var_sync_handle),
            jwk_init_listener: Some(jwk_init_handle),
        },
        soma_change_tx,
    })
}

fn start_sdk_server_subsystem(
    project_dir: PathBuf,
    sdk_runtime: SdkRuntime,
    restate_service_port: u16,
    shutdown_rx: broadcast::Receiver<()>,
    repository: crate::repository::Repository,
    crypto_cache: CryptoCache,
) -> Result<SubsystemHandle, CommonError> {
    use crate::sdk::{StartDevSdkParams, start_dev_sdk};

    let (handle, signal) = SubsystemHandle::new("SDK Server");

    tokio::spawn(async move {
        match start_dev_sdk(StartDevSdkParams {
            project_dir,
            sdk_runtime,
            restate_service_port,
            kill_signal_rx: shutdown_rx,
            repository: std::sync::Arc::new(repository),
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

fn start_mcp_subsystem(
    bridge_service: bridge::router::BridgeService,
    mcp_transport_rx: tokio::sync::mpsc::UnboundedReceiver<
        rmcp::transport::sse_server::SseServerTransport,
    >,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    use crate::logic::bridge::connection_manager::{
        StartMcpConnectionManagerParams, start_mcp_connection_manager,
    };

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

/// Waits for SDK server healthcheck to pass, retrying up to max_iterations times
async fn wait_for_sdk_healthcheck(
    client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
) -> Result<(), CommonError> {
    const MAX_ITERATIONS: u32 = 10;
    const RETRY_DELAY_MS: u64 = 200;

    info!("Waiting for SDK server healthcheck to pass...");

    for attempt in 1..=MAX_ITERATIONS {
        let health_request = tonic::Request::new(());
        match client.health_check(health_request).await {
            Ok(_) => {
                info!("SDK server healthcheck passed");
                return Ok(());
            }
            Err(e) => {
                if attempt < MAX_ITERATIONS {
                    warn!(
                        "SDK server healthcheck not ready yet (attempt {}/{}): {:?}. Retrying...",
                        attempt, MAX_ITERATIONS, e
                    );
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                } else {
                    error!(
                        "SDK server healthcheck failed after {} attempts: {:?}",
                        MAX_ITERATIONS, e
                    );
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "SDK server healthcheck failed after {MAX_ITERATIONS} attempts: {e}"
                    )));
                }
            }
        }
    }

    // Should never reach here, but handle it just in case
    Err(CommonError::Unknown(anyhow::anyhow!(
        "SDK server healthcheck failed after {MAX_ITERATIONS} attempts"
    )))
}

fn start_credential_rotation_subsystem(
    bridge_repo: bridge::repository::Repository,
    crypto_cache: CryptoCache,
    on_bridge_change_tx: OnConfigChangeTx,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("Credential Rotation");

    tokio::spawn(async move {
        bridge::logic::credential_rotation_task(
            bridge_repo,
            crypto_cache,
            on_bridge_change_tx,
            shutdown_rx,
        )
        .await;
        signal.signal_with_message("stopped gracefully");
    });

    Ok(handle)
}
