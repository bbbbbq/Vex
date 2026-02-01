pub mod complete_configs;
pub mod completions;
pub mod exec;
pub mod list;
pub mod print;
pub mod remove;
pub mod rename;
pub mod save;

pub use complete_configs::{CompleteConfigsArgs, complete_configs_command};
pub use completions::{CompletionsArgs, completions_command};
pub use exec::{ExecArgs, exec_command};
pub use list::{ListArgs, list_command};
pub use print::{PrintArgs, print_command};
pub use remove::{RemoveArgs, remove_command};
pub use rename::{RenameArgs, rename_command};
pub use save::{SaveArgs, save_command};

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Save(SaveArgs),
    Rename(RenameArgs),
    Rm(RemoveArgs),
    List(ListArgs),
    Print(PrintArgs),
    Exec(ExecArgs),
    Completions(CompletionsArgs),
    /// Hidden command for shell completion
    #[clap(hide = true)]
    CompleteConfigs(CompleteConfigsArgs),
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
