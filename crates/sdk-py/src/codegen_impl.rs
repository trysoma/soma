use shared::error::CommonError;
use std::path::PathBuf;
use tracing::info;

use crate::codegen;
use sdk_core as core_types;

/// Python code generator implementation
pub struct PythonCodeGenerator {
    project_dir: PathBuf,
}

impl PythonCodeGenerator {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }
}

#[tonic::async_trait]
impl core_types::SdkCodeGenerator for PythonCodeGenerator {
    async fn generate_bridge_client(
        &self,
        request: core_types::GenerateBridgeClientRequest,
    ) -> Result<core_types::GenerateBridgeClientResponse, CommonError> {
        tracing::trace!(
            functions = request.function_instances.len(),
            agents = request.agents.len(),
            "Generating Python code"
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

        // Create __init__.py if it doesn't exist
        let init_path = soma_dir.join("__init__.py");
        if !init_path.exists() {
            std::fs::write(&init_path, "\"\"\"Soma generated package.\"\"\"\n").map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to create __init__.py: {e}"))
            })?;
        }

        // Generate and write bridge.py
        let python_code = codegen::generate_python_code_from_api_data(&function_instances)
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to generate Python code: {e}"))
            })?;

        let bridge_path = soma_dir.join("bridge.py");
        std::fs::write(&bridge_path, python_code).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to write bridge client file: {e}"))
        })?;
        tracing::debug!(path = %bridge_path.display(), "Bridge client generated");

        // Generate and write agents.py (only if there are agents)
        if !agents.is_empty() {
            let agents_code = codegen::generate_python_agents_code(&agents).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to generate agents code: {e}"))
            })?;

            let agents_path = soma_dir.join("agents.py");
            std::fs::write(&agents_path, agents_code).map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to write agents file: {e}"))
            })?;
            tracing::debug!(path = %agents_path.display(), "Agents client generated");
        }

        Ok(core_types::GenerateBridgeClientResponse {
            result: Some(sdk_proto::generate_bridge_client_response::Result::Success(
                sdk_proto::GenerateBridgeClientSuccess {
                    message: format!(
                        "Python bridge and agents generated successfully at {}",
                        soma_dir.display()
                    ),
                },
            )),
        })
    }
}
