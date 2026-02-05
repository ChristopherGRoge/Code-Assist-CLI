use console::style;

/// Check if VS Code is installed
pub fn check_vscode() -> bool {
    let installed = is_vscode_installed();

    if installed {
        println!(
            "  {} VS Code",
            style("✓").green().bold()
        );
    } else {
        println!(
            "  {} VS Code - {}",
            style("✗").red().bold(),
            style("not installed").red()
        );
    }

    installed
}

/// Check if Git is installed
pub fn check_git() -> bool {
    let installed = is_git_installed();

    if installed {
        println!(
            "  {} Git",
            style("✓").green().bold()
        );
    } else {
        println!(
            "  {} Git - {}",
            style("✗").red().bold(),
            style("not installed").red()
        );
    }

    installed
}

fn is_vscode_installed() -> bool {
    // Check if VS Code app exists (platform-specific paths)
    #[cfg(target_os = "windows")]
    {
        let paths = [
            r"C:\Program Files\Microsoft VS Code\Code.exe",
            r"C:\Program Files (x86)\Microsoft VS Code\Code.exe",
        ];
        for path in &paths {
            if std::path::Path::new(path).exists() {
                return true;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if std::path::Path::new("/Applications/Visual Studio Code.app").exists() {
            return true;
        }
    }

    // Check if 'code' command is available (works on all platforms)
    std::process::Command::new("code")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn is_git_installed() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
