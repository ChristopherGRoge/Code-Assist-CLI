use super::PlatformPaths;
use anyhow::{Context, Result};
use console::style;
use std::path::PathBuf;

pub fn get_paths() -> PlatformPaths {
    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    let appdata = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir.join("AppData").join("Roaming"));

    PlatformPaths {
        home_dir: home_dir.clone(),
        claude_config_dir: home_dir.join(".claude"),
        vscode_settings_dir: appdata.join("Code").join("User"),
        certs_dir: home_dir.join(".continue").join("certs"),
    }
}

pub fn print_install_instructions() {
    println!(
        "{}\n",
        style("Please install the missing software via Software Center:").yellow()
    );
    println!("  1. Open Software Center from the Start menu");
    println!("  2. Search for and install:");
    println!("     - Visual Studio Code");
    println!("     - Git for Windows");
    println!("\nOnce installed, run this command again.");
}

pub fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    env.set_value(name, &value)
        .context(format!("Failed to set environment variable {}", name))?;

    // Notify the system of environment change
    broadcast_environment_change();

    Ok(())
}

pub fn add_to_path(dir: &str) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    let current_path: String = env.get_value("Path").unwrap_or_default();

    // Check if already in PATH
    if current_path
        .split(';')
        .any(|p| p.eq_ignore_ascii_case(dir))
    {
        return Ok(());
    }

    let new_path = if current_path.is_empty() {
        dir.to_string()
    } else {
        format!("{};{}", current_path, dir)
    };

    env.set_value("Path", &new_path)
        .context("Failed to update PATH")?;

    broadcast_environment_change();

    Ok(())
}

pub fn import_certificate(_cert_path: &std::path::Path) -> Result<()> {
    // On Windows, we use NODE_EXTRA_CA_CERTS environment variable
    // instead of importing to system store (which requires admin)
    // The certificate path is set via set_user_env_var
    Ok(())
}

fn broadcast_environment_change() {
    // This notifies Windows Explorer and other applications that
    // environment variables have changed
    #[cfg(target_os = "windows")]
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        #[link(name = "user32")]
        extern "system" {
            fn SendMessageTimeoutW(
                hwnd: *mut std::ffi::c_void,
                msg: u32,
                wparam: usize,
                lparam: *const u16,
                flags: u32,
                timeout: u32,
                result: *mut usize,
            ) -> isize;
        }

        const HWND_BROADCAST: *mut std::ffi::c_void = 0xffff as *mut _;
        const WM_SETTINGCHANGE: u32 = 0x001A;
        const SMTO_ABORTIFHUNG: u32 = 0x0002;

        let environment: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut result: usize = 0;
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            environment.as_ptr(),
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        );
    }
}

/// Check if VS Code is installed on Windows
pub fn check_vscode_installed() -> bool {
    // Check common installation paths
    let paths = [
        r"C:\Program Files\Microsoft VS Code\Code.exe",
        r"C:\Program Files (x86)\Microsoft VS Code\Code.exe",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return true;
        }
    }

    // Check if 'code' command is available
    std::process::Command::new("code")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Git is installed on Windows
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
