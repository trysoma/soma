mod a2a;
mod commands;
mod logic;
mod mcp;
mod repository;
mod router;
mod utils;
mod vite;

use shared::error::CommonError;
use tracing::error;

use crate::commands::dev::{DevParams, cmd_dev};
use crate::utils::config::get_or_init_cli_config;

pub fn unwrap_and_error<T>(cmd: Result<T, CommonError>) -> T {
    match cmd {
        Ok(value) => value,
        Err(e) => {
            error!("Error: {:?}", &e);
            panic!("Error: {:?}", &e);
        }
    }
}

pub async fn run_soma(dev_params: DevParams) -> Result<(), anyhow::Error> {
    // Initialize tracing
    shared::env::configure_env()?;
    shared::logging::configure_logging()?;
    shared::crypto::configure_crypto_provider()?;

    let mut config = get_or_init_cli_config()
        .await
        .inspect_err(|e| {
            error!("Failed to get or init CLI config: {:?}", e);
        })
        .unwrap();
    let res = cmd_dev(dev_params, &mut config).await;
    unwrap_and_error(res);
    Ok(())
}
