use anyhow::Result;
use clap::Args;
use std::fs;

use crate::config::config_dir;

#[derive(Args)]
#[clap(about = "List config names for shell completion (internal use)", hide = true)]
pub struct CompleteConfigsArgs;

/// Output config names for shell completion, one per line
pub fn complete_configs_command() -> Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                println!("{}", name);
            }
        }
    }

    Ok(())
}
