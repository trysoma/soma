use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;

pub type InvokeFunction = ThreadsafeFunction<InvokeFunctionRequest, InvokeFunctionResponse>;



#[napi(object)]
pub struct ProviderController {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub categories: Vec<String>,
    pub credential_controllers: Vec<ProviderCredentialController>,
}

#[napi(object)]
pub struct FunctionController {
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub output: String,
}


#[napi]
pub enum ProviderCredentialController {
    NoAuth,
    ApiKey,
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowConfiguration),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowConfiguration),
}

#[napi(object)]
pub struct Oauth2AuthorizationCodeFlowConfiguration {
    pub static_credential_configuration: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
}

#[napi(object)]
pub struct Oauth2JwtBearerAssertionFlowConfiguration {
    pub static_credential_configuration: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
}

#[napi(object)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<Metadata>>,
}

#[napi(object)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<Metadata>>,
}

#[napi(object)]
pub struct Metadata {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[napi]
pub struct InvokeFunctionRequest {
    pub provider_controller_type_id: String,
    pub function_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub credentials: String,
    pub parameters: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct InvokeFunctionResponse {
    pub data: Option<String>,
    pub error: Option<String>,
}
