mod a2a;
mod commands;
mod logic;
mod mcp;
mod repository;
mod router;
mod utils;
mod vite;

use std::time::Duration;

use shared::error::{CommonError, DynError};
use tokio_graceful_shutdown::{errors::GracefulShutdownError, SubsystemHandle, Toplevel};
use tracing::error;

use crate::commands::dev::{DevParams, cmd_dev};
use crate::utils::config::get_or_init_cli_config;


pub type Subsys = SubsystemHandle<DynError>;

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

    Toplevel::new(async move |subsys: SubsystemHandle| {
        // TODO: we should create dev() as a function that doesnt need the cli config
        let mut config = get_or_init_cli_config()
            .await
            .inspect_err(|e| {
                error!("Failed to get or init CLI config: {:?}", e);
            })
            .unwrap();
        let res = cmd_dev(&subsys, dev_params, &mut config).await;
        unwrap_and_error(res);

        subsys.request_shutdown();
    })
    .catch_signals()
    .handle_shutdown_requests(Duration::from_millis(30_000))
    .await
    .map_err(|err: GracefulShutdownError<DynError>| {
        let sub_errs = err.get_subsystem_errors();
        for sub_err in sub_errs {
            error!("error: {:?}", sub_err);
        }
        anyhow::anyhow!(err)
    })
}
