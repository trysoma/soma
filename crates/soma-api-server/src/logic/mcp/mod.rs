use std::sync::Arc;

use mcp::logic::{OnConfigChangeEvt, OnConfigChangeRx};
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::{debug, info, warn};

pub mod codegen;
pub mod providers;

/// Runs the mcp client generation loop - listens for mcp config changes and triggers codegen.
/// This function runs indefinitely until aborted by the process manager.
pub async fn run_mcp_client_generation_loop(
    mcp_repo: mcp::repository::Repository,
    sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    agent_cache: crate::sdk::sdk_agent_sync::AgentCache,
    mut on_mcp_config_change_rx: OnConfigChangeRx,
) {
    debug!("MCP client generation loop started");

    loop {
        match on_mcp_config_change_rx.recv().await {
            Ok(event) => {
                // Check if this event should trigger mcp client generation
                let should_trigger = matches!(
                    event,
                    OnConfigChangeEvt::FunctionInstanceAdded(_)
                        | OnConfigChangeEvt::FunctionInstanceRemoved(_, _, _)
                        | OnConfigChangeEvt::ProviderInstanceAdded(_)
                        | OnConfigChangeEvt::ProviderInstanceRemoved(_)
                );

                if should_trigger {
                    debug!("MCP change event detected, triggering mcp client generation");

                    // Get the SDK client and verify it's ready
                    let mut client_guard = sdk_client.lock().await;
                    if let Some(ref mut client) = *client_guard {
                        // Verify SDK server is ready by checking health
                        let health_ready = match client.health_check(tonic::Request::new(())).await
                        {
                            Ok(_) => true,
                            Err(e) => {
                                warn!(
                                    "SDK server healthcheck failed, skipping mcp client generation: {:?}",
                                    e
                                );
                                false
                            }
                        };

                        if health_ready {
                            match crate::logic::mcp::codegen::trigger_mcp_client_generation(
                                client,
                                &mcp_repo,
                                &agent_cache,
                            )
                            .await
                            {
                                Ok(()) => {
                                    debug!("MCP client generation completed successfully");
                                }
                                Err(e) => {
                                    warn!("Failed to trigger mcp client generation: {:?}", e);
                                    // Don't return error, just log it - we want to keep listening
                                }
                            }
                        }
                    } else {
                        warn!("SDK client not available, skipping mcp client generation");
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                info!(
                    "MCP config change receiver closed, stopping MCP client generation listener"
                );
                break;
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(
                    "MCP config change receiver lagged, skipped {} messages",
                    skipped
                );
                // Continue listening
            }
        }
    }
}
