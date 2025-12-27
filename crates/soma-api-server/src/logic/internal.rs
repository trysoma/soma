use std::sync::Arc;

use mcp::repository::ProviderRepositoryLike;
use encryption::logic::crypto_services::CryptoCache;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::uds::{DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client};
use tokio::sync::Mutex;
use tonic::{Request, transport::Channel};
use tracing::{debug, error, trace, warn};
use utoipa::ToSchema;

use crate::logic::environment_variable_sync::{
    fetch_all_environment_variables, sync_environment_variables_to_sdk,
};
use crate::logic::secret_sync::{fetch_and_decrypt_all_secrets, sync_secrets_to_sdk};
use crate::sdk::{sdk_agent_sync, sdk_provider_sync};

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
            trace!("SDK server health check passed");
            Ok(CheckSdkHealthResponse {})
        }
        Err(e) => {
            debug!(error = ?e, "SDK server health check failed");
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK server health check failed: {e}"
            )))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TriggerCodegenResponse {}

/// Triggers mcp client generation via gRPC call to SDK server
pub async fn trigger_codegen(
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    mcp_repo: &impl ProviderRepositoryLike,
    agent_cache: &sdk_agent_sync::AgentCache,
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

    crate::logic::mcp::codegen::trigger_mcp_client_generation(
        client,
        mcp_repo,
        agent_cache,
    )
    .await?;

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

/// Resync SDK: fetches metadata from SDK, syncs providers/agents to mcp registry,
/// registers Restate deployments, syncs secrets/env vars to SDK, and triggers codegen
pub async fn resync_sdk(
    repository: &std::sync::Arc<crate::repository::Repository>,
    crypto_cache: &CryptoCache,
    restate_params: &crate::restate::RestateServerParams,
    sdk_client: &Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    agent_cache: &sdk_agent_sync::AgentCache,
    mcp_repo: &impl mcp::repository::ProviderRepositoryLike,
) -> Result<ResyncSdkResponse, CommonError> {
    let mut sdk_client_guard = sdk_client.lock().await;

    // Try to reconnect to SDK server (it may have restarted)
    trace!("Reconnecting to SDK server");
    match create_soma_unix_socket_client(DEFAULT_SOMA_SERVER_SOCK).await {
        Ok(new_client) => {
            trace!("Reconnected to SDK server");
            *sdk_client_guard = Some(new_client);
        }
        Err(e) => {
            debug!(error = ?e, "Failed to reconnect to SDK server, using existing client");
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

    debug!(
        providers = metadata.mcp_providers.len(),
        agents = metadata.agents.len(),
        "Fetched SDK metadata"
    );

    // Sync providers to mcp registry
    sdk_provider_sync::sync_providers_from_metadata(&metadata)?;

    // Capture existing agent IDs before syncing to detect removed agents
    let old_agent_ids = sdk_agent_sync::get_all_agent_ids(agent_cache);

    // Sync agents to cache (this clears and repopulates)
    sdk_agent_sync::sync_agents_from_metadata(agent_cache, &metadata);

    let providers_synced = metadata.mcp_providers.len();
    let agents_synced = metadata.agents.len();

    // Find and unregister removed agents
    let removed_agents = sdk_agent_sync::find_removed_agents(&old_agent_ids, &metadata);
    for (project_id, agent_id) in &removed_agents {
        let restate_service_id = format!("{project_id}.{agent_id}");
        debug!(project_id, agent_id, "Unregistering removed agent");
        if let Err(e) = unregister_agent_deployment(&restate_service_id, restate_params).await {
            warn!(project_id, agent_id, error = ?e, "Failed to unregister agent");
        }
    }
    if !removed_agents.is_empty() {
        debug!(count = removed_agents.len(), "Unregistered removed agents");
    }

    // Register Restate deployments for agents
    if !metadata.agents.is_empty() {
        for agent in &metadata.agents {
            let restate_service_id = format!("{}.{}", agent.project_id, agent.id);
            register_agent_deployment(agent.clone(), restate_params, &restate_service_id).await?;
        }
    }

    // Sync secrets to SDK
    let secrets = fetch_and_decrypt_all_secrets(repository, crypto_cache).await?;
    let secrets_count = secrets.len();
    if !secrets.is_empty() {
        trace!(count = secrets_count, "Syncing secrets to SDK");
        sync_secrets_to_sdk(client, secrets).await?;
    }

    // Sync environment variables to SDK
    let env_vars = fetch_all_environment_variables(repository).await?;
    let env_vars_count = env_vars.len();
    if !env_vars.is_empty() {
        trace!(
            count = env_vars_count,
            "Syncing environment variables to SDK"
        );
        sync_environment_variables_to_sdk(client, env_vars).await?;
    }

    debug!(
        providers = providers_synced,
        agents = agents_synced,
        secrets = secrets_count,
        env_vars = env_vars_count,
        "SDK resync complete"
    );

    // Trigger mcp client generation (includes agents now that they're synced)
    trace!("Triggering mcp client generation");
    if let Err(e) = crate::logic::mcp::codegen::trigger_mcp_client_generation(
        client,
        mcp_repo,
        agent_cache,
    )
    .await
    {
        warn!(error = ?e, "Failed to trigger mcp client generation");
    }

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

    debug!(
        service_path = %restate_service_id,
        service_address = %service_address,
        "Registering Restate deployment"
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
            trace!(agent = %agent.name, service = %metadata.name, "Registered agent");
        }
        Err(e) => {
            error!(agent = %agent.name, error = ?e, "Failed to register agent");
        }
    }

    Ok(())
}

/// Unregister a Restate deployment for a removed agent.
/// Finds the deployment containing the service and removes it.
async fn unregister_agent_deployment(
    restate_service_id: &str,
    restate_server_params: &crate::restate::RestateServerParams,
) -> Result<(), CommonError> {
    use shared::restate::admin_client::AdminClient;
    use shared::restate::admin_interface::{AdminClientInterface, Deployment};

    let admin_url = restate_server_params.get_admin_address()?;
    let client = AdminClient::new(admin_url, restate_server_params.get_admin_token()).await?;
    // Ensure the server is healthy before querying deployments
    client.ensure_healthy().await?;

    // Get all deployments to find the one containing this service
    let deployments_response = client
        .get_deployments()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to get deployments: {e}")))?;

    let deployments = deployments_response
        .into_body()
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse deployments: {e}")))?;

    // Find and remove deployments containing this service
    for deployment_response in deployments.deployments {
        let (deployment_id, _deployment, services) =
            Deployment::from_deployment_response(deployment_response);

        let has_service = services.iter().any(|s| s.name == restate_service_id);
        if has_service {
            trace!(deployment_id = %deployment_id, service = %restate_service_id, "Removing deployment");

            match client
                .remove_deployment(&deployment_id.to_string(), true)
                .await
            {
                Ok(response) => {
                    if response.status_code().is_success() {
                        trace!(service = %restate_service_id, "Removed deployment");
                    } else {
                        warn!(
                            service = %restate_service_id,
                            status = %response.status_code(),
                            "Unexpected status removing deployment"
                        );
                    }
                }
                Err(e) => {
                    warn!(service = %restate_service_id, error = ?e, "Failed to remove deployment");
                }
            }
            break;
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RuntimeConfigResponse {}

pub async fn runtime_config() -> Result<RuntimeConfigResponse, CommonError> {
    Ok(RuntimeConfigResponse {})
}
