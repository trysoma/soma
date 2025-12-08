//! Soma Python SDK - Native bindings for the Soma platform
//!
//! This crate provides Python bindings for the Soma SDK using PyO3.

pub mod codegen;
pub mod codegen_impl;
pub mod types;

use codegen_impl::PythonCodeGenerator;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use sdk_core as core_types;
use shared::error::CommonError;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use types as py_types;

use once_cell::sync::OnceCell;

// Global gRPC service instance
static GRPC_SERVICE: OnceCell<Arc<core_types::GrpcService<PythonCodeGenerator>>> = OnceCell::new();

fn get_grpc_service() -> PyResult<&'static Arc<core_types::GrpcService<PythonCodeGenerator>>> {
    GRPC_SERVICE.get().ok_or_else(|| {
        pyo3::exceptions::PyRuntimeError::new_err(
            "gRPC service not initialized - call start_grpc_server first",
        )
    })
}

/// Convert Python ProviderCredentialController to core type
fn convert_credential_controller(
    py_cred: &py_types::ProviderCredentialController,
) -> core_types::ProviderCredentialController {
    match &py_cred.inner {
        py_types::ProviderCredentialControllerInner::NoAuth => {
            core_types::ProviderCredentialController::NoAuth
        }
        py_types::ProviderCredentialControllerInner::ApiKey => {
            core_types::ProviderCredentialController::ApiKey
        }
        py_types::ProviderCredentialControllerInner::Oauth2AuthorizationCodeFlow(config) => {
            let metadata = config
                .static_credential_configuration
                .metadata
                .as_ref()
                .map(|m| {
                    m.iter()
                        .map(|meta| core_types::Metadata {
                            key: meta.key.clone(),
                            value: meta.value.clone(),
                        })
                        .collect()
                });

            core_types::ProviderCredentialController::Oauth2AuthorizationCodeFlow(
                core_types::Oauth2AuthorizationCodeFlowConfiguration {
                    static_credential_configuration:
                        core_types::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                            auth_uri: config.static_credential_configuration.auth_uri.clone(),
                            token_uri: config.static_credential_configuration.token_uri.clone(),
                            userinfo_uri: config
                                .static_credential_configuration
                                .userinfo_uri
                                .clone(),
                            jwks_uri: config.static_credential_configuration.jwks_uri.clone(),
                            issuer: config.static_credential_configuration.issuer.clone(),
                            scopes: config.static_credential_configuration.scopes.clone(),
                            metadata,
                        },
                },
            )
        }
        py_types::ProviderCredentialControllerInner::Oauth2JwtBearerAssertionFlow(config) => {
            let metadata = config
                .static_credential_configuration
                .metadata
                .as_ref()
                .map(|m| {
                    m.iter()
                        .map(|meta| core_types::Metadata {
                            key: meta.key.clone(),
                            value: meta.value.clone(),
                        })
                        .collect()
                });

            core_types::ProviderCredentialController::Oauth2JwtBearerAssertionFlow(
                core_types::Oauth2JwtBearerAssertionFlowConfiguration {
                    static_credential_configuration:
                        core_types::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                            auth_uri: config.static_credential_configuration.auth_uri.clone(),
                            token_uri: config.static_credential_configuration.token_uri.clone(),
                            userinfo_uri: config
                                .static_credential_configuration
                                .userinfo_uri
                                .clone(),
                            jwks_uri: config.static_credential_configuration.jwks_uri.clone(),
                            issuer: config.static_credential_configuration.issuer.clone(),
                            scopes: config.static_credential_configuration.scopes.clone(),
                            metadata,
                        },
                },
            )
        }
    }
}

/// Convert Python ProviderController to core type
fn convert_provider_controller(
    provider: &py_types::ProviderController,
) -> core_types::ProviderController {
    let credential_controllers: Vec<core_types::ProviderCredentialController> = provider
        .credential_controllers
        .iter()
        .map(convert_credential_controller)
        .collect();

    core_types::ProviderController {
        type_id: provider.type_id.clone(),
        name: provider.name.clone(),
        documentation: provider.documentation.clone(),
        categories: provider.categories.clone(),
        functions: vec![],
        credential_controllers,
    }
}

/// Start the gRPC server on a Unix socket with Python code generation
#[pyfunction]
#[pyo3(signature = (socket_path, project_dir, /) -> "typing.Awaitable[None]")]
pub fn start_grpc_server(
    py: Python,
    socket_path: String,
    project_dir: String,
) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async {
        let socket_path = PathBuf::from(socket_path);
        let project_dir = PathBuf::from(project_dir);

        let code_generator = PythonCodeGenerator::new(project_dir);

        let service = core_types::start_grpc_server(vec![], socket_path, code_generator)
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        GRPC_SERVICE.set(service).map_err(|_| {
            pyo3::exceptions::PyRuntimeError::new_err("gRPC service already initialized")
        })?;

        Ok(())
    })
}

/// Add a provider controller to the running server
#[pyfunction]
#[pyo3(signature = (provider, /) -> "None")]
pub fn add_provider(provider: py_types::ProviderController) -> PyResult<()> {
    let core_provider = convert_provider_controller(&provider);
    get_grpc_service()?.add_provider(core_provider);
    Ok(())
}

/// Remove a provider controller by type_id
#[pyfunction]
#[pyo3(signature = (type_id, /) -> "bool")]
pub fn remove_provider(type_id: String) -> PyResult<bool> {
    Ok(get_grpc_service()?.remove_provider(&type_id))
}

/// Update a provider controller (removes old and inserts new)
#[pyfunction]
#[pyo3(signature = (provider, /) -> "bool")]
pub fn update_provider(provider: py_types::ProviderController) -> PyResult<bool> {
    let current_provider = get_grpc_service()?.get_provider(&provider.type_id);
    let current_provider = if let Some(current_provider) = current_provider {
        current_provider
    } else {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Provider not found",
        ));
    };

    let credential_controllers: Vec<core_types::ProviderCredentialController> = provider
        .credential_controllers
        .iter()
        .map(convert_credential_controller)
        .collect();

    let core_provider = core_types::ProviderController {
        type_id: provider.type_id.clone(),
        name: provider.name.clone(),
        documentation: provider.documentation.clone(),
        categories: provider.categories.clone(),
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
/// * `invoke_callback` - Python callable that will be called when the function is invoked
#[pyfunction]
#[pyo3(signature = (provider_type_id, function_metadata, invoke_callback: "typing.Callable[[InvokeFunctionRequest], InvokeFunctionResponse]", /) -> "bool")]
pub fn add_function(
    provider_type_id: String,
    function_metadata: py_types::FunctionMetadata,
    invoke_callback: Py<PyAny>,
) -> PyResult<bool> {
    let callback = Arc::new(invoke_callback);

    let core_function = core_types::FunctionController {
        name: function_metadata.name.clone(),
        description: function_metadata.description.clone(),
        parameters: function_metadata.parameters.clone(),
        output: function_metadata.output.clone(),
        invoke: Arc::new(move |req: core_types::InvokeFunctionRequest| {
            let callback = Arc::clone(&callback);
            Box::pin(async move {
                // Call Python callback from async context
                let result = Python::attach(|py| {
                    let py_req = py_types::InvokeFunctionRequest {
                        provider_controller_type_id: req.provider_controller_type_id.clone(),
                        function_controller_type_id: req.function_controller_type_id.clone(),
                        credential_controller_type_id: req.credential_controller_type_id.clone(),
                        credentials: req.credentials.clone(),
                        parameters: req.parameters.clone(),
                    };

                    // Call the Python function
                    let result = callback.call1(py, (py_req,));

                    match result {
                        Ok(py_result) => {
                            // Try to extract InvokeFunctionResponse
                            match py_result.extract::<py_types::InvokeFunctionResponse>(py) {
                                Ok(response) => {
                                    if let Some(data) = response.data {
                                        Ok(core_types::InvokeFunctionResponse { result: Ok(data) })
                                    } else if let Some(error) = response.error {
                                        Ok(core_types::InvokeFunctionResponse {
                                            result: Err(core_types::CallbackError {
                                                message: error.message,
                                            }),
                                        })
                                    } else {
                                        Ok(core_types::InvokeFunctionResponse {
                                            result: Err(core_types::CallbackError {
                                                message: "Python result must contain data or error"
                                                    .to_string(),
                                            }),
                                        })
                                    }
                                }
                                Err(e) => Ok(core_types::InvokeFunctionResponse {
                                    result: Err(core_types::CallbackError {
                                        message: format!("Failed to extract response: {e}"),
                                    }),
                                }),
                            }
                        }
                        Err(e) => Ok(core_types::InvokeFunctionResponse {
                            result: Err(core_types::CallbackError {
                                message: format!("Python function error: {e}"),
                            }),
                        }),
                    }
                });

                result.map_err(|e: core_types::InvokeFunctionResponse| {
                    CommonError::Unknown(anyhow::anyhow!("{:?}", e))
                })
            })
        }),
    };

    Ok(get_grpc_service()?.add_function(&provider_type_id, core_function))
}

/// Remove a function controller from a specific provider
#[pyfunction]
#[pyo3(signature = (provider_type_id, function_name, /) -> "bool")]
pub fn remove_function(provider_type_id: String, function_name: String) -> PyResult<bool> {
    Ok(get_grpc_service()?.remove_function(&provider_type_id, &function_name))
}

/// Update a function controller (removes old and inserts new)
#[pyfunction]
#[pyo3(signature = (provider_type_id, function_metadata, invoke_callback: "typing.Callable[[InvokeFunctionRequest], InvokeFunctionResponse]", /) -> "bool")]
pub fn update_function(
    provider_type_id: String,
    function_metadata: py_types::FunctionMetadata,
    invoke_callback: Py<PyAny>,
) -> PyResult<bool> {
    info!("update_function: {:?}", function_metadata.name);
    let callback = Arc::new(invoke_callback);

    let core_function = core_types::FunctionController {
        name: function_metadata.name.clone(),
        description: function_metadata.description.clone(),
        parameters: function_metadata.parameters.clone(),
        output: function_metadata.output.clone(),
        invoke: Arc::new(move |req: core_types::InvokeFunctionRequest| {
            let callback = Arc::clone(&callback);
            Box::pin(async move {
                let result = Python::attach(|py| {
                    let py_req = py_types::InvokeFunctionRequest {
                        provider_controller_type_id: req.provider_controller_type_id.clone(),
                        function_controller_type_id: req.function_controller_type_id.clone(),
                        credential_controller_type_id: req.credential_controller_type_id.clone(),
                        credentials: req.credentials.clone(),
                        parameters: req.parameters.clone(),
                    };

                    let result = callback.call1(py, (py_req,));

                    match result {
                        Ok(py_result) => {
                            match py_result.extract::<py_types::InvokeFunctionResponse>(py) {
                                Ok(response) => {
                                    if let Some(data) = response.data {
                                        Ok(core_types::InvokeFunctionResponse { result: Ok(data) })
                                    } else if let Some(error) = response.error {
                                        Ok(core_types::InvokeFunctionResponse {
                                            result: Err(core_types::CallbackError {
                                                message: error.message,
                                            }),
                                        })
                                    } else {
                                        Ok(core_types::InvokeFunctionResponse {
                                            result: Err(core_types::CallbackError {
                                                message: "Python result must contain data or error"
                                                    .to_string(),
                                            }),
                                        })
                                    }
                                }
                                Err(e) => Ok(core_types::InvokeFunctionResponse {
                                    result: Err(core_types::CallbackError {
                                        message: format!("Failed to extract response: {e}"),
                                    }),
                                }),
                            }
                        }
                        Err(e) => Ok(core_types::InvokeFunctionResponse {
                            result: Err(core_types::CallbackError {
                                message: format!("Python function error: {e}"),
                            }),
                        }),
                    }
                });

                result.map_err(|e: core_types::InvokeFunctionResponse| {
                    CommonError::Unknown(anyhow::anyhow!("{:?}", e))
                })
            })
        }),
    };

    Ok(get_grpc_service()?.update_function(&provider_type_id, core_function))
}

/// Add an agent to the running server
#[pyfunction]
#[pyo3(signature = (agent, /) -> "bool")]
pub fn add_agent(agent: py_types::Agent) -> PyResult<bool> {
    let core_agent = core_types::Agent {
        id: agent.id,
        project_id: agent.project_id,
        name: agent.name,
        description: agent.description,
    };
    Ok(get_grpc_service()?.add_agent(core_agent))
}

/// Remove an agent by id
#[pyfunction]
#[pyo3(signature = (id, /) -> "bool")]
pub fn remove_agent(id: String) -> PyResult<bool> {
    Ok(get_grpc_service()?.remove_agent(&id))
}

/// Update an agent (removes old and inserts new)
#[pyfunction]
#[pyo3(signature = (agent, /) -> "bool")]
pub fn update_agent(agent: py_types::Agent) -> PyResult<bool> {
    let core_agent = core_types::Agent {
        id: agent.id,
        project_id: agent.project_id,
        name: agent.name,
        description: agent.description,
    };
    Ok(get_grpc_service()?.update_agent(core_agent))
}

/// Set the secret handler callback that will be called when secrets are synced from Soma
/// The callback receives a list of secrets and should inject them into os.environ
#[pyfunction]
#[pyo3(signature = (callback: "typing.Callable[[list[Secret]], SetSecretsResponse]", /) -> "None")]
pub fn set_secret_handler(callback: Py<PyAny>) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::SecretHandler = Arc::new(move |secrets: Vec<core_types::Secret>| {
        let callback: Arc<Py<PyAny>> = Arc::clone(&callback);
        let secret_keys: Vec<String> = secrets.iter().map(|s| s.key.clone()).collect();
        info!(
            "Secret handler invoked with {} secrets: {:?}",
            secrets.len(),
            secret_keys
        );

        Box::pin(async move {
            let py_secrets: Vec<py_types::Secret> = secrets
                .into_iter()
                .map(|s| py_types::Secret {
                    key: s.key,
                    value: s.value,
                })
                .collect();

            info!(
                "Calling Python secret handler callback with {} secrets",
                py_secrets.len()
            );

            let result = Python::attach(|py| {
                let result = callback.call1(py, (py_secrets,));

                match result {
                    Ok(py_result) => match py_result.extract::<py_types::SetSecretsResponse>(py) {
                        Ok(response) => {
                            if let Some(data) = response.data {
                                Ok(core_types::SetSecretsResponse {
                                    result: Ok(core_types::SetSecretsSuccess {
                                        message: data.message,
                                    }),
                                })
                            } else if let Some(error) = response.error {
                                Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                            } else {
                                Err(CommonError::Unknown(anyhow::anyhow!(
                                    "Python result must contain data or error"
                                )))
                            }
                        }
                        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                            "Failed to extract response: {e}"
                        ))),
                    },
                    Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                        "Python function error: {e}"
                    ))),
                }
            });

            result
        })
    });

    info!("Registering secret handler");
    get_grpc_service()?.set_secret_handler(handler);
    info!("Secret handler registered successfully");
    Ok(())
}

/// Set the environment variable handler callback
#[pyfunction]
#[pyo3(signature = (callback: "typing.Callable[[list[EnvironmentVariable]], SetEnvironmentVariablesResponse]", /) -> "None")]
pub fn set_environment_variable_handler(callback: Py<PyAny>) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::EnvironmentVariableHandler =
        Arc::new(move |env_vars: Vec<core_types::EnvironmentVariable>| {
            let callback: Arc<Py<PyAny>> = Arc::clone(&callback);
            let env_var_keys: Vec<String> = env_vars.iter().map(|e| e.key.clone()).collect();
            info!(
                "Environment variable handler invoked with {} env vars: {:?}",
                env_vars.len(),
                env_var_keys
            );

            Box::pin(async move {
                let py_env_vars: Vec<py_types::EnvironmentVariable> = env_vars
                    .into_iter()
                    .map(|e| py_types::EnvironmentVariable {
                        key: e.key,
                        value: e.value,
                    })
                    .collect();

                info!(
                    "Calling Python environment variable handler callback with {} env vars",
                    py_env_vars.len()
                );

                let result = Python::attach(|py| {
                    let result = callback.call1(py, (py_env_vars,));

                    match result {
                        Ok(py_result) => {
                            match py_result.extract::<py_types::SetEnvironmentVariablesResponse>(py)
                            {
                                Ok(response) => {
                                    if let Some(data) = response.data {
                                        Ok(core_types::SetEnvironmentVariablesResponse {
                                            result: Ok(
                                                core_types::SetEnvironmentVariablesSuccess {
                                                    message: data.message,
                                                },
                                            ),
                                        })
                                    } else if let Some(error) = response.error {
                                        Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                                    } else {
                                        Err(CommonError::Unknown(anyhow::anyhow!(
                                            "Python result must contain data or error"
                                        )))
                                    }
                                }
                                Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                                    "Failed to extract response: {e}"
                                ))),
                            }
                        }
                        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                            "Python function error: {e}"
                        ))),
                    }
                });

                result
            })
        });

    info!("Registering environment variable handler");
    get_grpc_service()?.set_environment_variable_handler(handler);
    info!("Environment variable handler registered successfully");
    Ok(())
}

/// Set the unset secret handler callback
#[pyfunction]
#[pyo3(signature = (callback: "typing.Callable[[str], UnsetSecretResponse]", /) -> "None")]
pub fn set_unset_secret_handler(callback: Py<PyAny>) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::UnsetSecretHandler = Arc::new(move |key: String| {
        let callback: Arc<Py<PyAny>> = Arc::clone(&callback);
        info!("Unset secret handler invoked with key: {}", key);

        Box::pin(async move {
            info!(
                "Calling Python unset secret handler callback with key: {}",
                key
            );

            let result = Python::attach(|py| {
                let result = callback.call1(py, (key.clone(),));

                match result {
                    Ok(py_result) => match py_result.extract::<py_types::UnsetSecretResponse>(py) {
                        Ok(response) => {
                            if let Some(data) = response.data {
                                Ok(core_types::UnsetSecretResponse {
                                    result: Ok(core_types::UnsetSecretSuccess {
                                        message: data.message,
                                    }),
                                })
                            } else if let Some(error) = response.error {
                                Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                            } else {
                                Err(CommonError::Unknown(anyhow::anyhow!(
                                    "Python result must contain data or error"
                                )))
                            }
                        }
                        Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                            "Failed to extract response: {e}"
                        ))),
                    },
                    Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                        "Python function error: {e}"
                    ))),
                }
            });

            result
        })
    });

    info!("Registering unset secret handler");
    get_grpc_service()?.set_unset_secret_handler(handler);
    info!("Unset secret handler registered successfully");
    Ok(())
}

/// Set the unset environment variable handler callback
#[pyfunction]
#[pyo3(signature = (callback: "typing.Callable[[str], UnsetEnvironmentVariableResponse]", /) -> "None")]
pub fn set_unset_environment_variable_handler(callback: Py<PyAny>) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::UnsetEnvironmentVariableHandler = Arc::new(move |key: String| {
        let callback: Arc<Py<PyAny>> = Arc::clone(&callback);
        info!(
            "Unset environment variable handler invoked with key: {}",
            key
        );

        Box::pin(async move {
            info!(
                "Calling Python unset environment variable handler callback with key: {}",
                key
            );

            let result = Python::attach(|py| {
                let result = callback.call1(py, (key.clone(),));

                match result {
                    Ok(py_result) => {
                        match py_result.extract::<py_types::UnsetEnvironmentVariableResponse>(py) {
                            Ok(response) => {
                                if let Some(data) = response.data {
                                    Ok(core_types::UnsetEnvironmentVariableResponse {
                                        result: Ok(core_types::UnsetEnvironmentVariableSuccess {
                                            message: data.message,
                                        }),
                                    })
                                } else if let Some(error) = response.error {
                                    Err(CommonError::Unknown(anyhow::anyhow!(error.message)))
                                } else {
                                    Err(CommonError::Unknown(anyhow::anyhow!(
                                        "Python result must contain data or error"
                                    )))
                                }
                            }
                            Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                                "Failed to extract response: {e}"
                            ))),
                        }
                    }
                    Err(e) => Err(CommonError::Unknown(anyhow::anyhow!(
                        "Python function error: {e}"
                    ))),
                }
            });

            result
        })
    });

    info!("Registering unset environment variable handler");
    get_grpc_service()?.set_unset_environment_variable_handler(handler);
    info!("Unset environment variable handler registered successfully");
    Ok(())
}

/// Calls the internal resync endpoint on the Soma API server.
/// This triggers the API server to:
/// - Fetch metadata from the SDK (providers, agents)
/// - Sync providers to the bridge registry
/// - Register Restate deployments for agents
/// - Sync secrets to the SDK
/// - Sync environment variables to the SDK
#[pyfunction]
#[pyo3(signature = (base_url=None) -> "typing.Awaitable[None]")]
pub fn resync_sdk(py: Python, base_url: Option<String>) -> PyResult<Bound<PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async {
        core_types::resync_sdk(base_url)
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    })
}

/// A Python module implemented in Rust for the Soma SDK.
#[pymodule]
pub mod trysoma_sdk_core {
    // Functions
    #[pymodule_export]
    pub use super::add_agent;
    #[pymodule_export]
    pub use super::add_function;
    #[pymodule_export]
    pub use super::add_provider;
    #[pymodule_export]
    pub use super::remove_agent;
    #[pymodule_export]
    pub use super::remove_function;
    #[pymodule_export]
    pub use super::remove_provider;
    #[pymodule_export]
    pub use super::resync_sdk;
    #[pymodule_export]
    pub use super::set_environment_variable_handler;
    #[pymodule_export]
    pub use super::set_secret_handler;
    #[pymodule_export]
    pub use super::set_unset_environment_variable_handler;
    #[pymodule_export]
    pub use super::set_unset_secret_handler;
    #[pymodule_export]
    pub use super::start_grpc_server;
    #[pymodule_export]
    pub use super::update_agent;
    #[pymodule_export]
    pub use super::update_function;
    #[pymodule_export]
    pub use super::update_provider;

    // Types
    #[pymodule_export]
    pub use super::py_types::Agent;
    #[pymodule_export]
    pub use super::py_types::CallbackError;
    #[pymodule_export]
    pub use super::py_types::EnvironmentVariable;
    #[pymodule_export]
    pub use super::py_types::FunctionController;
    #[pymodule_export]
    pub use super::py_types::FunctionMetadata;
    #[pymodule_export]
    pub use super::py_types::InvokeFunctionRequest;
    #[pymodule_export]
    pub use super::py_types::InvokeFunctionResponse;
    #[pymodule_export]
    pub use super::py_types::Metadata;
    #[pymodule_export]
    pub use super::py_types::Oauth2AuthorizationCodeFlowConfiguration;
    #[pymodule_export]
    pub use super::py_types::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration;
    #[pymodule_export]
    pub use super::py_types::Oauth2JwtBearerAssertionFlowConfiguration;
    #[pymodule_export]
    pub use super::py_types::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration;
    #[pymodule_export]
    pub use super::py_types::ProviderController;
    #[pymodule_export]
    pub use super::py_types::ProviderCredentialController;
    #[pymodule_export]
    pub use super::py_types::Secret;
    #[pymodule_export]
    pub use super::py_types::SetEnvironmentVariablesResponse;
    #[pymodule_export]
    pub use super::py_types::SetEnvironmentVariablesSuccess;
    #[pymodule_export]
    pub use super::py_types::SetSecretsResponse;
    #[pymodule_export]
    pub use super::py_types::SetSecretsSuccess;
    #[pymodule_export]
    pub use super::py_types::UnsetEnvironmentVariableResponse;
    #[pymodule_export]
    pub use super::py_types::UnsetEnvironmentVariableSuccess;
    #[pymodule_export]
    pub use super::py_types::UnsetSecretResponse;
    #[pymodule_export]
    pub use super::py_types::UnsetSecretSuccess;
}
