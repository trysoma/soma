pub mod types;
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use shared::error::CommonError;
use tracing_subscriber::EnvFilter;
pub use types::*;

use sdk_proto::soma_sdk_service_server::{SomaSdkService, SomaSdkServiceServer};
use std::{path::PathBuf, sync::Arc};
use tonic::{Request, Response, Status, transport::Server};
use tracing::info;

pub struct GrpcService {
    providers: ArcSwap<Vec<ProviderController>>,
    agents: ArcSwap<Vec<Agent>>,
}

#[tonic::async_trait]
impl SomaSdkService for GrpcService {
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
}

impl GrpcService {
    pub fn new(providers: Vec<ProviderController>, agents: Vec<Agent>) -> Self {
        Self {
            providers: ArcSwap::from_pointee(providers),
            agents: ArcSwap::from_pointee(agents),
        }
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

static GRPC_SERVICE: Lazy<GrpcService> = Lazy::new(|| GrpcService::new(vec![], vec![]));

/// Starts a gRPC server that handles function invocations over a Unix socket
///
/// # Arguments
/// * `providers` - Array of ProviderController definitions with function implementations
/// * `socket_path` - Path to the Unix socket (e.g., "/tmp/soma-sdk.sock")
///
/// # Example
/// Each FunctionController must have an `invoke` function that handles the invocation.
pub async fn start_grpc_server(
    providers: Vec<ProviderController>,
    socket_path: PathBuf,
) -> Result<(), CommonError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // Set the providers in the global service
    GRPC_SERVICE.set_providers(providers);

    // Remove existing socket file if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .map_err(|e| anyhow::anyhow!("Failed to remove existing socket: {e}"))?;
    }

    info!("Starting gRPC server on Unix socket: {:?}", socket_path);

    // Create Unix socket listener
    let uds = tokio::net::UnixListener::bind(&socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to bind Unix socket: {e}"))?;

    let incoming = tokio_stream::wrappers::UnixListenerStream::new(uds);

    // Create a wrapper service that uses the global GRPC_SERVICE
    let service = GrpcServiceWrapper;

    Server::builder()
        .add_service(SomaSdkServiceServer::new(service))
        .serve_with_incoming(incoming)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start server: {e}"))?;

    Ok(())
}

/// Wrapper struct that delegates to the global GRPC_SERVICE
struct GrpcServiceWrapper;

#[tonic::async_trait]
impl SomaSdkService for GrpcServiceWrapper {
    async fn metadata(
        &self,
        request: Request<()>,
    ) -> Result<Response<sdk_proto::MetadataResponse>, Status> {
        GRPC_SERVICE.metadata(request).await
    }

    async fn health_check(&self, request: Request<()>) -> Result<Response<()>, Status> {
        GRPC_SERVICE.health_check(request).await
    }

    async fn invoke_function(
        &self,
        request: Request<sdk_proto::InvokeFunctionRequest>,
    ) -> Result<Response<sdk_proto::InvokeFunctionResponse>, Status> {
        GRPC_SERVICE.invoke_function(request).await
    }
}

/// Get a reference to the global GRPC service for dynamic provider management
pub fn get_grpc_service() -> &'static GrpcService {
    &GRPC_SERVICE
}
