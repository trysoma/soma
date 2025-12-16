pub mod codegen;
pub mod codegen_impl;
pub mod types;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use parking_lot::Mutex;
use shared::error::CommonError;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, trace};

use codegen_impl::TypeScriptCodeGenerator;
use sdk_core as core_types;
use types as js_types;

// Global gRPC service instance - uses Mutex<Option<...>> to allow resetting
static GRPC_SERVICE: Mutex<Option<Arc<core_types::GrpcService<TypeScriptCodeGenerator>>>> =
    Mutex::new(None);

/// Start the gRPC server on a Unix socket with TypeScript code generation
#[napi]
pub async fn start_grpc_server(socket_path: String, project_dir: String) -> Result<()> {
    let socket_path = PathBuf::from(socket_path);
    let project_dir = PathBuf::from(project_dir);

    let code_generator = TypeScriptCodeGenerator::new(project_dir);

    let service = core_types::start_grpc_server(vec![], socket_path, code_generator)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // Store the service, replacing any existing one
    let mut guard = GRPC_SERVICE.lock();
    *guard = Some(service);

    Ok(())
}

/// Kill/clear the gRPC service, removing all providers, agents, and handlers.
/// This allows the service to be restarted fresh.
#[napi]
pub fn kill_grpc_service() -> Result<()> {
    debug!("Killing gRPC service");
    let mut guard = GRPC_SERVICE.lock();
    if let Some(service) = guard.as_ref() {
        // Clear the service state
        service.clear();
    }
    // Remove the service from the global
    *guard = None;
    trace!("gRPC service cleared");
    Ok(())
}

fn get_grpc_service() -> Result<Arc<core_types::GrpcService<TypeScriptCodeGenerator>>> {
    GRPC_SERVICE.lock().clone().ok_or_else(|| {
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
                        result: Err(core_types::CallbackError {
                            message: e.reason.clone(),
                        }),
                    })
                    .unwrap()
                    .await
                    .map_err(|e| core_types::InvokeFunctionResponse {
                        result: Err(core_types::CallbackError {
                            message: e.reason.clone(),
                        }),
                    })
                    .unwrap();

                trace!("Function invocation complete");

                Ok::<core_types::InvokeFunctionResponse, CommonError>(
                    core_types::InvokeFunctionResponse {
                        result: if let Some(data) = result.data {
                            Ok(data)
                        } else if let Some(error) = result.error {
                            Err(core_types::CallbackError {
                                message: error.message,
                            })
                        } else {
                            Err(core_types::CallbackError {
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
    trace!(function = %function_metadata.name, "Updating function");
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
                trace!("Function invocation complete");

                match result {
                    Ok(js_response) => {
                        if let Some(data) = js_response.data {
                            Ok(core_types::InvokeFunctionResponse { result: Ok(data) })
                        } else if let Some(error) = js_response.error {
                            Ok(core_types::InvokeFunctionResponse {
                                result: Err(core_types::CallbackError {
                                    message: error.message,
                                }),
                            })
                        } else {
                            Ok(core_types::InvokeFunctionResponse {
                                result: Err(core_types::CallbackError {
                                    message: "JS result must contain .data or .error".to_string(),
                                }),
                            })
                        }
                    }
                    Err(e) => Ok(core_types::InvokeFunctionResponse {
                        result: Err(core_types::CallbackError {
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
        trace!(count = secrets.len(), "Secret handler invoked");
        Box::pin(async move {
            // Convert core secrets to JS secrets
            let js_secrets: Vec<js_types::Secret> = secrets
                .into_iter()
                .map(|s| js_types::Secret {
                    key: s.key,
                    value: s.value,
                })
                .collect();

            trace!(count = js_secrets.len(), "Calling JS secret handler");
            // Call the JS callback
            let result = callback
                .call_async(Ok(js_secrets))
                .await
                .map_err(|e| {
                    let error_msg = format!("Failed to call secret handler: {e}");
                    debug!(error = %error_msg, "Secret handler callback failed");
                    CommonError::Unknown(anyhow::anyhow!(error_msg))
                })?
                .await;

            match result {
                Ok(js_response) => {
                    if let Some(data) = js_response.data {
                        Ok(core_types::SetSecretsResponse {
                            result: Ok(core_types::SetSecretsSuccess {
                                message: data.message,
                            }),
                        })
                    } else if let Some(error) = js_response.error {
                        Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                    } else {
                        Err(CommonError::Unknown(anyhow::anyhow!(
                            "JS result must contain .data or .error"
                        )))
                    }
                }
                Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(format!(
                    "JavaScript function error: {e}"
                )))),
            }
        })
    });

    trace!("Registering secret handler");
    get_grpc_service()?.set_secret_handler(handler);
    trace!("Secret handler registered");
    Ok(())
}

/// Set the environment variable handler callback that will be called when environment variables are synced from Soma
/// The callback receives an array of environment variables and should inject them into process.env
#[napi]
pub fn set_environment_variable_handler(
    callback: ThreadsafeFunction<
        Vec<js_types::EnvironmentVariable>,
        Promise<js_types::SetEnvironmentVariablesResponse>,
    >,
) -> Result<()> {
    let callback = Arc::new(callback);

    let handler: core_types::EnvironmentVariableHandler =
        Arc::new(move |env_vars: Vec<core_types::EnvironmentVariable>| {
            let callback = Arc::clone(&callback);
            trace!(
                count = env_vars.len(),
                "Environment variable handler invoked"
            );
            Box::pin(async move {
                // Convert core environment variables to JS environment variables
                let js_env_vars: Vec<js_types::EnvironmentVariable> = env_vars
                    .into_iter()
                    .map(|e| js_types::EnvironmentVariable {
                        key: e.key,
                        value: e.value,
                    })
                    .collect();

                trace!(count = js_env_vars.len(), "Calling JS env var handler");
                // Call the JS callback
                let result = callback
                    .call_async(Ok(js_env_vars))
                    .await
                    .map_err(|e| {
                        let error_msg = format!("Failed to call environment variable handler: {e}");
                        debug!(error = %error_msg, "Env var handler callback failed");
                        CommonError::Unknown(anyhow::anyhow!(error_msg))
                    })?
                    .await;

                match result {
                    Ok(js_response) => {
                        if let Some(data) = js_response.data {
                            Ok(core_types::SetEnvironmentVariablesResponse {
                                result: Ok(core_types::SetEnvironmentVariablesSuccess {
                                    message: data.message,
                                }),
                            })
                        } else if let Some(error) = js_response.error {
                            Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                        } else {
                            Err(CommonError::Unknown(anyhow::anyhow!(
                                "JS result must contain .data or .error"
                            )))
                        }
                    }
                    Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(format!(
                        "JavaScript function error: {e}"
                    )))),
                }
            })
        });

    trace!("Registering environment variable handler");
    get_grpc_service()?.set_environment_variable_handler(handler);
    trace!("Environment variable handler registered");
    Ok(())
}

/// Set the unset secret handler callback that will be called when a secret is unset
/// The callback receives a secret key and should remove it from process.env
#[napi]
pub fn set_unset_secret_handler(
    callback: ThreadsafeFunction<String, Promise<js_types::UnsetSecretResponse>>,
) -> Result<()> {
    let callback = Arc::new(callback);

    let handler: core_types::UnsetSecretHandler = Arc::new(move |key: String| {
        let callback = Arc::clone(&callback);
        trace!(key = %key, "Unset secret handler invoked");
        Box::pin(async move {
            trace!(key = %key, "Calling JS unset secret handler");
            // Call the JS callback
            let result = callback
                .call_async(Ok(key))
                .await
                .map_err(|e| {
                    let error_msg = format!("Failed to call unset secret handler: {e}");
                    debug!(error = %error_msg, "Unset secret handler failed");
                    CommonError::Unknown(anyhow::anyhow!(error_msg))
                })?
                .await;

            match result {
                Ok(js_response) => {
                    if let Some(data) = js_response.data {
                        Ok(core_types::UnsetSecretResponse {
                            result: Ok(core_types::UnsetSecretSuccess {
                                message: data.message,
                            }),
                        })
                    } else if let Some(error) = js_response.error {
                        Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                    } else {
                        Err(CommonError::Unknown(anyhow::anyhow!(
                            "JS result must contain .data or .error"
                        )))
                    }
                }
                Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(format!(
                    "JavaScript function error: {e}"
                )))),
            }
        })
    });

    trace!("Registering unset secret handler");
    get_grpc_service()?.set_unset_secret_handler(handler);
    trace!("Unset secret handler registered");
    Ok(())
}

/// Set the unset environment variable handler callback that will be called when an environment variable is unset
/// The callback receives an environment variable key and should remove it from process.env
#[napi]
pub fn set_unset_environment_variable_handler(
    callback: ThreadsafeFunction<String, Promise<js_types::UnsetEnvironmentVariableResponse>>,
) -> Result<()> {
    let callback = Arc::new(callback);

    let handler: core_types::UnsetEnvironmentVariableHandler = Arc::new(move |key: String| {
        let callback = Arc::clone(&callback);
        trace!(key = %key, "Unset env var handler invoked");
        Box::pin(async move {
            trace!(key = %key, "Calling JS unset env var handler");
            // Call the JS callback
            let result = callback
                .call_async(Ok(key))
                .await
                .map_err(|e| {
                    let error_msg =
                        format!("Failed to call unset environment variable handler: {e}");
                    debug!(error = %error_msg, "Unset env var handler failed");
                    CommonError::Unknown(anyhow::anyhow!(error_msg))
                })?
                .await;

            match result {
                Ok(js_response) => {
                    if let Some(data) = js_response.data {
                        Ok(core_types::UnsetEnvironmentVariableResponse {
                            result: Ok(core_types::UnsetEnvironmentVariableSuccess {
                                message: data.message,
                            }),
                        })
                    } else if let Some(error) = js_response.error {
                        Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                    } else {
                        Err(CommonError::Unknown(anyhow::anyhow!(
                            "JS result must contain .data or .error"
                        )))
                    }
                }
                Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(format!(
                    "JavaScript function error: {e}"
                )))),
            }
        })
    });

    trace!("Registering unset environment variable handler");
    get_grpc_service()?.set_unset_environment_variable_handler(handler);
    trace!("Unset environment variable handler registered");
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

/// Response from resync_sdk operation
#[napi(object)]
pub struct ResyncSdkResponse {}

/// Calls the internal resync endpoint on the Soma API server.
/// This triggers the API server to:
/// - Fetch metadata from the SDK (providers, agents)
/// - Sync providers to the bridge registry
/// - Register Restate deployments for agents
/// - Sync secrets to the SDK
/// - Sync environment variables to the SDK
///
/// # Parameters
/// * `base_url` - Optional base URL of the Soma API server (defaults to SOMA_SERVER_BASE_URL env var or http://localhost:3000)
///
/// # Returns
/// The resync response from the server
#[napi]
pub async fn resync_sdk(base_url: Option<String>) -> Result<ResyncSdkResponse> {
    core_types::resync_sdk(base_url)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(ResyncSdkResponse {})
}
