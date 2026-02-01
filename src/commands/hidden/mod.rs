//! Hidden commands for internal use (shell completion, etc.)

pub mod complete_configs;

pub use complete_configs::{CompleteConfigsArgs, complete_configs_command};
