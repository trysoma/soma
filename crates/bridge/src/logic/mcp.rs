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
        ListFunctionInstancesParams, PROVIDER_REGISTRY, ProviderControllerLike,
        WithFunctionInstanceId, invoke_function, list_function_instances,
    },
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
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        // TODO: could use request.unwrap().cursor to get the next page token
        let function_instances = list_function_instances(
            self.repository(),
            ListFunctionInstancesParams {
                pagination: PaginationRequest {
                    page_size: 1000,
                    next_page_token: None,
                },
                provider_instance_id: None,
            },
        )
        .await?;
        let mut tools = function_instances.items;

        let mut next_page_token = function_instances.next_page_token;

        while let Some(token) = next_page_token {
            let function_instances = list_function_instances(
                self.repository(),
                ListFunctionInstancesParams {
                    pagination: PaginationRequest {
                        page_size: 1000,
                        next_page_token: Some(token),
                    },
                    provider_instance_id: None,
                },
            )
            .await?;
            tools.extend(function_instances.items);
            next_page_token = function_instances.next_page_token;
        }

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

        let mcp_tools: Vec<Tool> = tools
            .into_iter()
            .map(|fi| {
                let (provider_controller, function_controller) = function_provider_controllers
                    .iter()
                    .find(|(_, f)| f.type_id() == fi.function_controller_type_id)
                    .ok_or(CommonError::Unknown(anyhow::anyhow!(
                        "Function controller not found: {}",
                        fi.function_controller_type_id
                    )))?;
                Ok(Tool {
                    name: format!(
                        "{}.{}.{}",
                        provider_controller.type_id(),
                        function_controller.type_id(),
                        fi.provider_instance_id
                    )
                    .into(),
                    title: Some(function_controller.name().to_string()),
                    description: Some(function_controller.documentation().to_string().into()),
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
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let id_arr = request.name.split('.').collect::<Vec<&str>>();
        if id_arr.len() != 3 {
            return Err(rmcp::ErrorData::invalid_request("Invalid tool name", None));
        }
        let _provider_controller_type_id = id_arr[0].to_string();
        let function_controller_type_id = id_arr[1].to_string();
        let provider_instance_id = id_arr[2].to_string();

        let function_instance = invoke_function(
            self.repository(),
            self.encryption_service(),
            InvokeFunctionParams {
                provider_instance_id: provider_instance_id.clone(),
                inner: WithFunctionInstanceId {
                    function_controller_type_id: function_controller_type_id.clone(),
                    inner: InvokeFunctionParamsInner {
                        // TODO: we should allow to call invoke with no body params
                        params: WrappedJsonValue::new(serde_json::Value::Object(
                            request.arguments.unwrap(),
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
