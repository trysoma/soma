use std::path::PathBuf;
use std::sync::Arc;

use encryption::logic::crypto_services::CryptoCache;
use tool::logic::mcp::McpServerService;
use tool::logic::OnConfigChangeTx;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use shared::error::CommonError;
use shared::process_manager::{
    CustomProcessManager, OnStop, OnTerminalStop, RestartConfig, ThreadConfig,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace};

use crate::logic::on_change_pubsub::{SomaChangeTx, create_soma_change_channel, run_change_pubsub};
use crate::repository::setup_repository;
use crate::{ApiService, InitApiServiceParams};

pub struct CreateApiServiceParams {
    pub base_url: String,
    pub project_dir: PathBuf,
    pub db_conn_string: String,
    pub db_auth_token: Option<String>,
    pub process_manager: Arc<CustomProcessManager>,
}

pub struct ApiServiceBundle {
    pub api_service: ApiService,
    /// Unified change channel for external listeners to subscribe to mcp and encryption events
    pub soma_change_tx: SomaChangeTx,
    /// Bootstrap API key for initial sync and other basic config tasks.
    pub bootstrap_api_key: String,
}

/// Creates the API service and starts all subsystems
/// Returns the API service and subsystem handles for the caller to manage
pub async fn create_api_service(
    params: CreateApiServiceParams,
) -> Result<ApiServiceBundle, CommonError> {
    let CreateApiServiceParams {
        base_url,
        project_dir,
        db_conn_string,
        db_auth_token,
        process_manager,
    } = params;

    // Setup database and repositories
    trace!("Setting up database and repositories...");
    let db_url = url::Url::parse(&db_conn_string)?;
    let (_db, conn, _repository, tool_repo, encryption_repo, environment_repo) =
        setup_repository(&db_url, &db_auth_token).await?;
    trace!("Database and repositories setup");

    // Create identity repository (uses same connection)
    let identity_repo = identity::repository::Repository::new(conn.clone());

    // Create the mcp config change channel
    let (on_mcp_config_change_tx, _on_mcp_config_change_rx): (OnConfigChangeTx, _) =
        tokio::sync::broadcast::channel(100);

    // Create encryption event channel
    let (encryption_change_tx, _encryption_change_rx): (
        encryption::logic::EncryptionKeyEventSender,
        _,
    ) = tokio::sync::broadcast::channel(100);

    // Create secret event channel
    let (secret_change_tx, _secret_change_rx) =
        crate::logic::on_change_pubsub::create_secret_change_channel(100);

    // Create variable event channel
    let (variable_change_tx, _variable_change_rx) =
        crate::logic::on_change_pubsub::create_variable_change_channel(100);

    // Create the unified soma change channel
    let (soma_change_tx, _soma_change_rx) = create_soma_change_channel(100);

    // Initialize the crypto cache from the encryption repository
    trace!("Initializing crypto cache");
    let local_envelope_encryption_key_path = project_dir.join(".soma/envelope-encryption-keys");
    let crypto_cache = CryptoCache::new(
        encryption_repo.clone(),
        local_envelope_encryption_key_path.clone(),
    );
    encryption::logic::crypto_services::init_crypto_cache(&crypto_cache)
        .await
        .inspect_err(|_e| error!("Failed to initialize crypto cache"))?;
    trace!("Crypto cache initialized");

    // Create JWKS cache (JWKs will be created when default DEK alias is available)
    let internal_jwks_cache = identity::logic::jwk::cache::JwksCache::new(identity_repo.clone());

    // Create JWK rotation state to track initialization
    let jwk_rotation_state = crate::logic::identity::JwkRotationState::new();

    // Create MCP cancellation token for graceful shutdown
    let mcp_ct = CancellationToken::new();

    // Create the StreamableHttpService for MCP
    // Note: McpServerService is created fresh for each request by the service factory
    // Clone values for use in the service factory closure
    let tool_repo_for_mcp = tool_repo.clone();
    let crypto_cache_for_mcp = crypto_cache.clone();
    let mcp_service = StreamableHttpService::new(
        move || {
            Ok(McpServerService {
                repository: tool_repo_for_mcp.clone(),
                encryption_service: crypto_cache_for_mcp.clone(),
            })
        },
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig {
            cancellation_token: mcp_ct.child_token(),
            ..Default::default()
        },
    );

    // Initialize API service
    debug!("Initializing API service");
    let api_service = ApiService::new(InitApiServiceParams {
        base_url: base_url.clone(),
        internal_jwks_cache: internal_jwks_cache.clone(),
        environment_repository: environment_repo.clone(),
        mcp_service,
        on_mcp_config_change_tx: on_mcp_config_change_tx.clone(),
        crypto_cache: crypto_cache.clone(),
        tool_repository: tool_repo.clone(),
        identity_repository: identity_repo.clone(),
        on_encryption_change_tx: encryption_change_tx.clone(),
        on_secret_change_tx: secret_change_tx.clone(),
        on_variable_change_tx: variable_change_tx.clone(),
        encryption_repository: encryption_repo.clone(),
        local_envelope_encryption_key_path,
    })
    .await?;
    debug!("API service initialized");

    // Start the unified change pubsub forwarder
    trace!("Starting unified change pubsub");
    let soma_change_tx_for_pubsub = soma_change_tx.clone();
    let identity_change_tx_for_pubsub = api_service.identity_service.on_config_change_tx.clone();
    let on_mcp_config_change_tx_for_pubsub = on_mcp_config_change_tx.clone();
    let encryption_change_tx_for_pubsub = encryption_change_tx.clone();
    let secret_change_tx_for_pubsub = secret_change_tx.clone();
    let variable_change_tx_for_pubsub = variable_change_tx.clone();
    process_manager
        .start_thread(
            "change_pubsub",
            ThreadConfig {
                spawn_fn: {
                    let soma_change_tx = soma_change_tx_for_pubsub.clone();
                    let identity_change_tx = identity_change_tx_for_pubsub.clone();
                    let on_mcp_config_change_tx = on_mcp_config_change_tx_for_pubsub.clone();
                    let encryption_change_tx = encryption_change_tx_for_pubsub.clone();
                    let secret_change_tx = secret_change_tx_for_pubsub.clone();
                    let variable_change_tx = variable_change_tx_for_pubsub.clone();
                    move || {
                        let soma_change_tx = soma_change_tx.clone();
                        let identity_change_tx = identity_change_tx.clone();
                        let on_mcp_config_change_tx = on_mcp_config_change_tx.clone();
                        let encryption_change_tx = encryption_change_tx.clone();
                        let secret_change_tx = secret_change_tx.clone();
                        let variable_change_tx = variable_change_tx.clone();
                        tokio::spawn(async move {
                            run_change_pubsub(
                                soma_change_tx,
                                on_mcp_config_change_tx.subscribe(),
                                encryption_change_tx.subscribe(),
                                secret_change_tx.subscribe(),
                                variable_change_tx.subscribe(),
                                identity_change_tx.subscribe(),
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
            },
        )
        .await
        .inspect_err(|e| error!(error = %e, "Failed to start change pubsub thread"))?;

    // Start credential rotation
    trace!("Starting credential rotation");
    process_manager
        .start_thread(
            "credential_rotation",
            ThreadConfig {
                spawn_fn: {
                    let tool_repo = tool_repo.clone();
                    let crypto_cache = crypto_cache.clone();
                    let on_mcp_config_change_tx = on_mcp_config_change_tx.clone();
                    move || {
                        let tool_repo = tool_repo.clone();
                        let crypto_cache = crypto_cache.clone();
                        let on_mcp_config_change_tx = on_mcp_config_change_tx.clone();
                        tokio::spawn(async move {
                            tool::logic::credential_rotation_task(
                                tool_repo,
                                crypto_cache,
                                on_mcp_config_change_tx,
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
            },
        )
        .await
        .inspect_err(|e| error!(error = %e, "Failed to start credential rotation thread"))?;

    // Start JWK init listener (will start JWK rotation when default DEK is available)
    trace!("Starting JWK init listener");
    let encryption_change_rx_for_jwk = encryption_change_tx.subscribe();
    {
        let identity_repo_clone = identity_repo.clone();
        let crypto_cache_clone = crypto_cache.clone();
        let jwks_cache_clone = internal_jwks_cache.clone();
        let jwk_rotation_state_clone = jwk_rotation_state.clone();
        let encryption_change_rx_clone = encryption_change_rx_for_jwk.resubscribe();
        process_manager
            .start_thread(
                "jwk_init_listener",
                ThreadConfig {
                    spawn_fn: move || {
                        let identity_repo = identity_repo_clone.clone();
                        let crypto_cache = crypto_cache_clone.clone();
                        let jwks_cache = jwks_cache_clone.clone();
                        let jwk_rotation_state = jwk_rotation_state_clone.clone();
                        let encryption_change_rx = encryption_change_rx_clone.resubscribe();
                        tokio::spawn(async move {
                            crate::logic::identity::run_jwk_init_listener(
                                identity_repo,
                                crypto_cache,
                                jwks_cache,
                                jwk_rotation_state,
                                encryption_change_rx,
                            )
                            .await?;
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
                },
            )
            .await
            .inspect_err(|e| error!(error = %e, "Failed to start JWK init listener thread"))?;
    }

    trace!("Creating bootstrap API key");

    let bootstrap_api_key = identity::logic::api_key::bootstrap::create_bootstrap_api_key(Some(
        &api_service.identity_service.api_key_cache,
    ))
    .await?;
    trace!("Bootstrap API key created");

    Ok(ApiServiceBundle {
        api_service,
        soma_change_tx,
        bootstrap_api_key: bootstrap_api_key.api_key,
    })
}
