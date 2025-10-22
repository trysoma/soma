use crate::logic::FunctionControllerLike;
use crate::logic::api_key::{ApiKeyController, ApiKeyStaticCredentialConfiguration};
use crate::logic::controller::{CATEGORY_EMAIL, PROVIDER_REGISTRY};
use crate::logic::credential::oauth::{
    Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
    Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
};
use crate::logic::credential::oauth::{
    Oauth2JwtBearerAssertionFlowController, OauthAuthFlowController,
};
use crate::logic::credential::{
    ResourceServerCredentialSerialized, StaticCredentialConfigurationLike, UserCredentialSerialized,
};
use crate::logic::encryption::DecryptionService;
use crate::logic::*;
use crate::providers::*;
use async_trait::async_trait;
use base64::Engine;
use bridge_macros::define_provider;
use schemars::{JsonSchema, SchemaGenerator, schema_for};
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{WrappedJsonValue, WrappedSchema};
use std::sync::Arc;
use utoipa::ToSchema;
pub struct StripeProviderController;

#[async_trait]
impl ProviderControllerLike for StripeProviderController {
    fn type_id(&self) -> &'static str {
        "stripe"
    }

    fn documentation(&self) -> &'static str {
        "stripe documentation"
    }

    fn name(&self) -> &'static str {
        "Stripe"
    }

    fn categories(&self) -> Vec<&'static str> {
        vec![CATEGORY_PAYMENTS]
    }

    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>> {
        vec![Arc::new(ProcessRefundFunctionController)]
    }

    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>> {
        vec![Arc::new(ApiKeyController {
            static_credentials: ApiKeyStaticCredentialConfiguration {
                metadata: Metadata::default(),
            },
        })]
    }
}

struct ProcessRefundFunctionController;

#[derive(Serialize, Deserialize, ToSchema, Clone, JsonSchema)]
struct ProcessRefundFunctionParameters {
    refund_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, JsonSchema)]
struct ProcessRefundFunctionOutput {
    success: bool,
}

#[async_trait]
impl FunctionControllerLike for ProcessRefundFunctionController {
    fn type_id(&self) -> &'static str {
        "stripe_process_refund"
    }
    fn name(&self) -> &'static str {
        "Process a refund"
    }
    fn documentation(&self) -> &'static str {
        "# Process a refund
"
    }
    fn parameters(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(ProcessRefundFunctionParameters).into())
    }
    fn output(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(ProcessRefundFunctionOutput).into())
    }
    fn categories(&self) -> Vec<&'static str> {
        vec![CATEGORY_PAYMENTS]
    }

    async fn invoke(
        &self,
        crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        _static_credentials: &Box<dyn StaticCredentialConfigurationLike>,
        resource_server_credential: &ResourceServerCredentialSerialized,
        _user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<WrappedJsonValue, CommonError> {
        // Parse the function parameters
        let refund_params: ProcessRefundFunctionParameters = serde_json::from_value(params.into())
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid parameters: {}", e)))?;

        // Downcast to OAuth controller and decrypt credentials
        let cred_controller_type_id = credential_controller.type_id();

        let credentials = if cred_controller_type_id == ApiKeyController::static_type_id() {
            let controller = credential_controller
                .as_any()
                .downcast_ref::<ApiKeyController>()
                .ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to downcast to ApiKeyController"))
                })?;
            controller
                .decrypt_api_key_credentials(crypto_service, resource_server_credential)
                .await?
        } else {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Unsupported credential controller type: {}",
                cred_controller_type_id
            )));
        };

        // Make the HTTP request to Stripe API
        let client = reqwest::Client::new();
        let request_body = serde_json::json!({
            "refund_id": refund_params.refund_id
        });
        let response = client
            .post("https://api.stripe.com/v1/refunds")
            .header("Authorization", format!("Bearer {}", credentials.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("HTTP request failed: {}", e)))?;

        // Check if the request was successful
        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Stripe API error: {}",
                error_text
            )));
        }

        // Parse the response
        let stripe_response: serde_json::Value = response.json().await.map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to parse response: {}", e))
        })?;

        Ok(WrappedJsonValue::new(stripe_response))
    }
}
