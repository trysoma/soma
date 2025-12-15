use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bridge::logic::mcp::BridgeMcpService;
use bridge::logic::{OnConfigChangeTx, register_all_bridge_providers};
use encryption::logic::crypto_services::CryptoCache;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use shared::error::CommonError;
use shared::process_manager::{CustomProcessManager, OnStop, OnTerminalStop, RestartConfig, ThreadConfig};
use shared::soma_agent_definition::SomaAgentDefinitionLike;
use shared::subsystem::SubsystemHandle;
use shared::uds::{
    DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client, establish_connection_with_retry,
};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace};

use crate::logic::on_change_pubsub::{SomaChangeTx, create_soma_change_channel, run_change_pubsub};
use crate::logic::task::ConnectionManager;
use crate::repository::setup_repository;
use crate::restate::RestateServerParams;
use crate::sdk::{SdkRuntime, determine_sdk_runtime, sdk_agent_sync, sdk_provider_sync};
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
    pub process_manager: Arc<tokio::sync::Mutex<CustomProcessManager>>,
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
        process_manager,
    } = params;

    // Determine SDK runtime
    trace!("Determining SDK runtime");
    let sdk_runtime = match determine_sdk_runtime(&project_dir)? {
        Some(runtime) => runtime,
        None => {
            error!("No SDK runtime matched");
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "No SDK runtime matched"
            )));
        }
    };
    trace!("SDK runtime determined: {:?}", sdk_runtime);
    // Setup database and repositories
    trace!("Setting up database and repositories...");
    let connection_manager = ConnectionManager::new();
    let db_url = url::Url::parse(&db_conn_string)?;
    let (_db, conn, repository, bridge_repo, encryption_repo) =
        setup_repository(&db_url, &db_auth_token).await?;
    trace!("Database and repositories setup");
    // Create identity repository (uses same connection)
    let identity_repo = identity::repository::Repository::new(conn.clone());

    // Create the bridge config change channel
    let (on_bridge_config_change_tx, _on_bridge_config_change_rx): (OnConfigChangeTx, _) =
        tokio::sync::broadcast::channel(100);

    // Create encryption event channel
    let (encryption_change_tx, _encryption_change_rx): (
        encryption::logic::EncryptionKeyEventSender,
        _,
    ) = tokio::sync::broadcast::channel(100);

    // Create secret event channel
    let (secret_change_tx, _secret_change_rx) =
        crate::logic::on_change_pubsub::create_secret_change_channel(100);

    // Create environment variable event channel
    let (environment_variable_change_tx, _environment_variable_change_rx) =
        crate::logic::on_change_pubsub::create_environment_variable_change_channel(100);

    // Create the unified soma change channel
    let (soma_change_tx, _soma_change_rx) = create_soma_change_channel(100);

    // Initialize the crypto cache from the encryption repository
    trace!("Initializing crypto cache");
    let local_envelope_encryption_key_path = project_dir.join(".soma/envelope-encryption-keys");
    let crypto_cache = CryptoCache::new(
        encryption_repo.clone(),
        local_envelope_encryption_key_path.clone(),
    );
    encryption::logic::crypto_services::init_crypto_cache(&crypto_cache).await
        .inspect_err(|_e| error!("Failed to initialize crypto cache"))?;
    trace!("Crypto cache initialized");
    // Create the agent cache early (shared between services, needed for codegen)
    let agent_cache = sdk_agent_sync::create_agent_cache();

    // Create JWKS cache (JWKs will be created when default DEK alias is available)
    let internal_jwks_cache = identity::logic::jwk::cache::JwksCache::new(identity_repo.clone());

    // Create JWK rotation state to track initialization
    let jwk_rotation_state = crate::logic::identity::JwkRotationState::new();

    // Restate server is started by caller (soma crate)
    // We just use the passed-in handle

    // Wait for Restate to be ready
    debug!("Waiting for Restate server");
    let restate_admin_client = loop {
        match restate_params.get_admin_client().await {
            Ok(client) => {
                debug!("Restate server ready");
                break client;
            }
            Err(e) => {
                trace!(error = ?e, "Restate server not ready, retrying");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    };

    // Start SDK server subsystem
    debug!("Starting SDK server");
    let sdk_server_handle = start_sdk_server_subsystem(
        project_dir.clone(),
        sdk_runtime,
        soma_restate_service_port,
        repository.clone(),
        crypto_cache.clone(),
        process_manager.clone(),
    ).await?;

    // Wait for SDK server and sync providers
    let socket_path = DEFAULT_SOMA_SERVER_SOCK.to_string();
    debug!("Waiting for SDK server");
    let sdk_client = match tokio::time::timeout(
        Duration::from_secs(30),
        establish_connection_with_retry(&socket_path),
    )
    .await
    {
        Ok(Ok(_)) => {
            trace!("SDK server ready, syncing providers");
            let mut client = create_soma_unix_socket_client(&socket_path).await?;
            let request = tonic::Request::new(());
            let response = client.metadata(request).await.map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to get SDK metadata: {e}"))
            })?;
            let metadata = response.into_inner();
            sdk_provider_sync::sync_providers_from_metadata(&metadata)?;
            trace!("SDK providers synced");

            // Wait for SDK server healthcheck to pass before triggering bridge client generation
            wait_for_sdk_healthcheck(&mut client).await?;

            // Perform initial secret sync to SDK (after SDK is fully ready)
            trace!("Performing initial secret sync to SDK");
            let repository_arc_for_initial_sync = std::sync::Arc::new(repository.clone());
            match crate::logic::secret_sync::fetch_and_decrypt_all_secrets(
                &repository_arc_for_initial_sync,
                &crypto_cache,
            )
            .await
            {
                Ok(secrets) => {
                    if !secrets.is_empty() {
                        trace!(count = secrets.len(), "Syncing secrets to SDK");
                        match crate::logic::secret_sync::sync_secrets_to_sdk(&mut client, secrets)
                            .await
                        {
                            Ok(()) => {
                                trace!("Initial secret sync complete");
                            }
                            Err(e) => {
                                debug!(error = ?e, "Failed initial secret sync");
                                // Don't fail startup - secrets will be synced on next change
                            }
                        }
                    } else {
                        trace!("No secrets to sync on startup");
                    }
                }
                Err(e) => {
                    debug!(error = ?e, "Failed to fetch secrets for initial sync");
                    // Don't fail startup - secrets will be synced on next change
                }
            }

            // Perform initial environment variable sync to SDK (after SDK is fully ready)
            trace!("Performing initial environment variable sync to SDK");
            match crate::logic::environment_variable_sync::fetch_all_environment_variables(
                &repository_arc_for_initial_sync,
            )
            .await
            {
                Ok(env_vars) => {
                    if !env_vars.is_empty() {
                        trace!(count = env_vars.len(), "Syncing env vars to SDK");
                        match crate::logic::environment_variable_sync::sync_environment_variables_to_sdk(
                            &mut client,
                            env_vars,
                        )
                        .await
                        {
                            Ok(()) => {
                                trace!("Initial env var sync complete");
                            }
                            Err(e) => {
                                debug!(error = ?e, "Failed initial env var sync");
                                // Don't fail startup - env vars will be synced on next change
                            }
                        }
                    } else {
                        trace!("No environment variables to sync on startup");
                    }
                }
                Err(e) => {
                    debug!(error = ?e, "Failed to fetch env vars for initial sync");
                    // Don't fail startup - env vars will be synced on next change
                }
            }

            // Trigger initial bridge client generation on start
            trace!("Triggering initial bridge client generation");
            match crate::logic::bridge::codegen::trigger_bridge_client_generation(
                &mut client,
                &bridge_repo,
                &agent_cache,
            )
            .await
            {
                Ok(()) => {
                    trace!("Initial bridge client generation complete");
                }
                Err(e) => {
                    debug!(error = ?e, "Failed initial bridge client generation");
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

    // Create MCP cancellation token for graceful shutdown
    let mcp_ct = CancellationToken::new();

    // Subscribe to system shutdown to cancel MCP service
    let process_manager_clone_for_lock = process_manager.clone();
    let process_manager_clone_for_spawn = process_manager.clone();
    let mcp_ct_for_thread = mcp_ct.clone();
    process_manager_clone_for_lock.lock().await.start_thread("mcp_shutdown_listener", ThreadConfig {
        spawn_fn: move || {
            let process_manager_for_mcp = process_manager_clone_for_spawn.clone();
            let ct = mcp_ct_for_thread.clone();
            tokio::spawn(async move {
                process_manager_for_mcp.lock().await.wait_for_shutdown().await;
                ct.cancel();
                Ok(())
            })
        },
        health_check: None,
        on_terminal_stop: OnTerminalStop::Ignore,
        on_stop: OnStop::Nothing,
        shutdown_priority: 1,
        follow_logs: false,
        on_shutdown_triggered: None,
        on_shutdown_complete: None,
    }).await
    .inspect_err(|e| error!(error = %e, "Failed to start MCP shutdown listener thread"))?;

    // Create the StreamableHttpService for MCP
    // Note: BridgeMcpService is created fresh for each request by the service factory
    // Clone values for use in the service factory closure
    let bridge_repo_for_mcp = bridge_repo.clone();
    let crypto_cache_for_mcp = crypto_cache.clone();
    let mcp_service = StreamableHttpService::new(
        move || {
            Ok(BridgeMcpService {
                repository: bridge_repo_for_mcp.clone(),
                encryption_service: crypto_cache_for_mcp.clone(),
            })
        },
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig {
            cancellation_token: mcp_ct.child_token(),
            // Disable stateful mode to prevent 500 errors on GET resume attempts.
            // When stateful_mode is false, the server returns 405 for GET requests,
            // telling clients to use POST-only mode. The MCP protocol still works
            // correctly - responses are returned via SSE in the POST response body.
            // stateful_mode: false,
            ..Default::default()
        },
    );

    // Register built-in bridge providers (google_mail, stripe, etc.) BEFORE creating API service
    trace!("Registering built-in bridge providers");
    register_all_bridge_providers().await?;
    trace!("Built-in providers registered");

    // Initialize API service
    debug!("Initializing API service");
    let local_envelope_encryption_key_path = project_dir.join(".soma/envelope-encryption-keys");
    let api_service = ApiService::new(InitApiServiceParams {
        base_url: base_url.clone(),
        host: host.clone(),
        port,
        internal_jwks_cache: internal_jwks_cache.clone(),
        soma_restate_service_port,
        connection_manager: connection_manager.clone(),
        repository: repository.clone(),
        mcp_service,
        soma_definition: soma_definition.clone(),
        restate_ingress_client: restate_params.get_ingress_client()?,
        restate_admin_client: restate_admin_client.clone(),
        restate_params: restate_params.clone(),
        on_bridge_config_change_tx: on_bridge_config_change_tx.clone(),
        crypto_cache: crypto_cache.clone(),
        bridge_repository: bridge_repo.clone(),
        identity_repository: identity_repo.clone(),
        sdk_client: sdk_client.clone(),
        on_encryption_change_tx: encryption_change_tx.clone(),
        on_secret_change_tx: secret_change_tx.clone(),
        on_environment_variable_change_tx: environment_variable_change_tx.clone(),
        encryption_repository: encryption_repo.clone(),
        local_envelope_encryption_key_path,
        agent_cache: agent_cache.clone(),
    })
    .await?;
    debug!("API service initialized");

    // Start the unified change pubsub forwarder (after api_service is created so we can subscribe to identity events)
    trace!("Starting unified change pubsub");
    let soma_change_tx_for_pubsub = soma_change_tx.clone();
    let identity_change_tx_for_pubsub = api_service.identity_service.on_config_change_tx.clone();
    let on_bridge_config_change_tx_for_pubsub = on_bridge_config_change_tx.clone();
    let encryption_change_tx_for_pubsub = encryption_change_tx.clone();
    let secret_change_tx_for_pubsub = secret_change_tx.clone();
    let environment_variable_change_tx_for_pubsub = environment_variable_change_tx.clone();
    let process_manager_clone = process_manager.clone();
    process_manager_clone.lock().await.start_thread("change_pubsub", ThreadConfig {
        spawn_fn: {
            let soma_change_tx = soma_change_tx_for_pubsub.clone();
            let process_manager_for_pubsub = process_manager_clone.clone();
            let identity_change_tx = identity_change_tx_for_pubsub.clone();
            let on_bridge_config_change_tx = on_bridge_config_change_tx_for_pubsub.clone();
            let encryption_change_tx = encryption_change_tx_for_pubsub.clone();
            let secret_change_tx = secret_change_tx_for_pubsub.clone();
            let environment_variable_change_tx = environment_variable_change_tx_for_pubsub.clone();
            move || {
                let soma_change_tx = soma_change_tx.clone();
                let process_manager_for_pubsub = process_manager_for_pubsub.clone();
                let identity_change_tx = identity_change_tx.clone();
                let on_bridge_config_change_tx = on_bridge_config_change_tx.clone();
                let encryption_change_tx = encryption_change_tx.clone();
                let secret_change_tx = secret_change_tx.clone();
                let environment_variable_change_tx = environment_variable_change_tx.clone();
                tokio::spawn(async move {
                    // Create a broadcast channel for shutdown_rx (run_change_pubsub expects this)
                    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
                    // Spawn a task to monitor process manager shutdown and signal change_pubsub
                    tokio::spawn(async move {
                        process_manager_for_pubsub.lock().await.wait_for_shutdown().await;
                        let _ = shutdown_tx.send(());
                    });
                    run_change_pubsub(
                        soma_change_tx,
                        on_bridge_config_change_tx.subscribe(),
                        encryption_change_tx.subscribe(),
                        secret_change_tx.subscribe(),
                        environment_variable_change_tx.subscribe(),
                        identity_change_tx.subscribe(),
                        shutdown_rx,
                    )
                    .await;
                    Ok(())
                })
            }
        },
        health_check: None,
        on_terminal_stop: OnTerminalStop::TriggerShutdown,
        on_stop: OnStop::Restart(RestartConfig {
            max_restarts: 5,
            restart_delay: 1000,
        }),
        shutdown_priority: 5,
        follow_logs: false,
        on_shutdown_triggered: None,
        on_shutdown_complete: None,
    }).await
    .inspect_err(|e| error!(error = %e, "Failed to start change pubsub thread"))?;

    // Note: MCP service is now nested directly in the router as a Tower service.
    // No separate subsystem is needed.

    // Note: SDK sync is now SDK-initiated. When the SDK server starts (or restarts due to HMR),
    // it calls the /_internal/v1/resync_sdk endpoint to trigger sync of providers, agents,
    // secrets, and environment variables. This replaces the old connection-monitoring approach.

    // Start credential rotation
    trace!("Starting credential rotation");
    let credential_rotation_handle = start_credential_rotation_subsystem(
        bridge_repo.clone(),
        crypto_cache.clone(),
        on_bridge_config_change_tx.clone(),
        process_manager.clone(),
    ).await?;

    // Start bridge client generation listener
    trace!("Starting bridge client generation listener");
    let bridge_client_gen_handle = {
        let bridge_repo_clone = bridge_repo.clone();
        let sdk_client_clone = sdk_client.clone();
        let agent_cache_clone = agent_cache.clone();
        let bridge_client_gen_rx_clone = bridge_client_gen_rx.resubscribe();
        let process_manager_for_bridge = process_manager.clone();
        let (handle, _signal) = shared::subsystem::SubsystemHandle::new("Bridge Client Generation");
        process_manager.lock().await.start_thread("bridge_client_generation", ThreadConfig {
            spawn_fn: move || {
                let bridge_repo = bridge_repo_clone.clone();
                let sdk_client = sdk_client_clone.clone();
                let agent_cache = agent_cache_clone.clone();
                let mut on_bridge_config_change_rx = bridge_client_gen_rx_clone.resubscribe();
                let process_manager_for_bridge = process_manager_for_bridge.clone();
                tokio::spawn(async move {
                    // Create a broadcast channel for shutdown_rx (start_bridge_client_generation_subsystem expects this)
                    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
                    // Spawn a task to monitor process manager shutdown and signal bridge client generation
                    tokio::spawn(async move {
                        process_manager_for_bridge.lock().await.wait_for_shutdown().await;
                        let _ = shutdown_tx.send(());
                    });
                    crate::logic::bridge::start_bridge_client_generation_subsystem(
                        bridge_repo,
                        sdk_client,
                        agent_cache,
                        on_bridge_config_change_rx,
                        shutdown_rx,
                    )?;
                    Ok(())
                })
            },
            health_check: None,
            on_terminal_stop: OnTerminalStop::Ignore,
            on_stop: OnStop::Nothing,
            shutdown_priority: 4,
            follow_logs: false,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }).await
        .inspect_err(|e| error!(error = %e, "Failed to start bridge client generation thread"))?;
        handle
    };

    // Start secret sync subsystem
    trace!("Starting secret sync subsystem");
    let secret_sync_rx = secret_change_tx.subscribe();
    let socket_path_clone = socket_path.clone();
    let secret_sync_handle = {
        let repository_clone = repository.clone();
        let crypto_cache_clone = crypto_cache.clone();
        let socket_path_clone = socket_path_clone.clone();
        let secret_sync_rx_clone = secret_sync_rx.resubscribe();
        let process_manager_for_secret = process_manager.clone();
        let (handle, _signal) = shared::subsystem::SubsystemHandle::new("Secret Sync");
        process_manager.lock().await.start_thread("secret_sync", ThreadConfig {
            spawn_fn: move || {
                let repository = repository_clone.clone();
                let crypto_cache = crypto_cache_clone.clone();
                let socket_path = socket_path_clone.clone();
                let mut secret_change_rx = secret_sync_rx_clone.resubscribe();
                let process_manager_for_secret = process_manager_for_secret.clone();
                tokio::spawn(async move {
                    // Create a broadcast channel for shutdown_rx
                    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
                    // Spawn a task to monitor process manager shutdown
                    tokio::spawn(async move {
                        process_manager_for_secret.lock().await.wait_for_shutdown().await;
                        let _ = shutdown_tx.send(());
                    });
                    crate::logic::secret_sync::run_secret_sync_loop(crate::logic::secret_sync::SecretSyncParams {
                        repository: Arc::new(repository),
                        crypto_cache,
                        socket_path,
                        secret_change_rx,
                        shutdown_rx,
                    }).await?;
                    Ok(())
                })
            },
            health_check: None,
            on_terminal_stop: OnTerminalStop::Ignore,
            on_stop: OnStop::Nothing,
            shutdown_priority: 5,
            follow_logs: false,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }).await
        .inspect_err(|e| error!(error = %e, "Failed to start secret sync thread"))?;
        handle
    };

    // Start environment variable sync subsystem
    trace!("Starting environment variable sync subsystem");
    let env_var_sync_rx = environment_variable_change_tx.subscribe();
    let socket_path_for_env_sync = socket_path.clone();
    let env_var_sync_handle = {
        let repository_clone = repository.clone();
        let socket_path_clone = socket_path_for_env_sync.clone();
        let env_var_sync_rx_clone = env_var_sync_rx.resubscribe();
        let process_manager_for_env = process_manager.clone();
        let (handle, _signal) = shared::subsystem::SubsystemHandle::new("Environment Variable Sync");
        process_manager.lock().await.start_thread("environment_variable_sync", ThreadConfig {
            spawn_fn: move || {
                let repository = repository_clone.clone();
                let socket_path = socket_path_clone.clone();
                let mut env_var_change_rx = env_var_sync_rx_clone.resubscribe();
                let process_manager_for_env = process_manager_for_env.clone();
                tokio::spawn(async move {
                    // Create a broadcast channel for shutdown_rx
                    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
                    // Spawn a task to monitor process manager shutdown
                    tokio::spawn(async move {
                        process_manager_for_env.lock().await.wait_for_shutdown().await;
                        let _ = shutdown_tx.send(());
                    });
                    crate::logic::environment_variable_sync::run_environment_variable_sync_loop(crate::logic::environment_variable_sync::EnvironmentVariableSyncParams {
                        repository: Arc::new(repository),
                        socket_path,
                        environment_variable_change_rx: env_var_change_rx,
                        shutdown_rx,
                    }).await?;
                    Ok(())
                })
            },
            health_check: None,
            on_terminal_stop: OnTerminalStop::Ignore,
            on_stop: OnStop::Nothing,
            shutdown_priority: 5,
            follow_logs: false,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }).await
        .inspect_err(|e| error!(error = %e, "Failed to start environment variable sync thread"))?;
        handle
    };

    // Start JWK init listener (will start JWK rotation when default DEK is available)
    trace!("Starting JWK init listener");
    let encryption_change_rx_for_jwk = encryption_change_tx.subscribe();
    let jwk_init_handle = {
        let identity_repo_clone = identity_repo.clone();
        let crypto_cache_clone = crypto_cache.clone();
        let jwks_cache_clone = internal_jwks_cache.clone();
        let jwk_rotation_state_clone = jwk_rotation_state.clone();
        let mut encryption_change_rx_clone = encryption_change_rx_for_jwk.resubscribe();
        let process_manager_for_jwk = process_manager.clone();
        let (handle, _signal) = shared::subsystem::SubsystemHandle::new("JWK Init Listener");
        process_manager.lock().await.start_thread("jwk_init_listener", ThreadConfig {
            spawn_fn: move || {
                let identity_repo = identity_repo_clone.clone();
                let crypto_cache = crypto_cache_clone.clone();
                let jwks_cache = jwks_cache_clone.clone();
                let jwk_rotation_state = jwk_rotation_state_clone.clone();
                let mut encryption_change_rx = encryption_change_rx_clone.resubscribe();
                let process_manager_for_jwk = process_manager_for_jwk.clone();
                tokio::spawn(async move {
                    // Create a broadcast channel for shutdown_rx
                    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
                    // Create another channel for system_shutdown_signal (used internally by JWK init)
                    let (system_shutdown_tx, _) = broadcast::channel(1);
                    let system_shutdown_tx_for_spawn = system_shutdown_tx.clone();
                    // Spawn a task to monitor process manager shutdown
                    tokio::spawn(async move {
                        process_manager_for_jwk.lock().await.wait_for_shutdown().await;
                        let _ = shutdown_tx.send(());
                        let _ = system_shutdown_tx_for_spawn.send(());
                    });
                    crate::logic::identity::run_jwk_init_listener(
                        identity_repo,
                        crypto_cache,
                        jwks_cache,
                        jwk_rotation_state,
                        encryption_change_rx,
                        shutdown_rx,
                        system_shutdown_tx,
                    ).await?;
                    Ok(())
                })
            },
            health_check: None,
            on_terminal_stop: OnTerminalStop::Ignore,
            on_stop: OnStop::Nothing,
            shutdown_priority: 2,
            follow_logs: false,
            on_shutdown_triggered: None,
            on_shutdown_complete: None,
        }).await
        .inspect_err(|e| error!(error = %e, "Failed to start JWK init listener thread"))?;
        handle
    };

    // Note: Initial sync of secrets and environment variables now happens AFTER SDK server
    // healthcheck passes (see above, around line 171). This ensures the SDK server's gRPC
    // handlers are fully registered before we try to sync.

    Ok(ApiServiceBundle {
        api_service,
        subsystems: Subsystems {
            sdk_server: Some(sdk_server_handle),
            credential_rotation: Some(credential_rotation_handle),
            bridge_client_generation: Some(bridge_client_gen_handle),
            secret_sync: Some(secret_sync_handle),
            environment_variable_sync: Some(env_var_sync_handle),
            jwk_init_listener: Some(jwk_init_handle),
        },
        soma_change_tx,
    })
}

async fn start_sdk_server_subsystem(
    project_dir: PathBuf,
    sdk_runtime: SdkRuntime,
    restate_service_port: u16,
    repository: crate::repository::Repository,
    crypto_cache: CryptoCache,
    process_manager: Arc<tokio::sync::Mutex<CustomProcessManager>>,
) -> Result<SubsystemHandle, CommonError> {
    use crate::sdk::{StartDevSdkParams, start_dev_sdk};

    let (handle, _signal) = SubsystemHandle::new("SDK Server");
    let process_manager_clone = process_manager.clone();

    process_manager_clone.lock().await.start_thread("sdk_server", ThreadConfig {
        spawn_fn: {
            let project_dir = project_dir.clone();
            let sdk_runtime = sdk_runtime.clone();
            let restate_service_port = restate_service_port;
            let repository = repository.clone();
            let crypto_cache = crypto_cache.clone();
            let process_manager_for_thread = process_manager.clone();
            move || {
                let project_dir = project_dir.clone();
                let sdk_runtime = sdk_runtime.clone();
                let repository = repository.clone();
                let crypto_cache = crypto_cache.clone();
                let process_manager = process_manager_for_thread.clone();
                tokio::spawn(async move {
                    start_dev_sdk(StartDevSdkParams {
                        project_dir,
                        sdk_runtime,
                        restate_service_port,
                        repository: std::sync::Arc::new(repository),
                        crypto_cache,
                        process_manager,
                    })
                    .await
                })
            }
        },
        health_check: None,
        on_terminal_stop: OnTerminalStop::TriggerShutdown,
        on_stop: OnStop::Restart(RestartConfig {
            max_restarts: 10,
            restart_delay: 2000,
        }),
        shutdown_priority: 8,
        follow_logs: false,
        on_shutdown_triggered: None,
        on_shutdown_complete: None,
    }).await
    .inspect_err(|e| error!(error = %e, "Failed to start SDK server thread"))?;

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

    trace!("Waiting for SDK server healthcheck");

    for attempt in 1..=MAX_ITERATIONS {
        let health_request = tonic::Request::new(());
        match client.health_check(health_request).await {
            Ok(_) => {
                trace!("SDK server healthcheck passed");
                return Ok(());
            }
            Err(e) => {
                if attempt < MAX_ITERATIONS {
                    trace!(
                        attempt = attempt,
                        max = MAX_ITERATIONS,
                        error = ?e,
                        "SDK server healthcheck not ready, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                } else {
                    error!(
                        attempts = MAX_ITERATIONS,
                        error = ?e,
                        "SDK server healthcheck failed"
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

async fn start_credential_rotation_subsystem(
    bridge_repo: bridge::repository::Repository,
    crypto_cache: CryptoCache,
    on_bridge_change_tx: OnConfigChangeTx,
    process_manager: Arc<tokio::sync::Mutex<CustomProcessManager>>,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, _signal) = SubsystemHandle::new("Credential Rotation");
    let process_manager_clone = process_manager.clone();

    process_manager_clone.lock().await.start_thread("credential_rotation", ThreadConfig {
        spawn_fn: {
            let bridge_repo = bridge_repo.clone();
            let crypto_cache = crypto_cache.clone();
            let on_bridge_change_tx = on_bridge_change_tx.clone();
            let process_manager_for_cred = process_manager.clone();
            move || {
                let bridge_repo = bridge_repo.clone();
                let crypto_cache = crypto_cache.clone();
                let on_bridge_change_tx = on_bridge_change_tx.clone();
                let process_manager_for_cred = process_manager_for_cred.clone();
                tokio::spawn(async move {
                    // Create a broadcast channel for shutdown_rx (credential rotation expects this)
                    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
                    // Spawn a task to monitor process manager shutdown and signal credential rotation
                    tokio::spawn(async move {
                        process_manager_for_cred.lock().await.wait_for_shutdown().await;
                        let _ = shutdown_tx.send(());
                    });
                    bridge::logic::credential_rotation_task(
                        bridge_repo,
                        crypto_cache,
                        on_bridge_change_tx,
                        shutdown_rx,
                    )
                    .await;
                    Ok(())
                })
            }
        },
        health_check: None,
        on_terminal_stop: OnTerminalStop::Ignore,
        on_stop: OnStop::Restart(RestartConfig {
            max_restarts: 5,
            restart_delay: 1000,
        }),
        shutdown_priority: 3,
        follow_logs: false,
        on_shutdown_triggered: None,
        on_shutdown_complete: None,
    }).await
    .inspect_err(|e| error!(error = %e, "Failed to start credential rotation thread"))?;

    Ok(handle)
}
