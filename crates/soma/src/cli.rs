use clap::{Parser, Subcommand};
use clap_complete::{generate, shells::Bash};
use shared::error::CommonError;
use tracing::error;

use crate::{commands::{self, dev::DevParams}, utils::config::get_or_init_cli_config};


pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn unwrap_and_error<T>(cmd: Result<T, CommonError>) -> T {
    match cmd {
        Ok(value) => value,
        Err(e) => {
            error!("Error: {:?}", &e);
            panic!("Error: {:?}", &e);
        }
    }
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum Commands {
    Dev(DevParams),
    Codegen,
    Completions
    // #[command(subcommand)]
    // Bridge(BridgeCommands),
}

pub async fn run_cli(cli: Cli) -> Result<(), anyhow::Error> {
    let mut config = get_or_init_cli_config()
        .await
        .inspect_err(|e| {
            error!("Failed to get or init CLI config: {:?}", e);
        })
        .unwrap();

    let cmd_res = match cli.command {
        Commands::Dev(params) => commands::dev::cmd_dev(params, &mut config).await,
        Commands::Codegen => commands::codegen::cmd_codegen(&mut config).await,
        Commands::Completions => commands::completions::cmd_completions(),
    };

    unwrap_and_error(cmd_res);
    Ok(())
}
