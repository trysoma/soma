mod a2a;
mod cli;
mod commands;
mod logic;
mod mcp;
mod repository;
mod router;
mod utils;
mod vite;

use clap::{CommandFactory, Parser, Subcommand};
use shared::error::CommonError;
use tracing::error;

use crate::{cli::{Cli, run_cli}, commands::dev::DevParams, utils::config::get_or_init_cli_config};



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
