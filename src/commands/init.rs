use anyhow::{Context, Result};
use clap::Args;
use clap_complete::Shell;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Args)]
#[clap(about = "Initialize shell completion for vex")]
pub struct InitArgs {
    #[arg(
        short,
        long,
        help = "Shell type (bash, zsh, fish). Auto-detected if not specified"
    )]
    pub shell: Option<Shell>,

    #[arg(long, help = "Print the shell configuration line without installing")]
    pub print: bool,
}

pub fn init_command(shell: Option<Shell>, print_only: bool) -> Result<()> {
    let detected_shell = shell.or_else(detect_shell).context(
        "Could not detect shell type. Please specify with --shell (bash, zsh, fish)",
    )?;

    let config_line = get_completion_line(detected_shell);

    if print_only {
        println!("{}", config_line);
        return Ok(());
    }

    let rc_file = get_shell_rc_file(detected_shell)?;

    // Check if already configured
    if let Ok(content) = fs::read_to_string(&rc_file) {
        if content.contains("vex completions") || content.contains("vex init") {
            println!("✓ Shell completion for vex is already configured in {:?}", rc_file);
            println!("  To reconfigure, remove the existing vex completion line first.");
            return Ok(());
        }
    }

    // Append completion line to rc file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)
        .with_context(|| format!("Failed to open {:?}", rc_file))?;

    writeln!(file)?;
    writeln!(file, "# vex shell completion")?;
    writeln!(file, "{}", config_line)?;

    println!("✓ Shell completion installed for {:?}", detected_shell);
    println!("  Added to: {:?}", rc_file);
    println!();
    println!("To activate, run:");
    println!("  source {:?}", rc_file);
    println!();
    println!("Or restart your terminal.");

    Ok(())
}

/// Detect current shell from environment
fn detect_shell() -> Option<Shell> {
    let shell_path = env::var("SHELL").ok()?;
    let shell_name = shell_path.rsplit('/').next()?;

    match shell_name {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "powershell" | "pwsh" => Some(Shell::PowerShell),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

/// Get the completion configuration line for a shell
fn get_completion_line(shell: Shell) -> String {
    match shell {
        Shell::Bash => r#"eval "$(vex completions bash)""#.to_string(),
        Shell::Zsh => r#"eval "$(vex completions zsh)""#.to_string(),
        Shell::Fish => "vex completions fish | source".to_string(),
        Shell::PowerShell => "Invoke-Expression (& vex completions powershell)".to_string(),
        Shell::Elvish => "eval (vex completions elvish)".to_string(),
        _ => format!("# Unsupported shell: {:?}", shell),
    }
}

/// Get the shell rc file path
fn get_shell_rc_file(shell: Shell) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    let rc_file = match shell {
        Shell::Bash => {
            // Prefer .bashrc, fall back to .bash_profile
            let bashrc = home.join(".bashrc");
            if bashrc.exists() {
                bashrc
            } else {
                home.join(".bash_profile")
            }
        }
        Shell::Zsh => home.join(".zshrc"),
        Shell::Fish => home.join(".config/fish/config.fish"),
        Shell::PowerShell => {
            // PowerShell profile location varies by platform
            if cfg!(windows) {
                home.join("Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1")
            } else {
                home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
            }
        }
        Shell::Elvish => home.join(".elvish/rc.elv"),
        _ => anyhow::bail!("Unsupported shell: {:?}", shell),
    };

    Ok(rc_file)
}
