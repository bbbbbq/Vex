use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use std::io;

use crate::commands::Cli;

#[derive(Args)]
#[clap(about = "Generate shell completion scripts")]
pub struct CompletionsArgs {
    #[arg(help = "Shell type (bash, zsh, fish, powershell, elvish)")]
    pub shell: Shell,
}

pub fn completions_command(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    generate(shell, &mut cmd, bin_name, &mut io::stdout());

    Ok(())
}

