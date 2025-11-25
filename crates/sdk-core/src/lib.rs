pub mod types;
mod unix_socket;

use arc_swap::ArcSwap;
use shared::error::CommonError;
use tracing_subscriber::EnvFilter;
pub use types::*;
use unix_socket::{bind_unix_listener, create_listener_stream};

use sdk_proto::soma_sdk_service_server::{SomaSdkService, SomaSdkServiceServer};
use std::{path::PathBuf, sync::Arc};
use tonic::{Request, Response, Status, transport::Server};
use tracing::info;

pub type GenerateBridgeClientResponse = sdk_proto::GenerateBridgeClientResponse;
pub type GenerateBridgeClientRequest = sdk_proto::GenerateBridgeClientRequest;

/// Trait for SDK-specific code generation (TypeScript, Python, etc.)
#[tonic::async_trait]
pub trait SdkCodeGenerator: Send + Sync {
    /// Generate bridge client code from function instance metadata
    async fn generate_bridge_client(
        &self,
        request: GenerateBridgeClientRequest,
    ) -> Result<GenerateBridgeClientResponse, CommonError>;
}

pub struct GrpcService<G: SdkCodeGenerator> {
    providers: ArcSwap<Vec<ProviderController>>,
    agents: ArcSwap<Vec<Agent>>,
    code_generator: Arc<G>,
    secret_handler: ArcSwap<Option<SecretHandler>>,
}

#[tonic::async_trait]
impl<G: SdkCodeGenerator + 'static> SomaSdkService for GrpcService<G> {
    async fn metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<sdk_proto::MetadataResponse>, Status> {
        let providers = self.providers.load();
        let proto_providers: Vec<sdk_proto::ProviderController> =
            providers.iter().map(Into::into).collect();

        let agents = self.agents.load();
        let proto_agents: Vec<sdk_proto::Agent> = agents.iter().cloned().map(Into::into).collect();

        let response = sdk_proto::MetadataResponse {
            bridge_providers: proto_providers,
            agents: proto_agents,
        };

        Ok(Response::new(response))
    }

    async fn health_check(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn invoke_function(
        &self,
        request: Request<sdk_proto::InvokeFunctionRequest>,
    ) -> Result<Response<sdk_proto::InvokeFunctionResponse>, Status> {
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

        info!("invoking function: {:?}", function.name);

        // Invoke the function (Arc keeps providers alive during the call)
        let result = (function.invoke)(req)
            .await
            .map_err(|e| Status::internal(format!("Function invocation failed: {e}")));

        info!("invoke_function result: {:?}", result);

        let result = result?;

        Ok(Response::new(result.into()))
    }

    async fn generate_bridge_client(
        &self,
        request: Request<sdk_proto::GenerateBridgeClientRequest>,
    ) -> Result<Response<sdk_proto::GenerateBridgeClientResponse>, Status> {
        info!("generate_bridge_client called - delegating to code generator");

        let req = request.into_inner();
        match self.code_generator.generate_bridge_client(req).await {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => {
                info!("Code generator returned error: {}", e);
                Ok(Response::new(sdk_proto::GenerateBridgeClientResponse {
                    result: Some(sdk_proto::generate_bridge_client_response::Result::Error(
                        sdk_proto::GenerateBridgeClientError {
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
        info!(
            "set_secrets called with {} secrets",
            request.get_ref().secrets.len()
        );

        let proto_req = request.into_inner();
        let secrets: Vec<Secret> = proto_req.secrets.into_iter().map(Into::into).collect();

        // Get the secret handler
        let handler_guard = self.secret_handler.load();
        let handler = match handler_guard.as_ref() {
            Some(h) => h.clone(),
            None => {
                info!("No secret handler registered");
                return Ok(Response::new(sdk_proto::SetSecretsResponse {
                    success: false,
                    message: "No secret handler registered".to_string(),
                }));
            }
        };

        // Call the handler
        match handler(secrets).await {
            Ok(response) => {
                info!("set_secrets completed successfully: {}", response.message);
                Ok(Response::new(response.into()))
            }
            Err(e) => {
                info!("set_secrets failed: {}", e);
                Ok(Response::new(sdk_proto::SetSecretsResponse {
                    success: false,
                    message: format!("Failed to set secrets: {e}"),
                }))
            }
        }
    }
}

impl<G: SdkCodeGenerator + 'static> GrpcService<G> {
    pub fn new(providers: Vec<ProviderController>, agents: Vec<Agent>, code_generator: G) -> Self {
        Self {
            providers: ArcSwap::from_pointee(providers),
            agents: ArcSwap::from_pointee(agents),
            code_generator: Arc::new(code_generator),
            secret_handler: ArcSwap::from_pointee(None),
        }
    }

    /// Set the secret handler callback that will be invoked when secrets are synced
    pub fn set_secret_handler(&self, handler: SecretHandler) {
        self.secret_handler.store(Arc::new(Some(handler)));
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

    /// Update a function controller (removes old and inserts new)
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
                    provider.functions.remove(pos);
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
}

/// Starts a gRPC server that handles function invocations over a Unix socket
///
/// # Arguments
/// * `providers` - Array of ProviderController definitions with function implementations
/// * `socket_path` - Path to the Unix socket (e.g., "/tmp/soma-sdk.sock")
/// * `code_generator` - Implementation of SdkCodeGenerator for bridge client generation
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

    info!("Starting gRPC server on Unix socket: {:?}", socket_path);

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

    async fn generate_bridge_client(
        &self,
        request: Request<sdk_proto::GenerateBridgeClientRequest>,
    ) -> Result<Response<sdk_proto::GenerateBridgeClientResponse>, Status> {
        self.0.generate_bridge_client(request).await
    }

    async fn set_secrets(
        &self,
        request: Request<sdk_proto::SetSecretsRequest>,
    ) -> Result<Response<sdk_proto::SetSecretsResponse>, Status> {
        self.0.set_secrets(request).await
    }
}
