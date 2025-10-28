use pyo3::prelude::*;
use sdk_core::{
    start_grpc_server, FunctionController, InvokeFunctionRequest, InvokeFunctionResponse,
    ProviderController, ProviderCredentialController,
};
use std::path::PathBuf;
use std::sync::Arc;

#[pyclass]
struct InvocationRequest {
    #[pyo3(get)]
    provider_controller_type_id: String,
    #[pyo3(get)]
    function_controller_type_id: String,
    #[pyo3(get)]
    credential_controller_type_id: String,
    #[pyo3(get)]
    credentials: String,
    #[pyo3(get)]
    parameters: String,
}

#[pyclass]
struct InvocationResponse {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    data: Option<String>,
    #[pyo3(get)]
    error: Option<String>,
}

/// Start the gRPC server with the given providers over a Unix socket
///
/// This is a simplified example. In a real implementation, you would:
/// 1. Parse the providers configuration
/// 2. Create FunctionController instances with actual implementations
/// 3. Register the providers with the server
#[pyfunction]
fn start_sdk_server(socket_path: String) -> PyResult<()> {
    // Example: Create a simple provider with a test function
    let providers = vec![ProviderController {
        type_id: "example_provider".to_string(),
        name: "Example Provider".to_string(),
        documentation: "Example provider for testing".to_string(),
        categories: vec!["example".to_string()],
        functions: vec![FunctionController {
            name: "example_function".to_string(),
            description: "Example function".to_string(),
            parameters: "{}".to_string(),
            output: "{}".to_string(),
            invoke: Arc::new(|req: InvokeFunctionRequest| {
                Box::pin(async move {
                    Ok(InvokeFunctionResponse {
                        result: Ok(format!(
                            "{{\"message\": \"Hello from {}\", \"params\": {}}}",
                            req.function_controller_type_id, req.parameters
                        )),
                    })
                })
            }),
        }],
        credential_controllers: vec![ProviderCredentialController::NoAuth],
    }];

    let path = PathBuf::from(socket_path);

    // Start the gRPC server in a new tokio runtime
    let runtime = tokio::runtime::Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create runtime: {}",
            e
        ))
    })?;

    runtime.block_on(async move {
        start_grpc_server(providers, path)
            .await
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to start server: {}",
                    e
                ))
            })
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn sdk_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_sdk_server, m)?)?;
    m.add_class::<InvocationRequest>()?;
    m.add_class::<InvocationResponse>()?;
    Ok(())
}
