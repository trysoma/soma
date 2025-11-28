use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use std::sync::Arc;

#[derive(Clone)]
pub struct Agent {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
}

#[derive(Clone)]
pub struct ProviderController {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub categories: Vec<String>,
    pub functions: Vec<FunctionController>,
    pub credential_controllers: Vec<ProviderCredentialController>,
}

#[derive(Clone)]
pub struct FunctionController {
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub output: String,
    pub invoke: Arc<
        dyn Fn(
                InvokeFunctionRequest,
            ) -> BoxFuture<'static, Result<InvokeFunctionResponse, CommonError>>
            + Send
            + Sync
            + 'static,
    >,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderCredentialController {
    NoAuth,
    ApiKey,
    Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowConfiguration),
    Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowConfiguration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2AuthorizationCodeFlowConfiguration {
    pub static_credential_configuration: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2JwtBearerAssertionFlowConfiguration {
    pub static_credential_configuration: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<Metadata>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<Metadata>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct InvokeFunctionRequest {
    pub provider_controller_type_id: String,
    pub function_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub credentials: String,
    pub parameters: String,
}

#[derive(Debug, Clone)]
pub struct InvokeError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CallbackError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct InvokeFunctionResponse {
    pub result: Result<String, CallbackError>,
}

pub struct MetadataResponse {
    pub bridge_providers: Vec<ProviderController>,
    pub agents: Vec<Agent>,
}

// Conversions from proto types to our types using TryFrom

// impl TryFrom<sdk_proto::ProviderController> for ProviderController {
//     type Error = CommonError;

//     fn try_from(proto: sdk_proto::ProviderController) -> Result<Self, Self::Error> {
//         Ok(Self {
//             type_id: proto.type_id,
//             name: proto.name,
//             documentation: proto.documentation,
//             categories: proto.categories,
//             functions: proto
//                 .functions
//                 .into_iter()
//                 .map(TryInto::try_into)
//                 .collect::<Result<Vec<_>, _>>()?,
//             credential_controllers: proto
//                 .credential_controllers
//                 .into_iter()
//                 .map(TryInto::try_into)
//                 .collect::<Result<Vec<_>, _>>()?,
//         })
//     }
// }

impl From<ProviderController> for sdk_proto::ProviderController {
    fn from(pc: ProviderController) -> Self {
        Self {
            type_id: pc.type_id,
            name: pc.name,
            documentation: pc.documentation,
            categories: pc.categories,
            functions: pc.functions.into_iter().map(Into::into).collect(),
            credential_controllers: pc
                .credential_controllers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<&ProviderController> for sdk_proto::ProviderController {
    fn from(pc: &ProviderController) -> Self {
        Self {
            type_id: pc.type_id.clone(),
            name: pc.name.clone(),
            documentation: pc.documentation.clone(),
            categories: pc.categories.clone(),
            functions: pc.functions.iter().map(Into::into).collect(),
            credential_controllers: pc.credential_controllers.iter().map(Into::into).collect(),
        }
    }
}

// impl TryFrom<sdk_proto::FunctionController> for FunctionController {
//     type Error = CommonError;

//     fn try_from(proto: sdk_proto::FunctionController) -> Result<Self, Self::Error> {
//         Ok(Self {
//             name: proto.name,
//             description: proto.description,
//             parameters: proto.parameters,
//             output: proto.output,
//         })
//     }
// }

impl From<FunctionController> for sdk_proto::FunctionController {
    fn from(fc: FunctionController) -> Self {
        Self {
            name: fc.name,
            description: fc.description,
            parameters: fc.parameters,
            output: fc.output,
        }
    }
}

impl From<&FunctionController> for sdk_proto::FunctionController {
    fn from(fc: &FunctionController) -> Self {
        Self {
            name: fc.name.clone(),
            description: fc.description.clone(),
            parameters: fc.parameters.clone(),
            output: fc.output.clone(),
        }
    }
}

impl TryFrom<sdk_proto::ProviderCredentialController> for ProviderCredentialController {
    type Error = CommonError;

    fn try_from(proto: sdk_proto::ProviderCredentialController) -> Result<Self, Self::Error> {
        use sdk_proto::provider_credential_controller::Kind;

        match proto.kind {
            Some(Kind::NoAuth(_)) => Ok(ProviderCredentialController::NoAuth),
            Some(Kind::ApiKey(_)) => Ok(ProviderCredentialController::ApiKey),
            Some(Kind::Oauth2(oauth2)) => {
                let config = oauth2.static_credential_configuration.ok_or_else(|| {
                    CommonError::InvalidRequest {
                        msg: "OAuth2 credential controller missing static_credential_configuration"
                            .to_string(),
                        source: None,
                    }
                })?;

                Ok(ProviderCredentialController::Oauth2AuthorizationCodeFlow(
                    Oauth2AuthorizationCodeFlowConfiguration {
                        static_credential_configuration: config.try_into()?,
                    },
                ))
            }
            Some(Kind::Oauth2JwtBearerAssertionFlow(jwt)) => {
                let config = jwt
                    .static_credential_configuration
                    .ok_or_else(|| CommonError::InvalidRequest {
                        msg: "Oauth2JwtBearerAssertionFlow credential controller missing static_credential_configuration"
                            .to_string(),
                        source: None,
                    })?;

                Ok(ProviderCredentialController::Oauth2JwtBearerAssertionFlow(
                    Oauth2JwtBearerAssertionFlowConfiguration {
                        static_credential_configuration: config.try_into()?,
                    },
                ))
            }
            None => Err(CommonError::InvalidRequest {
                msg: "ProviderCredentialController missing kind".to_string(),
                source: None,
            }),
        }
    }
}

impl From<ProviderCredentialController> for sdk_proto::ProviderCredentialController {
    fn from(pcc: ProviderCredentialController) -> Self {
        use sdk_proto::provider_credential_controller::Kind;

        let kind = match pcc {
            ProviderCredentialController::NoAuth => Some(Kind::NoAuth(sdk_proto::NoAuth {})),
            ProviderCredentialController::ApiKey => Some(Kind::ApiKey(sdk_proto::ApiKey {})),
            ProviderCredentialController::Oauth2AuthorizationCodeFlow(config) => {
                Some(Kind::Oauth2(sdk_proto::Oauth2AuthorizationCodeFlow {
                    static_credential_configuration: Some(
                        config.static_credential_configuration.into(),
                    ),
                }))
            }
            ProviderCredentialController::Oauth2JwtBearerAssertionFlow(config) => Some(
                Kind::Oauth2JwtBearerAssertionFlow(sdk_proto::Oauth2JwtBearerAssertionFlow {
                    static_credential_configuration: Some(
                        config.static_credential_configuration.into(),
                    ),
                }),
            ),
        };

        Self { kind }
    }
}

impl From<&ProviderCredentialController> for sdk_proto::ProviderCredentialController {
    fn from(pcc: &ProviderCredentialController) -> Self {
        use sdk_proto::provider_credential_controller::Kind;

        let kind = match pcc {
            ProviderCredentialController::NoAuth => Some(Kind::NoAuth(sdk_proto::NoAuth {})),
            ProviderCredentialController::ApiKey => Some(Kind::ApiKey(sdk_proto::ApiKey {})),
            ProviderCredentialController::Oauth2AuthorizationCodeFlow(config) => {
                Some(Kind::Oauth2(sdk_proto::Oauth2AuthorizationCodeFlow {
                    static_credential_configuration: Some(
                        config.static_credential_configuration.clone().into(),
                    ),
                }))
            }
            ProviderCredentialController::Oauth2JwtBearerAssertionFlow(config) => Some(
                Kind::Oauth2JwtBearerAssertionFlow(sdk_proto::Oauth2JwtBearerAssertionFlow {
                    static_credential_configuration: Some(
                        config.static_credential_configuration.clone().into(),
                    ),
                }),
            ),
        };

        Self { kind }
    }
}

impl TryFrom<sdk_proto::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration>
    for Oauth2AuthorizationCodeFlowStaticCredentialConfiguration
{
    type Error = CommonError;

    fn try_from(
        proto: sdk_proto::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            auth_uri: proto.auth_uri,
            token_uri: proto.token_uri,
            userinfo_uri: proto.userinfo_uri,
            jwks_uri: proto.jwks_uri,
            issuer: proto.issuer,
            scopes: proto.scopes,
            metadata: if !proto.metadata.is_empty() {
                Some(
                    proto
                        .metadata
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()?,
                )
            } else {
                None
            },
        })
    }
}

impl TryFrom<sdk_proto::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration>
    for Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration
{
    type Error = CommonError;

    fn try_from(
        proto: sdk_proto::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            auth_uri: proto.auth_uri,
            token_uri: proto.token_uri,
            userinfo_uri: proto.userinfo_uri,
            jwks_uri: proto.jwks_uri,
            issuer: proto.issuer,
            scopes: proto.scopes,
            metadata: if !proto.metadata.is_empty() {
                Some(
                    proto
                        .metadata
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()?,
                )
            } else {
                None
            },
        })
    }
}

impl From<Oauth2AuthorizationCodeFlowStaticCredentialConfiguration>
    for sdk_proto::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration
{
    fn from(config: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration) -> Self {
        Self {
            auth_uri: config.auth_uri,
            token_uri: config.token_uri,
            userinfo_uri: config.userinfo_uri,
            jwks_uri: config.jwks_uri,
            issuer: config.issuer,
            scopes: config.scopes,
            metadata: match config.metadata {
                Some(m) => m.into_iter().map(Into::into).collect(),
                None => vec![],
            },
        }
    }
}

impl From<Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration>
    for sdk_proto::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration
{
    fn from(config: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration) -> Self {
        Self {
            auth_uri: config.auth_uri,
            token_uri: config.token_uri,
            userinfo_uri: config.userinfo_uri,
            jwks_uri: config.jwks_uri,
            issuer: config.issuer,
            scopes: config.scopes,
            metadata: match config.metadata {
                Some(m) => m.into_iter().map(Into::into).collect(),
                None => vec![],
            },
        }
    }
}

impl TryFrom<sdk_proto::Metadata> for Metadata {
    type Error = CommonError;

    fn try_from(proto: sdk_proto::Metadata) -> Result<Self, Self::Error> {
        Ok(Self {
            key: proto.key,
            value: proto.value,
        })
    }
}

impl From<Metadata> for sdk_proto::Metadata {
    fn from(m: Metadata) -> Self {
        Self {
            key: m.key,
            value: m.value,
        }
    }
}

impl TryFrom<sdk_proto::InvokeFunctionRequest> for InvokeFunctionRequest {
    type Error = CommonError;

    fn try_from(proto: sdk_proto::InvokeFunctionRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_controller_type_id: proto.provider_controller_type_id,
            function_controller_type_id: proto.function_controller_type_id,
            credential_controller_type_id: proto.credential_controller_type_id,
            credentials: proto.credentials,
            parameters: proto.parameters,
        })
    }
}

impl From<CallbackError> for sdk_proto::CallbackError {
    fn from(error: CallbackError) -> Self {
        Self {
            message: error.message,
        }
    }
}

impl From<InvokeFunctionResponse> for sdk_proto::InvokeFunctionResponse {
    fn from(response: InvokeFunctionResponse) -> Self {
        use sdk_proto::invoke_function_response::Kind;

        let kind = match response.result {
            Ok(data) => Some(Kind::Data(data)),
            Err(error) => Some(Kind::Error(error.into())),
        };

        Self { kind }
    }
}

impl From<MetadataResponse> for sdk_proto::MetadataResponse {
    fn from(response: MetadataResponse) -> Self {
        Self {
            bridge_providers: response
                .bridge_providers
                .into_iter()
                .map(Into::into)
                .collect(),
            agents: response.agents.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Agent> for sdk_proto::Agent {
    fn from(agent: Agent) -> Self {
        Self {
            id: agent.id,
            project_id: agent.project_id,
            name: agent.name,
            description: agent.description,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Secret {
    pub key: String,
    pub value: String,
}

impl From<sdk_proto::Secret> for Secret {
    fn from(proto: sdk_proto::Secret) -> Self {
        Self {
            key: proto.key,
            value: proto.value,
        }
    }
}

impl From<Secret> for sdk_proto::Secret {
    fn from(secret: Secret) -> Self {
        Self {
            key: secret.key,
            value: secret.value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetSecretsSuccess {
    pub message: String,
}

impl From<SetSecretsSuccess> for sdk_proto::CallbackSuccess {
    fn from(success: SetSecretsSuccess) -> Self {
        Self {
            message: success.message,
        }
    }
}

/// Response from setting secrets
#[derive(Debug, Clone)]
pub struct SetSecretsResponse {
    pub result: Result<SetSecretsSuccess, CallbackError>,
}

impl From<SetSecretsResponse> for sdk_proto::SetSecretsResponse {
    fn from(response: SetSecretsResponse) -> Self {
        use sdk_proto::set_secrets_response::Kind;

        Self {
            kind: match response.result {
                Ok(data) => Some(Kind::Data(sdk_proto::CallbackSuccess {
                    message: data.message,
                })),
                Err(error) => Some(Kind::Error(error.into())),
            },
        }
    }
}

/// Type alias for the secret handler callback
pub type SecretHandler = Arc<
    dyn Fn(Vec<Secret>) -> BoxFuture<'static, Result<SetSecretsResponse, CommonError>>
        + Send
        + Sync
        + 'static,
>;

#[derive(Debug, Clone)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
}

impl From<sdk_proto::EnvironmentVariable> for EnvironmentVariable {
    fn from(proto: sdk_proto::EnvironmentVariable) -> Self {
        Self {
            key: proto.key,
            value: proto.value,
        }
    }
}

impl From<EnvironmentVariable> for sdk_proto::EnvironmentVariable {
    fn from(env_var: EnvironmentVariable) -> Self {
        Self {
            key: env_var.key,
            value: env_var.value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetEnvironmentVariablesSuccess {
    pub message: String,
}

impl From<SetEnvironmentVariablesSuccess> for sdk_proto::CallbackSuccess {
    fn from(success: SetEnvironmentVariablesSuccess) -> Self {
        Self {
            message: success.message,
        }
    }
}

/// Response from setting environment variables
#[derive(Debug, Clone)]
pub struct SetEnvironmentVariablesResponse {
    pub result: Result<SetEnvironmentVariablesSuccess, CallbackError>,
}

impl From<SetEnvironmentVariablesResponse> for sdk_proto::SetEnvironmentVariablesResponse {
    fn from(response: SetEnvironmentVariablesResponse) -> Self {
        use sdk_proto::set_environment_variables_response::Kind;

        Self {
            kind: match response.result {
                Ok(data) => Some(Kind::Data(sdk_proto::CallbackSuccess {
                    message: data.message,
                })),
                Err(error) => Some(Kind::Error(error.into())),
            },
        }
    }
}

/// Type alias for the environment variable handler callback
pub type EnvironmentVariableHandler = Arc<
    dyn Fn(
            Vec<EnvironmentVariable>,
        ) -> BoxFuture<'static, Result<SetEnvironmentVariablesResponse, CommonError>>
        + Send
        + Sync
        + 'static,
>;

// Unset secret types
#[derive(Debug, Clone)]
pub struct UnsetSecretRequest {
    pub key: String,
}

impl From<sdk_proto::UnsetSecretRequest> for UnsetSecretRequest {
    fn from(proto: sdk_proto::UnsetSecretRequest) -> Self {
        Self { key: proto.key }
    }
}

impl From<UnsetSecretRequest> for sdk_proto::UnsetSecretRequest {
    fn from(req: UnsetSecretRequest) -> Self {
        Self { key: req.key }
    }
}

#[derive(Debug, Clone)]
pub struct UnsetSecretSuccess {
    pub message: String,
}

impl From<UnsetSecretSuccess> for sdk_proto::CallbackSuccess {
    fn from(success: UnsetSecretSuccess) -> Self {
        Self {
            message: success.message,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnsetSecretResponse {
    pub result: Result<UnsetSecretSuccess, CallbackError>,
}

impl From<UnsetSecretResponse> for sdk_proto::UnsetSecretResponse {
    fn from(response: UnsetSecretResponse) -> Self {
        use sdk_proto::unset_secret_response::Kind;

        Self {
            kind: match response.result {
                Ok(data) => Some(Kind::Data(data.into())),
                Err(error) => Some(Kind::Error(error.into())),
            },
        }
    }
}

/// Type alias for the unset secret handler callback
pub type UnsetSecretHandler = Arc<
    dyn Fn(String) -> BoxFuture<'static, Result<UnsetSecretResponse, CommonError>>
        + Send
        + Sync
        + 'static,
>;

// Unset environment variable types
#[derive(Debug, Clone)]
pub struct UnsetEnvironmentVariableRequest {
    pub key: String,
}

impl From<sdk_proto::UnsetEnvironmentVariableRequest> for UnsetEnvironmentVariableRequest {
    fn from(proto: sdk_proto::UnsetEnvironmentVariableRequest) -> Self {
        Self { key: proto.key }
    }
}

impl From<UnsetEnvironmentVariableRequest> for sdk_proto::UnsetEnvironmentVariableRequest {
    fn from(req: UnsetEnvironmentVariableRequest) -> Self {
        Self { key: req.key }
    }
}

#[derive(Debug, Clone)]
pub struct UnsetEnvironmentVariableSuccess {
    pub message: String,
}

impl From<UnsetEnvironmentVariableSuccess> for sdk_proto::CallbackSuccess {
    fn from(success: UnsetEnvironmentVariableSuccess) -> Self {
        Self {
            message: success.message,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnsetEnvironmentVariableResponse {
    pub result: Result<UnsetEnvironmentVariableSuccess, CallbackError>,
}

impl From<UnsetEnvironmentVariableResponse> for sdk_proto::UnsetEnvironmentVariableResponse {
    fn from(response: UnsetEnvironmentVariableResponse) -> Self {
        use sdk_proto::unset_environment_variable_response::Kind;

        Self {
            kind: match response.result {
                Ok(data) => Some(Kind::Data(data.into())),
                Err(error) => Some(Kind::Error(error.into())),
            },
        }
    }
}

/// Type alias for the unset environment variable handler callback
pub type UnsetEnvironmentVariableHandler = Arc<
    dyn Fn(String) -> BoxFuture<'static, Result<UnsetEnvironmentVariableResponse, CommonError>>
        + Send
        + Sync
        + 'static,
>;
