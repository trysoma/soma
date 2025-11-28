use bridge::repository::ProviderRepositoryLike;
use encryption::logic::crypto_services::CryptoCache;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::error::CommonError;
use shared::uds::{DEFAULT_SOMA_SERVER_SOCK, create_soma_unix_socket_client};
use tonic::{Request, transport::Channel};
use tracing::{error, info, warn};

use crate::logic::environment_variable_sync::{
    fetch_all_environment_variables, sync_environment_variables_to_sdk,
};
use crate::logic::secret_sync::{fetch_and_decrypt_all_secrets, sync_secrets_to_sdk};
use crate::sdk::sdk_provider_sync;

/// Checks SDK server health via gRPC
pub async fn check_sdk_health(
    sdk_client: &mut SomaSdkServiceClient<Channel>,
) -> Result<(), CommonError> {
    let request = Request::new(());
    match sdk_client.health_check(request).await {
        Ok(_) => {
            info!("SDK server health check passed");
            Ok(())
        }
        Err(e) => {
            warn!("SDK server health check failed: {:?}", e);
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK server health check failed: {e}"
            )))
        }
    }
}

/// Triggers bridge client generation via gRPC call to SDK server
pub async fn trigger_codegen(
    sdk_client: &mut SomaSdkServiceClient<Channel>,
    bridge_repo: &impl ProviderRepositoryLike,
) -> Result<String, CommonError> {
    crate::logic::bridge::codegen::trigger_bridge_client_generation(sdk_client, bridge_repo)
        .await?;

    Ok("Bridge client generation completed successfully".to_string())
}

/// Result of resync operation
pub struct ResyncResult {
    pub providers_synced: usize,
    pub agents_synced: usize,
    pub secrets_synced: usize,
    pub env_vars_synced: usize,
}

/// Resync SDK: fetches metadata from SDK, syncs providers/agents to bridge registry,
/// registers Restate deployments, and syncs secrets/env vars to SDK
pub async fn resync_sdk(
    repository: &std::sync::Arc<crate::repository::Repository>,
    crypto_cache: &CryptoCache,
    restate_params: &crate::restate::RestateServerParams,
    sdk_port: u16,
) -> Result<ResyncResult, CommonError> {
    let socket_path = DEFAULT_SOMA_SERVER_SOCK;

    info!("Starting SDK resync...");

    // Create SDK client
    let mut client = create_soma_unix_socket_client(socket_path).await?;

    // Fetch metadata from SDK (providers and agents)
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
        if let Err(e) =
            register_agent_deployments(metadata.agents, restate_params, sdk_port).await
        {
            error!("Failed to register agent deployments: {:?}", e);
        }
    }

    // Sync secrets to SDK
    let secrets = fetch_and_decrypt_all_secrets(repository, crypto_cache).await?;
    let secrets_count = secrets.len();
    info!("Syncing {} secrets to SDK", secrets_count);
    if !secrets.is_empty() {
        let mut client = create_soma_unix_socket_client(socket_path).await?;
        sync_secrets_to_sdk(&mut client, secrets).await?;
    }

    // Sync environment variables to SDK
    let env_vars = fetch_all_environment_variables(repository).await?;
    let env_vars_count = env_vars.len();
    info!("Syncing {} environment variables to SDK", env_vars_count);
    if !env_vars.is_empty() {
        let mut client = create_soma_unix_socket_client(socket_path).await?;
        sync_environment_variables_to_sdk(&mut client, env_vars).await?;
    }

    info!(
        "SDK resync complete: {} providers, {} agents, {} secrets, {} env vars",
        providers_synced, agents_synced, secrets_count, env_vars_count
    );

    Ok(ResyncResult {
        providers_synced,
        agents_synced,
        secrets_synced: secrets_count,
        env_vars_synced: env_vars_count,
    })
}

/// Register Restate deployments for all agents
async fn register_agent_deployments(
    agents: Vec<sdk_proto::Agent>,
    restate_params: &crate::restate::RestateServerParams,
    sdk_port: u16,
) -> Result<(), CommonError> {
    use shared::restate;
    use std::collections::HashMap;

    info!(
        "Registering {} agent deployment(s) with Restate",
        agents.len()
    );

    for agent in agents {
        let service_uri = format!("http://127.0.0.1:{sdk_port}");
        let deployment_type = restate::deploy::DeploymentType::Http {
            uri: service_uri.clone(),
            additional_headers: HashMap::new(),
        };

        // Use the project_id.agent_id format as the service path (matches Restate service name)
        let service_path = format!("{}.{}", agent.project_id, agent.id);

        info!(
            "Registering agent '{}' (project_id={}, agent_id={}) at {} with service path: {}",
            agent.name, agent.project_id, agent.id, service_uri, service_path
        );

        let admin_url = restate_params.get_admin_address()?;
        let config = restate::deploy::DeploymentRegistrationConfig {
            admin_url: admin_url.to_string(),
            service_path: service_path.clone(),
            deployment_type,
            bearer_token: restate_params.get_admin_token(),
            private: restate_params.get_private(),
            insecure: restate_params.get_insecure(),
            force: restate_params.get_force(),
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
    }

    Ok(())
}
