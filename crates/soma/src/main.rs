
mod a2a;
mod commands;
mod logic;
mod mcp;
mod repository;
mod router;
mod utils;
mod vite;

use clap::{Parser, Subcommand};
use soma::unwrap_and_error;
use tracing::error;

use crate::{commands::dev::DevParams, utils::config::get_or_init_cli_config};



pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Dev(DevParams),
    Codegen,
    // #[command(subcommand)]
    // Bridge(BridgeCommands),
}

// #[derive(Subcommand)]
// pub enum BridgeCommands {
//     Init(commands::BridgeInitParams),
// }


async fn run_cli(cli: Cli) -> Result<(), anyhow::Error> {
    let mut config = get_or_init_cli_config()
        .await
        .inspect_err(|e| {
            error!("Failed to get or init CLI config: {:?}", e);
        })
        .unwrap();

    let cmd_res = match cli.command {
        Commands::Dev(params) => commands::cmd_dev(params, &mut config).await,
        Commands::Codegen => commands::cmd_codegen(&mut config).await,
    };

    unwrap_and_error(cmd_res);
    Ok(())
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
