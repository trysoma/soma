use axum::extract::OriginalUri;
use encryption::logic::CryptoCache;
use http::request::Parts;
use rmcp::{
    ServerHandler,
    model::{Annotated, Annotations, Extensions, RawContent, RawTextContent, Tool},
};
use shared::primitives::{PaginationRequest, WrappedJsonValue};

use crate::{
    logic::{
        InvokeToolParams, InvokeToolParamsInner, InvokeResult,
        WithToolInstanceId, invoke_tool_internal,
    },
    repository::{Repository, ProviderRepositoryLike},
};

/// Extracts the mcp_server_instance_id from the HTTP request path.
/// Expected path format: /api/mcp/v1/mcp-server/{mcp_server_instance_id}/mcp
fn extract_mcp_server_instance_id_from_path(path: &str) -> Option<String> {
    // Path format: /api/mcp/v1/mcp-server/{mcp_server_instance_id}/mcp
    let segments: Vec<&str> = path.split('/').collect();
    // Find "mcp-server" and get the next segment
    for (i, part) in segments.iter().enumerate() {
        if *part == "mcp-server" && i + 1 < segments.len() {
            let instance_id = segments[i + 1];
            // Make sure it's not "mcp" (the endpoint suffix)
            if !instance_id.is_empty() && instance_id != "mcp" {
                return Some(instance_id.to_string());
            }
        }
    }
    None
}

/// Extracts the mcp_server_instance_id from the request context.
/// The rmcp library stores http::request::Parts in the rmcp Extensions.
/// First tries to use OriginalUri from Parts.extensions (preserves full path before nest_service strips it),
/// then falls back to Parts.uri.
fn extract_mcp_server_instance_id(extensions: &Extensions) -> Result<String, rmcp::ErrorData> {
    // Get Parts from rmcp extensions (injected by rmcp StreamableHttpService)
    if let Some(parts) = extensions.get::<Parts>() {
        // First try OriginalUri from the http request extensions (stored by our middleware)
        if let Some(original_uri) = parts.extensions.get::<OriginalUri>() {
            if let Some(id) = extract_mcp_server_instance_id_from_path(original_uri.path()) {
                return Ok(id);
            }
        }

        // Fall back to Parts.uri (will be stripped by nest_service, but try anyway)
        if let Some(id) = extract_mcp_server_instance_id_from_path(parts.uri.path()) {
            return Ok(id);
        }
    }

    Err(rmcp::ErrorData::internal_error(
        "Could not extract MCP server instance ID from request path",
        None,
    ))
}

pub struct McpServerService {
    pub repository: Repository,
    pub encryption_service: CryptoCache,
}

impl ServerHandler for McpServerService {
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        // Extract mcp_server_instance_id from the request context
        let mcp_server_instance_id = extract_mcp_server_instance_id(&context.extensions)?;

        // Fetch all functions for this MCP server instance with pagination
        let mut all_functions = Vec::new();
        let mut next_page_token: Option<String> = None;

        loop {
            let pagination = PaginationRequest {
                page_size: 1000,
                next_page_token: next_page_token.clone(),
            };

            let result = self
                .repository
                .list_mcp_server_instance_tools(&mcp_server_instance_id, &pagination)
                .await?;

            all_functions.extend(result.items);

            match result.next_page_token {
                Some(token) => next_page_token = Some(token),
                None => break,
            }
        }

        // TODO: Refactor to fetch tool schemas from repository instead of hardcoded sources
        // The old implementation used list_all_tool_group_sources() which has been removed.
        // Tool schemas should be stored in the database and fetched from there.
        // For now, return empty tools list.
        //
        // Required changes:
        // 1. Store tool parameter/output schemas in database when tools are registered
        // 2. Fetch schemas from repository using tool_deployment_type_id
        // 3. Use stored schemas to construct Tool objects

        let mcp_tools: Vec<Tool> = vec![];

        Ok(rmcp::model::ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: mcp_tools,
        })
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        // Extract mcp_server_instance_id from the request context
        let mcp_server_instance_id = extract_mcp_server_instance_id(&context.extensions)?;

        // The tool name is now the tool_name from mcp_server_instance_function
        let tool_name = request.name.to_string();

        // Look up the function by mcp_server_instance_id and tool_name
        let mcp_function = self
            .repository
            .get_mcp_server_instance_tool_by_name(&mcp_server_instance_id, &tool_name)
            .await?
            .ok_or_else(|| {
                rmcp::ErrorData::invalid_request(
                    format!(
                        "Function '{tool_name}' not found in MCP server instance '{mcp_server_instance_id}'"
                    ),
                    None,
                )
            })?;

        // Now we have the tool_deployment_type_id and tool_group_instance_id to invoke the tool
        let tool_instance = invoke_tool_internal(
            &self.repository,
            &self.encryption_service,
            InvokeToolParams {
                tool_group_instance_id: mcp_function.tool_group_instance_id.clone(),
                inner: WithToolInstanceId {
                    tool_deployment_type_id: mcp_function.tool_deployment_type_id.clone(),
                    inner: InvokeToolParamsInner {
                        params: WrappedJsonValue::new(serde_json::Value::Object(
                            request.arguments.unwrap_or_default(),
                        )),
                    },
                },
            },
        )
        .await
        .inspect_err(|e| tracing::debug!(error = ?e, "Function invocation failed"));

        match tool_instance {
            Ok(invoke_response) => match invoke_response {
                InvokeResult::Success(response) => Ok(rmcp::model::CallToolResult {
                    content: vec![],
                    structured_content: Some(response.into_inner()),
                    is_error: None,
                    meta: None,
                }),
                InvokeResult::Error(error) => {
                    Ok(rmcp::model::CallToolResult::error(vec![Annotated::new(
                        RawContent::Text(RawTextContent {
                            text: error.message,
                            meta: None,
                        }),
                        Some(Annotations::default()),
                    )]))
                }
            },
            Err(e) => Ok(rmcp::model::CallToolResult::error(vec![Annotated::new(
                RawContent::Text(RawTextContent {
                    text: e.to_string(),
                    meta: None,
                }),
                Some(Annotations::default()),
            )])),
        }
    }

    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            capabilities: rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("This is the MCP Service".to_string()),
            ..Default::default()
        }
    }
}
