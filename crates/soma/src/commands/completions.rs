use clap::{CommandFactory, ValueEnum};
use shared::error::CommonError;
use clap_complete::{generate, Shell};
use std::io;
use crate::cli::Cli;

#[derive(Debug, Clone, ValueEnum)]
pub enum CompletionShell {
    /// Bourne Again `SHell` (bash)
    Bash,
    /// Elvish shell
    Elvish,
    /// Friendly Interactive `SHell` (fish)
    Fish,
    /// `PowerShell`
    PowerShell,
    /// Z `SHell` (zsh)
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::Elvish => Shell::Elvish,
            CompletionShell::PowerShell => Shell::PowerShell,
        }
    }
}

pub fn cmd_completions(shell: CompletionShell) -> Result<(), CommonError> {
    let shell_type: Shell = shell.into();
    generate(shell_type, &mut Cli::command(), "soma", &mut io::stdout());
    Ok(())
}