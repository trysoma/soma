use crate::logic::FunctionControllerLike;
use crate::logic::controller::CATEGORY_EMAIL;
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
use crate::logic::*;
use ::encryption::logic::crypto_services::DecryptionService;
use async_trait::async_trait;
use base64::Engine;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{WrappedJsonValue, WrappedSchema};
use std::sync::Arc;
use utoipa::ToSchema;
pub struct GoogleMailProviderController;

#[async_trait]
impl ProviderControllerLike for GoogleMailProviderController {
    fn type_id(&self) -> String {
        "google_mail".to_string()
    }

    fn documentation(&self) -> String {
        "lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 2
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 3
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 4
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 5
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
".to_string()
    }

    fn name(&self) -> String {
        "Google Mail".to_string()
    }

    fn categories(&self) -> Vec<String> {
        vec![CATEGORY_EMAIL.to_string()]
    }

    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike>> {
        vec![Arc::new(SendEmailFunctionController)]
    }

    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike>> {
        vec![
            Arc::new(OauthAuthFlowController {
                static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                    auth_uri: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                    token_uri: "https://oauth2.googleapis.com/token".to_string(),
                    userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
                    jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".to_string(),
                    issuer: "https://accounts.google.com".to_string(),
                    scopes: vec![
                        "https://www.googleapis.com/auth/gmail.send".to_string(),
                        "https://www.googleapis.com/auth/gmail.compose".to_string(),
                        "https://www.googleapis.com/auth/gmail.readonly".to_string(),
                        "https://www.googleapis.com/auth/userinfo.email".to_string(),
                        "https://www.googleapis.com/auth/userinfo.profile".to_string(),
                        "openid".to_string(),
                    ],
                    metadata: Metadata::new(),
                },
            }),
            Arc::new(Oauth2JwtBearerAssertionFlowController {
                static_credentials: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                    auth_uri: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                    token_uri: "https://oauth2.googleapis.com/token".to_string(),
                    userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
                    jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".to_string(),
                    issuer: "https://accounts.google.com".to_string(),
                    scopes: vec![
                        "https://www.googleapis.com/auth/gmail.send".to_string(),
                        "https://www.googleapis.com/auth/gmail.compose".to_string(),
                        "https://www.googleapis.com/auth/gmail.readonly".to_string(),
                    ],
                    metadata: Metadata::new(),
                },
            }),
        ]
    }

    fn metadata(&self) -> Metadata {
        Metadata::new()
    }
}

struct SendEmailFunctionController;

#[derive(Serialize, Deserialize, ToSchema, Clone, JsonSchema)]
struct SendEmailFunctionParameters {
    to: String,
    subject: String,
    body: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, JsonSchema)]
struct SendEmailFunctionOutput {
    message_id: String,
}

#[async_trait]
impl FunctionControllerLike for SendEmailFunctionController {
    fn type_id(&self) -> String {
        "google_mail_send_email".to_string()
    }
    fn name(&self) -> String {
        "Send an email".to_string()
    }
    fn documentation(&self) -> String {
        "# Send an email

lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 2
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 3
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 4
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

# lorem 5
lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

        ".to_string()
    }
    fn parameters(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(SendEmailFunctionParameters))
    }
    fn output(&self) -> WrappedSchema {
        WrappedSchema::new(schema_for!(SendEmailFunctionOutput))
    }
    fn categories(&self) -> Vec<String> {
        vec![CATEGORY_EMAIL.to_string()]
    }

    async fn invoke(
        &self,
        crypto_service: &DecryptionService,
        credential_controller: &Arc<dyn ProviderCredentialControllerLike>,
        _static_credentials: &dyn StaticCredentialConfigurationLike,
        _resource_server_credential: &ResourceServerCredentialSerialized,
        user_credential: &UserCredentialSerialized,
        params: WrappedJsonValue,
    ) -> Result<InvokeResult, CommonError> {
        // Parse the function parameters
        let email_params: SendEmailFunctionParameters = serde_json::from_value(params.into())
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Invalid parameters: {e}")))?;

        // Downcast to OAuth controller and decrypt credentials
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
            controller
                .decrypt_oauth_credentials(crypto_service, user_credential)
                .await?
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
            controller
                .decrypt_oauth_credentials(crypto_service, user_credential)
                .await?
        } else {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Unsupported credential controller type: {cred_controller_type_id}"
            )));
        };

        // Build the email in RFC2822 format
        let email_content = format!(
            "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            email_params.to, email_params.subject, email_params.body
        );

        // Base64url encode the email
        let encoded_email =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(email_content.as_bytes());

        // Prepare the Gmail API request body
        let request_body = serde_json::json!({
            "raw": encoded_email
        });

        // Make the HTTP request to Gmail API
        let client = reqwest::Client::new();
        let response = client
            .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
            .header(
                "Authorization",
                format!("Bearer {}", credentials.access_token),
            )
            .header("Content-Type", "application/json")
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
        let gmail_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to parse response: {e}")))?;

        // Extract the message ID from the response
        let message_id = gmail_response
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("No message ID in response")))?
            .to_string();

        Ok(InvokeResult::Success(WrappedJsonValue::new(
            serde_json::json!({
                "message_id": message_id
            }),
        )))
    }
}
