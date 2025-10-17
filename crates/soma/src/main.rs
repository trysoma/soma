use std::time::Duration;

use clap::{Parser, Subcommand};
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel, errors::GracefulShutdownError};
use tracing::error;

use crate::{commands::StartParams, utils::config::get_or_init_cli_config};
use shared::error::{CommonError, DynError};

mod a2a;
mod commands;
mod logic;
mod mcp;
mod repository;
mod router;
mod utils;
mod vite;

pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start(StartParams),
    Codegen,
    // #[command(subcommand)]
    // Bridge(BridgeCommands),
}

// #[derive(Subcommand)]
// pub enum BridgeCommands {
//     Init(commands::BridgeInitParams),
// }

pub type Subsys = SubsystemHandle<DynError>;

fn unwrap_and_error<T>(cmd: Result<T, CommonError>) -> T {
    match cmd {
        Ok(value) => value,
        Err(e) => {
            error!("Error: {:?}", &e);
            panic!("Error: {:?}", &e);
        }
    }
}

async fn run_cli(cli: Cli) -> Result<(), anyhow::Error> {
    Toplevel::new(async move |subsys: SubsystemHandle| {
        let mut config = get_or_init_cli_config()
            .await
            .inspect_err(|e| {
                error!("Failed to get or init CLI config: {:?}", e);
            })
            .unwrap();

        let cmd_res = match cli.command {
            Commands::Start(params) => commands::cmd_start(&subsys, params, &mut config).await,
            Commands::Codegen => commands::cmd_codegen(&subsys, &mut config).await,
            // Commands::Bridge(bridge_cmd) => match bridge_cmd {
            //     BridgeCommands::Init(params) => {
            //         commands::cmd_bridge_init(&subsys, params, &mut config).await
            //     }
            // },
        };

        unwrap_and_error(cmd_res);

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    shared::env::configure_env()?;
    shared::logging::configure_logging()?;
    shared::crypto::configure_crypto_provider()?;

    // Parse CLI arguments with precedence: actual CLI args > SOMA_COMMAND env var
    let cli = Cli::parse();

    run_cli(cli).await
}
