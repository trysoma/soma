use bridge::{logic::mcp::handle_mcp_transport, router::BridgeService};
use shared::error::CommonError;
use tokio::sync::mpsc;
use tracing::{error, info};
pub struct StartMcpConnectionManagerParams {
    pub bridge_service: BridgeService,
    pub mcp_transport_rx: mpsc::UnboundedReceiver<rmcp::transport::sse_server::SseServerTransport>,
    pub system_shutdown_signal_rx: tokio::sync::broadcast::Receiver<()>,
}

#[allow(clippy::needless_return)]
pub async fn start_mcp_connection_manager(
    params: StartMcpConnectionManagerParams,
) -> Result<(), CommonError> {
    let StartMcpConnectionManagerParams {
        bridge_service,
        mut mcp_transport_rx,
        mut system_shutdown_signal_rx,
    } = params;

    let mcp_ct = tokio_util::sync::CancellationToken::new();

    let mcp_ct_clone = mcp_ct.clone();

    tokio::spawn(async move {
        loop {
            let maybe_incoming_mcp_transport = mcp_transport_rx.recv().await;

            match handle_mcp_transport(maybe_incoming_mcp_transport, &bridge_service, &mcp_ct_clone)
                .await
            {
                Ok(should_break) => {
                    if should_break {
                        break;
                    }
                }
                Err(e) => {
                    error!("MCP transport processor error: {:?}", e);
                    break;
                }
            }
        }

        mcp_ct_clone.cancel();
    });

    tokio::select! {
        _ = system_shutdown_signal_rx.recv() => {
            info!("MCP transport processor shutdown requested");
            mcp_ct.cancel();
            return Ok(());
        }
        _ = mcp_ct.cancelled() => {
            info!("MCP transport processor loop broke unexpectedly");
            return Err(CommonError::Unknown(anyhow::anyhow!("MCP transport processor loop broke unexpectedly")));
        }
    }
}

// tokio::spawn(async move {
//     let mut mcp_transport_rx_local = mcp_transport_rx;
//     loop {
//       tokio::select! {
//         _ = mcp_system_shutdown_signal_rx.recv() => {
//           info!("MCP transport processor shutdown requested");
//           mcp_ct.cancel();
//           break;
//         }
//         maybe_transport = mcp_transport_rx_local.recv() => {
//           match handle_mcp_transport(maybe_transport, &bridge_service_clone, &mcp_ct).await {
//             Ok(should_break) => {
//               if should_break {
//                 break;
//               }
//             }
//             Err(e) => {
//               error!("MCP transport processor error: {:?}", e);
//               break;
//             }
//           }
//         }
//       }
//     }
//     mcp_ct.cancel();
//     let _ = mcp_shutdown_complete_signal_trigger.send(());
//   });
