use std::sync::Arc;

use rmcp::{
    ServerHandler,
    model::{Annotated, Annotations, RawContent, RawTextContent, Tool},
    service::serve_directly_with_ct,
};
use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedJsonValue},
};
use tokio_util::sync::CancellationToken;

use crate::{
    logic::{
        FunctionControllerLike, InvokeFunctionParams, InvokeFunctionParamsInner, InvokeResult,
        McpServiceInstanceExt, PROVIDER_REGISTRY, ProviderControllerLike, WithFunctionInstanceId,
        invoke_function,
    },
    repository::ProviderRepositoryLike,
    router::BridgeService,
};

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
        _request: Option<rmcp::model::PaginatedRequestParam>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        // Extract the MCP server instance ID from extensions
        let ext_data = context
            .extensions
            .get::<McpServiceInstanceExt>()
            .ok_or_else(|| {
                rmcp::ErrorData::internal_error(
                    "MCP server instance ID not found in request context",
                    None,
                )
            })?;
        let mcp_server_instance_id = ext_data.mcp_server_instance_id.clone();

        // Fetch all functions for this MCP server instance with pagination
        let mut all_functions = Vec::new();
        let mut next_page_token: Option<String> = None;

        loop {
            let pagination = PaginationRequest {
                page_size: 1000,
                next_page_token: next_page_token.clone(),
            };

            let result = self
                .repository()
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
            next_cursor: None,
            tools: mcp_tools,
        })
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        // Extract the MCP server instance ID from extensions
        let ext_data = context
            .extensions
            .get::<McpServiceInstanceExt>()
            .ok_or_else(|| {
                rmcp::ErrorData::internal_error(
                    "MCP server instance ID not found in request context",
                    None,
                )
            })?;
        let mcp_server_instance_id = ext_data.mcp_server_instance_id.clone();

        // The tool name is now the function_name from mcp_server_instance_function
        let function_name = request.name.to_string();

        // Look up the function by mcp_server_instance_id and function_name
        let mcp_function = self
            .repository()
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
        let function_instance = invoke_function(
            self.repository(),
            self.encryption_service(),
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
        .inspect_err(|e| tracing::error!("Error invoking function: {:?}", e));

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
            instructions: Some("This is the Bridge Service".to_string()),
            ..Default::default()
        }
    }
}
