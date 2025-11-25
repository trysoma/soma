use bridge::logic::{FunctionInstanceWithMetadata, get_function_instances};
use bridge::repository::ProviderRepositoryLike;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::error::CommonError;
use tonic::transport::Channel;
use tracing::{error, info};

/// Triggers bridge client generation via gRPC call to SDK server
pub async fn trigger_bridge_client_generation(
    sdk_client: &mut SomaSdkServiceClient<Channel>,
    bridge_repo: &impl ProviderRepositoryLike,
) -> Result<(), CommonError> {
    info!("Triggering bridge client generation via SDK server");

    // Get function instances from bridge
    let function_instances = get_function_instances(bridge_repo).await?;

    // Convert to proto messages
    let proto_function_instances: Vec<sdk_proto::FunctionInstanceData> = function_instances
        .iter()
        .map(convert_to_proto_function_instance)
        .collect();

    // Call gRPC method
    let request = tonic::Request::new(sdk_proto::GenerateBridgeClientRequest {
        function_instances: proto_function_instances,
    });

    match sdk_client.generate_bridge_client(request).await {
        Ok(response) => {
            let result = response.into_inner();
            match result.result {
                Some(sdk_proto::generate_bridge_client_response::Result::Success(success)) => {
                    info!("Bridge client generation succeeded: {}", success.message);
                    Ok(())
                }
                Some(sdk_proto::generate_bridge_client_response::Result::Error(error)) => {
                    error!("Bridge client generation failed: {}", error.message);
                    Err(CommonError::Unknown(anyhow::anyhow!(
                        "Bridge client generation failed: {}",
                        error.message
                    )))
                }
                None => {
                    error!("Bridge client generation returned empty response");
                    Err(CommonError::Unknown(anyhow::anyhow!(
                        "Bridge client generation returned empty response"
                    )))
                }
            }
        }
        Err(e) => {
            error!("Failed to call generate_bridge_client gRPC method: {:?}", e);
            Err(CommonError::Unknown(anyhow::anyhow!(
                "Failed to call generate_bridge_client: {e}"
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
