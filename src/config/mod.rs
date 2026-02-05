use anyhow::{Context, Result};
use console::style;
use std::path::Path;

use crate::platform::{self, PlatformPaths};

fn get_platform_config_dir(local_dir: &Path) -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        local_dir.join("WIN").join("USER-DIRECTORY")
    }

    #[cfg(target_os = "macos")]
    {
        local_dir.join("MACOS").join("USER-DIRECTORY")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux fallback for development - not actually supported at runtime
        local_dir.join("LINUX").join("USER-DIRECTORY")
    }
}

fn get_vscode_settings_source(config_dir: &Path) -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        config_dir
            .join("AppData")
            .join("Roaming")
            .join("Code")
            .join("User")
            .join("settings.json")
    }

    #[cfg(target_os = "macos")]
    {
        config_dir
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User")
            .join("settings.json")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux fallback for development
        config_dir
            .join(".config")
            .join("Code")
            .join("User")
            .join("settings.json")
    }
}

/// Deploy configuration files for a tool
pub fn deploy_configs(local_dir: &Path, paths: &PlatformPaths) -> Result<()> {
    let platform_config_dir = get_platform_config_dir(local_dir);

    if !platform_config_dir.exists() {
        println!(
            "  {} No platform-specific configs found",
            style("!").yellow().bold()
        );
        return Ok(());
    }

    // Deploy .claude/settings.json
    deploy_claude_settings(&platform_config_dir, paths)?;

    // Deploy certificates
    deploy_certificates(&platform_config_dir, paths)?;

    // Deploy VS Code settings
    deploy_vscode_settings(&platform_config_dir, paths)?;

    // Set environment variables
    configure_environment(paths)?;

    Ok(())
}

fn deploy_claude_settings(config_dir: &Path, paths: &PlatformPaths) -> Result<()> {
    let source = config_dir.join(".claude").join("settings.json");
    if !source.exists() {
        return Ok(());
    }

    let dest_dir = &paths.claude_config_dir;
    std::fs::create_dir_all(dest_dir).context("Failed to create .claude directory")?;

    let dest = dest_dir.join("settings.json");

    // If settings already exist, merge them
    if dest.exists() {
        merge_json_settings(&source, &dest)?;
        println!(
            "  {} Merged Claude settings",
            style("✓").green().bold()
        );
    } else {
        std::fs::copy(&source, &dest).context("Failed to copy Claude settings")?;
        println!(
            "  {} Deployed Claude settings",
            style("✓").green().bold()
        );
    }

    Ok(())
}

fn deploy_certificates(config_dir: &Path, paths: &PlatformPaths) -> Result<()> {
    // Look for certificates in different possible locations
    let cert_sources = [
        config_dir.join(".continue").join("certs"),
        config_dir.join("certs"),
    ];

    let mut found_certs = false;

    for cert_source in &cert_sources {
        if !cert_source.exists() {
            continue;
        }

        std::fs::create_dir_all(&paths.certs_dir).context("Failed to create certs directory")?;

        for entry in std::fs::read_dir(cert_source)? {
            let entry = entry?;
            let path = entry.path();

            // Skip macOS resource fork files
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("._"))
                .unwrap_or(false)
            {
                continue;
            }

            if path.extension().map(|e| e == "crt").unwrap_or(false) {
                let dest = paths.certs_dir.join(entry.file_name());
                std::fs::copy(&path, &dest).context("Failed to copy certificate")?;

                println!(
                    "  {} Deployed certificate: {}",
                    style("✓").green().bold(),
                    entry.file_name().to_string_lossy()
                );

                // Try to import the certificate
                if let Err(e) = platform::import_certificate(&dest) {
                    println!(
                        "  {} Certificate import: {}",
                        style("!").yellow().bold(),
                        e
                    );
                }

                found_certs = true;
            }
        }
    }

    if !found_certs {
        println!(
            "  {} No certificates to deploy",
            style("-").dim()
        );
    }

    Ok(())
}

fn deploy_vscode_settings(config_dir: &Path, paths: &PlatformPaths) -> Result<()> {
    let platform_source = get_vscode_settings_source(config_dir);

    // Also check for a simpler path structure
    let alt_source = config_dir.join("vscode-settings.json");

    let source = if platform_source.exists() {
        platform_source
    } else if alt_source.exists() {
        alt_source
    } else {
        println!(
            "  {} No VS Code settings to deploy",
            style("-").dim()
        );
        return Ok(());
    };

    std::fs::create_dir_all(&paths.vscode_settings_dir)
        .context("Failed to create VS Code settings directory")?;

    let dest = paths.vscode_settings_dir.join("settings.json");

    if dest.exists() {
        merge_json_settings(&source, &dest)?;
        println!(
            "  {} Merged VS Code settings",
            style("✓").green().bold()
        );
    } else {
        std::fs::copy(&source, &dest).context("Failed to copy VS Code settings")?;
        println!(
            "  {} Deployed VS Code settings",
            style("✓").green().bold()
        );
    }

    Ok(())
}

fn configure_environment(paths: &PlatformPaths) -> Result<()> {
    // Set NODE_EXTRA_CA_CERTS if we have certificates
    let zscaler_cert = paths.certs_dir.join("ZscalerRootCertificate-2048-SHA256.crt");
    let alt_cert = paths.certs_dir.join("zscaler-root.crt");

    let cert_path = if zscaler_cert.exists() {
        Some(zscaler_cert)
    } else if alt_cert.exists() {
        Some(alt_cert)
    } else {
        None
    };

    if let Some(cert) = cert_path {
        platform::set_user_env_var("NODE_EXTRA_CA_CERTS", cert.to_str().unwrap())?;
        println!(
            "  {} Set NODE_EXTRA_CA_CERTS environment variable",
            style("✓").green().bold()
        );
    }

    Ok(())
}

fn merge_json_settings(source: &Path, dest: &Path) -> Result<()> {
    let source_content = std::fs::read_to_string(source)?;
    let dest_content = std::fs::read_to_string(dest)?;

    let source_json: serde_json::Value = serde_json::from_str(&source_content)
        .context("Failed to parse source settings JSON")?;
    let mut dest_json: serde_json::Value = serde_json::from_str(&dest_content)
        .context("Failed to parse destination settings JSON")?;

    // Merge source into dest (source values override dest)
    if let (serde_json::Value::Object(source_obj), serde_json::Value::Object(dest_obj)) =
        (source_json, &mut dest_json)
    {
        for (key, value) in source_obj {
            dest_obj.insert(key, value);
        }
    }

    let merged = serde_json::to_string_pretty(&dest_json)?;
    std::fs::write(dest, merged)?;

    Ok(())
}

/// Install VSIX extensions from a directory
pub fn install_vsix_extensions(vsix_dir: &Path) -> Result<()> {
    if !vsix_dir.exists() {
        println!(
            "  {} No VSIX extensions to install",
            style("-").dim()
        );
        return Ok(());
    }

    let vscode_cli = get_vscode_cli();

    for entry in std::fs::read_dir(vsix_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "vsix").unwrap_or(false) {
            let filename = entry.file_name();
            println!(
                "  Installing extension: {}",
                style(filename.to_string_lossy()).cyan()
            );

            let output = std::process::Command::new(vscode_cli)
                .args(["--install-extension", path.to_str().unwrap()])
                .output()
                .context("Failed to run VS Code CLI")?;

            if output.status.success() {
                println!(
                    "  {} Installed {}",
                    style("✓").green().bold(),
                    filename.to_string_lossy()
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "  {} Failed to install {}: {}",
                    style("✗").red().bold(),
                    filename.to_string_lossy(),
                    stderr.trim()
                );
            }
        }
    }

    Ok(())
}

fn get_vscode_cli() -> &'static str {
    "code"
}
