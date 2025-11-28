use crate::error::CommonError;
use std::path::Path;

use hyper_util::rt::TokioIo;
use sdk_proto::soma_sdk_service_client::SomaSdkServiceClient;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

/// Default Unix socket path for the SDK gRPC server
pub const DEFAULT_SOMA_SERVER_SOCK: &str = "/tmp/soma-sdk.sock";

pub async fn create_soma_unix_socket_client(
    socket_path: &str,
) -> Result<SomaSdkServiceClient<tonic::transport::Channel>, CommonError> {
    let channel = create_unix_socket_client(socket_path).await?;
    Ok(SomaSdkServiceClient::new(channel))
}

/// Create a gRPC client connected to a Unix socket
pub async fn create_unix_socket_client(
    socket_path: &str,
) -> Result<tonic::transport::Channel, CommonError> {
    // Convert to String to avoid lifetime issues
    let socket_path = socket_path.to_string();

    // Create a channel that connects to the Unix socket
    let channel = Endpoint::try_from("http://[::]:50051")
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create endpoint: {e}")))?
        .connect_with_connector(service_fn(move |_: Uri| {
            let socket_path = socket_path.clone();
            async move {
                let stream = connect_unix_stream(&socket_path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to connect to Unix socket: {e}"))
        })?;

    Ok(channel)
}

/// Establish connection with retry logic
pub async fn establish_connection_with_retry(socket_path: &str) -> Result<(), CommonError> {
    use tokio::time::{Duration, interval};

    let mut ticker = interval(Duration::from_millis(500));
    let max_attempts = 20; // 10 seconds total
    let mut attempts = 0;

    loop {
        ticker.tick().await;

        // Check if socket exists
        if !Path::new(socket_path).exists() {
            attempts += 1;
            if attempts >= max_attempts {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "Socket file does not exist after {max_attempts} attempts: {socket_path}"
                )));
            }
            continue;
        }

        // Try to create client
        match create_unix_socket_client(socket_path).await {
            Ok(_) => {
                info!("Successfully connected to SDK server");
                return Ok(());
            }
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Failed to connect after {max_attempts} attempts: {e}"
                    )));
                }
            }
        }
    }
}

/// Monitor connection health by keeping a persistent connection and making health check calls
/// Returns when the connection is lost (assumes server restart/hot reload)
pub async fn monitor_connection_health(socket_path: &str) {
    // Create a persistent client connection
    let mut client = match create_soma_unix_socket_client(socket_path).await {
        Ok(client) => client,
        Err(_) => return, // Connection already failed
    };

    // Keep making health check calls on the same connection
    // When the server restarts, the persistent connection will break
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if !Path::new(socket_path).exists() {
            // Socket file disappeared - server is restarting
            info!("Socket file disappeared, server is restarting");
            return;
        }

        // Make an actual health check RPC call on the existing connection
        // This will fail when the server restarts because the connection is broken
        let health_request = tonic::Request::new(());
        match client.health_check(health_request).await {
            Ok(_) => {
                // Connection is still alive, continue monitoring
            }
            Err(e) => {
                // Health check failed - server likely restarted
                info!("Health check failed, server likely restarted: {:?}", e);
                return;
            }
        }
    }
}

// Platform-specific UnixStream implementation for client connections
// On Unix: uses tokio::net::UnixStream
// On Windows: uses uds_windows::UnixStream wrapped with SyncIoBridge

#[cfg(unix)]
mod unix_impl {

    use tokio::net::UnixStream as TokioUnixStream;

    pub type UnixStream = TokioUnixStream;

    pub async fn connect_unix_stream(path: &str) -> std::io::Result<UnixStream> {
        TokioUnixStream::connect(path).await
    }
}

#[cfg(windows)]
mod windows_impl {
    use tokio_util::io::SyncIoBridge;
    use uds_windows::UnixStream as UdsUnixStream;

    pub type UnixStream = SyncIoBridge<UdsUnixStream>;

    pub async fn connect_unix_stream(path: &str) -> std::io::Result<UnixStream> {
        let stream = tokio::task::spawn_blocking(move || UdsUnixStream::connect(path)).await??;
        Ok(SyncIoBridge::new(stream))
    }
}

use tracing::info;
#[cfg(unix)]
pub use unix_impl::*;

#[cfg(windows)]
pub use windows_impl::*;
