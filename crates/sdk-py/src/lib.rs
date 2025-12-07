//! Soma Python SDK - Native bindings for the Soma platform
//!
//! This crate provides Python bindings for the Soma SDK using PyO3.

pub mod codegen;
pub mod codegen_impl;
pub mod types;

use codegen_impl::PythonCodeGenerator;
use pyo3::prelude::*;
use sdk_core as core_types;
use shared::error::CommonError;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use types as py_types;

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

// Global runtime for async operations
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

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
fn start_grpc_server(socket_path: String, project_dir: String) -> PyResult<()> {
    let socket_path = PathBuf::from(socket_path);
    let project_dir = PathBuf::from(project_dir);

    let code_generator = PythonCodeGenerator::new(project_dir);

    let service = get_runtime()
        .block_on(async {
            core_types::start_grpc_server(vec![], socket_path, code_generator).await
        })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

    GRPC_SERVICE.set(service).map_err(|_| {
        pyo3::exceptions::PyRuntimeError::new_err("gRPC service already initialized")
    })?;

    Ok(())
}

/// Add a provider controller to the running server
#[pyfunction]
fn add_provider(provider: py_types::ProviderController) -> PyResult<()> {
    let core_provider = convert_provider_controller(&provider);
    get_grpc_service()?.add_provider(core_provider);
    Ok(())
}

/// Remove a provider controller by type_id
#[pyfunction]
fn remove_provider(type_id: String) -> PyResult<bool> {
    Ok(get_grpc_service()?.remove_provider(&type_id))
}

/// Update a provider controller (removes old and inserts new)
#[pyfunction]
fn update_provider(provider: py_types::ProviderController) -> PyResult<bool> {
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
fn add_function(
    provider_type_id: String,
    function_metadata: py_types::FunctionMetadata,
    invoke_callback: PyObject,
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
                let result = Python::with_gil(|py| {
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
fn remove_function(provider_type_id: String, function_name: String) -> PyResult<bool> {
    Ok(get_grpc_service()?.remove_function(&provider_type_id, &function_name))
}

/// Update a function controller (removes old and inserts new)
#[pyfunction]
fn update_function(
    provider_type_id: String,
    function_metadata: py_types::FunctionMetadata,
    invoke_callback: PyObject,
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
                let result = Python::with_gil(|py| {
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
fn add_agent(agent: py_types::Agent) -> PyResult<bool> {
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
fn remove_agent(id: String) -> PyResult<bool> {
    Ok(get_grpc_service()?.remove_agent(&id))
}

/// Update an agent (removes old and inserts new)
#[pyfunction]
fn update_agent(agent: py_types::Agent) -> PyResult<bool> {
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
fn set_secret_handler(callback: PyObject) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::SecretHandler = Arc::new(move |secrets: Vec<core_types::Secret>| {
        let callback: Arc<PyObject> = Arc::clone(&callback);
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

            let result = Python::with_gil(|py| {
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
fn set_environment_variable_handler(callback: PyObject) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::EnvironmentVariableHandler =
        Arc::new(move |env_vars: Vec<core_types::EnvironmentVariable>| {
            let callback: Arc<PyObject> = Arc::clone(&callback);
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

                let result = Python::with_gil(|py| {
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
fn set_unset_secret_handler(callback: PyObject) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::UnsetSecretHandler = Arc::new(move |key: String| {
        let callback: Arc<PyObject> = Arc::clone(&callback);
        info!("Unset secret handler invoked with key: {}", key);

        Box::pin(async move {
            info!(
                "Calling Python unset secret handler callback with key: {}",
                key
            );

            let result = Python::with_gil(|py| {
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
fn set_unset_environment_variable_handler(callback: PyObject) -> PyResult<()> {
    let callback = Arc::new(callback);

    let handler: core_types::UnsetEnvironmentVariableHandler = Arc::new(move |key: String| {
        let callback: Arc<PyObject> = Arc::clone(&callback);
        info!(
            "Unset environment variable handler invoked with key: {}",
            key
        );

        Box::pin(async move {
            info!(
                "Calling Python unset environment variable handler callback with key: {}",
                key
            );

            let result = Python::with_gil(|py| {
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
#[pyo3(signature = (base_url=None))]
fn resync_sdk(base_url: Option<String>) -> PyResult<()> {
    get_runtime()
        .block_on(async { core_types::resync_sdk(base_url).await })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

    Ok(())
}

/// A Python module implemented in Rust for the Soma SDK.
#[pymodule]
fn sdk_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(start_grpc_server, m)?)?;
    m.add_function(wrap_pyfunction!(add_provider, m)?)?;
    m.add_function(wrap_pyfunction!(remove_provider, m)?)?;
    m.add_function(wrap_pyfunction!(update_provider, m)?)?;
    m.add_function(wrap_pyfunction!(add_function, m)?)?;
    m.add_function(wrap_pyfunction!(remove_function, m)?)?;
    m.add_function(wrap_pyfunction!(update_function, m)?)?;
    m.add_function(wrap_pyfunction!(add_agent, m)?)?;
    m.add_function(wrap_pyfunction!(remove_agent, m)?)?;
    m.add_function(wrap_pyfunction!(update_agent, m)?)?;
    m.add_function(wrap_pyfunction!(set_secret_handler, m)?)?;
    m.add_function(wrap_pyfunction!(set_environment_variable_handler, m)?)?;
    m.add_function(wrap_pyfunction!(set_unset_secret_handler, m)?)?;
    m.add_function(wrap_pyfunction!(set_unset_environment_variable_handler, m)?)?;
    m.add_function(wrap_pyfunction!(resync_sdk, m)?)?;

    // Types
    m.add_class::<py_types::Agent>()?;
    m.add_class::<py_types::ProviderController>()?;
    m.add_class::<py_types::FunctionController>()?;
    m.add_class::<py_types::ProviderCredentialController>()?;
    m.add_class::<py_types::Oauth2AuthorizationCodeFlowConfiguration>()?;
    m.add_class::<py_types::Oauth2JwtBearerAssertionFlowConfiguration>()?;
    m.add_class::<py_types::Oauth2AuthorizationCodeFlowStaticCredentialConfiguration>()?;
    m.add_class::<py_types::Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration>()?;
    m.add_class::<py_types::Metadata>()?;
    m.add_class::<py_types::InvokeFunctionRequest>()?;
    m.add_class::<py_types::InvokeFunctionResponse>()?;
    m.add_class::<py_types::CallbackError>()?;
    m.add_class::<py_types::Secret>()?;
    m.add_class::<py_types::SetSecretsResponse>()?;
    m.add_class::<py_types::SetSecretsSuccess>()?;
    m.add_class::<py_types::EnvironmentVariable>()?;
    m.add_class::<py_types::SetEnvironmentVariablesResponse>()?;
    m.add_class::<py_types::SetEnvironmentVariablesSuccess>()?;
    m.add_class::<py_types::UnsetSecretResponse>()?;
    m.add_class::<py_types::UnsetSecretSuccess>()?;
    m.add_class::<py_types::UnsetEnvironmentVariableResponse>()?;
    m.add_class::<py_types::UnsetEnvironmentVariableSuccess>()?;
    m.add_class::<py_types::FunctionMetadata>()?;

    Ok(())
}
