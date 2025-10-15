// use enum_dispatch::enum_dispatch;
// use reqwest::Request;
// use serde::{Deserialize, Serialize};
// use shared::{error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4}};

// use crate::logic::{CredentialInjectorLike, DatabaseCredential, Metadata, Oauth2AuthorizationCodeFlowResourceServerCredential, Oauth2AuthorizationCodeFlowUserCredential, Oauth2JwtBearerAssertionFlowUserCredential, Oauth2StaticCredentialConfiguration, ProviderControllerLike, ResourceServerCredential, ResourceServerCredentialVariant, UserCredential, UserCredentialVariant};

// pub struct GoogleMailController;

// impl ProviderControllerLike for GoogleMailController {
//     type ProviderInstance = GoogleMailInstance;
    
//     async fn save_resource_server_credential(input: ResourceServerCredentialVariant) -> Result<ResourceServerCredential, shared::error::CommonError> {
//         let input = match input.clone() {
//             ResourceServerCredentialVariant::NoAuth => return Err(CommonError::InvalidRequest { msg: "No auth is not supported for Google Mail".to_string(), source: None }),
//             ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(credentials) => input,
//             ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow(credentials) => input,
//             ResourceServerCredentialVariant::Custom(credentials) => return Err(CommonError::InvalidRequest { msg: "Custom resource server credentials are not supported for Google Mail".to_string(), source: None }),
//         };

//         // TODO: save the resource server credential to the database

//         Ok(ResourceServerCredential {
//             id: WrappedUuidV4::new(),
//             created_at: WrappedChronoDateTime::now(),
//             updated_at: WrappedChronoDateTime::now(),
//             inner: input,
//             metadata: Metadata::new(),
//         })
//     }
    
//     async fn save_user_credential(input: UserCredentialVariant) -> Result<UserCredential, shared::error::CommonError> {
//         let input = match input.clone() {
//             UserCredentialVariant::NoAuth => return Err(CommonError::InvalidRequest { msg: "No auth is not supported for Google Mail".to_string(), source: None }),
//             UserCredentialVariant::Oauth2AuthorizationCodeFlow(credentials) => input,
//             UserCredentialVariant::Oauth2JwtBearerAssertionFlow(credentials) => input,
//             UserCredentialVariant::Custom(credentials) => return Err(CommonError::InvalidRequest { msg: "Custom user credentials are not supported for Google Mail".to_string(), source: None }),
//         };

//         // TODO: save the user credential to the database

//         Ok(UserCredential {
//             id: WrappedUuidV4::new(),
//             created_at: WrappedChronoDateTime::now(),
//             updated_at: WrappedChronoDateTime::now(),
//             inner: input,
//             metadata: Metadata::new(),
//         })
//     }

//     fn id() -> String {
//         "google_mail".to_string()
//     }

//     fn documentation_url() -> String {
//         "https://developers.google.com/gmail/api/guides/concepts".to_string()
//     }

//     fn name() -> String {
//         "Google Mail".to_string()
//     }
// }


// fn default_static_credentials() -> Oauth2StaticCredentialConfiguration {
//     Oauth2StaticCredentialConfiguration {
//         auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
//         token_uri: "https://oauth2.googleapis.com/token".to_string(),
//         userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
//         jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
//         issuer: "https://accounts.google.com".to_string(),
//         scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
//         metadata: Metadata::new(),
//     }
// }

// // a struct instance represnts an instance of a provider persisted in the database


// #[derive(Serialize, Deserialize)]
// #[serde(transparent)]
// pub struct GoogleMailInstance(GoogleMailVariant);

// #[enum_dispatch(CredentialInjectorLike)]
// #[derive(Serialize, Deserialize)]
// #[serde(tag = "type")]
// pub enum GoogleMailVariant{
//     Oauth2AuthorizationCodeFlow(GoogleMailOauth2AuthorizationCodeFlowInstance),
//     Oauth2JwtBearerAssertionFlow(GoogleMailOauth2JwtBearerAssertionFlowInstance),
// }

// #[derive(Serialize, Deserialize)]
// pub struct GoogleMailOauth2AuthorizationCodeFlowInstance {
//     static_credentials: Oauth2StaticCredentialConfiguration,
//     resource_server_credentials: Oauth2AuthorizationCodeFlowResourceServerCredential,
//     user_credentials: Oauth2AuthorizationCodeFlowUserCredential,
// }

// impl CredentialInjectorLike for GoogleMailOauth2AuthorizationCodeFlowInstance {
//     fn inject_credentials(&self, request: &mut Request) {
//     }
// }


// #[derive(Serialize, Deserialize)]
// pub struct GoogleMailOauth2JwtBearerAssertionFlowInstance {
//     static_credentials: Oauth2StaticCredentialConfiguration,
//     resource_server_credentials: Oauth2JwtBearerAssertionFlowUserCredential,
//     user_credentials: Oauth2JwtBearerAssertionFlowUserCredential,
// }

// impl CredentialInjectorLike for GoogleMailOauth2JwtBearerAssertionFlowInstance {
//     fn inject_credentials(&self, request: &mut Request) {
//     }
// }

// use crate::define_provider;


// define_provider!(google_mail {
//     id: "google_mail",
//     name: "Google Mail",
//     docs: "https://developers.google.com/gmail/api/guides/concepts",
//     flows: [{
//         Oauth2AuthorizationCodeFlow: {
//             static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
//                 auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
//                 token_uri: "https://oauth2.googleapis.com/token".to_string(),
//                 userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
//                 jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
//                 issuer: "https://accounts.google.com".to_string(),
//             },
//             ..potentially_other_macro keys
//         } 
//         Oauth2JwtBearerAssertionFlow: {
//             static_credentials: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
//                 auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
//                 token_uri: "https://oauth2.googleapis.com/token".to_string(),
//                 userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
//                 jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
//                 issuer: "https://accounts.google.com".to_string(),
//             }
//             ..potentially_other_macro keys
//         }
//     }],
//     default_scopes: ["https://www.googleapis.com/auth/gmail.readonly"]
// });

use bridge_macros::define_provider;

define_provider!(google_mail {
    id: "google_mail",
    name: "Google Mail",
    docs: "https://developers.google.com/gmail/api/guides/concepts",
    flows: [
        {
            Oauth2AuthorizationCodeFlow: {
                static_credentials: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                    auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
                    token_uri: "https://oauth2.googleapis.com/token".to_string(),
                    userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
                    jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
                    issuer: "https://accounts.google.com".to_string(),
                    scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
                    metadata: Metadata::new(),
                }
            }
        },
        {
            Oauth2JwtBearerAssertionFlow: {
                static_credentials: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                    auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
                    token_uri: "https://oauth2.googleapis.com/token".to_string(),
                    userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
                    jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks".to_string(),
                    issuer: "https://accounts.google.com".to_string(),
                    scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
                    metadata: Metadata::new(),
                }
            }
        }
    ],
    default_scopes: ["https://www.googleapis.com/auth/gmail.readonly"],
});