use std::sync::Arc;

use async_trait::async_trait;
use bridge::logic::DecryptionService;
use bridge::logic::FunctionControllerLike;
use bridge::logic::Metadata;
use bridge::logic::PROVIDER_REGISTRY;
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
use schemars::schema_for;
use serde_json::json;
use shared::primitives::WrappedJsonValue;
use shared::primitives::WrappedSchema;

use shared::error::CommonError;

use crate::logic::GetTaskTimelineItemsRequest;
use crate::logic::GetTaskTimelineItemsResponse;
use crate::logic::get_task_timeline_items;
use crate::repository::Repository;

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
    type_id: String,
    name: String,
    documentation: String,
    categories: Vec<String>,
    functions: Vec<DynamicFunctionControllerParams>,
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
            functions: params.functions.into_iter().map(|f| Arc::new(DynamicFunctionController::new(f)) as Arc<dyn FunctionControllerLike>).collect(),
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
}

impl DynamicProviderController {
    fn functions_mut(&mut self) -> &mut Vec<Arc<dyn FunctionControllerLike>> {
        &mut self.functions
    }
}

/// Function controller for getting task timeline items
struct DynamicFunctionController {
    type_id: String,
    name: String,
    documentation: String,
    parameters: WrappedSchema,
    output: WrappedSchema,
    categories: Vec<String>,
}

pub struct DynamicFunctionControllerParams {
    type_id: String,
    name: String,
    documentation: String,
    parameters: WrappedSchema,
    output: WrappedSchema,
    categories: Vec<String>,
}

impl DynamicFunctionController {
    pub fn new(params: DynamicFunctionControllerParams) -> Self {
        Self {
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
        static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        resource_server_credential: &ResourceServerCredentialSerialized,
        user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<WrappedJsonValue, CommonError> {
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
                "Unsupported credential controller type for dynamic function: {}",
                cred_controller_type_id
            )));
        };

        // this would need to be implemented in JS, TS, or Python, etc.
        // Parse the function parameters
        // let params: GetTaskTimelineItemsRequest = serde_json::from_value(params.into())
        //     .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid parameters: {}", e)))?;

        // // Downcast to OAuth controller and decrypt credentials
        // let cred_controller_type_id = credential_controller.type_id();

        // if cred_controller_type_id == NoAuthController::static_type_id() {
        //     let _controller = credential_controller
        //         .as_any()
        //         .downcast_ref::<NoAuthController>()
        //         .ok_or_else(|| {
        //             CommonError::Unknown(anyhow::anyhow!(
        //                 "Failed to downcast to NoAuthController"
        //             ))
        //         })?;

        // }  else {
        //     return Err(CommonError::Unknown(anyhow::anyhow!(
        //         "Unsupported credential controller type: {}",
        //         cred_controller_type_id
        //     )));
        // };

        // let res = get_task_timeline_items(
        //     &self.repository,
        //     params,
        // )
        // .await;

        // Ok(WrappedJsonValue::new(serde_json::json!(res)))
        Ok(WrappedJsonValue::new(json!({})))
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
