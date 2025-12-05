use std::sync::Arc;

use encryption::logic::EncryptionKeyEvent;
use encryption::logic::crypto_services::CryptoCache;
use identity::logic::jwk::DEFAULT_JWK_DEK_ALIAS;
use identity::logic::jwk::cache::JwksCache;
use identity::repository::Repository as IdentityRepository;
use shared::error::CommonError;
use shared::subsystem::SubsystemHandle;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// State to track whether JWK rotation has been initialized
#[derive(Clone)]
pub struct JwkRotationState {
    initialized: Arc<std::sync::atomic::AtomicBool>,
}

impl JwkRotationState {
    pub fn new() -> Self {
        Self {
            initialized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_initialized(&self) {
        self.initialized
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for JwkRotationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts a listener that watches for encryption key events
/// and initializes JWK rotation when the default DEK alias is created
pub fn start_jwk_init_on_dek_listener(
    identity_repo: IdentityRepository,
    crypto_cache: CryptoCache,
    jwks_cache: JwksCache,
    jwk_rotation_state: JwkRotationState,
    encryption_change_rx: broadcast::Receiver<EncryptionKeyEvent>,
    system_shutdown_signal: broadcast::Sender<()>,
) -> Result<SubsystemHandle, CommonError> {
    use shared::subsystem::SubsystemHandle;

    let (handle, signal) = SubsystemHandle::new("JWK Init Listener");
    let shutdown_rx = system_shutdown_signal.subscribe();

    tokio::spawn(async move {
        match run_jwk_init_listener(
            identity_repo,
            crypto_cache,
            jwks_cache,
            jwk_rotation_state,
            encryption_change_rx,
            shutdown_rx,
            system_shutdown_signal,
        )
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                tracing::error!("JWK init listener stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}

async fn run_jwk_init_listener(
    identity_repo: IdentityRepository,
    crypto_cache: CryptoCache,
    jwks_cache: JwksCache,
    jwk_rotation_state: JwkRotationState,
    mut encryption_change_rx: broadcast::Receiver<EncryptionKeyEvent>,
    mut shutdown_rx: broadcast::Receiver<()>,
    system_shutdown_signal: broadcast::Sender<()>,
) -> Result<(), CommonError> {
    info!("Starting JWK init listener, waiting for default DEK alias...");

    loop {
        tokio::select! {
            event = encryption_change_rx.recv() => {
                match event {
                    Ok(encryption_evt) => {
                        if let Err(e) = handle_encryption_event(
                            &encryption_evt,
                            &identity_repo,
                            &crypto_cache,
                            &jwks_cache,
                            &jwk_rotation_state,
                            &system_shutdown_signal,
                        ).await {
                            warn!("Error handling encryption event for JWK init: {:?}", e);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Encryption change channel closed, stopping JWK init listener");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Encryption change channel lagged, skipped {} messages", skipped);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, stopping JWK init listener");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_encryption_event(
    event: &EncryptionKeyEvent,
    identity_repo: &IdentityRepository,
    crypto_cache: &CryptoCache,
    jwks_cache: &JwksCache,
    jwk_rotation_state: &JwkRotationState,
    system_shutdown_signal: &broadcast::Sender<()>,
) -> Result<(), CommonError> {
    // Only handle DEK alias events
    let alias = match event {
        EncryptionKeyEvent::DataEncryptionKeyAliasAdded { alias, .. } => alias,
        EncryptionKeyEvent::DataEncryptionKeyAliasUpdated { alias, .. } => alias,
        _ => return Ok(()),
    };

    // Check if this is the default JWK DEK alias
    if alias != DEFAULT_JWK_DEK_ALIAS {
        return Ok(());
    }

    // Check if already initialized
    if jwk_rotation_state.is_initialized() {
        info!("JWK rotation already initialized, skipping");
        return Ok(());
    }

    info!(
        "Default DEK alias '{}' detected, initializing JWK system...",
        DEFAULT_JWK_DEK_ALIAS
    );

    // Create initial JWKs if needed
    identity::logic::jwk::check_jwks_exists_on_start(
        identity_repo,
        crypto_cache,
        jwks_cache,
        DEFAULT_JWK_DEK_ALIAS,
    )
    .await?;

    // Mark as initialized
    jwk_rotation_state.set_initialized();

    // Start the JWK rotation task
    let identity_repo_clone = identity_repo.clone();
    let crypto_cache_clone = crypto_cache.clone();
    let jwks_cache_clone = jwks_cache.clone();
    let shutdown_rx = system_shutdown_signal.subscribe();

    tokio::spawn(async move {
        identity::logic::jwk::jwk_rotation_task(
            identity_repo_clone,
            crypto_cache_clone,
            jwks_cache_clone,
            DEFAULT_JWK_DEK_ALIAS.to_string(),
            shutdown_rx,
        )
        .await;
        info!("JWK rotation task stopped");
    });

    info!("JWK rotation task started successfully");

    Ok(())
}
