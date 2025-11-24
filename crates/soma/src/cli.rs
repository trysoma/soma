use clap::{Parser, Subcommand};
use shared::error::CommonError;
use tracing::error;

use crate::{
    commands::{
        self, codegen::CodegenParams, completions::CompletionShell, dev::DevParams,
        encryption::EncryptionParams, init::InitParams,
    },
    utils::get_or_init_cli_config,
};

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
    /// Start Soma development server
    Dev(DevParams),
    /// Generate bridge client for current project
    Codegen(CodegenParams),
    /// Generate shell completions for soma
    Completions {
        /// Shell to generate completions for
        shell: CompletionShell,
    },
    /// Manage encryption keys
    Encryption(EncryptionParams),
    /// Initialize a new Soma project
    Init(InitParams),
    /// Show Soma version
    Version,
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
        Commands::Codegen(params) => commands::codegen::cmd_codegen(params, &mut config).await,
        Commands::Completions { shell } => commands::completions::cmd_completions(shell),
        Commands::Encryption(params) => commands::encryption::cmd_encryption(params, &mut config).await,
        Commands::Init(params) => commands::init::cmd_init(params).await,
        Commands::Version => {
            println!("Soma CLI version: {CLI_VERSION}");
            Ok(())
        }
    };

    unwrap_and_error(cmd_res);
    Ok(())
}
