use std::sync::Arc;

use rmcp::{ServerHandler, service::serve_directly_with_ct};
use shared::error::CommonError;
use tokio_util::sync::CancellationToken;

use crate::router::bridge::BridgeService;

pub async fn handle_mcp_transport<R, T, E, A>(
    maybe_transport: Option<T>,
    mcp_server_instance: &BridgeService,
    mcp_ct: &CancellationToken,
) -> Result<bool, CommonError>
where
    R: rmcp::service::ServiceRole,
    BridgeService: rmcp::Service<R>,
    T: rmcp::transport::IntoTransport<R, E, A>,
    E: std::error::Error + Send + Sync + 'static,
{
    match maybe_transport {
        Some(transport) => {
            let service = mcp_server_instance.clone();
            let ct = mcp_ct.child_token();

            tokio::spawn(async move {
                let server = serve_directly_with_ct(service, transport, None, ct);
                if let Err(e) = server.waiting().await {
                    tracing::error!("MCP transport handler exited with error: {:?}", e);
                }
            });

            Ok(false) // continue loop
        }
        None => {
            tracing::info!("MCP transport channel closed — no more transports to serve.");
            Ok(true) // break loop
        }
    }
}

impl ServerHandler for BridgeService {
    async fn list_tools(
        &self,
        request: Option<rmcp::model::PaginatedRequestParam>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListToolsResult {
            next_cursor: None,
            tools: vec![],
        })
    }

    
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            capabilities: rmcp::model::ServerCapabilities::builder().enable_tools().build(),
            instructions: Some("This is the Bridge Service".to_string()),
            ..Default::default()

        }
    }
}
