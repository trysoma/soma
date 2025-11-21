use clap::CommandFactory;
use shared::error::CommonError;
use clap_complete::{generate, shells::Bash};
use std::io;
use crate::cli::Cli;

pub fn cmd_completions() -> Result<(), CommonError> {
    generate(Bash, &mut Cli::command(), "soma", &mut io::stdout());
    Ok(())
}