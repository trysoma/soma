use pyo3::prelude::*;

/// Agent metadata
#[pyclass]
#[derive(Clone)]
pub struct Agent {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub project_id: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub description: String,
}

#[pymethods]
impl Agent {
    #[new]
    #[pyo3(signature = (id, project_id, name, description, /)-> "Agent")]
    fn new(id: String, project_id: String, name: String, description: String) -> Self {
        Self {
            id,
            project_id,
            name,
            description,
        }
    }
}

/// Provider controller definition
#[pyclass]
#[derive(Clone, Debug)]
pub struct ProviderController {
    #[pyo3(get, set)]
    pub type_id: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub documentation: String,
    #[pyo3(get, set)]
    pub categories: Vec<String>,
    #[pyo3(get, set)]
    pub credential_controllers: Vec<ProviderCredentialController>,
}

#[pymethods]
impl ProviderController {
    #[new]
    #[pyo3(signature = (type_id, name, documentation, categories: "list[str]", credential_controllers: "list[ProviderCredentialController]", /) -> "ProviderController")]
    fn new(
        type_id: String,
        name: String,
        documentation: String,
        categories: Vec<String>,
        credential_controllers: Vec<ProviderCredentialController>,
    ) -> Self {
        Self {
            type_id,
            name,
            documentation,
            categories,
            credential_controllers,
        }
    }
}

/// Function controller definition
#[pyclass]
#[derive(Clone, Debug)]
pub struct FunctionController {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub description: String,
    #[pyo3(get, set)]
    pub parameters: String,
    #[pyo3(get, set)]
    pub output: String,
}

#[pymethods]
impl FunctionController {
    #[new]
    #[pyo3(signature = (name, description, parameters, output, /) -> "FunctionController")]
    fn new(name: String, description: String, parameters: String, output: String) -> Self {
        Self {
            name,
            description,
            parameters,
            output,
        }
    }
}

/// Credential controller types
#[pyclass]
#[derive(Clone, Debug)]
pub struct ProviderCredentialController {
    #[pyo3(get)]
    pub credential_type: String,
    pub inner: ProviderCredentialControllerInner,
}

#[derive(Clone, Debug)]
pub enum ProviderCredentialControllerInner {
    NoAuth,
    ApiKey,
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowConfiguration),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowConfiguration),
}

#[pymethods]
impl ProviderCredentialController {
    #[staticmethod]
    #[pyo3(signature = () -> "ProviderCredentialController")]
    fn no_auth() -> Self {
        Self {
            credential_type: "NoAuth".to_string(),
            inner: ProviderCredentialControllerInner::NoAuth,
        }
    }

    #[staticmethod]
    #[pyo3(signature = () -> "ProviderCredentialController")]
    fn api_key() -> Self {
        Self {
            credential_type: "ApiKey".to_string(),
            inner: ProviderCredentialControllerInner::ApiKey,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (config, /) -> "ProviderCredentialController")]
    fn oauth2_authorization_code_flow(config: Oauth2AuthorizationCodeFlowConfiguration) -> Self {
        Self {
            credential_type: "Oauth2AuthorizationCodeFlow".to_string(),
            inner: ProviderCredentialControllerInner::Oauth2AuthorizationCodeFlow(config),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (config, /) -> "ProviderCredentialController")]
    fn oauth2_jwt_bearer_flow(config: Oauth2JwtBearerAssertionFlowConfiguration) -> Self {
        Self {
            credential_type: "Oauth2JwtBearerAssertionFlow".to_string(),
            inner: ProviderCredentialControllerInner::Oauth2JwtBearerAssertionFlow(config),
        }
    }

    #[pyo3(signature = () -> "Oauth2AuthorizationCodeFlowConfiguration | None")]
    fn get_oauth2_authorization_code_config(
        &self,
    ) -> Option<Oauth2AuthorizationCodeFlowConfiguration> {
        match &self.inner {
            ProviderCredentialControllerInner::Oauth2AuthorizationCodeFlow(c) => Some(c.clone()),
            _ => None,
        }
    }

    #[pyo3(signature = () -> "Oauth2JwtBearerAssertionFlowConfiguration | None")]
    fn get_oauth2_jwt_bearer_config(&self) -> Option<Oauth2JwtBearerAssertionFlowConfiguration> {
        match &self.inner {
            ProviderCredentialControllerInner::Oauth2JwtBearerAssertionFlow(c) => Some(c.clone()),
            _ => None,
        }
    }
}

/// OAuth2 Authorization Code Flow configuration
#[pyclass]
#[derive(Clone, Debug)]
pub struct Oauth2AuthorizationCodeFlowConfiguration {
    #[pyo3(get, set)]
    pub static_credential_configuration: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
}

#[pymethods]
impl Oauth2AuthorizationCodeFlowConfiguration {
    #[new]
    #[pyo3(signature = (static_credential_configuration, /) -> "Oauth2AuthorizationCodeFlowConfiguration")]
    fn new(
        static_credential_configuration: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
    ) -> Self {
        Self {
            static_credential_configuration,
        }
    }
}

/// OAuth2 JWT Bearer Assertion Flow configuration
#[pyclass]
#[derive(Clone, Debug)]
pub struct Oauth2JwtBearerAssertionFlowConfiguration {
    #[pyo3(get, set)]
    pub static_credential_configuration: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
}

#[pymethods]
impl Oauth2JwtBearerAssertionFlowConfiguration {
    #[new]
    #[pyo3(signature = (static_credential_configuration, /) -> "Oauth2JwtBearerAssertionFlowConfiguration")]
    fn new(
        static_credential_configuration: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
    ) -> Self {
        Self {
            static_credential_configuration,
        }
    }
}

/// Static credential configuration for OAuth2 Authorization Code Flow
#[pyclass]
#[derive(Clone, Debug)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    #[pyo3(get, set)]
    pub auth_uri: String,
    #[pyo3(get, set)]
    pub token_uri: String,
    #[pyo3(get, set)]
    pub userinfo_uri: String,
    #[pyo3(get, set)]
    pub jwks_uri: String,
    #[pyo3(get, set)]
    pub issuer: String,
    #[pyo3(get, set)]
    pub scopes: Vec<String>,
    #[pyo3(get, set)]
    pub metadata: Option<Vec<Metadata>>,
}

#[pymethods]
impl Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    #[new]
    #[pyo3(signature = (auth_uri, token_uri, userinfo_uri, jwks_uri, issuer, scopes: "list[str]", /, metadata: "list[Metadata] | None" = None) -> "Oauth2AuthorizationCodeFlowStaticCredentialConfiguration")]
    fn new(
        auth_uri: String,
        token_uri: String,
        userinfo_uri: String,
        jwks_uri: String,
        issuer: String,
        scopes: Vec<String>,
        metadata: Option<Vec<Metadata>>,
    ) -> Self {
        Self {
            auth_uri,
            token_uri,
            userinfo_uri,
            jwks_uri,
            issuer,
            scopes,
            metadata,
        }
    }
}

/// Static credential configuration for OAuth2 JWT Bearer Assertion Flow
#[pyclass]
#[derive(Clone, Debug)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    #[pyo3(get, set)]
    pub auth_uri: String,
    #[pyo3(get, set)]
    pub token_uri: String,
    #[pyo3(get, set)]
    pub userinfo_uri: String,
    #[pyo3(get, set)]
    pub jwks_uri: String,
    #[pyo3(get, set)]
    pub issuer: String,
    #[pyo3(get, set)]
    pub scopes: Vec<String>,
    #[pyo3(get, set)]
    pub metadata: Option<Vec<Metadata>>,
}

#[pymethods]
impl Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    #[new]
    #[pyo3(signature = (auth_uri, token_uri, userinfo_uri, jwks_uri, issuer, scopes: "list[str]", /, metadata: "list[Metadata] | None" = None) -> "Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration")]
    fn new(
        auth_uri: String,
        token_uri: String,
        userinfo_uri: String,
        jwks_uri: String,
        issuer: String,
        scopes: Vec<String>,
        metadata: Option<Vec<Metadata>>,
    ) -> Self {
        Self {
            auth_uri,
            token_uri,
            userinfo_uri,
            jwks_uri,
            issuer,
            scopes,
            metadata,
        }
    }
}

/// Metadata key-value pair
#[pyclass]
#[derive(Clone, Debug)]
pub struct Metadata {
    #[pyo3(get, set)]
    pub key: String,
    #[pyo3(get, set)]
    pub value: String,
}

#[pymethods]
impl Metadata {
    #[new]
    #[pyo3(signature = (key, value, /) -> "Metadata")]
    fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

/// Function invocation request
#[pyclass]
#[derive(Clone, Debug)]
pub struct InvokeFunctionRequest {
    #[pyo3(get, set)]
    pub provider_controller_type_id: String,
    #[pyo3(get, set)]
    pub function_controller_type_id: String,
    #[pyo3(get, set)]
    pub credential_controller_type_id: String,
    #[pyo3(get, set)]
    pub credentials: String,
    #[pyo3(get, set)]
    pub parameters: String,
}

#[pymethods]
impl InvokeFunctionRequest {
    #[new]
    #[pyo3(signature = (provider_controller_type_id, function_controller_type_id, credential_controller_type_id, credentials, parameters, /) -> "InvokeFunctionRequest")]
    fn new(
        provider_controller_type_id: String,
        function_controller_type_id: String,
        credential_controller_type_id: String,
        credentials: String,
        parameters: String,
    ) -> Self {
        Self {
            provider_controller_type_id,
            function_controller_type_id,
            credential_controller_type_id,
            credentials,
            parameters,
        }
    }
}

/// Callback error
#[pyclass]
#[derive(Clone, Debug)]
pub struct CallbackError {
    #[pyo3(get, set)]
    pub message: String,
}

#[pymethods]
impl CallbackError {
    #[new]
    #[pyo3(signature = (message, /) -> "CallbackError")]
    fn new(message: String) -> Self {
        Self { message }
    }
}

/// Function invocation response
#[pyclass]
#[derive(Clone, Debug)]
pub struct InvokeFunctionResponse {
    #[pyo3(get, set)]
    pub data: Option<String>,
    #[pyo3(get, set)]
    pub error: Option<CallbackError>,
}

#[pymethods]
impl InvokeFunctionResponse {
    #[new]
    #[pyo3(signature = (data=None, error=None) -> "InvokeFunctionResponse")]
    fn new(data: Option<String>, error: Option<CallbackError>) -> Self {
        Self { data, error }
    }

    #[staticmethod]
    #[pyo3(signature = (data, /) -> "InvokeFunctionResponse")]
    fn success(data: String) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "InvokeFunctionResponse")]
    fn failure(message: String) -> Self {
        Self {
            data: None,
            error: Some(CallbackError { message }),
        }
    }
}

/// Secret key-value pair
#[pyclass]
#[derive(Clone, Debug)]
pub struct Secret {
    #[pyo3(get, set)]
    pub key: String,
    #[pyo3(get, set)]
    pub value: String,
}

#[pymethods]
impl Secret {
    #[new]
    #[pyo3(signature = (key, value, /) -> "Secret")]
    fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

/// Success response for setting secrets
#[pyclass]
#[derive(Clone, Debug)]
pub struct SetSecretsSuccess {
    #[pyo3(get, set)]
    pub message: String,
}

#[pymethods]
impl SetSecretsSuccess {
    #[new]
    #[pyo3(signature = (message, /) -> "SetSecretsSuccess")]
    fn new(message: String) -> Self {
        Self { message }
    }
}

/// Response from setting secrets
#[pyclass]
#[derive(Clone, Debug)]
pub struct SetSecretsResponse {
    #[pyo3(get, set)]
    pub data: Option<SetSecretsSuccess>,
    #[pyo3(get, set)]
    pub error: Option<CallbackError>,
}

#[pymethods]
impl SetSecretsResponse {
    #[new]
    #[pyo3(signature = (data=None, error=None) -> "SetSecretsResponse")]
    fn new(data: Option<SetSecretsSuccess>, error: Option<CallbackError>) -> Self {
        Self { data, error }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "SetSecretsResponse")]
    fn success(message: String) -> Self {
        Self {
            data: Some(SetSecretsSuccess { message }),
            error: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "SetSecretsResponse")]
    fn failure(message: String) -> Self {
        Self {
            data: None,
            error: Some(CallbackError { message }),
        }
    }
}

/// Environment variable key-value pair
#[pyclass]
#[derive(Clone, Debug)]
pub struct EnvironmentVariable {
    #[pyo3(get, set)]
    pub key: String,
    #[pyo3(get, set)]
    pub value: String,
}

#[pymethods]
impl EnvironmentVariable {
    #[new]
    #[pyo3(signature = (key, value, /) -> "EnvironmentVariable")]
    fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

/// Success response for setting environment variables
#[pyclass]
#[derive(Clone, Debug)]
pub struct SetEnvironmentVariablesSuccess {
    #[pyo3(get, set)]
    pub message: String,
}

#[pymethods]
impl SetEnvironmentVariablesSuccess {
    #[new]
    #[pyo3(signature = (message, /) -> "SetEnvironmentVariablesSuccess")]
    fn new(message: String) -> Self {
        Self { message }
    }
}

/// Response from setting environment variables
#[pyclass]
#[derive(Clone, Debug)]
pub struct SetEnvironmentVariablesResponse {
    #[pyo3(get, set)]
    pub data: Option<SetEnvironmentVariablesSuccess>,
    #[pyo3(get, set)]
    pub error: Option<CallbackError>,
}

#[pymethods]
impl SetEnvironmentVariablesResponse {
    #[new]
    #[pyo3(signature = (data=None, error=None) -> "SetEnvironmentVariablesResponse")]
    fn new(data: Option<SetEnvironmentVariablesSuccess>, error: Option<CallbackError>) -> Self {
        Self { data, error }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "SetEnvironmentVariablesResponse")]
    fn success(message: String) -> Self {
        Self {
            data: Some(SetEnvironmentVariablesSuccess { message }),
            error: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "SetEnvironmentVariablesResponse")]
    fn failure(message: String) -> Self {
        Self {
            data: None,
            error: Some(CallbackError { message }),
        }
    }
}

/// Success response for unsetting a secret
#[pyclass]
#[derive(Clone, Debug)]
pub struct UnsetSecretSuccess {
    #[pyo3(get, set)]
    pub message: String,
}

#[pymethods]
impl UnsetSecretSuccess {
    #[new]
    #[pyo3(signature = (message, /) -> "UnsetSecretSuccess")]
    fn new(message: String) -> Self {
        Self { message }
    }
}

/// Response from unsetting a secret
#[pyclass]
#[derive(Clone, Debug)]
pub struct UnsetSecretResponse {
    #[pyo3(get, set)]
    pub data: Option<UnsetSecretSuccess>,
    #[pyo3(get, set)]
    pub error: Option<CallbackError>,
}

#[pymethods]
impl UnsetSecretResponse {
    #[new]
    #[pyo3(signature = (data=None, error=None) -> "UnsetSecretResponse")]
    fn new(data: Option<UnsetSecretSuccess>, error: Option<CallbackError>) -> Self {
        Self { data, error }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "UnsetSecretResponse")]
    fn success(message: String) -> Self {
        Self {
            data: Some(UnsetSecretSuccess { message }),
            error: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "UnsetSecretResponse")]
    fn failure(message: String) -> Self {
        Self {
            data: None,
            error: Some(CallbackError { message }),
        }
    }
}

/// Success response for unsetting an environment variable
#[pyclass]
#[derive(Clone, Debug)]
pub struct UnsetEnvironmentVariableSuccess {
    #[pyo3(get, set)]
    pub message: String,
}

#[pymethods]
impl UnsetEnvironmentVariableSuccess {
    #[new]
    #[pyo3(signature = (message, /) -> "UnsetEnvironmentVariableSuccess")]
    fn new(message: String) -> Self {
        Self { message }
    }
}

/// Response from unsetting an environment variable
#[pyclass]
#[derive(Clone, Debug)]
pub struct UnsetEnvironmentVariableResponse {
    #[pyo3(get, set)]
    pub data: Option<UnsetEnvironmentVariableSuccess>,
    #[pyo3(get, set)]
    pub error: Option<CallbackError>,
}

#[pymethods]
impl UnsetEnvironmentVariableResponse {
    #[new]
    #[pyo3(signature = (data=None, error=None) -> "UnsetEnvironmentVariableResponse")]
    fn new(data: Option<UnsetEnvironmentVariableSuccess>, error: Option<CallbackError>) -> Self {
        Self { data, error }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "UnsetEnvironmentVariableResponse")]
    fn success(message: String) -> Self {
        Self {
            data: Some(UnsetEnvironmentVariableSuccess { message }),
            error: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (message, /) -> "UnsetEnvironmentVariableResponse")]
    fn failure(message: String) -> Self {
        Self {
            data: None,
            error: Some(CallbackError { message }),
        }
    }
}

/// Function metadata for registration
#[pyclass]
#[derive(Clone, Debug)]
pub struct FunctionMetadata {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub description: String,
    #[pyo3(get, set)]
    pub parameters: String,
    #[pyo3(get, set)]
    pub output: String,
}

#[pymethods]
impl FunctionMetadata {
    #[new]
    #[pyo3(signature = (name, description, parameters, output, /) -> "FunctionMetadata")]
    fn new(name: String, description: String, parameters: String, output: String) -> Self {
        Self {
            name,
            description,
            parameters,
            output,
        }
    }
}
