use std::sync::Arc;

use encryption::logic::EncryptionKeyEvent;
use encryption::logic::crypto_services::CryptoCache;
use identity::logic::jwk::DEFAULT_JWK_DEK_ALIAS;
use identity::logic::jwk::cache::JwksCache;
use identity::repository::Repository as IdentityRepository;
use shared::error::CommonError;
use tokio::sync::broadcast;
use tracing::{debug, trace, warn};

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

/// Runs the JWK init listener loop - watches for encryption key events
/// and initializes JWK rotation when the default DEK alias is created.
/// This function runs indefinitely until aborted by the process manager.
pub async fn run_jwk_init_listener(
    identity_repo: IdentityRepository,
    crypto_cache: CryptoCache,
    jwks_cache: JwksCache,
    jwk_rotation_state: JwkRotationState,
    mut encryption_change_rx: broadcast::Receiver<EncryptionKeyEvent>,
) -> Result<(), CommonError> {
    debug!("JWK init listener started, waiting for DEK alias");

    loop {
        match encryption_change_rx.recv().await {
            Ok(encryption_evt) => {
                if let Err(e) = handle_encryption_event(
                    &encryption_evt,
                    &identity_repo,
                    &crypto_cache,
                    &jwks_cache,
                    &jwk_rotation_state,
                ).await {
                    warn!(error = ?e, "Error handling encryption event for JWK init");
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                debug!("Encryption change channel closed");
                break;
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(skipped, "Encryption change channel lagged");
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
        trace!("JWK rotation already initialized");
        return Ok(());
    }

    debug!(dek_alias = DEFAULT_JWK_DEK_ALIAS, "Initializing JWK system");

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

    // Start the JWK rotation task - runs indefinitely until aborted by process manager
    let identity_repo_clone = identity_repo.clone();
    let crypto_cache_clone = crypto_cache.clone();
    let jwks_cache_clone = jwks_cache.clone();

    tokio::spawn(async move {
        identity::logic::jwk::jwk_rotation_task(
            identity_repo_clone,
            crypto_cache_clone,
            jwks_cache_clone,
            DEFAULT_JWK_DEK_ALIAS.to_string(),
        )
        .await;
        debug!("JWK rotation task stopped");
    });

    debug!("JWK rotation task started");

    Ok(())
}
