use std::path::Path;

use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use tracing::info;

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
        .map_err(|e| CommonError::Unknown(anyhow::anyhow!("Failed to create endpoint: {e}")))?
        .connect_with_connector(service_fn(move |_: Uri| {
            let socket_path = socket_path.clone();
            async move {
                let stream = UnixStream::connect(socket_path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to connect to Unix socket: {e}"))
        })?;

    Ok(SomaSdkServiceClient::new(channel))
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

/// Monitor connection health by keeping the socket connection alive
/// Returns when the connection is lost (assumes server restart/hot reload)
pub async fn monitor_connection_health(socket_path: &str) {
    // Simply check if the socket file exists - when the server restarts, it will be removed briefly
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if !Path::new(socket_path).exists() {
            // Socket file disappeared - server is restarting
            return;
        }

        // Try a simple health check to see if connection is still alive
        match create_unix_socket_client(socket_path).await {
            Ok(_) => {
                // Connection is still alive, continue monitoring
            }
            Err(_) => {
                // Connection failed - server likely restarted
                return;
            }
        }
    }
}
