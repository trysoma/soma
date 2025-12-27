mod mcp;
mod cli;
mod commands;
mod restate_server;
mod server;
mod utils;
use clap::Parser;
use human_panic::setup_panic;

use crate::cli::{Cli, run_cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_panic!();
    // Initialize tracing
    shared::env::configure_env()?;
    shared::logging::configure_logging()?;
    shared::crypto::configure_crypto_provider()?;

    // Parse CLI arguments with precedence: actual CLI args > SOMA_COMMAND env var
    let cli = Cli::parse();

    run_cli(cli).await
}
