use bridge::repository::ProviderRepositoryLike;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::error::CommonError;
use tonic::{Request, transport::Channel};
use tracing::{info, warn};

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
