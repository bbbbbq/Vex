use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Vex - QEMU auxiliary command-line tool, simplifying startup parameter management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Vex {
    /// Save QEMU startup parameters as a configuration
    Save {
        /// Force overwrite existing configuration (no prompt)
        #[arg(short = 'y')]
        force: bool,

        /// Configuration name (for later execution/deletion)
        name: String,

        /// Configuration description (optional, use double quotes)
        #[arg(short = 'd')]
        desc: Option<String>,

        /// Path to QEMU executable (e.g., qemu-system-x86_64)
        qemu_bin: String,

        /// QEMU startup arguments (e.g., -m 512 -hda disk.img)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        qemu_args: Vec<String>,
    },

    /// Execute a saved QEMU configuration
    Exec {
        /// Name of the configuration to execute
        name: String,

        /// Enable debug mode (adds -s -S parameters for GDB debugging)
        #[arg(short = 'd')]
        debug: bool,
    },

    /// Delete a saved QEMU configuration
    Rm {
        /// Name of the configuration to delete
        name: String,
    },

    /// List all saved configurations
    List,

    /// Rename a saved QEMU configuration
    Rename {
        /// New description for the configuration (optional)
        #[arg(short = 'd')]
        desc: Option<String>,

        /// Force overwrite if new name already exists (no prompt)
        #[arg(short = 'y')]
        force: bool,

        /// Current name of the configuration
        old_name: String,

        /// New name for the configuration
        new_name: String,
    },
}

/// Stored QEMU configuration structure
#[derive(Debug, Serialize, Deserialize)]
struct QemuConfig {
    /// Path to QEMU executable
    qemu_bin: String,
    /// List of QEMU startup arguments
    args: Vec<String>,
    /// Configuration description (optional)
    desc: Option<String>,
}

/// Get Vex config file storage directory (~/.vex/configs)
fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get user home directory")?;
    let dir = home.join(".vex").join("configs");
    if !dir.exists() {
        fs::create_dir_all(&dir).context("Failed to create config directory")?;
    }
    Ok(dir)
}

/// Get path to the config file for a given name
fn config_file(name: &str) -> Result<PathBuf> {
    let dir = config_dir()?;
    Ok(dir.join(format!("{}.json", name)))
}

/// List all saved configurations
fn list_configs() -> Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        println!("No configurations saved yet.");
        return Ok(());
    }

    let entries = fs::read_dir(&dir).context("Failed to read config directory")?;
    let mut configs = Vec::new();

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        if let Ok(config) = serde_json::from_str::<QemuConfig>(&content) {
                            configs.push((name.to_string(), config));
                        }
                    }
                    Err(_) => {
                        // Skip invalid config files
                        continue;
                    }
                }
            }
        }
    }

    if configs.is_empty() {
        println!("No configurations found.");
    } else {
        println!("Saved configurations:");
        for (name, config) in configs {
            if let Some(desc) = config.desc {
                println!("  {} - {}", name, desc);
            } else {
                println!("  {} - (no description)", name);
            }
            println!("    QEMU: {}", config.qemu_bin);
            println!("    Args: {:?}", config.args);
            println!();
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let vex = Vex::parse();

    match vex {
        Vex::Save {
            force,
            name,
            desc,
            qemu_bin,
            qemu_args,
        } => {
            let config_path = config_file(&name)?;
            
            // Check if debug parameters -s or -S are present
            let has_debug_args = qemu_args.iter().any(|arg| arg == "-s" || arg == "-S");
            
            let mut final_args = qemu_args.clone();
            
            if has_debug_args {
                println!("Debug parameters '-s' or '-S' detected in startup arguments");
                println!("These parameters are used to start GDB debugging server, but saving them to configuration may not be the best practice.");
                println!("Suggestion: Skip saving these parameters and use 'vex exec -d' to start remote debugging mode");
                println!("Skip saving debug parameters and use exec -d for remote debugging? [Y/n]");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                
                if input.is_empty() || input == "y" || input == "yes" {
                    // User chose to skip debug parameters
                    final_args = qemu_args.iter()
                        .filter(|&arg| arg != "-s" && arg != "-S")
                        .cloned()
                        .collect();
                    println!("Debug parameters have been skipped, saved configuration will not include -s or -S parameters");
                    println!("To start debugging mode, use: vex exec -d {}", name);
                } else {
                    println!("Debug parameters will be included in the saved configuration");
                }
            }

            let config = QemuConfig {
                qemu_bin: qemu_bin.clone(),
                args: final_args,
                desc,
            };

            if config_path.exists() && !force {
                println!("Configuration '{}' already exists, overwrite? [y/N]", name);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                if input != "y" && input != "yes" {
                    println!("Save cancelled");
                    return Ok(());
                }
            }

            let config_json = serde_json::to_string_pretty(&config).context("Failed to serialize configuration")?;
            fs::write(&config_path, config_json).context("Failed to save config file")?;
            
            if let Some(desc) = &config.desc {
                println!("Configuration '{}' with description '{}' saved to {:?}", name, desc, config_path);
            } else {
                println!("Configuration '{}' saved to {:?}", name, config_path);
            }
        }

        Vex::Exec { name, debug } => {
            let config_path = config_file(&name)?;
            if !config_path.exists() {
                anyhow::bail!("Configuration '{}' does not exist. Create it first with 'vex save'", name);
            }

            let config_json = fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: QemuConfig = serde_json::from_str(&config_json).context("Failed to deserialize configuration")?;

            let mut exec_args = config.args.clone();
            
            if debug {
                // Add debug parameters
                exec_args.push("-s".to_string());
                exec_args.push("-S".to_string());
                if let Some(desc) = &config.desc {
                    println!("Starting configuration '{}' ({}) in DEBUG mode: {} {:?}", name, desc, config.qemu_bin, exec_args);
                } else {
                    println!("Starting configuration '{}' in DEBUG mode: {} {:?}", name, config.qemu_bin, exec_args);
                }
                println!("GDB debugging server started, you can connect to localhost:1234 using gdb");
            } else {
                if let Some(desc) = &config.desc {
                    println!("Starting configuration '{}' ({}): {} {:?}", name, desc, config.qemu_bin, exec_args);
                } else {
                    println!("Starting configuration '{}': {} {:?}", name, config.qemu_bin, exec_args);
                }
            }
            
            let status = Command::new(&config.qemu_bin)
                .args(&exec_args)
                .status()
                .with_context(|| format!("Failed to execute QEMU: {}", config.qemu_bin))?;

            if !status.success() {
                anyhow::bail!("QEMU execution failed with exit code: {}", status.code().unwrap_or(-1));
            }
        }

        Vex::Rm { name } => {
            let config_path = config_file(&name)?;
            if !config_path.exists() {
                anyhow::bail!("Configuration '{}' does not exist, cannot delete", name);
            }

            fs::remove_file(&config_path).context("Failed to delete config file")?;
            println!("Configuration '{}' deleted", name);
        }

        Vex::List => {
            list_configs()?;
        }

        Vex::Rename {
            desc,
            force,
            old_name,
            new_name,
        } => {
            let old_config_path = config_file(&old_name)?;
            if !old_config_path.exists() {
                anyhow::bail!("Configuration '{}' does not exist, cannot rename", old_name);
            }

            let new_config_path = config_file(&new_name)?;
            if new_config_path.exists() && !force {
                println!("Configuration '{}' already exists, overwrite? [y/N]", new_name);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                if input != "y" && input != "yes" {
                    println!("Rename cancelled");
                    return Ok(());
                }
            }

            // Read the old configuration
            let config_json = fs::read_to_string(&old_config_path).context("Failed to read config file")?;
            let mut config: QemuConfig = serde_json::from_str(&config_json).context("Failed to deserialize configuration")?;

            // Update description if provided
            if let Some(new_desc) = desc {
                config.desc = Some(new_desc);
            }

            // Save to new location
            let new_config_json = serde_json::to_string_pretty(&config).context("Failed to serialize configuration")?;
            fs::write(&new_config_path, new_config_json).context("Failed to save new config file")?;

            // Remove old configuration
            fs::remove_file(&old_config_path).context("Failed to delete old config file")?;

            if let Some(desc) = &config.desc {
                println!("Configuration '{}' renamed to '{}' with description '{}'", old_name, new_name, desc);
            } else {
                println!("Configuration '{}' renamed to '{}'", old_name, new_name);
            }
        }
    }

    Ok(())
}