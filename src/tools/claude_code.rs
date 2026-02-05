use anyhow::{anyhow, Context, Result};
use console::style;
use std::path::PathBuf;

use super::Tool;
use crate::config;
use crate::download;
use crate::platform;

pub struct ClaudeCode {
    local_dir: PathBuf,
}

impl ClaudeCode {
    pub fn new() -> Self {
        // Get the directory where the executable is located
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // Look for local directory relative to executable or current directory
        let local_dir = if exe_dir.join("local").exists() {
            exe_dir.join("local")
        } else {
            std::env::current_dir().unwrap().join("local")
        };

        Self { local_dir }
    }

    fn get_install_dir(&self) -> PathBuf {
        let paths = platform::get_paths();
        paths.home_dir.join(".claude").join("bin")
    }

    fn get_binary_path(&self) -> PathBuf {
        self.get_install_dir().join(platform::get_binary_name())
    }
}

impl Tool for ClaudeCode {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn is_installed(&self) -> Result<bool> {
        let binary_path = self.get_binary_path();
        Ok(binary_path.exists())
    }

    fn install(&self) -> Result<()> {
        println!(
            "{} Installing Claude Code...\n",
            style("→").cyan().bold()
        );

        // Step 1: Get version
        println!("  Fetching latest version...");
        let (version, source) = download::get_latest_version(&self.local_dir)?;
        println!(
            "  {} Version: {} ({})",
            style("✓").green().bold(),
            style(&version).cyan(),
            match source {
                download::DownloadSource::Remote => "remote",
                download::DownloadSource::LocalFallback => "local fallback",
            }
        );

        // Step 2: Get manifest
        println!("\n  Fetching manifest...");
        let (manifest, _) = download::get_manifest(&version, &self.local_dir)?;

        let platform_id = platform::get_platform_id();
        let binary_name = platform::get_binary_name();

        let checksum = manifest["platforms"][platform_id]["checksum"]
            .as_str()
            .ok_or_else(|| anyhow!("Platform {} not found in manifest", platform_id))?;

        println!(
            "  {} Platform: {}",
            style("✓").green().bold(),
            style(platform_id).cyan()
        );

        // Step 3: Download binary
        println!("\n  Downloading binary...");
        let download_dir = platform::get_paths().home_dir.join(".claude").join("downloads");
        std::fs::create_dir_all(&download_dir)?;

        let temp_binary = download_dir.join(format!("claude-{}-{}", version, platform_id));

        let _source = download::download_binary(
            &version,
            platform_id,
            binary_name,
            &self.local_dir,
            &temp_binary,
            checksum,
        )?;

        // Step 4: Make executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_binary)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&temp_binary, perms)?;
        }

        // Step 5: Run claude install
        println!(
            "\n{} Running Claude Code setup...\n",
            style("→").cyan().bold()
        );

        let output = std::process::Command::new(&temp_binary)
            .arg("install")
            .output()
            .context("Failed to run claude install")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Claude install failed: {}", stderr));
        }

        // Clean up temp binary
        std::fs::remove_file(&temp_binary).ok();

        // Step 6: Install VSIX extensions
        println!(
            "\n{} Installing VS Code extensions...\n",
            style("→").cyan().bold()
        );
        let vsix_dir = self.local_dir.join("VSIX");
        config::install_vsix_extensions(&vsix_dir)?;

        // Step 7: Deploy configurations
        println!(
            "\n{} Deploying configurations...\n",
            style("→").cyan().bold()
        );
        let paths = platform::get_paths();
        config::deploy_configs(&self.local_dir, &paths)?;

        // Step 8: Add to PATH
        let install_dir = self.get_install_dir();
        if let Err(e) = platform::add_to_path(install_dir.to_str().unwrap()) {
            println!(
                "  {} Could not add to PATH: {}",
                style("!").yellow().bold(),
                e
            );
        } else {
            println!(
                "  {} Added to PATH: {}",
                style("✓").green().bold(),
                install_dir.display()
            );
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        println!(
            "{} Uninstalling Claude Code...\n",
            style("→").cyan().bold()
        );

        let binary_path = self.get_binary_path();

        // Try to run claude uninstall first
        if binary_path.exists() {
            println!("  Running Claude Code uninstaller...");
            let output = std::process::Command::new(&binary_path)
                .arg("uninstall")
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    println!(
                        "  {} Claude Code uninstalled",
                        style("✓").green().bold()
                    );
                }
                _ => {
                    // Manual cleanup
                    println!("  {} Performing manual cleanup...", style("!").yellow().bold());

                    // Remove binary
                    std::fs::remove_file(&binary_path).ok();

                    // Remove .claude directory (but keep downloads as backup)
                    let claude_dir = platform::get_paths().claude_config_dir;
                    if claude_dir.exists() {
                        // Only remove specific subdirectories, not the whole thing
                        std::fs::remove_dir_all(claude_dir.join("bin")).ok();
                    }
                }
            }
        } else {
            println!(
                "  {} Claude Code is not installed",
                style("-").dim()
            );
        }

        Ok(())
    }

    fn configure(&self) -> Result<()> {
        // Install VSIX extensions
        println!("  Installing VS Code extensions...\n");
        let vsix_dir = self.local_dir.join("VSIX");
        config::install_vsix_extensions(&vsix_dir)?;

        // Deploy configurations
        println!("\n  Deploying configurations...\n");
        let paths = platform::get_paths();
        config::deploy_configs(&self.local_dir, &paths)?;

        Ok(())
    }
}
