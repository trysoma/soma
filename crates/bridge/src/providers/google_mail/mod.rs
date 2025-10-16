use async_trait::async_trait;
use bridge_macros::define_provider;
use schemars::{JsonSchema, SchemaGenerator};
use base64::Engine;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use std::sync::Arc;
use crate::logic::FunctionControllerLike;
use crate::logic::*;
use crate::providers::*;
use crate::oauth::{OauthAuthFlowController, Oauth2JwtBearerAssertionFlowController};

// define_provider!(google_mail {
//     id: "google_mail",
//     name: "Google Mail",
//     docs: "example documentation",
//     flows: [
//         {
//             Oauth2AuthorizationCodeFlow: {
//                 static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
//                     auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
//                     token_uri: "https://oauth2.googleapis.com/token".to_string(),
//                     userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
//                     jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
//                     issuer: "https://accounts.google.com".to_string(),
//                     scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
//                     metadata: Metadata::new(),
//                 }
//             }
//         },
//         {
//             Oauth2JwtBearerAssertionFlow: {
//                 static_credentials: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
//                     auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
//                     token_uri: "https://oauth2.googleapis.com/token".to_string(),
//                     userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
//                     jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
//                     issuer: "https://accounts.google.com".to_string(),
//                     scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
//                     metadata: Metadata::new(),
//                 }
//             }
//         }
//     ],
//     default_scopes: ["https://www.googleapis.com/auth/gmail.readonly"],
//     functions: [
//         FunctionController::GoogleMailSendEmail(GoogleMailFnSendEmailController),
//     ],
// });

// #[derive(Serialize, Deserialize, Clone, JsonSchema)]
// pub struct GoogleMailFnSendEmailParams {
//     pub to: String,
//     pub subject: String,
//     pub body: String,
//     pub cc: Option<String>,
//     pub bcc: Option<String>,
//     // TODO: how to handle blob / non-json data? and keep compatable with MCP protocol....
//     // pub attachments: Option<Vec<Attachment>>,
// }


// #[derive(Serialize, Deserialize, Clone, JsonSchema)]
// pub struct GoogleMailFnSendEmailOutput {
//     pub message_id: String,
// }

// pub struct GoogleMailFnSendEmailController;

// impl FunctionControllerLike for GoogleMailFnSendEmailController {
//     async fn handle(provider_instance: Self::ProviderInstance, params: Self::Params) -> Result<Self::Output, CommonError> {
//         // TODO: think about how to handle this through trait instead of match
//         // Extract access token from the provider instance
//         let access_token = match &provider_instance.0 {
//             GoogleMailVariant::Oauth2AuthorizationCodeFlow(creds) => &creds.user_cred.access_token,
//             GoogleMailVariant::Oauth2JwtBearerAssertionFlow(creds) => &creds.user_cred.token,
//             _ => {
//                 return Err(CommonError::InvalidRequest {
//                     msg: "Unsupported credential type for sending email".to_string(),
//                     source: None,
//                 });
//             }
//         };

//         // Build the email in RFC 2822 format
//         let mut email_content = format!(
//             "From: me\r\nTo: {}\r\nSubject: {}\r\n",
//             params.to, params.subject
//         );

//         if let Some(cc) = &params.cc {
//             email_content.push_str(&format!("Cc: {}\r\n", cc));
//         }

//         if let Some(bcc) = &params.bcc {
//             email_content.push_str(&format!("Bcc: {}\r\n", bcc));
//         }

//         email_content.push_str(&format!("\r\n{}", params.body));

//         // Base64url encode the email (Gmail API requires base64url encoding)
//         let encoded_email = base64::engine::general_purpose::URL_SAFE_NO_PAD
//             .encode(email_content.as_bytes());

//         // Prepare the request body
//         let request_body = serde_json::json!({
//             "raw": encoded_email
//         });

//         // Make the API request using reqwest
//         let client = reqwest::Client::new();
//         let response = client
//             .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
//             .bearer_auth(access_token)
//             .header("Content-Type", "application/json")
//             .json(&request_body)
//             .send()
//             .await?;

//         // Check if the request was successful
//         if !response.status().is_success() {
//             let status = response.status();
//             let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
//             return Err(CommonError::InvalidResponse {
//                 msg: format!("Gmail API returned error {}: {}", status, error_text),
//                 source: None,
//             });
//         }

//         // Parse the response to extract the message ID
//         let response_json: serde_json::Value = response
//             .json()
//             .await?;

//         let message_id = response_json["id"]
//             .as_str()
//             .ok_or_else(|| CommonError::InvalidResponse {
//                 msg: "Gmail API response missing message ID".to_string(),
//                 source: None,
//             })?
//             .to_string();

//         Ok(Self::Output { message_id })
//     }
    
//     type ProviderInstance = GoogleMailInstance;
    
//     type Params = GoogleMailFnSendEmailParams;
    
//     type Output = GoogleMailFnSendEmailOutput;
    
//     fn id() -> String {
//         "google_mail_fn_send_email".to_string()
//     }
    
//     fn name() -> String {
//         "Send Email".to_string()
//     }
    
//     fn documentation() -> String {
//         "Send an email".to_string()
//     }
    
//     fn parameters() -> schemars::Schema {
//         schemars::schema_for!(GoogleMailFnSendEmailParams)
//     }
    
//     fn output() -> schemars::Schema {
//         schemars::schema_for!(GoogleMailFnSendEmailOutput)
//     }
// }

pub struct GoogleMailProviderController;

#[async_trait]
impl ProviderControllerLike for GoogleMailProviderController {
    fn type_id(&self) ->  &'static str {
        "google_mail"
    }
    
    fn documentation(&self) ->  &'static str {
        "Google Mail"
    }
    
    fn name(&self) ->  &'static str {
        "Google Mail"
    }
    
    fn functions(&self) -> Vec<Arc<dyn FunctionControllerLike> >  {
        vec![]
    }
    
    fn credential_controllers(&self) -> Vec<Arc<dyn ProviderCredentialControllerLike> >  {
        vec![
            Arc::new(OauthAuthFlowController),
            Arc::new(Oauth2JwtBearerAssertionFlowController),
        ]
    }
}

