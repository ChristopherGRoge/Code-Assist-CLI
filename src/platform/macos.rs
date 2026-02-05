use super::PlatformPaths;
use anyhow::{Context, Result};
use console::style;
use std::path::PathBuf;

pub fn get_paths() -> PlatformPaths {
    let home_dir = dirs::home_dir().expect("Could not determine home directory");

    PlatformPaths {
        home_dir: home_dir.clone(),
        claude_config_dir: home_dir.join(".claude"),
        vscode_settings_dir: home_dir
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User"),
        certs_dir: home_dir.join("certs"),
    }
}

pub fn print_install_instructions() {
    println!(
        "{}\n",
        style("Please install the missing software via Self-Service:").yellow()
    );
    println!("  1. Open Self-Service from your Applications folder or Dock");
    println!("  2. Search for and install:");
    println!("     - Visual Studio Code");
    println!("     - Git (or Xcode Command Line Tools)");
    println!("\nOnce installed, run this command again.");
}

pub fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    // On macOS, we add to shell config files
    let home = dirs::home_dir().context("Could not determine home directory")?;

    // Determine which shell config to use
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    let config_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else if shell.contains("bash") {
        // On macOS, .bash_profile is typically used for login shells
        home.join(".bash_profile")
    } else {
        home.join(".profile")
    };

    let export_line = format!("export {}=\"{}\"", name, value);

    // Read existing content
    let existing = std::fs::read_to_string(&config_file).unwrap_or_default();

    // Check if already set
    if existing.contains(&format!("export {}=", name)) {
        // Update existing line
        let updated: Vec<String> = existing
            .lines()
            .map(|line| {
                if line.trim_start().starts_with(&format!("export {}=", name)) {
                    export_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect();
        std::fs::write(&config_file, updated.join("\n") + "\n")
            .context("Failed to update shell config")?;
    } else {
        // Append new line
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config_file)
            .context("Failed to open shell config")?;

        use std::io::Write;
        writeln!(file, "\n# Added by code-assist")?;
        writeln!(file, "{}", export_line)?;
    }

    Ok(())
}

pub fn add_to_path(dir: &str) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    let config_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else if shell.contains("bash") {
        home.join(".bash_profile")
    } else {
        home.join(".profile")
    };

    let path_line = format!("export PATH=\"{}:$PATH\"", dir);

    let existing = std::fs::read_to_string(&config_file).unwrap_or_default();

    // Check if this path is already added
    if existing.contains(dir) {
        return Ok(());
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)
        .context("Failed to open shell config")?;

    use std::io::Write;
    writeln!(file, "\n# Added by code-assist")?;
    writeln!(file, "{}", path_line)?;

    Ok(())
}

pub fn import_certificate(cert_path: &std::path::Path) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let keychain = home.join("Library/Keychains/login.keychain-db");

    let output = std::process::Command::new("security")
        .args([
            "add-trusted-cert",
            "-k",
            keychain.to_str().unwrap(),
            cert_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to run security command")?;

    if !output.status.success() {
        // If security command fails, try opening the cert for manual import
        println!(
            "{} Automatic certificate import failed. Opening certificate for manual import...",
            style("!").yellow().bold()
        );
        std::process::Command::new("open")
            .arg(cert_path)
            .spawn()
            .context("Failed to open certificate")?;
    }

    Ok(())
}

/// Check if VS Code is installed on macOS
pub fn check_vscode_installed() -> bool {
    // Check Application folder
    let app_path = std::path::Path::new("/Applications/Visual Studio Code.app");
    if app_path.exists() {
        return true;
    }

    // Check if 'code' command is available
    std::process::Command::new("code")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Git is installed on macOS
pub fn check_git_installed() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the VS Code CLI path
pub fn get_vscode_cli() -> &'static str {
    "code"
}
