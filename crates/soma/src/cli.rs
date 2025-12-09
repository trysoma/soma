use clap::{Parser, Subcommand};
use shared::error::CommonError;
use tracing::error;

use crate::{
    commands::{
        self, api_key::ApiKeyParams, auth::AuthParams, codegen::CodegenParams,
        completions::CompletionShell, dev::DevParams, encryption::EncKeyParams,
        environment::EnvironmentParams, init::InitParams, secret::SecretParams, sts::StsParams,
    },
    utils::get_or_init_cli_config,
};

pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn unwrap_and_error<T>(cmd: Result<T, CommonError>) -> T {
    match cmd {
        Ok(value) => value,
        Err(e) => {
            error!("Error: {:?}", &e);
            std::process::exit(1);
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
    #[command(name = "enc-key")]
    EncKey(EncKeyParams),
    /// Initialize a new Soma project
    Init(InitParams),
    /// Manage secrets
    Secret(SecretParams),
    /// Manage environment variables
    #[command(name = "env")]
    Environment(EnvironmentParams),
    /// Manage API keys
    #[command(name = "api-key")]
    ApiKey(ApiKeyParams),
    /// Manage user authentication flow configurations (OAuth/OIDC)
    Auth(AuthParams),
    /// Manage STS (Security Token Service) configurations
    Sts(StsParams),
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
        Commands::EncKey(params) => commands::encryption::cmd_enc_key(params, &mut config).await,
        Commands::Init(params) => commands::init::cmd_init(params).await,
        Commands::Secret(params) => commands::secret::cmd_secret(params, &mut config).await,
        Commands::Environment(params) => {
            commands::environment::cmd_environment(params, &mut config).await
        }
        Commands::ApiKey(params) => commands::api_key::cmd_api_key(params, &mut config).await,
        Commands::Auth(params) => commands::auth::cmd_auth(params, &mut config).await,
        Commands::Sts(params) => commands::sts::cmd_sts(params, &mut config).await,
        Commands::Version => {
            println!("Soma CLI version: {CLI_VERSION}");
            Ok(())
        }
    };

    unwrap_and_error(cmd_res);
    Ok(())
}
