use clap::{Parser, Subcommand};
use shared::error::CommonError;
use tracing::error;

use crate::{commands::{self, dev::DevParams, init::InitParams, completions::CompletionShell}, utils::config::get_or_init_cli_config};


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
    /// Generate shell completions for soma
    Completions {
        /// Shell to generate completions for
        shell: CompletionShell,
    },
    Init(InitParams),
    Version,
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
        Commands::Completions { shell } => commands::completions::cmd_completions(shell),
        Commands::Init(params) => commands::init::cmd_init(params).await,
        Commands::Version => {
            println!("Soma CLI version: {}", CLI_VERSION);
            Ok(())
        },
    };

    unwrap_and_error(cmd_res);
    Ok(())
}
