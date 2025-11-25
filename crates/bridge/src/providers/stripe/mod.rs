use crate::logic::FunctionControllerLike;
use crate::logic::api_key::{ApiKeyController, ApiKeyStaticCredentialConfiguration};
use crate::logic::credential::{
    ResourceServerCredentialSerialized, StaticCredentialConfigurationLike, UserCredentialSerialized,
};
use crate::logic::*;
use ::encryption::logic::crypto_services::DecryptionService;
use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{WrappedJsonValue, WrappedSchema};
use std::sync::Arc;
use utoipa::ToSchema;
pub struct StripeProviderController;

#[async_trait]
impl ProviderControllerLike for StripeProviderController {
    fn type_id(&self) -> String {
        "stripe".to_string()
    }

    fn documentation(&self) -> String {
        "stripe documentation".to_string()
    }

    fn name(&self) -> String {
        "Stripe".to_string()
    }

    fn categories(&self) -> Vec<String> {
        vec![CATEGORY_PAYMENTS.to_string()]
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

    fn metadata(&self) -> Metadata {
        Metadata::new()
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
    fn type_id(&self) -> String {
        "stripe_process_refund".to_string()
    }
    fn name(&self) -> String {
        "Process a refund".to_string()
    }
    fn documentation(&self) -> String {
        "# Process a refund
"
        .to_string()
    }
    fn parameters(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(ProcessRefundFunctionParameters))
    }
    fn output(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(ProcessRefundFunctionOutput))
    }
    fn categories(&self) -> Vec<String> {
        vec![CATEGORY_PAYMENTS.to_string()]
    }

    async fn invoke(
        &self,
        crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        resource_server_credential: &ResourceServerCredentialSerialized,
        _user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<InvokeResult, CommonError> {
        // Parse the function parameters
        let refund_params: ProcessRefundFunctionParameters = serde_json::from_value(params.into())
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid parameters: {e}")))?;

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
                "Unsupported credential controller type: {cred_controller_type_id}"
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
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("HTTP request failed: {e}")))?;

        // Check if the request was successful
        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Ok(InvokeResult::Error(InvokeError {
                message: error_text,
            }));
        }

        // Parse the response
        let stripe_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse response: {e}")))?;

        Ok(InvokeResult::Success(WrappedJsonValue::new(
            stripe_response,
        )))
    }
}
