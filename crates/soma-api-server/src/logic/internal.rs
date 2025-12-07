use std::sync::Arc;

use bridge::repository::ProviderRepositoryLike;
use encryption::logic::crypto_services::CryptoCache;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::uds::{DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client};
use tokio::sync::Mutex;
use tonic::{Request, transport::Channel};
use tracing::{error, info, warn};
use utoipa::ToSchema;

use crate::logic::environment_variable_sync::{
    fetch_all_environment_variables, sync_environment_variables_to_sdk,
};
use crate::logic::secret_sync::{fetch_and_decrypt_all_secrets, sync_secrets_to_sdk};
use crate::sdk::sdk_provider_sync;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CheckSdkHealthResponse {}

/// Checks SDK server health via gRPC
pub async fn check_sdk_health(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
) -> Result<CheckSdkHealthResponse, CommonError> {
    let mut sdk_client_guard = sdk_client.lock().await;
    let client = match sdk_client_guard.as_mut() {
        Some(client) => client,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK client not available. Please ensure the SDK server is running."
            )));
        }
    };

    let request = Request::new(());
    match client.health_check(request).await {
        Ok(_) => {
            info!("SDK server health check passed");
            Ok(CheckSdkHealthResponse {})
        }
        Err(e) => {
            warn!("SDK server health check failed: {:?}", e);
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK server health check failed: {e}"
            )))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TriggerCodegenResponse {}

/// Triggers bridge client generation via gRPC call to SDK server
pub async fn trigger_codegen(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    bridge_repo: &impl ProviderRepositoryLike,
) -> Result<TriggerCodegenResponse, CommonError> {
    let mut sdk_client_guard = sdk_client.lock().await;

    let client = match sdk_client_guard.as_mut() {
        Some(client) => client,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK client not available. Please ensure the SDK server is running."
            )));
        }
    };

    crate::logic::bridge::codegen::trigger_bridge_client_generation(client, bridge_repo).await?;

    Ok(TriggerCodegenResponse {})
}

/// Result of resync operation
pub struct ResyncResult {
    pub providers_synced: usize,
    pub agents_synced: usize,
    pub secrets_synced: usize,
    pub env_vars_synced: usize,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResyncSdkResponse {}

/// Resync SDK: fetches metadata from SDK, syncs providers/agents to bridge registry,
/// registers Restate deployments, and syncs secrets/env vars to SDK
pub async fn resync_sdk(
    repository: &std::sync::Arc<crate::repository::Repository>,
    crypto_cache: &CryptoCache,
    restate_params: &crate::restate::RestateServerParams,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
) -> Result<ResyncSdkResponse, CommonError> {
    let mut sdk_client_guard = sdk_client.lock().await;

    // Try to reconnect to SDK server (it may have restarted)
    info!("Resync: Reconnecting to SDK server...");
    match create_soma_unix_socket_client(DEFAULT_SOMA_SERVER_SOCK).await {
        Ok(new_client) => {
            info!("Resync: Successfully reconnected to SDK server");
            *sdk_client_guard = Some(new_client);
        }
        Err(e) => {
            warn!("Resync: Failed to reconnect to SDK server: {:?}", e);
            // Continue with existing client if reconnect fails
        }
    }

    let client = match sdk_client_guard.as_mut() {
        Some(client) => client,
        None => {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK client not available. Please ensure the SDK server is running."
            )));
        }
    };

    let request = Request::new(());
    let response = client
        .metadata(request)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to get SDK metadata: {e}")))?;

    let metadata = response.into_inner();

    info!(
        "Fetched SDK metadata: {} providers, {} agents",
        metadata.bridge_providers.len(),
        metadata.agents.len()
    );

    // Sync providers to bridge registry
    sdk_provider_sync::sync_providers_from_metadata(&metadata)?;

    let providers_synced = metadata.bridge_providers.len();
    let agents_synced = metadata.agents.len();

    // Register Restate deployments for agents
    if !metadata.agents.is_empty() {
        for agent in metadata.agents {
            let restate_service_id = format!("{}.{}", agent.project_id, agent.id);
            register_agent_deployment(agent, restate_params, &restate_service_id).await?;
        }
    }

    // Sync secrets to SDK
    let secrets = fetch_and_decrypt_all_secrets(repository, crypto_cache).await?;
    let secrets_count = secrets.len();
    info!("Syncing {} secrets to SDK", secrets_count);
    if !secrets.is_empty() {
        sync_secrets_to_sdk(client, secrets).await?;
    }

    // Sync environment variables to SDK
    let env_vars = fetch_all_environment_variables(repository).await?;
    let env_vars_count = env_vars.len();
    info!("Syncing {} environment variables to SDK", env_vars_count);
    if !env_vars.is_empty() {
        sync_environment_variables_to_sdk(client, env_vars).await?;
    }

    info!(
        "SDK resync complete: {} providers, {} agents, {} secrets, {} env vars",
        providers_synced, agents_synced, secrets_count, env_vars_count
    );

    Ok(ResyncSdkResponse {})
}

/// Register Restate deployments for all agents
async fn register_agent_deployment(
    agent: sdk_proto::Agent,
    restate_server_params: &crate::restate::RestateServerParams,
    restate_service_id: &str,
) -> Result<(), CommonError> {
    use shared::restate;

    let service_address = restate_server_params.get_soma_restate_service_address();
    let deployment_type = restate::deploy::DeploymentType::Http {
        uri: service_address.to_string(),
        additional_headers: restate_server_params.get_soma_restate_service_additional_headers(),
    };

    info!(
        "Registering service path: {} with service address: {}",
        restate_service_id, service_address
    );

    let admin_url = restate_server_params.get_admin_address()?;
    let config = restate::deploy::DeploymentRegistrationConfig {
        admin_url: admin_url.to_string(),
        service_path: restate_service_id.to_string(),
        deployment_type,
        bearer_token: restate_server_params.get_admin_token(),
        private: restate_server_params.get_private(),
        insecure: restate_server_params.get_insecure(),
        force: restate_server_params.get_force(),
    };

    match restate::deploy::register_deployment(config).await {
        Ok(metadata) => {
            info!(
                "Successfully registered agent '{}' (service: {})",
                agent.name, metadata.name
            );
        }
        Err(e) => {
            error!("Failed to register agent '{}': {:?}", agent.name, e);
            // Continue with other agents even if one fails
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RuntimeConfigResponse {}

pub async fn runtime_config() -> Result<RuntimeConfigResponse, CommonError> {
    Ok(RuntimeConfigResponse {})
}
