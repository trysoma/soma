pub mod codegen;
pub mod codegen_impl;
pub mod types;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use shared::error::CommonError;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use codegen_impl::TypeScriptCodeGenerator;
use sdk_core as core_types;
use types as js_types;

use once_cell::sync::OnceCell;

static GRPC_SERVICE: OnceCell<Arc<core_types::GrpcService<TypeScriptCodeGenerator>>> =
    OnceCell::new();

/// Start the gRPC server on a Unix socket with TypeScript code generation
#[napi]
pub async fn start_grpc_server(socket_path: String, project_dir: String) -> Result<()> {
    let socket_path = PathBuf::from(socket_path);
    let project_dir = PathBuf::from(project_dir);

    let code_generator = TypeScriptCodeGenerator::new(project_dir);

    let service = core_types::start_grpc_server(vec![], socket_path, code_generator)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    GRPC_SERVICE
        .set(service)
        .map_err(|_| napi::Error::from_reason("gRPC service already initialized"))?;

    Ok(())
}

fn get_grpc_service() -> Result<&'static Arc<core_types::GrpcService<TypeScriptCodeGenerator>>> {
    GRPC_SERVICE.get().ok_or_else(|| {
        napi::Error::from_reason("gRPC service not initialized - call start_grpc_server first")
    })
}

/// Add a provider controller to the running server
#[napi]
pub fn add_provider(provider: js_types::ProviderController) -> Result<()> {
    // Convert credential controllers
    let credential_controllers: Vec<core_types::ProviderCredentialController> = provider
        .credential_controllers
        .into_iter()
        .map(|js_cred| {
            Ok(match js_cred {
                js_types::ProviderCredentialController::NoAuth => {
                    core_types::ProviderCredentialController::NoAuth
                }
                js_types::ProviderCredentialController::ApiKey => {
                    core_types::ProviderCredentialController::ApiKey
                }
                js_types::ProviderCredentialController::Oauth2AuthorizationCodeFlow(config) => {
                    let metadata = config.static_credential_configuration.metadata.map(|m| {
                        m.into_iter()
                            .map(|js_meta| core_types::Metadata {
                                key: js_meta.key,
                                value: js_meta.value,
                            })
                            .collect()
                    });

                    core_types::ProviderCredentialController::Oauth2AuthorizationCodeFlow(
                        core_types::Oauth2AuthorizationCodeFlowConfiguration {
                            static_credential_configuration: core_types::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                                auth_uri: config.static_credential_configuration.auth_uri,
                                token_uri: config.static_credential_configuration.token_uri,
                                userinfo_uri: config.static_credential_configuration.userinfo_uri,
                                jwks_uri: config.static_credential_configuration.jwks_uri,
                                issuer: config.static_credential_configuration.issuer,
                                scopes: config.static_credential_configuration.scopes,
                                metadata,
                            },
                        },
                    )
                }
                js_types::ProviderCredentialController::Oauth2JwtBearerAssertionFlow(config) => {
                    let metadata = config.static_credential_configuration.metadata.map(|m| {
                        m.into_iter()
                            .map(|js_meta| core_types::Metadata {
                                key: js_meta.key,
                                value: js_meta.value,
                            })
                            .collect()
                    });

                    core_types::ProviderCredentialController::Oauth2JwtBearerAssertionFlow(
                        core_types::Oauth2JwtBearerAssertionFlowConfiguration {
                            static_credential_configuration: core_types::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                                auth_uri: config.static_credential_configuration.auth_uri,
                                token_uri: config.static_credential_configuration.token_uri,
                                userinfo_uri: config.static_credential_configuration.userinfo_uri,
                                jwks_uri: config.static_credential_configuration.jwks_uri,
                                issuer: config.static_credential_configuration.issuer,
                                scopes: config.static_credential_configuration.scopes,
                                metadata,
                            },
                        },
                    )
                }
            })
        })
        .collect::<std::result::Result<Vec<_>, CommonError>>()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let core_provider = core_types::ProviderController {
        type_id: provider.type_id,
        name: provider.name,
        documentation: provider.documentation,
        categories: provider.categories,
        functions: vec![],
        credential_controllers,
    };

    get_grpc_service()?.add_provider(core_provider);
    Ok(())
}

/// Remove a provider controller by type_id
#[napi]
pub fn remove_provider(type_id: String) -> Result<bool> {
    Ok(get_grpc_service()?.remove_provider(&type_id))
}

/// Update a provider controller (removes old and inserts new)
#[napi]
pub fn update_provider(provider: js_types::ProviderController) -> Result<bool> {
    let current_provider = get_grpc_service()?.get_provider(&provider.type_id);
    let current_provider = if let Some(current_provider) = current_provider {
        current_provider
    } else {
        return Err(napi::Error::from_reason("Provider not found"));
    };

    let credential_controllers: Vec<core_types::ProviderCredentialController> = provider
        .credential_controllers
        .into_iter()
        .map(|js_cred| {
            Ok(match js_cred {
                js_types::ProviderCredentialController::NoAuth => {
                    core_types::ProviderCredentialController::NoAuth
                }
                js_types::ProviderCredentialController::ApiKey => {
                    core_types::ProviderCredentialController::ApiKey
                }
                js_types::ProviderCredentialController::Oauth2AuthorizationCodeFlow(config) => {
                    let metadata = config.static_credential_configuration.metadata.map(|m| {
                        m.into_iter()
                            .map(|js_meta| core_types::Metadata {
                                key: js_meta.key,
                                value: js_meta.value,
                            })
                            .collect()
                    });

                    core_types::ProviderCredentialController::Oauth2AuthorizationCodeFlow(
                        core_types::Oauth2AuthorizationCodeFlowConfiguration {
                            static_credential_configuration: core_types::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                                auth_uri: config.static_credential_configuration.auth_uri,
                                token_uri: config.static_credential_configuration.token_uri,
                                userinfo_uri: config.static_credential_configuration.userinfo_uri,
                                jwks_uri: config.static_credential_configuration.jwks_uri,
                                issuer: config.static_credential_configuration.issuer,
                                scopes: config.static_credential_configuration.scopes,
                                metadata,
                            },
                        },
                    )
                }
                js_types::ProviderCredentialController::Oauth2JwtBearerAssertionFlow(config) => {
                    let metadata = config.static_credential_configuration.metadata.map(|m| {
                        m.into_iter()
                            .map(|js_meta| core_types::Metadata {
                                key: js_meta.key,
                                value: js_meta.value,
                            })
                            .collect()
                    });

                    core_types::ProviderCredentialController::Oauth2JwtBearerAssertionFlow(
                        core_types::Oauth2JwtBearerAssertionFlowConfiguration {
                            static_credential_configuration: core_types::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                                auth_uri: config.static_credential_configuration.auth_uri,
                                token_uri: config.static_credential_configuration.token_uri,
                                userinfo_uri: config.static_credential_configuration.userinfo_uri,
                                jwks_uri: config.static_credential_configuration.jwks_uri,
                                issuer: config.static_credential_configuration.issuer,
                                scopes: config.static_credential_configuration.scopes,
                                metadata,
                            },
                        },
                    )
                }
            })
        })
        .collect::<std::result::Result<Vec<_>, CommonError>>()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let core_provider = core_types::ProviderController {
        type_id: provider.type_id,
        name: provider.name,
        documentation: provider.documentation,
        categories: provider.categories,
        functions: current_provider.functions,
        credential_controllers,
    };

    Ok(get_grpc_service()?.update_provider(core_provider))
}

/// Add a function controller to a specific provider
///
/// # Parameters
/// * `provider_type_id` - The type_id of the provider to add the function to
/// * `function_metadata` - Object containing name, description, parameters, and output
/// * `invoke_callback` - ThreadsafeFunction that will be called when the function is invoked
#[napi(object)]
pub struct FunctionMetadata {
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub output: String,
}

#[napi]
pub fn add_function(
    provider_type_id: String,
    function_metadata: FunctionMetadata,
    invoke_callback: ThreadsafeFunction<
        js_types::InvokeFunctionRequest,
        Promise<js_types::InvokeFunctionResponse>,
    >,
) -> Result<bool> {
    let invoke_fn = Arc::new(invoke_callback);

    let core_function = core_types::FunctionController {
        name: function_metadata.name,
        description: function_metadata.description,
        parameters: function_metadata.parameters,
        output: function_metadata.output,
        invoke: Arc::new(move |req: core_types::InvokeFunctionRequest| {
            let invoke_fn = Arc::clone(&invoke_fn);
            Box::pin(async move {
                let js_req = js_types::InvokeFunctionRequest {
                    provider_controller_type_id: req.provider_controller_type_id,
                    function_controller_type_id: req.function_controller_type_id,
                    credential_controller_type_id: req.credential_controller_type_id,
                    credentials: req.credentials,
                    parameters: req.parameters,
                };

                let result = invoke_fn
                    .call_async(Ok(js_req))
                    .await
                    .map_err(|e| core_types::InvokeFunctionResponse {
                        result: Err(core_types::InvokeError {
                            message: e.reason.clone(),
                        }),
                    })
                    .unwrap()
                    .await
                    .map_err(|e| core_types::InvokeFunctionResponse {
                        result: Err(core_types::InvokeError {
                            message: e.reason.clone(),
                        }),
                    })
                    .unwrap();

                info!("invoke_fn result: {:?}", result);

                Ok::<core_types::InvokeFunctionResponse, CommonError>(
                    core_types::InvokeFunctionResponse {
                        result: if let Some(data) = result.data {
                            Ok(data)
                        } else if let Some(error) = result.error {
                            Err(core_types::InvokeError {
                                message: error.message,
                            })
                        } else {
                            Err(core_types::InvokeError {
                                message: "JS result must contain .data or .error".to_string(),
                            })
                        },
                    },
                )
            })
        }),
    };

    Ok(get_grpc_service()?.add_function(&provider_type_id, core_function))
}

/// Remove a function controller from a specific provider
#[napi]
pub fn remove_function(provider_type_id: String, function_name: String) -> Result<bool> {
    Ok(get_grpc_service()?.remove_function(&provider_type_id, &function_name))
}

/// Update a function controller (removes old and inserts new)
#[napi]
pub fn update_function(
    provider_type_id: String,
    function_metadata: FunctionMetadata,
    invoke_callback: ThreadsafeFunction<
        js_types::InvokeFunctionRequest,
        js_types::InvokeFunctionResponse,
    >,
) -> Result<bool> {
    info!("update_function: {:?}", function_metadata.name);
    let invoke_fn = Arc::new(invoke_callback);

    let core_function = core_types::FunctionController {
        name: function_metadata.name,
        description: function_metadata.description,
        parameters: function_metadata.parameters,
        output: function_metadata.output,
        invoke: Arc::new(move |req: core_types::InvokeFunctionRequest| {
            let invoke_fn = Arc::clone(&invoke_fn);
            Box::pin(async move {
                let js_req = js_types::InvokeFunctionRequest {
                    provider_controller_type_id: req.provider_controller_type_id,
                    function_controller_type_id: req.function_controller_type_id,
                    credential_controller_type_id: req.credential_controller_type_id,
                    credentials: req.credentials,
                    parameters: req.parameters,
                };

                let result = invoke_fn.call_async(Ok(js_req)).await;
                info!("invoke_fn result: {:?}", result);

                match result {
                    Ok(js_response) => {
                        if let Some(data) = js_response.data {
                            Ok(core_types::InvokeFunctionResponse { result: Ok(data) })
                        } else if let Some(error) = js_response.error {
                            Ok(core_types::InvokeFunctionResponse {
                                result: Err(core_types::InvokeError {
                                    message: error.message,
                                }),
                            })
                        } else {
                            Ok(core_types::InvokeFunctionResponse {
                                result: Err(core_types::InvokeError {
                                    message: "JS result must contain .data or .error".to_string(),
                                }),
                            })
                        }
                    }
                    Err(e) => Ok(core_types::InvokeFunctionResponse {
                        result: Err(core_types::InvokeError {
                            message: format!("JavaScript function error: {e}"),
                        }),
                    }),
                }
            })
        }),
    };

    Ok(get_grpc_service()?.update_function(&provider_type_id, core_function))
}

#[napi]
pub fn add_agent(agent: js_types::Agent) -> Result<bool> {
    let core_agent = core_types::Agent {
        id: agent.id,
        project_id: agent.project_id,
        name: agent.name,
        description: agent.description,
    };
    Ok(get_grpc_service()?.add_agent(core_agent))
}

/// Set the secret handler callback that will be called when secrets are synced from Soma
/// The callback receives an array of secrets and should inject them into process.env
#[napi]
pub fn set_secret_handler(
    callback: ThreadsafeFunction<Vec<js_types::Secret>, Promise<js_types::SetSecretsResponse>>,
) -> Result<()> {
    let callback = Arc::new(callback);

    let handler: core_types::SecretHandler = Arc::new(move |secrets: Vec<core_types::Secret>| {
        let callback = Arc::clone(&callback);
        Box::pin(async move {
            // Convert core secrets to JS secrets
            let js_secrets: Vec<js_types::Secret> = secrets
                .into_iter()
                .map(|s| js_types::Secret {
                    key: s.key,
                    value: s.value,
                })
                .collect();

            // Call the JS callback
            let result = callback
                .call_async(Ok(js_secrets))
                .await
                .map_err(|e| {
                    CommonError::Unknown(anyhow::anyhow!("Failed to call secret handler: {e}"))
                })?
                .await
                .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Secret handler failed: {e}")))?;

            Ok(core_types::SetSecretsResponse {
                success: result.success,
                message: result.message,
            })
        })
    });

    get_grpc_service()?.set_secret_handler(handler);
    info!("Secret handler registered");
    Ok(())
}

/// Remove an agent by id
#[napi]
pub fn remove_agent(id: String) -> Result<bool> {
    Ok(get_grpc_service()?.remove_agent(&id))
}

/// Update an agent (removes old and inserts new)
#[napi]
pub fn update_agent(agent: js_types::Agent) -> Result<bool> {
    let core_agent = core_types::Agent {
        id: agent.id,
        project_id: agent.project_id,
        name: agent.name,
        description: agent.description,
    };
    Ok(get_grpc_service()?.update_agent(core_agent))
}
