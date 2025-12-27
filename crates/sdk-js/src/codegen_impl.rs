use shared::error::CommonError;
use std::path::PathBuf;

use crate::codegen;
use sdk_core as core_types;

/// TypeScript code generator implementation
pub struct TypeScriptCodeGenerator {
    project_dir: PathBuf,
}

impl TypeScriptCodeGenerator {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }
}

#[tonic::async_trait]
impl core_types::SdkCodeGenerator for TypeScriptCodeGenerator {
    async fn generate_mcp_client(
        &self,
        request: core_types::GenerateMcpClientRequest,
    ) -> Result<core_types::GenerateMcpClientResponse, CommonError> {
        tracing::trace!(
            functions = request.function_instances.len(),
            agents = request.agents.len(),
            "Generating TypeScript code"
        );

        // Convert proto function instances to codegen types
        let function_instances: Vec<codegen::FunctionInstanceData> = request
            .function_instances
            .iter()
            .map(|fi| codegen::FunctionInstanceData {
                provider_instance_id: fi.provider_instance_id.clone(),
                provider_instance_display_name: fi.provider_instance_display_name.clone(),
                provider_controller: codegen::ProviderControllerData {
                    type_id: fi
                        .provider_controller
                        .as_ref()
                        .map(|pc| pc.type_id.clone())
                        .unwrap_or_default(),
                    display_name: fi
                        .provider_controller
                        .as_ref()
                        .map(|pc| pc.display_name.clone())
                        .unwrap_or_default(),
                },
                function_controller: codegen::FunctionControllerData {
                    type_id: fi
                        .function_controller
                        .as_ref()
                        .map(|fc| fc.type_id.clone())
                        .unwrap_or_default(),
                    display_name: fi
                        .function_controller
                        .as_ref()
                        .map(|fc| fc.display_name.clone())
                        .unwrap_or_default(),
                    params_json_schema: fi
                        .function_controller
                        .as_ref()
                        .and_then(|fc| serde_json::from_str(&fc.params_json_schema).ok()),
                    return_value_json_schema: fi
                        .function_controller
                        .as_ref()
                        .and_then(|fc| serde_json::from_str(&fc.return_value_json_schema).ok()),
                },
            })
            .collect();

        // Convert proto agents to codegen types
        let agents: Vec<codegen::AgentData> = request
            .agents
            .iter()
            .map(|agent| codegen::AgentData {
                id: agent.id.clone(),
                project_id: agent.project_id.clone(),
                name: agent.name.clone(),
                description: agent.description.clone(),
            })
            .collect();

        // Ensure soma directory exists
        let soma_dir = self.project_dir.join("soma");
        std::fs::create_dir_all(&soma_dir).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to create soma directory: {e}"))
        })?;

        // Generate and write mcp.ts
        let typescript_code = codegen::generate_typescript_code_from_api_data(&function_instances)
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to generate TypeScript code: {e}"))
            })?;

        let mcp_path = soma_dir.join("mcp.ts");
        std::fs::write(&mcp_path, typescript_code).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to write mcp client file: {e}"))
        })?;
        tracing::debug!(path = %mcp_path.display(), "MCP client generated");

        // Generate and write agents.ts (only if there are agents)
        if !agents.is_empty() {
            let agents_code = codegen::generate_typescript_agents_code(&agents).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to generate agents code: {e}"))
            })?;

            let agents_path = soma_dir.join("agents.ts");
            std::fs::write(&agents_path, agents_code).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to write agents file: {e}"))
            })?;
            tracing::debug!(path = %agents_path.display(), "Agents client generated");
        }

        Ok(core_types::GenerateMcpClientResponse {
            result: Some(sdk_proto::generate_mcp_client_response::Result::Success(
                sdk_proto::GenerateMcpClientSuccess {
                    message: format!(
                        "TypeScript mcp and agents generated successfully at {}",
                        soma_dir.display()
                    ),
                },
            )),
        })
    }
}
