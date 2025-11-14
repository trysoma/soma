use std::time::Duration;

use shared::error::CommonError;

#[derive(vite_rs::Embed)]
#[root = "./app"]
#[dev_server_port = 21012]
pub(crate) struct Assets;

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
