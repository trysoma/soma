use std::sync::Arc;

use bridge::logic::{OnConfigChangeEvt, OnConfigChangeRx};
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::{debug, info, warn};

pub mod codegen;
pub mod providers;

/// Runs the bridge client generation loop - listens for bridge config changes and triggers codegen.
/// This function runs indefinitely until aborted by the process manager.
pub async fn run_bridge_client_generation_loop(
    bridge_repo: bridge::repository::Repository,
    sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    agent_cache: crate::sdk::sdk_agent_sync::AgentCache,
    mut on_bridge_config_change_rx: OnConfigChangeRx,
) {
    debug!("Bridge client generation loop started");

    loop {
        match on_bridge_config_change_rx.recv().await {
            Ok(event) => {
                // Check if this event should trigger bridge client generation
                let should_trigger = matches!(
                    event,
                    OnConfigChangeEvt::FunctionInstanceAdded(_)
                        | OnConfigChangeEvt::FunctionInstanceRemoved(_, _, _)
                        | OnConfigChangeEvt::ProviderInstanceAdded(_)
                        | OnConfigChangeEvt::ProviderInstanceRemoved(_)
                );

                if should_trigger {
                    debug!("Bridge change event detected, triggering bridge client generation");

                    // Get the SDK client and verify it's ready
                    let mut client_guard = sdk_client.lock().await;
                    if let Some(ref mut client) = *client_guard {
                        // Verify SDK server is ready by checking health
                        let health_ready = match client.health_check(tonic::Request::new(())).await
                        {
                            Ok(_) => true,
                            Err(e) => {
                                warn!(
                                    "SDK server healthcheck failed, skipping bridge client generation: {:?}",
                                    e
                                );
                                false
                            }
                        };

                        if health_ready {
                            match crate::logic::bridge::codegen::trigger_bridge_client_generation(
                                client,
                                &bridge_repo,
                                &agent_cache,
                            )
                            .await
                            {
                                Ok(()) => {
                                    debug!("Bridge client generation completed successfully");
                                }
                                Err(e) => {
                                    warn!("Failed to trigger bridge client generation: {:?}", e);
                                    // Don't return error, just log it - we want to keep listening
                                }
                            }
                        }
                    } else {
                        warn!("SDK client not available, skipping bridge client generation");
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                info!(
                    "Bridge config change receiver closed, stopping bridge client generation listener"
                );
                break;
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(
                    "Bridge config change receiver lagged, skipped {} messages",
                    skipped
                );
                // Continue listening
            }
        }
    }
}
