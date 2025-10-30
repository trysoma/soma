use std::path::Path;

use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use tracing::{info, error};

use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use shared::error::CommonError;

/// Create a gRPC client connected to a Unix socket
pub async fn create_unix_socket_client(
    socket_path: &str,
) -> Result<SomaSdkServiceClient<tonic::transport::Channel>, CommonError> {
    // Convert to String to avoid lifetime issues
    let socket_path = socket_path.to_string();

    // Create a channel that connects to the Unix socket
    let channel = Endpoint::try_from("http://[::]:50051")
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create endpoint: {}", e)))?
        .connect_with_connector(service_fn(move |_: Uri| {
            let socket_path = socket_path.clone();
            async move {
                let stream = UnixStream::connect(socket_path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to connect to Unix socket: {}", e)))?;

    Ok(SomaSdkServiceClient::new(channel))
}

/// Test the gRPC connection by calling the Metadata endpoint
pub async fn test_grpc_connection(socket_path: &str) -> Result<(), CommonError> {
    info!("Testing gRPC connection to socket: {}", socket_path);

    // Wait for the socket to exist with retries (up to 10 seconds)
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 20;
    const RETRY_DELAY_MS: u64 = 500;

    while !Path::new(socket_path).exists() && attempts < MAX_ATTEMPTS {
        if attempts == 0 {
            info!("Waiting for socket file to be created: {}", socket_path);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
        attempts += 1;
    }

    if !Path::new(socket_path).exists() {
        error!("Socket file does not exist after {} seconds: {}", MAX_ATTEMPTS as f64 * RETRY_DELAY_MS as f64 / 1000.0, socket_path);
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Socket file does not exist: {}",
            socket_path
        )));
    }

    info!("Socket file found after {} ms, attempting connection...", attempts * RETRY_DELAY_MS as u32);

    // Create client with retry logic
    let mut client = None;
    attempts = 0;

    while client.is_none() && attempts < MAX_ATTEMPTS {
        match create_unix_socket_client(socket_path).await {
            Ok(c) => {
                client = Some(c);
                break;
            }
            Err(e) => {
                if attempts == 0 {
                    info!("Waiting for gRPC server to be ready...");
                }
                if attempts == MAX_ATTEMPTS - 1 {
                    error!("Failed to connect to gRPC server after {} attempts: {:?}", MAX_ATTEMPTS, e);
                    return Err(e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                attempts += 1;
            }
        }
    }

    let mut client = client.ok_or_else(|| {
        CommonError::Unknown(anyhow::anyhow!("Failed to create gRPC client"))
    })?;

    info!("Connected to gRPC server successfully!");

    // Wait a bit more for providers to be registered
    // The server starts empty and providers are added via addProvider() after startup
    info!("Waiting for providers to be registered...");
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Call Metadata endpoint
    let request = tonic::Request::new(());
    let response = client
        .metadata(request)
        .await
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("gRPC call failed: {}", e)))?;

    let metadata = response.into_inner();

    info!("=== gRPC Metadata Response ===");
    info!("Provider count: {}", metadata.bridge_providers.len());

    for (i, provider) in metadata.bridge_providers.iter().enumerate() {
        info!("Provider {}: type_id={}, name={}", i + 1, provider.type_id, provider.name);
        info!("  Documentation: {}", provider.documentation);
        info!("  Categories: {:?}", provider.categories);
        info!("  Function count: {}", provider.functions.len());

        for (j, func) in provider.functions.iter().enumerate() {
            info!("    Function {}: name={}, description={}", j + 1, func.name, func.description);
            info!("      Parameters: {}", func.parameters);
            info!("      Output: {}", func.output);
        }

        info!("  Credential controller count: {}", provider.credential_controllers.len());
    }

    info!("=== End gRPC Metadata Response ===");

    Ok(())
}
