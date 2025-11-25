use std::sync::Arc;

use async_trait::async_trait;
use bridge::logic::FunctionControllerLike;
use bridge::logic::InvokeError;
use bridge::logic::InvokeResult;
use bridge::logic::Metadata;
use bridge::logic::ProviderControllerLike;
use bridge::logic::ProviderCredentialControllerLike;
use bridge::logic::ResourceServerCredentialSerialized;
use bridge::logic::StaticCredentialConfigurationLike;
use bridge::logic::StaticProviderCredentialControllerLike;
use bridge::logic::UserCredentialSerialized;
use bridge::logic::api_key::ApiKeyController;
use bridge::logic::no_auth::NoAuthController;
use bridge::logic::no_auth::NoAuthStaticCredentialConfiguration;
use bridge::logic::oauth::Oauth2JwtBearerAssertionFlowController;
use bridge::logic::oauth::OauthAuthFlowController;
use encryption::logic::crypto_services::DecryptionService;
use serde_json::json;
use shared::primitives::WrappedJsonValue;
use shared::primitives::WrappedSchema;

use shared::error::CommonError;
use shared::uds::DEFAULT_SOMA_SERVER_SOCK;
use shared::uds::create_soma_unix_socket_client;

/// Soma provider controller that provides soma-specific functions
pub struct DynamicProviderController {
    type_id: String,
    name: String,
    documentation: String,
    categories: Vec<String>,
    functions: Vec<Arc<dyn FunctionControllerLike>>,
    credential_controllers: Vec<Arc<dyn ProviderCredentialControllerLike>>,
}

pub struct DynamicProviderControllerParams {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub categories: Vec<String>,
    pub functions: Vec<DynamicFunctionControllerParams>,
    // TODO: need to add credential controllers once it works, for now, just NoAuthController
    // credential_controllers: Vec<Arc<dyn ProviderCredentialControllerLike>>,
}

impl DynamicProviderController {
    pub fn new(params: DynamicProviderControllerParams) -> Self {
        Self {
            type_id: params.type_id,
            name: params.name,
            documentation: params.documentation,
            categories: params.categories,
            functions: params
                .functions
                .into_iter()
                .map(|f| {
                    Arc::new(DynamicFunctionController::new(f)) as Arc<dyn FunctionControllerLike>
                })
                .collect(),
            credential_controllers: vec![Arc::new(NoAuthController {
                static_credentials: NoAuthStaticCredentialConfiguration {
                    metadata: Metadata::new(),
                },
            })],
            // credential_controllers: params.credential_controllers,
        }
    }
}

#[async_trait]
impl ProviderControllerLike for DynamicProviderController {
    fn type_id(&self) -> String {
        self.type_id.clone()
    }

    fn documentation(&self) -> String {
        self.documentation.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn categories(&self) -> Vec<String> {
        self.categories.clone()
    }

    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>> {
        self.functions.clone()
    }

    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>> {
        self.credential_controllers.clone()
    }

    fn metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        metadata.0.insert("is_dynamic".to_string(), true.into());
        metadata
    }
}

impl DynamicProviderController {
    #[allow(dead_code)]
    fn functions_mut(&mut self) -> &mut Vec<Arc<dyn FunctionControllerLike>> {
        &mut self.functions
    }
}

/// Function controller for SDK functions
struct DynamicFunctionController {
    provider_type_id: String,
    type_id: String,
    name: String,
    documentation: String,
    parameters: WrappedSchema,
    output: WrappedSchema,
    categories: Vec<String>,
}

pub struct DynamicFunctionControllerParams {
    pub provider_type_id: String,
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub parameters: WrappedSchema,
    pub output: WrappedSchema,
    pub categories: Vec<String>,
}

impl DynamicFunctionController {
    pub fn new(params: DynamicFunctionControllerParams) -> Self {
        Self {
            provider_type_id: params.provider_type_id,
            type_id: params.type_id,
            name: params.name,
            documentation: params.documentation,
            parameters: params.parameters,
            output: params.output,
            categories: params.categories,
        }
    }
}

#[async_trait]
impl FunctionControllerLike for DynamicFunctionController {
    fn type_id(&self) -> String {
        self.type_id.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn documentation(&self) -> String {
        self.documentation.clone()
    }
    fn parameters(&self) -> WrappedSchema {
        self.parameters.clone()
    }
    fn output(&self) -> WrappedSchema {
        self.output.clone()
    }
    fn categories(&self) -> Vec<String> {
        self.categories.clone()
    }

    async fn invoke(
        &self,
        decryption_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_credential: &ResourceServerCredentialSerialized,
        user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<InvokeResult, CommonError> {
        let cred_controller_type_id = credential_controller.type_id();

        let credentials = if cred_controller_type_id == OauthAuthFlowController::static_type_id() {
            let controller = credential_controller
                .as_any()
                .downcast_ref::<OauthAuthFlowController>()
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to downcast to OauthAuthFlowController"
                    ))
                })?;
            let creds = controller
                .decrypt_oauth_credentials(decryption_service, user_credential)
                .await?;

            json!({
                "access_token": creds.access_token
            })
        } else if cred_controller_type_id
            == Oauth2JwtBearerAssertionFlowController::static_type_id()
        {
            let controller = credential_controller
                .as_any()
                .downcast_ref::<Oauth2JwtBearerAssertionFlowController>()
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!(
                        "Failed to downcast to Oauth2JwtBearerAssertionFlowController"
                    ))
                })?;
            let creds = controller
                .decrypt_oauth_credentials(decryption_service, user_credential)
                .await?;

            json!({
                "access_token": creds.access_token
            })
        } else if cred_controller_type_id == ApiKeyController::static_type_id() {
            let controller = credential_controller
                .as_any()
                .downcast_ref::<ApiKeyController>()
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to downcast to ApiKeyController"))
                })?;
            let creds = controller
                .decrypt_api_key_credentials(decryption_service, resource_server_credential)
                .await?;

            json!({
                "api_key": creds.api_key
            })
        } else if cred_controller_type_id == NoAuthController::static_type_id() {
            json!({})
        } else {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Unsupported credential controller type for dynamic function: {cred_controller_type_id}"
            )));
        };

        tracing::info!(
            "Invoking SDK function: provider={}, function={}, credential_type={}",
            self.provider_type_id,
            self.type_id,
            credential_controller.type_id()
        );

        // Create gRPC client
        let mut client = create_soma_unix_socket_client(DEFAULT_SOMA_SERVER_SOCK)
            .await
            .map_err(|e| {
                CommonError::Unknown(anyhow::anyhow!("Failed to connect to SDK server: {e}"))
            })?;

        let credentials_json = serde_json::to_string(&credentials).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to serialize credentials: {e}"))
        })?;
        let parameters_json = serde_json::to_string(params.get_inner()).map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to serialize parameters: {e}"))
        })?;

        tracing::debug!(
            "SDK function call details: credentials={}, parameters={}",
            credentials_json,
            parameters_json
        );

        // Build InvokeFunctionRequest
        let request = tonic::Request::new(sdk_proto::InvokeFunctionRequest {
            provider_controller_type_id: self.provider_type_id.clone(),
            function_controller_type_id: self.type_id.clone(),
            credential_controller_type_id: credential_controller.type_id().to_string(),
            credentials: credentials_json,
            parameters: parameters_json,
        });

        // Call the SDK server
        let response = client.invoke_function(request).await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("gRPC invoke_function failed: {e}"))
        })?;

        let result = response.into_inner();

        // Check the oneof kind field
        match result.kind {
            Some(sdk_proto::invoke_function_response::Kind::Error(error)) => {
                tracing::error!(
                    "SDK function execution error: provider={}, function={}, error={}",
                    self.provider_type_id,
                    self.type_id,
                    error.message
                );
                return Ok(InvokeResult::Error(InvokeError {
                    message: error.message,
                }));
            }
            Some(sdk_proto::invoke_function_response::Kind::Data(data_str)) => {
                tracing::debug!(
                    "SDK function returned data: provider={}, function={}, data_length={}",
                    self.provider_type_id,
                    self.type_id,
                    data_str.len()
                );

                let data_value: serde_json::Value = serde_json::from_str(&data_str)
                    .map_err(|e| {
                        tracing::error!(
                            "Failed to parse SDK function result: provider={}, function={}, error={}, data={}",
                            self.provider_type_id,
                            self.type_id,
                            e,
                            data_str
                        );
                        CommonError::Unknown(anyhow::anyhow!(
                            "Failed to parse result from SDK function '{}': {}",
                            self.type_id,
                            e
                        ))
                    })?;

                Ok(InvokeResult::Success(WrappedJsonValue::new(data_value)))
            }
            None => {
                tracing::error!(
                    "SDK function returned neither data nor error: provider={}, function={}",
                    self.provider_type_id,
                    self.type_id
                );
                Err(CommonError::Unknown(anyhow::anyhow!(
                    "SDK function '{}' returned no data or error",
                    self.type_id
                )))
            }
        }
    }
}

// pub const DYNAMIC_FN_TYPE_ID: &str = "dynamic";

// pub fn register_dynamic_functions(
//     provider_params: DynamicProviderControllerParams,
// ) -> Result<(), CommonError> {
//     let registry = PROVIDER_REGISTRY
//         .get_mut()
//         .map_err(|e| {
//             CommonError::Unknown(anyhow::anyhow!("Failed to get provider registry: {}", e))
//         })?
//         .push(Arc::new(DynamicProviderController::new(provider_params)));

//     Ok(())
// }
