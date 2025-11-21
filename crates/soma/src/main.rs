mod a2a;
mod cli;
mod codegen;
mod commands;
mod logic;
mod mcp;
mod repository;
mod router;
mod utils;
mod vite;

use clap::Parser;

use crate::cli::{Cli, run_cli};



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
