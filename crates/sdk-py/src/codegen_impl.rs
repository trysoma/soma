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
        info!(
            "Python code generator invoked with {} function instances",
            request.function_instances.len()
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

        // Generate Python code
        let python_code = codegen::generate_python_code_from_api_data(&function_instances)
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to generate Python code: {e}"))
            })?;

        // Write to file
        let soma_dir = self.project_dir.join("soma");
        let output_path = soma_dir.join("bridge.py");

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

        std::fs::write(&output_path, python_code).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to write bridge client file: {e}"))
        })?;

        info!("Bridge client written to: {}", output_path.display());

        Ok(core_types::GenerateBridgeClientResponse {
            result: Some(sdk_proto::generate_bridge_client_response::Result::Success(
                sdk_proto::GenerateBridgeClientSuccess {
                    message: format!(
                        "Python bridge client generated successfully at {}",
                        output_path.display()
                    ),
                },
            )),
        })
    }
}
