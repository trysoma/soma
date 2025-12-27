use mcp::logic::{FunctionInstanceWithMetadata, get_function_instances_internal};
use mcp::repository::ProviderRepositoryLike;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::error::CommonError;
use tonic::transport::Channel;
use tracing::{debug, error, trace};

use crate::sdk::sdk_agent_sync::{AgentCache, get_all_agents};

/// Triggers mcp client generation via gRPC call to SDK server
pub async fn trigger_mcp_client_generation(
    sdk_client: &mut SomaSdkServiceClient<Channel>,
    mcp_repo: &impl ProviderRepositoryLike,
    agent_cache: &AgentCache,
) -> Result<(), CommonError> {
    trace!("Triggering mcp client generation");

    // Get function instances from mcp (internal version, no auth needed for internal calls)
    let function_instances = get_function_instances_internal(mcp_repo).await?;

    // Convert to proto messages
    let proto_function_instances: Vec<sdk_proto::FunctionInstanceData> = function_instances
        .iter()
        .map(convert_to_proto_function_instance)
        .collect();

    // Get agents from cache and convert to proto
    let agents = get_all_agents(agent_cache);
    let proto_agents: Vec<sdk_proto::Agent> = agents
        .into_iter()
        .map(|agent| sdk_proto::Agent {
            id: agent.id,
            project_id: agent.project_id,
            name: agent.name,
            description: agent.description,
        })
        .collect();

    // Call gRPC method
    let request = tonic::Request::new(sdk_proto::GenerateMcpClientRequest {
        function_instances: proto_function_instances,
        agents: proto_agents,
    });

    match sdk_client.generate_mcp_client(request).await {
        Ok(response) => {
            let result = response.into_inner();
            match result.result {
                Some(sdk_proto::generate_mcp_client_response::Result::Success(_)) => {
                    debug!("MCP client generated");
                    Ok(())
                }
                Some(sdk_proto::generate_mcp_client_response::Result::Error(error)) => {
                    error!(error = %error.message, "MCP client generation failed");
                    Err(CommonError::Unknown(anyhow::anyhow!(
                        "MCP client generation failed: {}",
                        error.message
                    )))
                }
                None => {
                    error!("MCP client generation returned empty response");
                    Err(CommonError::Unknown(anyhow::anyhow!(
                        "MCP client generation returned empty response"
                    )))
                }
            }
        }
        Err(e) => {
            error!(error = ?e, "Failed to call generate_mcp_client gRPC");
            Err(CommonError::Unknown(anyhow::anyhow!(
                "Failed to call generate_mcp_client: {e}"
            )))
        }
    }
}

fn convert_to_proto_function_instance(
    func: &FunctionInstanceWithMetadata,
) -> sdk_proto::FunctionInstanceData {
    let params_schema = func.function_controller.parameters();
    let output_schema = func.function_controller.output();

    sdk_proto::FunctionInstanceData {
        provider_instance_id: func.provider_instance.id.to_string(),
        provider_instance_display_name: func.provider_instance.display_name.clone(),
        provider_controller: Some(sdk_proto::ProviderControllerData {
            type_id: func.provider_controller.type_id(),
            display_name: func.provider_controller.name(),
        }),
        function_controller: Some(sdk_proto::FunctionControllerData {
            type_id: func.function_controller.type_id(),
            display_name: func.function_controller.name(),
            params_json_schema: serde_json::to_string(params_schema.get_inner().as_value())
                .unwrap_or_default(),
            return_value_json_schema: serde_json::to_string(output_schema.get_inner().as_value())
                .unwrap_or_default(),
        }),
    }
}
