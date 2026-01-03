use std::sync::Arc;

use axum::extract::OriginalUri;
use encryption::logic::CryptoCache;
use http::request::Parts;
use rmcp::{
    ServerHandler,
    model::{Annotated, Annotations, Extensions, RawContent, RawTextContent, Tool},
};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedJsonValue},
};

use crate::{
    logic::{
        FunctionControllerLike, InvokeFunctionParams, InvokeFunctionParamsInner, InvokeResult,
        PROVIDER_REGISTRY, ProviderControllerLike, WithFunctionInstanceId,
        invoke_function_internal,
    },
    repository::{ProviderRepositoryLike, Repository},
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
                .list_mcp_server_instance_functions(&mcp_server_instance_id, &pagination)
                .await?;

            all_functions.extend(result.items);

            match result.next_page_token {
                Some(token) => next_page_token = Some(token),
                None => break,
            }
        }

        // Get function controllers for parameter/output schemas
        let function_provider_controllers = PROVIDER_REGISTRY
            .read()
            .map_err(|_e| CommonError::Unknown(anyhow::anyhow!("Poison error")))?
            .iter()
            .flat_map(|pc| {
                pc.functions()
                    .iter()
                    .map(|f| (pc.clone(), f.clone()))
                    .collect::<Vec<(
                        Arc<dyn ProviderControllerLike>,
                        Arc<dyn FunctionControllerLike>,
                    )>>()
            })
            .collect::<Vec<(
                Arc<dyn ProviderControllerLike>,
                Arc<dyn FunctionControllerLike>,
            )>>();

        // Convert to MCP tools, using function_name as the tool name and function_description as description
        let mcp_tools: Vec<Tool> = all_functions
            .into_iter()
            .map(|mcp_fn| {
                let (_, function_controller) = function_provider_controllers
                    .iter()
                    .find(|(_, f)| f.type_id() == mcp_fn.function_controller_type_id)
                    .ok_or(CommonError::Unknown(anyhow::anyhow!(
                        "Function controller not found: {}",
                        mcp_fn.function_controller_type_id
                    )))?;

                // Use function_name from mcp_server_instance_function as the tool name
                // Use function_description if available, otherwise fall back to controller documentation
                let description = mcp_fn
                    .function_description
                    .unwrap_or_else(|| function_controller.documentation().to_string());

                Ok(Tool {
                    meta: None,
                    name: mcp_fn.function_name.clone().into(),
                    title: Some(function_controller.name().to_string()),
                    description: Some(description.into()),
                    input_schema: match function_controller.parameters().get_inner().as_object() {
                        Some(object) => Arc::new(object.clone()),
                        None => {
                            return Err(rmcp::ErrorData::internal_error(
                                format!(
                                    "Parameters schema is not an object: {:?}",
                                    function_controller.parameters().get_inner()
                                ),
                                None,
                            ));
                        }
                    },
                    output_schema: match function_controller.output().get_inner().as_object() {
                        Some(output) => Some(Arc::new(output.clone())),
                        None => {
                            return Err(rmcp::ErrorData::internal_error(
                                format!(
                                    "Output schema is not an object: {:?}",
                                    function_controller.output().get_inner()
                                ),
                                None,
                            ));
                        }
                    },
                    annotations: Some(rmcp::model::ToolAnnotations::new()),
                    icons: None,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

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

        // The tool name is now the function_name from mcp_server_instance_function
        let function_name = request.name.to_string();

        // Look up the function by mcp_server_instance_id and function_name
        let mcp_function = self
            .repository
            .get_mcp_server_instance_function_by_name(&mcp_server_instance_id, &function_name)
            .await?
            .ok_or_else(|| {
                rmcp::ErrorData::invalid_request(
                    format!(
                        "Function '{function_name}' not found in MCP server instance '{mcp_server_instance_id}'"
                    ),
                    None,
                )
            })?;

        // Now we have the function_controller_type_id and provider_instance_id to invoke the function
        let function_instance = invoke_function_internal(
            &self.repository,
            &self.encryption_service,
            InvokeFunctionParams {
                provider_instance_id: mcp_function.provider_instance_id.clone(),
                inner: WithFunctionInstanceId {
                    function_controller_type_id: mcp_function.function_controller_type_id.clone(),
                    inner: InvokeFunctionParamsInner {
                        params: WrappedJsonValue::new(serde_json::Value::Object(
                            request.arguments.unwrap_or_default(),
                        )),
                    },
                },
            },
        )
        .await
        .inspect_err(|e| tracing::debug!(error = ?e, "Function invocation failed"));

        match function_instance {
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
