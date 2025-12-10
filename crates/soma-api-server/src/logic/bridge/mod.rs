use std::sync::Arc;

use bridge::logic::{OnConfigChangeEvt, OnConfigChangeRx};
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::{error::CommonError, subsystem::SubsystemHandle};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::{info, warn};

pub mod codegen;
pub mod providers;

/// Starts the bridge client generation listener subsystem
pub fn start_bridge_client_generation_subsystem(
    bridge_repo: bridge::repository::Repository,
    sdk_client: Arc<Mutex<Option<SomaSdkServiceClient<Channel>>>>,
    agent_cache: crate::sdk::sdk_agent_sync::AgentCache,
    mut on_bridge_config_change_rx: OnConfigChangeRx,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<SubsystemHandle, CommonError> {
    let (handle, signal) = SubsystemHandle::new("Bridge Client Generation");

    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = on_bridge_config_change_rx.recv() => {
                    match event {
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
                                info!("Bridge change event detected, triggering bridge client generation");

                                // Get the SDK client and verify it's ready
                                let mut client_guard = sdk_client.lock().await;
                                if let Some(ref mut client) = *client_guard {
                                    // Verify SDK server is ready by checking health
                                    let health_ready = match client.health_check(tonic::Request::new(())).await {
                                        Ok(_) => true,
                                        Err(e) => {
                                            warn!("SDK server healthcheck failed, skipping bridge client generation: {:?}", e);
                                            false
                                        }
                                    };

                                    if health_ready {
                                        match crate::logic::bridge::codegen::trigger_bridge_client_generation(client, &bridge_repo, &agent_cache).await {
                                            Ok(()) => {
                                                info!("Bridge client generation completed successfully");
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
                            info!("Bridge config change receiver closed, stopping bridge client generation listener");
                            signal.signal_with_message("stopped gracefully");
                            break;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("Bridge config change receiver lagged, skipped {} messages", skipped);
                            // Continue listening
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping bridge client generation listener");
                    signal.signal_with_message("stopped by shutdown signal");
                    break;
                }
            }
        }
    });

    Ok(handle)
}
