use std::time::Duration;

use clap::Args;
use tracing::info;

use shared::error::CommonError;
use soma_api_client::apis::{configuration::Configuration, default_api};

use crate::utils::config::CliConfig;

#[derive(Args, Debug, Clone)]
pub struct CodegenParams {
    #[arg(long, default_value = "http://localhost:3000")]
    pub api_url: String,

    #[arg(long, default_value = "30")]
    pub timeout_secs: u64,
}

pub async fn cmd_codegen(
    params: CodegenParams,
    _config: &mut CliConfig,
) -> Result<(), CommonError> {
    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(params.timeout_secs))
        .build()
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!("Failed to create HTTP client: {e}"))
        })?;

    // Wait for API to be ready
    info!(
        "Waiting for Soma API server at {} to be ready...",
        params.api_url
    );

    let max_retries = params.timeout_secs / 2; // Check every 2 seconds
    let mut connected = false;

    // Create API config for health check
    let api_config = Configuration {
        base_path: params.api_url.clone(),
        user_agent: Some("soma-cli/codegen".to_string()),
        client: client.clone(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    };

    for attempt in 1..=max_retries {
        match default_api::agent_card(&api_config).await {
            Ok(_) => {
                info!("Connected to Soma API server successfully");
                connected = true;
                break;
            }
            Err(e) => {
                if attempt == max_retries {
                    return Err(CommonError::Unknown(anyhow::anyhow!(
                        "Failed to connect to Soma API server after {} attempts: {:?}. Please ensure 'soma dev' is running.",
                        max_retries,
                        e
                    )));
                }
                if attempt == 1 {
                    info!("Waiting for server... (attempt {}/{})", attempt, max_retries);
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    if !connected {
        return Err(CommonError::Unknown(anyhow::anyhow!(
            "Failed to connect to Soma API server. Please ensure 'soma dev' is running at {}",
            params.api_url
        )));
    }

    // Trigger codegen via API
    info!("Triggering bridge client generation...");

    let response = default_api::trigger_codegen(&api_config)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call trigger_codegen endpoint: {e:?}"
            ))
        })?;

    info!("Bridge client generation complete!");
    info!("{}", response.message);

    Ok(())
}
