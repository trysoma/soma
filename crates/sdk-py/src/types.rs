use pyo3::prelude::*;

pub type InvokeFunction = ThreadsafeFunction<InvokeFunctionRequest, InvokeFunctionResponse>;

#[pyclass]
pub struct Agent {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
}

#[pyclass]
pub struct ProviderController {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub categories: Vec<String>,
    pub credential_controllers: Vec<ProviderCredentialController>,
}

#[pyclass]
pub struct FunctionController {
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub output: String,
}

#[pyclass]
pub enum ProviderCredentialControllerType {
    NoAuth,
    ApiKey,
    Oauth2AuthorizationCodeFlow,
    Oauth2JwtBearerAssertionFlow
    // Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowConfiguration),
    // Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowConfiguration),
}

enum ProviderCredentialControllerInner {
    NoAuth,
    ApiKey,
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowConfiguration),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowConfiguration),
}

#[pyclass]
pub struct ProviderCredentialController {
    pub r#type: ProviderCredentialControllerType,
    pub inner: ProviderCredentialControllerInner,
}

#[pymethods]
impl ProviderCredentialController {
    #[staticmethod]
    fn no_auth() -> Self {
        Self { r#type: ProviderCredentialControllerType::NoAuth, inner: ProviderCredentialControllerInner::NoAuth }
    }

    #[staticmethod]
    fn api_key() -> Self {
        Self { r#type: ProviderCredentialControllerType::ApiKey, inner: ProviderCredentialControllerInner::ApiKey }
    }

    #[staticmethod]
    fn oauth2_authorization_code_flow(cfg: Oauth2AuthorizationCodeFlowConfiguration) -> Self {
        Self {
            r#type: ProviderCredentialControllerType::Oauth2AuthorizationCodeFlow,
            inner: ProviderCredentialControllerInner::Oauth2AuthorizationCodeFlow(cfg)
        }
    }

    #[staticmethod]
    fn oauth2_jwt_bearer_flow(cfg: Oauth2JwtBearerAssertionFlowConfiguration) -> Self {
        Self {
            r#type: ProviderCredentialControllerType::Oauth2JwtBearerAssertionFlow,
            inner: ProviderCredentialControllerInner::Oauth2JwtBearerAssertionFlow(cfg)
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Oauth2AuthorizationCodeFlowConfiguration {
    pub static_credential_configuration: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
}

#[pyclass]
#[derive(Clone)]
pub struct Oauth2JwtBearerAssertionFlowConfiguration {
    pub static_credential_configuration: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
}

#[pyclass]
#[derive(Clone)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<Metadata>>,
}

#[pyclass]
#[derive(Clone)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<Metadata>>,
}

#[pyclass]
#[derive(Clone)]
pub struct Metadata {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct InvokeFunctionRequest {
    pub provider_controller_type_id: String,
    pub function_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub credentials: String,
    pub parameters: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct CallbackError {
    pub message: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct InvokeFunctionResponse {
    pub data: Option<String>,
    pub error: Option<CallbackError>,
}

#[pyclass]
pub struct GenerateBridgeClientRequest {
    pub function_instances: Vec<FunctionInstanceData>,
}

#[pyclass]
pub struct FunctionInstanceData {
    pub provider_instance_id: String,
    pub provider_instance_display_name: String,
    pub provider_controller: Option<ProviderControllerData>,
    pub function_controller: Option<FunctionControllerData>,
}

#[pyclass]
pub struct ProviderControllerData {
    pub type_id: String,
    pub display_name: String,
}

#[pyclass]
pub struct FunctionControllerData {
    pub type_id: String,
    pub display_name: String,
    pub params_json_schema: String,
    pub return_value_json_schema: String,
}

#[pyclass]
pub struct GenerateBridgeClientResponse {
    pub success: Option<GenerateBridgeClientSuccess>,
    pub error: Option<GenerateBridgeClientError>,
}

#[pyclass]
pub struct GenerateBridgeClientSuccess {
    pub message: String,
}

#[pyclass]
pub struct GenerateBridgeClientError {
    pub message: String,
}

#[pyclass]
pub struct Secret {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct SetSecretsSuccess {
    pub message: String,
}

/// Response from setting secrets
#[derive(Debug, Clone)]
#[pyclass]
pub struct SetSecretsResponse {
    pub data: Option<SetSecretsSuccess>,
    pub error: Option<CallbackError>,
}

#[pyclass]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct SetEnvironmentVariablesSuccess {
    pub message: String,
}

/// Response from setting environment variables
#[derive(Debug, Clone)]
#[pyclass]
pub struct SetEnvironmentVariablesResponse {
    pub data: Option<SetEnvironmentVariablesSuccess>,
    pub error: Option<CallbackError>,
}

// Unset secret types
#[derive(Debug, Clone)]
#[pyclass]
pub struct UnsetSecretSuccess {
    pub message: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct UnsetSecretResponse {
    pub data: Option<UnsetSecretSuccess>,
    pub error: Option<CallbackError>,
}

// Unset environment variable types
#[derive(Debug, Clone)]
#[pyclass]
pub struct UnsetEnvironmentVariableSuccess {
    pub message: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct UnsetEnvironmentVariableResponse {
    pub data: Option<UnsetEnvironmentVariableSuccess>,
    pub error: Option<CallbackError>,
}
