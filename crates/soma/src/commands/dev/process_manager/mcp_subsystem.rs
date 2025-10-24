use bridge::logic::mcp::handle_mcp_transport;
use bridge::router::bridge::BridgeService;
use tokio::sync::mpsc;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle};
use tracing::info;

use shared::error::CommonError;

use crate::router;

pub struct StartMcpTransportProcessorParams {
    pub bridge_service: BridgeService,
    pub mcp_transport_rx: mpsc::UnboundedReceiver<rmcp::transport::sse_server::SseServerTransport>,
}

/// Starts the MCP transport processor subsystem
pub fn start_mcp_transport_processor_subsystem(
    subsys: &SubsystemHandle,
    params: StartMcpTransportProcessorParams,
) {
    let StartMcpTransportProcessorParams {
        bridge_service,
        mut mcp_transport_rx,
    } = params;

    let mcp_ct = tokio_util::sync::CancellationToken::new();

    subsys.start(SubsystemBuilder::new(
        "mcp-transport-processor",
        move |subsys: SubsystemHandle| {
            async move {
                loop {
                    tokio::select! {
                        _ = subsys.on_shutdown_requested() => {
                            info!("mcp-server subsystem shutdown requested.");
                            mcp_ct.cancel();
                            break;
                        }
                        maybe_transport = mcp_transport_rx.recv() => {
                            handle_mcp_transport(maybe_transport, &bridge_service, &mcp_ct).await?;
                        }
                    }
                }

                // Ensure any in-flight sessions are asked to shut down.
                mcp_ct.cancel();

                Ok::<(), CommonError>(())
            }
        },
    ));
}
