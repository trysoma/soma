use shared::error::CommonError;
use std::path::PathBuf;
use tracing::info;

use sdk_core as core_types;
use crate::codegen;

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
    async fn generate_bridge_client(
        &self,
        request: core_types::GenerateBridgeClientRequest,
    ) -> Result<core_types::GenerateBridgeClientResponse, CommonError> {
        info!("TypeScript code generator invoked with {} function instances", request.function_instances.len());

        // Convert proto function instances to codegen types
        let function_instances: Vec<codegen::FunctionInstanceData> = request
            .function_instances
            .iter()
            .map(|fi| {
                codegen::FunctionInstanceData {
                    provider_instance_id: fi.provider_instance_id.clone(),
                    provider_instance_display_name: fi.provider_instance_display_name.clone(),
                    provider_controller: codegen::ProviderControllerData {
                        type_id: fi.provider_controller.as_ref().map(|pc| pc.type_id.clone()).unwrap_or_default(),
                        display_name: fi.provider_controller.as_ref().map(|pc| pc.display_name.clone()).unwrap_or_default(),
                    },
                    function_controller: codegen::FunctionControllerData {
                        type_id: fi.function_controller.as_ref().map(|fc| fc.type_id.clone()).unwrap_or_default(),
                        display_name: fi.function_controller.as_ref().map(|fc| fc.display_name.clone()).unwrap_or_default(),
                        params_json_schema: fi.function_controller.as_ref()
                            .and_then(|fc| serde_json::from_str(&fc.params_json_schema).ok()),
                        return_value_json_schema: fi.function_controller.as_ref()
                            .and_then(|fc| serde_json::from_str(&fc.return_value_json_schema).ok()),
                    },
                }
            })
            .collect();

        // Generate TypeScript code
        let typescript_code = codegen::generate_typescript_code_from_api_data(&function_instances)
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to generate TypeScript code: {e}")))?;

        // Write to file
        let soma_dir = self.project_dir.join(".soma");
        let output_path = soma_dir.join("bridge.ts");

        std::fs::create_dir_all(&soma_dir)
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create .soma directory: {e}")))?;

        std::fs::write(&output_path, typescript_code)
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to write bridge client file: {e}")))?;

        info!("Bridge client written to: {}", output_path.display());

        Ok(core_types::GenerateBridgeClientResponse {
            result: Some(sdk_proto::generate_bridge_client_response::Result::Success(
                sdk_proto::GenerateBridgeClientSuccess {
                    message: format!("TypeScript bridge client generated successfully at {}", output_path.display()),
                }
            ))
        })
    }
}
