use encryption::logic::crypto_services::{CryptoCache, EncryptedString};
use environment::repository::SecretRepositoryLike;
use shared::error::CommonError;
use shared::primitives::PaginationRequest;
use tokio::sync::broadcast;
use tracing::{debug, error, trace, warn};

use crate::logic::on_change_pubsub::SecretChangeRx;

/// A decrypted secret ready to be sent to the SDK
#[derive(Debug, Clone)]
pub struct DecryptedSecret {
    pub key: String,
    pub value: String,
}

/// Fetch all secrets from the database and decrypt them
pub async fn fetch_and_decrypt_all_secrets(
    repository: &std::sync::Arc<environment::repository::Repository>,
    crypto_cache: &CryptoCache,
) -> Result<Vec<DecryptedSecret>, CommonError> {
    let mut all_secrets = Vec::new();
    let mut page_token = None;

    // Paginate through all secrets
    loop {
        let pagination = PaginationRequest {
            page_size: 100,
            next_page_token: page_token.clone(),
        };

        let page = repository.as_ref().get_secrets(&pagination).await?;

        for secret in page.items {
            trace!(key = %secret.key, dek_alias = %secret.dek_alias, "Decrypting secret");
            // Get decryption service for this secret's DEK alias
            match crypto_cache.get_decryption_service(&secret.dek_alias).await {
                Ok(decryption_service) => {
                    // Decrypt the secret value
                    match decryption_service
                        .decrypt_data(EncryptedString(secret.encrypted_secret.clone()))
                        .await
                    {
                        Ok(decrypted_value) => {
                            trace!(
                                key = %secret.key,
                                decrypted_len = decrypted_value.len(),
                                "Secret decrypted"
                            );
                            all_secrets.push(DecryptedSecret {
                                key: secret.key,
                                value: decrypted_value,
                            });
                        }
                        Err(e) => {
                            warn!(
                                key = %secret.key,
                                error = ?e,
                                "Failed to decrypt secret, skipping"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        dek_alias = %secret.dek_alias,
                        key = %secret.key,
                        error = ?e,
                        "Failed to get decryption service, skipping secret"
                    );
                }
            }
        }

        page_token = page.next_page_token;
        if page_token.is_none() {
            break;
        }
    }

    Ok(all_secrets)
}

/// Sync secrets to the SDK via gRPC (for initial sync - sends all secrets)
pub async fn sync_secrets_to_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    secrets: Vec<DecryptedSecret>,
) -> Result<(), CommonError> {
    let proto_secrets: Vec<sdk_proto::Secret> = secrets
        .into_iter()
        .map(|s| sdk_proto::Secret {
            key: s.key,
            value: s.value,
        })
        .collect();

    let request = tonic::Request::new(sdk_proto::SetSecretsRequest {
        secrets: proto_secrets,
    });

    let response = sdk_client.set_secrets(request).await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to call set_secrets RPC: {e}"))
    })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::set_secrets_response::Kind::Data(_)) => {
            trace!("Secrets synced to SDK");
            Ok(())
        }
        Some(sdk_proto::set_secrets_response::Kind::Error(error)) => Err(CommonError::Unknown(
            anyhow::anyhow!("SDK rejected secrets: {}", error.message),
        )),
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected secrets: unknown error"
        ))),
    }
}

/// Incrementally sync a single secret to the SDK via gRPC
pub async fn sync_secret_to_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    key: String,
    value: String,
) -> Result<(), CommonError> {
    let request = tonic::Request::new(sdk_proto::SetSecretsRequest {
        secrets: vec![sdk_proto::Secret { key, value }],
    });

    let response = sdk_client.set_secrets(request).await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to call set_secrets RPC: {e}"))
    })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::set_secrets_response::Kind::Data(_)) => {
            trace!("Secret synced to SDK");
            Ok(())
        }
        Some(sdk_proto::set_secrets_response::Kind::Error(error)) => Err(CommonError::Unknown(
            anyhow::anyhow!("SDK rejected secret: {}", error.message),
        )),
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected secret: unknown error"
        ))),
    }
}

/// Unset a secret in the SDK via gRPC
pub async fn unset_secret_in_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    key: String,
) -> Result<(), CommonError> {
    let request = tonic::Request::new(sdk_proto::UnsetSecretRequest { key });

    let response = sdk_client.unset_secrets(request).await.map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("Failed to call unset_secrets RPC: {e}"))
    })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::unset_secret_response::Kind::Data(_)) => {
            trace!("Unset secret in SDK");
            Ok(())
        }
        Some(sdk_proto::unset_secret_response::Kind::Error(error)) => Err(CommonError::Unknown(
            anyhow::anyhow!("SDK rejected unset secret: {}", error.message),
        )),
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected unset secret: unknown error"
        ))),
    }
}

pub struct SecretSyncParams {
    pub repository: std::sync::Arc<environment::repository::Repository>,
    pub crypto_cache: CryptoCache,
    pub socket_path: String,
    pub secret_change_rx: SecretChangeRx,
}

/// Run the secret sync loop - listens for secret changes and syncs to SDK.
/// This function runs indefinitely until aborted by the process manager.
pub async fn run_secret_sync_loop(params: SecretSyncParams) -> Result<(), CommonError> {
    let SecretSyncParams {
        repository,
        crypto_cache,
        socket_path,
        mut secret_change_rx,
    } = params;
    let repository = repository.clone();

    debug!("Secret sync loop started");

    loop {
        match secret_change_rx.recv().await {
            Ok(evt) => {
                trace!(event = ?evt, "Secret change event");

                // On any secret change, re-sync all secrets
                // This is simpler than tracking individual changes and ensures consistency
                match sync_all_secrets(&repository, &crypto_cache, &socket_path).await {
                    Ok(()) => {
                        trace!("Secrets re-synced");
                    }
                    Err(e) => {
                        error!(error = ?e, "Failed to re-sync secrets");
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                debug!("Secret change channel closed");
                break;
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(skipped, "Secret change channel lagged, re-syncing");
                // Re-sync all secrets to ensure we're in a consistent state
                if let Err(e) = sync_all_secrets(&repository, &crypto_cache, &socket_path).await {
                    error!(error = ?e, "Failed to re-sync secrets after lag");
                }
            }
        }
    }

    Ok(())
}

/// Helper to sync all secrets to SDK
async fn sync_all_secrets(
    repository: &std::sync::Arc<environment::repository::Repository>,
    crypto_cache: &CryptoCache,
    socket_path: &str,
) -> Result<(), CommonError> {
    // Fetch and decrypt all secrets
    let secrets = fetch_and_decrypt_all_secrets(repository, crypto_cache).await?;
    trace!(count = secrets.len(), "Syncing secrets to SDK");

    // Connect to SDK and sync
    let mut client = shared::uds::create_soma_unix_socket_client(socket_path).await?;
    sync_secrets_to_sdk(&mut client, secrets).await
}
