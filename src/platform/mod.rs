#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

use std::path::PathBuf;

/// Platform-specific configuration paths
pub struct PlatformPaths {
    pub home_dir: PathBuf,
    pub claude_config_dir: PathBuf,
    pub vscode_settings_dir: PathBuf,
    pub certs_dir: PathBuf,
}

/// Get platform-specific paths
pub fn get_paths() -> PlatformPaths {
    #[cfg(target_os = "windows")]
    {
        return windows::get_paths();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::get_paths();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux/other - for development only
        let home_dir = dirs::home_dir().expect("Could not determine home directory");
        PlatformPaths {
            home_dir: home_dir.clone(),
            claude_config_dir: home_dir.join(".claude"),
            vscode_settings_dir: home_dir.join(".config").join("Code").join("User"),
            certs_dir: home_dir.join("certs"),
        }
    }
}

/// Print platform-specific installation instructions for missing prerequisites
pub fn print_install_instructions() {
    #[cfg(target_os = "windows")]
    {
        windows::print_install_instructions();
    }

    #[cfg(target_os = "macos")]
    {
        macos::print_install_instructions();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        println!("Linux is not supported. Please use Windows or macOS.");
    }
}

/// Get the platform identifier for downloads
pub fn get_platform_id() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "win32-x64";
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "darwin-x64";
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "darwin-arm64";
    }

    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    {
        // For development on Linux x64
        "linux-x64"
    }
}

/// Get the binary name for the platform
pub fn get_binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        return "claude.exe";
    }

    #[cfg(not(target_os = "windows"))]
    {
        return "claude";
    }
}

/// Set an environment variable persistently for the user
pub fn set_user_env_var(name: &str, value: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        return windows::set_user_env_var(name, value);
    }

    #[cfg(target_os = "macos")]
    {
        return macos::set_user_env_var(name, value);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (name, value);
        anyhow::bail!("Linux is not supported")
    }
}

/// Add a directory to the user's PATH
pub fn add_to_path(dir: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        return windows::add_to_path(dir);
    }

    #[cfg(target_os = "macos")]
    {
        return macos::add_to_path(dir);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = dir;
        anyhow::bail!("Linux is not supported")
    }
}

/// Import a certificate into the system trust store
pub fn import_certificate(cert_path: &std::path::Path) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        return windows::import_certificate(cert_path);
    }

    #[cfg(target_os = "macos")]
    {
        return macos::import_certificate(cert_path);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = cert_path;
        anyhow::bail!("Linux is not supported")
    }
}
