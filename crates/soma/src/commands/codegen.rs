use clap::Args;
use tracing::debug;

use shared::error::CommonError;
use soma_api_client::apis::internal_api;

use crate::utils::{CliConfig, create_and_wait_for_api_client};

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
    // Create API client and wait for server to be ready
    let api_config =
        create_and_wait_for_api_client(&params.api_url, params.timeout_secs, None).await?;

    // Trigger codegen via API
    debug!("Triggering mcp client generation...");

    internal_api::trigger_codegen(&api_config)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call trigger_codegen endpoint: {e:?}"
            ))
        })?;

    debug!("MCP client generation complete!");

    Ok(())
}
