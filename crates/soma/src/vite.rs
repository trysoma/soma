use std::time::Duration;

use shared::error::CommonError;

// The vite_rs::Embed proc macro embeds frontend assets at compile time
// The path is relative to CARGO_MANIFEST_DIR
// Only compile when app directory exists and frontend is built (set by build.rs)
#[cfg(all(has_app_dir, has_frontend_dist))]
#[derive(vite_rs::Embed)]
#[root = "app"]
#[dev_server_port = 21012]
pub(crate) struct Assets;

// Stub for when app directory or dist doesn't exist (e.g., during cross-compilation)
#[cfg(not(all(has_app_dir, has_frontend_dist)))]
pub(crate) struct Assets;

#[cfg(not(all(has_app_dir, has_frontend_dist)))]
impl Assets {
    // The proc macro should generate this, but when it doesn't run, we provide a stub
    // Note: This will panic if called - frontend should be built before cross-compilation
    pub fn boxed() -> std::pin::Pin<Box<dyn vite_rs::LiveLoad + Send + Sync>> {
        // Return a minimal stub implementation that panics
        // Frontend should be built before cross-compilation
        struct StubLiveLoad;
        impl vite_rs::LiveLoad for StubLiveLoad {
            fn get(&self, _path: &str) -> Option<vite_rs::Asset> {
                None
            }
        }
        Box::pin(StubLiveLoad)
    }

    pub fn start_dev_server(_quiet: bool) -> Option<impl Drop> {
        None
    }

    pub fn stop_dev_server() {
        // No-op
    }
}

#[allow(dead_code)]
async fn ping_vite_dev_server() -> Result<(), CommonError> {
    let client = reqwest::Client::new();
    let response = client.get("http://localhost:21012").send().await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to ping vite dev server"
        )))
    }
}

#[allow(dead_code)]
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
