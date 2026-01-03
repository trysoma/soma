pub mod types;
mod unix_socket;

use arc_swap::ArcSwap;
use shared::error::CommonError;
use std::{path::PathBuf, sync::Arc};
use tracing_subscriber::EnvFilter;
pub use types::*;
use unix_socket::{bind_unix_listener, create_listener_stream};

use sdk_proto::soma_sdk_service_server::{SomaSdkService, SomaSdkServiceServer};
use tonic::{Request, Response, Status, transport::Server};
use tracing::{debug, trace};

pub type GenerateMcpClientResponse = sdk_proto::GenerateMcpClientResponse;
pub type GenerateMcpClientRequest = sdk_proto::GenerateMcpClientRequest;

/// Trait for SDK-specific code generation (TypeScript, Python, etc.)
#[tonic::async_trait]
pub trait SdkCodeGenerator: Send + Sync {
    /// Generate mcp client code from function instance metadata
    async fn generate_mcp_client(
        &self,
        request: GenerateMcpClientRequest,
    ) -> Result<GenerateMcpClientResponse, CommonError>;
}

pub struct GrpcService<G: SdkCodeGenerator> {
    providers: ArcSwap<Vec<ProviderController>>,
    agents: ArcSwap<Vec<Agent>>,
    code_generator: Arc<G>,
    secret_handler: ArcSwap<Option<SecretHandler>>,
    environment_variable_handler: ArcSwap<Option<EnvironmentVariableHandler>>,
    unset_secret_handler: ArcSwap<Option<UnsetSecretHandler>>,
    unset_environment_variable_handler: ArcSwap<Option<UnsetEnvironmentVariableHandler>>,
}

#[tonic::async_trait]
impl<G: SdkCodeGenerator + 'static> SomaSdkService for GrpcService<G> {
    async fn metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<sdk_proto::MetadataResponse>, Status> {
        trace!("Getting SDK metadata");
        let providers = self.providers.load();
        let proto_providers: Vec<sdk_proto::ProviderController> =
            providers.iter().map(Into::into).collect();

        let agents = self.agents.load();
        let proto_agents: Vec<sdk_proto::Agent> = agents.iter().cloned().map(Into::into).collect();

        let provider_count = proto_providers.len();
        let agent_count = proto_agents.len();

        let response = sdk_proto::MetadataResponse {
            mcp_providers: proto_providers,
            agents: proto_agents,
        };

        trace!(
            provider_count,
            agent_count, "Getting SDK metadata completed"
        );
        Ok(Response::new(response))
    }

    async fn health_check(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        trace!("Health check");
        trace!("Health check completed");
        Ok(Response::new(()))
    }

    async fn invoke_function(
        &self,
        request: Request<sdk_proto::InvokeFunctionRequest>,
    ) -> Result<Response<sdk_proto::InvokeFunctionResponse>, Status> {
        trace!("Invoking function");
        let proto_req = request.into_inner();
        let req: InvokeFunctionRequest =
            proto_req
                .try_into()
                .map_err(|e: shared::error::CommonError| {
                    Status::invalid_argument(format!("Invalid request: {e}"))
                })?;

        let providers = self.providers.load();

        // Find the provider and function
        let provider = providers
            .iter()
            .find(|p| p.type_id == req.provider_controller_type_id)
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Provider not found: {}",
                    req.provider_controller_type_id
                ))
            })?;

        let function = provider
            .functions
            .iter()
            .find(|f| f.name == req.function_controller_type_id)
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Function not found: {}",
                    req.function_controller_type_id
                ))
            })?;

        trace!(function = %function.name, provider = %provider.type_id, "Executing function");

        // Invoke the function (Arc keeps providers alive during the call)
        let result = (function.invoke)(req)
            .await
            .map_err(|e| Status::internal(format!("Function invocation failed: {e}")));

        trace!(success = result.is_ok(), "Invoking function completed");

        let result = result?;

        Ok(Response::new(result.into()))
    }

    async fn generate_mcp_client(
        &self,
        request: Request<sdk_proto::GenerateMcpClientRequest>,
    ) -> Result<Response<sdk_proto::GenerateMcpClientResponse>, Status> {
        trace!("Generating MCP client");

        let req = request.into_inner();

        match self.code_generator.generate_mcp_client(req).await {
            Ok(response) => {
                trace!("Generating MCP client completed");
                Ok(Response::new(response))
            }
            Err(e) => {
                debug!(error = %e, "Code generation failed");
                trace!("Generating MCP client completed (with error)");
                Ok(Response::new(sdk_proto::GenerateMcpClientResponse {
                    result: Some(sdk_proto::generate_mcp_client_response::Result::Error(
                        sdk_proto::GenerateMcpClientError {
                            message: e.to_string(),
                        },
                    )),
                }))
            }
        }
    }

    async fn set_secrets(
        &self,
        request: Request<sdk_proto::SetSecretsRequest>,
    ) -> Result<Response<sdk_proto::SetSecretsResponse>, Status> {
        use sdk_proto::set_secrets_response::Kind;
        let count = request.get_ref().secrets.len();
        trace!(count, "Setting secrets");

        let proto_req = request.into_inner();
        let secrets: Vec<Secret> = proto_req.secrets.into_iter().map(Into::into).collect();

        // Get the secret handler
        let handler_guard = self.secret_handler.load();
        let handler = match handler_guard.as_ref() {
            Some(h) => h.clone(),
            None => {
                debug!("No secret handler registered");
                trace!(count, "Setting secrets completed (no handler)");
                return Ok(Response::new(sdk_proto::SetSecretsResponse {
                    kind: Some(Kind::Error(sdk_proto::CallbackError {
                        message: "No secret handler registered".to_string(),
                    })),
                }));
            }
        };
        // Call the handler
        trace!("Invoking secret handler");

        let result = handler(secrets)
            .await
            .map_err(|e| Status::internal(format!("Function invocation failed: {e}")));

        trace!(count, success = result.is_ok(), "Setting secrets completed");

        let result = result?;

        Ok(Response::new(result.into()))
    }

    async fn set_environment_variables(
        &self,
        request: Request<sdk_proto::SetEnvironmentVariablesRequest>,
    ) -> Result<Response<sdk_proto::SetEnvironmentVariablesResponse>, Status> {
        use sdk_proto::set_environment_variables_response::Kind;
        let count = request.get_ref().environment_variables.len();
        trace!(count, "Setting environment variables");

        let proto_req = request.into_inner();
        let env_vars: Vec<EnvironmentVariable> = proto_req
            .environment_variables
            .into_iter()
            .map(Into::into)
            .collect();

        // Get the environment variable handler
        let handler_guard = self.environment_variable_handler.load();
        let handler = match handler_guard.as_ref() {
            Some(h) => h.clone(),
            None => {
                debug!("No environment variable handler registered");
                trace!(
                    count,
                    "Setting environment variables completed (no handler)"
                );
                return Ok(Response::new(sdk_proto::SetEnvironmentVariablesResponse {
                    kind: Some(Kind::Error(sdk_proto::CallbackError {
                        message: "No environment variable handler registered".to_string(),
                    })),
                }));
            }
        };
        // Call the handler
        trace!("Invoking environment variable handler");

        let result = handler(env_vars)
            .await
            .map_err(|e| Status::internal(format!("Function invocation failed: {e}")));

        trace!(
            count,
            success = result.is_ok(),
            "Setting environment variables completed"
        );

        let result = result?;

        Ok(Response::new(result.into()))
    }

    async fn unset_secrets(
        &self,
        request: Request<sdk_proto::UnsetSecretRequest>,
    ) -> Result<Response<sdk_proto::UnsetSecretResponse>, Status> {
        use sdk_proto::unset_secret_response::Kind;
        let key = request.get_ref().key.clone();
        trace!(key = %key, "Unsetting secret");

        let proto_req = request.into_inner();
        let req: UnsetSecretRequest = proto_req.into();

        // Get the unset secret handler
        let handler_guard = self.unset_secret_handler.load();
        let handler = match handler_guard.as_ref() {
            Some(h) => h.clone(),
            None => {
                debug!("No unset secret handler registered");
                trace!(key = %key, "Unsetting secret completed (no handler)");
                return Ok(Response::new(sdk_proto::UnsetSecretResponse {
                    kind: Some(Kind::Error(sdk_proto::CallbackError {
                        message: "No unset secret handler registered".to_string(),
                    })),
                }));
            }
        };
        // Call the handler
        trace!("Invoking unset secret handler");

        let result = handler(req.key)
            .await
            .map_err(|e| Status::internal(format!("Function invocation failed: {e}")));

        trace!(key = %key, success = result.is_ok(), "Unsetting secret completed");

        let result = result?;

        Ok(Response::new(result.into()))
    }

    async fn unset_environment_variables(
        &self,
        request: Request<sdk_proto::UnsetEnvironmentVariableRequest>,
    ) -> Result<Response<sdk_proto::UnsetEnvironmentVariableResponse>, Status> {
        use sdk_proto::unset_environment_variable_response::Kind;
        let key = request.get_ref().key.clone();
        trace!(key = %key, "Unsetting environment variable");

        let proto_req = request.into_inner();
        let req: UnsetEnvironmentVariableRequest = proto_req.into();

        // Get the unset environment variable handler
        let handler_guard = self.unset_environment_variable_handler.load();
        let handler = match handler_guard.as_ref() {
            Some(h) => h.clone(),
            None => {
                debug!("No unset environment variable handler registered");
                trace!(key = %key, "Unsetting environment variable completed (no handler)");
                return Ok(Response::new(sdk_proto::UnsetEnvironmentVariableResponse {
                    kind: Some(Kind::Error(sdk_proto::CallbackError {
                        message: "No unset environment variable handler registered".to_string(),
                    })),
                }));
            }
        };
        // Call the handler
        trace!("Invoking unset environment variable handler");

        let result = handler(req.key)
            .await
            .map_err(|e| Status::internal(format!("Function invocation failed: {e}")));

        trace!(key = %key, success = result.is_ok(), "Unsetting environment variable completed");

        let result = result?;

        Ok(Response::new(result.into()))
    }
}

impl<G: SdkCodeGenerator + 'static> GrpcService<G> {
    pub fn new(providers: Vec<ProviderController>, agents: Vec<Agent>, code_generator: G) -> Self {
        Self {
            providers: ArcSwap::from_pointee(providers),
            agents: ArcSwap::from_pointee(agents),
            code_generator: Arc::new(code_generator),
            secret_handler: ArcSwap::from_pointee(None),
            environment_variable_handler: ArcSwap::from_pointee(None),
            unset_secret_handler: ArcSwap::from_pointee(None),
            unset_environment_variable_handler: ArcSwap::from_pointee(None),
        }
    }

    /// Set the secret handler callback that will be invoked when secrets are synced
    pub fn set_secret_handler(&self, handler: SecretHandler) {
        self.secret_handler.store(Arc::new(Some(handler)));
    }

    /// Set the environment variable handler callback that will be invoked when environment variables are synced
    pub fn set_environment_variable_handler(&self, handler: EnvironmentVariableHandler) {
        self.environment_variable_handler
            .store(Arc::new(Some(handler)));
    }

    /// Set the unset secret handler callback that will be invoked when a secret is unset
    pub fn set_unset_secret_handler(&self, handler: UnsetSecretHandler) {
        self.unset_secret_handler.store(Arc::new(Some(handler)));
    }

    /// Set the unset environment variable handler callback that will be invoked when an environment variable is unset
    pub fn set_unset_environment_variable_handler(&self, handler: UnsetEnvironmentVariableHandler) {
        self.unset_environment_variable_handler
            .store(Arc::new(Some(handler)));
    }

    /// Add a new provider controller
    pub fn add_provider(&self, provider: ProviderController) {
        self.providers.rcu(|current| {
            let mut new_providers = (**current).clone();
            new_providers.push(provider.clone());
            new_providers
        });
    }

    /// Remove a provider controller by type_id
    pub fn remove_provider(&self, type_id: &str) -> bool {
        let mut removed = false;
        self.providers.rcu(|current| {
            let initial_len = current.len();
            let new_providers: Vec<ProviderController> = current
                .iter()
                .filter(|p| p.type_id != type_id)
                .cloned()
                .collect();
            removed = new_providers.len() != initial_len;
            new_providers
        });
        removed
    }

    /// Update a provider controller (removes old and inserts new)
    pub fn update_provider(&self, provider: ProviderController) -> bool {
        let mut updated = false;
        self.providers.rcu(|current| {
            let mut new_providers = (**current).clone();
            if let Some(pos) = new_providers
                .iter()
                .position(|p| p.type_id == provider.type_id)
            {
                new_providers.remove(pos);
                new_providers.push(provider.clone());
                updated = true;
            }
            new_providers
        });
        updated
    }

    /// Add a function controller to a specific provider
    pub fn add_function(&self, provider_type_id: &str, function: FunctionController) -> bool {
        let mut added = false;
        self.providers.rcu(|current| {
            let mut new_providers = (**current).clone();
            if let Some(provider) = new_providers
                .iter_mut()
                .find(|p| p.type_id == provider_type_id)
            {
                provider.functions.push(function.clone());
                added = true;
            }
            new_providers
        });
        added
    }

    /// Remove a function controller from a specific provider
    pub fn remove_function(&self, provider_type_id: &str, function_name: &str) -> bool {
        let mut removed = false;
        self.providers.rcu(|current| {
            let mut new_providers = (**current).clone();
            if let Some(provider) = new_providers
                .iter_mut()
                .find(|p| p.type_id == provider_type_id)
            {
                let initial_len = provider.functions.len();
                provider.functions.retain(|f| f.name != function_name);
                removed = provider.functions.len() != initial_len;
            }
            new_providers
        });
        removed
    }

    /// Update a function controller (removes old and inserts new), or add if it doesn't exist
    pub fn update_function(&self, provider_type_id: &str, function: FunctionController) -> bool {
        let mut updated = false;
        self.providers.rcu(|current| {
            let mut new_providers = (**current).clone();
            if let Some(provider) = new_providers
                .iter_mut()
                .find(|p| p.type_id == provider_type_id)
            {
                if let Some(pos) = provider
                    .functions
                    .iter()
                    .position(|f| f.name == function.name)
                {
                    // Function exists, replace it
                    provider.functions.remove(pos);
                    provider.functions.push(function.clone());
                    updated = true;
                } else {
                    // Function doesn't exist, add it (upsert behavior)
                    provider.functions.push(function.clone());
                    updated = true;
                }
            }
            new_providers
        });
        updated
    }

    /// Add a new agent
    pub fn add_agent(&self, agent: Agent) -> bool {
        let mut added = false;
        self.agents.rcu(|current| {
            let mut new_agents = (**current).clone();
            new_agents.push(agent.clone());
            added = true;
            new_agents
        });
        added
    }

    /// Remove an agent by id
    pub fn remove_agent(&self, id: &str) -> bool {
        let removed = false;
        self.agents.rcu(|current| {
            let mut new_agents = (**current).clone();
            new_agents.retain(|a| a.id != id);
            new_agents
        });
        removed
    }

    /// Update an agent (removes old and inserts new)
    pub fn update_agent(&self, agent: Agent) -> bool {
        let mut updated = false;
        self.agents.rcu(|current| {
            let mut new_agents = (**current).clone();
            if let Some(pos) = new_agents.iter().position(|a| a.id == agent.id) {
                new_agents.remove(pos);
                new_agents.push(agent.clone());
                updated = true;
            }
            new_agents
        });
        updated
    }
    /// Replace all providers
    pub fn set_providers(&self, providers: Vec<ProviderController>) {
        self.providers.store(Arc::new(providers));
    }

    /// Get a provider by type_id
    pub fn get_provider(&self, type_id: &str) -> Option<ProviderController> {
        self.providers
            .load()
            .iter()
            .find(|p| p.type_id == type_id)
            .cloned()
    }

    /// Clears all providers, agents, and handlers from the service.
    /// This effectively resets the service state without shutting down the gRPC server.
    pub fn clear(&self) {
        self.providers.store(Arc::new(vec![]));
        self.agents.store(Arc::new(vec![]));
        self.secret_handler.store(Arc::new(None));
        self.environment_variable_handler.store(Arc::new(None));
        self.unset_secret_handler.store(Arc::new(None));
        self.unset_environment_variable_handler
            .store(Arc::new(None));
    }
}

/// Starts a gRPC server that handles function invocations over a Unix socket
///
/// # Arguments
/// * `providers` - Array of ProviderController definitions with function implementations
/// * `socket_path` - Path to the Unix socket (e.g., "/tmp/soma-sdk.sock")
/// * `code_generator` - Implementation of SdkCodeGenerator for mcp client generation
///
/// # Returns
/// A handle to the GrpcService for dynamic provider/function management
///
/// # Example
/// Each FunctionController must have an `invoke` function that handles the invocation.
pub async fn start_grpc_server<G: SdkCodeGenerator + 'static>(
    providers: Vec<ProviderController>,
    socket_path: PathBuf,
    code_generator: G,
) -> Result<Arc<GrpcService<G>>, CommonError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // Remove existing socket file if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .map_err(|e| anyhow::anyhow!("Failed to remove existing socket: {e}"))?;
    }

    debug!(socket = ?socket_path, "Starting gRPC server");

    // Create the gRPC service with code generator
    let service = Arc::new(GrpcService::new(providers, vec![], code_generator));
    let service_clone = Arc::clone(&service);

    // Spawn the server in a background task
    tokio::spawn(async move {
        // Create Unix socket listener (platform-specific)
        let uds = match bind_unix_listener(&socket_path).await {
            Ok(uds) => uds,
            Err(e) => {
                tracing::error!("Failed to bind Unix socket: {e}");
                return;
            }
        };

        let incoming = create_listener_stream(uds);

        if let Err(e) = Server::builder()
            .add_service(SomaSdkServiceServer::new(GrpcServiceWrapper(service_clone)))
            .serve_with_incoming(incoming)
            .await
        {
            tracing::error!("gRPC server error: {e}");
        }
    });

    Ok(service)
}

/// Wrapper to allow Arc<GrpcService> to implement SomaSdkService
struct GrpcServiceWrapper<G: SdkCodeGenerator>(Arc<GrpcService<G>>);

#[tonic::async_trait]
impl<G: SdkCodeGenerator + 'static> SomaSdkService for GrpcServiceWrapper<G> {
    async fn metadata(
        &self,
        request: Request<()>,
    ) -> Result<Response<sdk_proto::MetadataResponse>, Status> {
        self.0.metadata(request).await
    }

    async fn health_check(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.0.health_check(request).await
    }

    async fn invoke_function(
        &self,
        request: Request<sdk_proto::InvokeFunctionRequest>,
    ) -> Result<Response<sdk_proto::InvokeFunctionResponse>, Status> {
        self.0.invoke_function(request).await
    }

    async fn generate_mcp_client(
        &self,
        request: Request<sdk_proto::GenerateMcpClientRequest>,
    ) -> Result<Response<sdk_proto::GenerateMcpClientResponse>, Status> {
        self.0.generate_mcp_client(request).await
    }

    async fn set_secrets(
        &self,
        request: Request<sdk_proto::SetSecretsRequest>,
    ) -> Result<Response<sdk_proto::SetSecretsResponse>, Status> {
        self.0.set_secrets(request).await
    }

    async fn set_environment_variables(
        &self,
        request: Request<sdk_proto::SetEnvironmentVariablesRequest>,
    ) -> Result<Response<sdk_proto::SetEnvironmentVariablesResponse>, Status> {
        self.0.set_environment_variables(request).await
    }

    async fn unset_secrets(
        &self,
        request: Request<sdk_proto::UnsetSecretRequest>,
    ) -> Result<Response<sdk_proto::UnsetSecretResponse>, Status> {
        self.0.unset_secrets(request).await
    }

    async fn unset_environment_variables(
        &self,
        request: Request<sdk_proto::UnsetEnvironmentVariableRequest>,
    ) -> Result<Response<sdk_proto::UnsetEnvironmentVariableResponse>, Status> {
        self.0.unset_environment_variables(request).await
    }
}

/// Response from resync_sdk operation
#[derive(Debug, Clone)]
pub struct ResyncSdkResponse {}

/// Calls the internal resync endpoint on the Soma API server.
/// This triggers the API server to:
/// - Fetch metadata from the SDK (providers, agents)
/// - Sync providers to the mcp registry
/// - Register Restate deployments for agents
/// - Sync secrets to the SDK
/// - Sync environment variables to the SDK
///
/// # Arguments
/// * `base_url` - Optional base URL of the Soma API server (defaults to SOMA_SERVER_BASE_URL env var or http://localhost:3000)
///
/// # Returns
/// The resync response from the server
pub async fn resync_sdk(base_url: Option<String>) -> Result<ResyncSdkResponse, CommonError> {
    let api_base_url = base_url
        .or_else(|| std::env::var("SOMA_SERVER_BASE_URL").ok())
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    trace!(url = %api_base_url, "Calling SDK resync endpoint");

    let config = soma_api_client::apis::configuration::Configuration {
        base_path: api_base_url.clone(),
        ..Default::default()
    };

    let result = soma_api_client::apis::internal_api::resync_sdk(&config)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Resync failed: {e:?}")));

    trace!(
        success = result.is_ok(),
        "Calling SDK resync endpoint completed"
    );
    result?;

    Ok(ResyncSdkResponse {})
}
