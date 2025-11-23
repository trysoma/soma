use std::time::Duration;

use shared::error::CommonError;
use tracing::info;

// The vite_rs::Embed proc macro embeds frontend assets at compile time
// The path is relative to CARGO_MANIFEST_DIR
#[derive(vite_rs::Embed)]
#[root = "app"]
#[dev_server_port = 21012]
pub struct Assets;

async fn ping_vite_dev_server() -> Result<(), CommonError> {
    let client = reqwest::Client::new();
    let response = client.get("http://localhost:21012").send().await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(CommonError::Unknown(anyhow::anyhow!("Failed to ping vite dev server")))
    }
}

pub async fn wait_for_vite_dev_server_shutdown() -> Result<(), CommonError> {
    let mut attempts = 0;
    let max_attempts = 10;
    loop {
        if ping_vite_dev_server().await.is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        attempts += 1;
        if attempts >= max_attempts {
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "Failed to wait for vite dev server to shutdown"
            )));
        }
    }
    Ok(())
}


/// Starts the Vite dev server (debug builds only)
/// Returns a guard that stops the server when dropped
#[cfg(debug_assertions)]
pub fn start_vite_dev_server() -> impl Drop {
    info!("Starting vite dev server");
    // The return value is a scope guard that stops the server when dropped
    let guard = Assets::start_dev_server(false);
    guard.unwrap_or_else(|| {
        panic!("Failed to start vite dev server");
    })
}

/// Stops the Vite dev server and waits for shutdown (debug builds only)
#[cfg(debug_assertions)]
pub async fn stop_vite_dev_server() -> Result<(), CommonError> {
    info!("Stopping vite dev server");
    Assets::stop_dev_server();
    wait_for_vite_dev_server_shutdown().await?;
    Ok(())
}