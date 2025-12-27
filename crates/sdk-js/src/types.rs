use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;

pub type InvokeFunction = ThreadsafeFunction<InvokeFunctionRequest, InvokeFunctionResponse>;

#[napi(object)]
pub struct Agent {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
}

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
pub struct CallbackError {
    pub message: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct InvokeFunctionResponse {
    pub data: Option<String>,
    pub error: Option<CallbackError>,
}

#[napi(object)]
pub struct GenerateMcpClientRequest {
    pub function_instances: Vec<FunctionInstanceData>,
}

#[napi(object)]
pub struct FunctionInstanceData {
    pub provider_instance_id: String,
    pub provider_instance_display_name: String,
    pub provider_controller: Option<ProviderControllerData>,
    pub function_controller: Option<FunctionControllerData>,
}

#[napi(object)]
pub struct ProviderControllerData {
    pub type_id: String,
    pub display_name: String,
}

#[napi(object)]
pub struct FunctionControllerData {
    pub type_id: String,
    pub display_name: String,
    pub params_json_schema: String,
    pub return_value_json_schema: String,
}

#[napi(object)]
pub struct GenerateMcpClientResponse {
    pub success: Option<GenerateMcpClientSuccess>,
    pub error: Option<GenerateMcpClientError>,
}

#[napi(object)]
pub struct GenerateMcpClientSuccess {
    pub message: String,
}

#[napi(object)]
pub struct GenerateMcpClientError {
    pub message: String,
}

#[napi(object)]
pub struct Secret {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct SetSecretsSuccess {
    pub message: String,
}

/// Response from setting secrets
#[derive(Debug, Clone)]
#[napi(object)]
pub struct SetSecretsResponse {
    pub data: Option<SetSecretsSuccess>,
    pub error: Option<CallbackError>,
}

#[napi(object)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct SetEnvironmentVariablesSuccess {
    pub message: String,
}

/// Response from setting environment variables
#[derive(Debug, Clone)]
#[napi(object)]
pub struct SetEnvironmentVariablesResponse {
    pub data: Option<SetEnvironmentVariablesSuccess>,
    pub error: Option<CallbackError>,
}

// Unset secret types
#[derive(Debug, Clone)]
#[napi(object)]
pub struct UnsetSecretSuccess {
    pub message: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct UnsetSecretResponse {
    pub data: Option<UnsetSecretSuccess>,
    pub error: Option<CallbackError>,
}

// Unset environment variable types
#[derive(Debug, Clone)]
#[napi(object)]
pub struct UnsetEnvironmentVariableSuccess {
    pub message: String,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct UnsetEnvironmentVariableResponse {
    pub data: Option<UnsetEnvironmentVariableSuccess>,
    pub error: Option<CallbackError>,
}
