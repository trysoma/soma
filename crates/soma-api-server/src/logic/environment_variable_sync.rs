use shared::error::CommonError;
use shared::primitives::PaginationRequest;
use shared::subsystem::SubsystemHandle;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::logic::on_change_pubsub::EnvironmentVariableChangeRx;
use crate::repository::EnvironmentVariableRepositoryLike;

/// An environment variable ready to be sent to the SDK
#[derive(Debug, Clone)]
pub struct EnvironmentVariableData {
    pub key: String,
    pub value: String,
}

/// Fetch all environment variables from the database
pub async fn fetch_all_environment_variables(
    repository: &std::sync::Arc<crate::repository::Repository>,
) -> Result<Vec<EnvironmentVariableData>, CommonError> {
    let mut all_env_vars = Vec::new();
    let mut page_token = None;

    // Paginate through all environment variables
    loop {
        let pagination = PaginationRequest {
            page_size: 100,
            next_page_token: page_token.clone(),
        };

        let page = repository
            .as_ref()
            .get_environment_variables(&pagination)
            .await?;

        for env_var in page.items {
            all_env_vars.push(EnvironmentVariableData {
                key: env_var.key,
                value: env_var.value,
            });
        }

        page_token = page.next_page_token;
        if page_token.is_none() {
            break;
        }
    }

    Ok(all_env_vars)
}

/// Sync environment variables to the SDK via gRPC
pub async fn sync_environment_variables_to_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    env_vars: Vec<EnvironmentVariableData>,
) -> Result<(), CommonError> {
    let proto_env_vars: Vec<sdk_proto::EnvironmentVariable> = env_vars
        .into_iter()
        .map(|e| sdk_proto::EnvironmentVariable {
            key: e.key,
            value: e.value,
        })
        .collect();

    let request = tonic::Request::new(sdk_proto::SetEnvironmentVariablesRequest {
        environment_variables: proto_env_vars,
    });

    let response = sdk_client
        .set_environment_variables(request)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call set_environment_variables RPC: {e}"
            ))
        })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::set_environment_variables_response::Kind::Data(data)) => {
            info!(
                "Successfully synced environment variables to SDK: {}",
                data.message
            );
            Ok(())
        }
        Some(sdk_proto::set_environment_variables_response::Kind::Error(error)) => {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK rejected environment variables: {}",
                error.message
            )))
        }
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected environment variables: unknown error"
        ))),
    }
}

pub struct EnvironmentVariableSyncParams {
    pub repository: std::sync::Arc<crate::repository::Repository>,
    pub socket_path: String,
    pub environment_variable_change_rx: EnvironmentVariableChangeRx,
    pub shutdown_rx: broadcast::Receiver<()>,
}

/// Run the environment variable sync loop - listens for env var changes and syncs to SDK
pub async fn run_environment_variable_sync_loop(
    params: EnvironmentVariableSyncParams,
) -> Result<(), CommonError> {
    let EnvironmentVariableSyncParams {
        repository,
        socket_path,
        mut environment_variable_change_rx,
        mut shutdown_rx,
    } = params;
    let repository = repository.clone();

    info!("Starting environment variable sync loop");

    loop {
        tokio::select! {
            // Handle environment variable change events
            event = environment_variable_change_rx.recv() => {
                match event {
                    Ok(evt) => {
                        info!("Environment variable change event received: {:?}", evt);

                        // On any env var change, re-sync all env vars
                        // This is simpler than tracking individual changes and ensures consistency
                        match sync_all_environment_variables(&repository, &socket_path).await {
                            Ok(()) => {
                                info!("Environment variables re-synced after change event");
                            }
                            Err(e) => {
                                error!("Failed to re-sync environment variables after change: {:?}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Environment variable change channel closed, stopping env var sync");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Environment variable change channel lagged, skipped {} messages. Re-syncing all env vars.", skipped);
                        // Re-sync all env vars to ensure we're in a consistent state
                        if let Err(e) = sync_all_environment_variables(&repository, &socket_path).await {
                            error!("Failed to re-sync environment variables after lag: {:?}", e);
                        }
                    }
                }
            }
            // Handle shutdown
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, stopping environment variable sync");
                break;
            }
        }
    }

    Ok(())
}

/// Helper to sync all environment variables to SDK
async fn sync_all_environment_variables(
    repository: &std::sync::Arc<crate::repository::Repository>,
    socket_path: &str,
) -> Result<(), CommonError> {
    // Fetch all environment variables
    let env_vars = fetch_all_environment_variables(repository).await?;
    info!("Fetched {} environment variables to sync", env_vars.len());

    // Connect to SDK and sync
    let mut client = shared::uds::create_soma_unix_socket_client(socket_path).await?;
    sync_environment_variables_to_sdk(&mut client, env_vars).await
}

/// Start the environment variable sync subsystem
pub fn start_environment_variable_sync_subsystem(
    repository: crate::repository::Repository,
    socket_path: String,
    environment_variable_change_rx: EnvironmentVariableChangeRx,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("Environment Variable Sync");
    let repository = std::sync::Arc::new(repository);

    tokio::spawn(async move {
        match run_environment_variable_sync_loop(EnvironmentVariableSyncParams {
            repository,
            socket_path,
            environment_variable_change_rx,
            shutdown_rx,
        })
        .await
        {
            Ok(()) => {
                signal.signal_with_message("stopped gracefully");
            }
            Err(e) => {
                error!("Environment variable sync stopped with error: {:?}", e);
                signal.signal();
            }
        }
    });

    Ok(handle)
}
