# Code Assist CLI - Rust Design Document

## Overview

A cross-platform CLI tool for installing and configuring AI coding assistants in enterprise environments.

## Supported Platforms

- Windows 11 (x64)
- macOS (ARM64 and x64)
- Linux: **Not supported** at this time

## Command Structure

```
code-assist [OPTIONS] <COMMAND>

Commands:
  install     Install a tool and configure environment
  uninstall   Remove a tool and its configuration
  configure   Apply/update configuration without reinstalling
  check       Verify prerequisites and current installation status
  list        List available tools and their status

Options:
  -t, --tool <TOOL>    Tool to operate on (e.g., claude-code)
  -v, --verbose        Enable verbose output
  -y, --yes            Skip confirmation prompts
  -h, --help           Print help
  -V, --version        Print version
```

## Example Usage

```bash
# Check prerequisites
./code-assist check

# Install Claude Code
./code-assist --tool claude-code install

# Uninstall Claude Code
./code-assist --tool claude-code uninstall

# Update configuration only
./code-assist --tool claude-code configure

# List all tools and status
./code-assist list
```

## Installation Flow

### 1. Prerequisites Check

| Prerequisite | Windows 11 | macOS |
|--------------|------------|-------|
| VS Code | Check registry/path | Check `/Applications/Visual Studio Code.app` |
| Git | Check `git --version` | Check `git --version` |

**If missing:** Display message directing user to:
- **Windows 11:** "Please install VS Code and Git via Software Center"
- **macOS:** "Please install VS Code and Git via Self-Service"

Exit with error code if prerequisites not met.

### 2. Tool Installation (Claude Code)

```
┌─────────────────────────────────────────────────────────┐
│  1. Download claude binary                              │
│     ├─ Try: Remote GCS bucket                          │
│     └─ Fallback: local/{version}/{platform}/claude     │
│                                                         │
│  2. Verify SHA256 checksum                             │
│                                                         │
│  3. Run: claude install                                │
│     (This sets up launcher and shell integration)      │
└─────────────────────────────────────────────────────────┘
```

### 3. VSIX Installation

Install all extensions from `local/VSIX/`:
```bash
code --install-extension local/VSIX/afs-code-cred-2.1.0.vsix
```

### 4. Configuration Deployment

#### Windows 11

| Source | Destination |
|--------|-------------|
| `local/WIN/USER-DIRECTORY/.claude/settings.json` | `%USERPROFILE%\.claude\settings.json` |
| `local/WIN/USER-DIRECTORY/.continue/certs/*.crt` | `%USERPROFILE%\.continue\certs\` |
| `local/WIN/USER-DIRECTORY/AppData/Roaming/Code/User/settings.json` | `%APPDATA%\Code\User\settings.json` |

**Environment Variables (User-level):**
- `NODE_EXTRA_CA_CERTS` = `%USERPROFILE%\.continue\certs\ZscalerRootCertificate-2048-SHA256.crt`
- Add Claude Code binary location to `PATH`

#### macOS

| Source | Destination |
|--------|-------------|
| `local/MACOS/USER-DIRECTORY/.claude/settings.json` | `~/.claude/settings.json` |
| `local/MACOS/USER-DIRECTORY/certs/*.crt` | `~/certs/` |

**Certificate Import:**
```bash
security add-trusted-cert -k "$HOME/Library/Keychains/login.keychain-db" ~/certs/zscaler-root.crt
```

**Shell Configuration:**
- Set `NODE_EXTRA_CA_CERTS` in `~/.zshrc` or `~/.bashrc`

## Project Structure

```
code-assist/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── lib.rs               # Library root
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands.rs      # Command handlers
│   ├── platform/
│   │   ├── mod.rs
│   │   ├── windows.rs       # Windows-specific logic
│   │   └── macos.rs         # macOS-specific logic
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── claude_code.rs   # Claude Code installer
│   │   └── registry.rs      # Tool registry
│   ├── config/
│   │   ├── mod.rs
│   │   └── deployment.rs    # Config file deployment
│   ├── prerequisites/
│   │   ├── mod.rs
│   │   ├── vscode.rs        # VS Code detection
│   │   └── git.rs           # Git detection
│   └── download/
│       ├── mod.rs
│       └── fallback.rs      # Download with local fallback
├── local/                   # Config files and fallback binaries
│   ├── latest
│   ├── VSIX/
│   ├── WIN/
│   ├── MACOS/
│   └── {version}/
└── tests/
    └── integration/
```

## Rust Dependencies

```toml
[package]
name = "code-assist"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["blocking"] }
sha2 = "0.10"
hex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"                    # Cross-platform directories
indicatif = "0.17"           # Progress bars
console = "0.15"             # Terminal colors/styling
anyhow = "1"                 # Error handling
thiserror = "1"              # Custom errors
tracing = "0.1"              # Logging
tracing-subscriber = "0.3"

[target.'cfg(windows)'.dependencies]
winreg = "0.52"              # Windows registry access

[target.'cfg(target_os = "macos")'.dependencies]
# macOS-specific deps if needed
```

## Key Implementation Details

### Platform Detection

```rust
#[cfg(target_os = "windows")]
pub mod platform {
    pub use crate::platform::windows::*;
}

#[cfg(target_os = "macos")]
pub mod platform {
    pub use crate::platform::macos::*;
}

#[cfg(target_os = "linux")]
compile_error!("Linux is not supported at this time");
```

### Tool Registry (Extensible)

```rust
pub trait Tool {
    fn name(&self) -> &str;
    fn check_installed(&self) -> Result<bool>;
    fn install(&self, config: &InstallConfig) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn configure(&self) -> Result<()>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}
```

### Download with Fallback

```rust
pub async fn download_with_fallback(
    remote_url: &str,
    local_path: &Path,
    output: &Path,
) -> Result<DownloadSource> {
    // Try remote first
    match download_remote(remote_url, output).await {
        Ok(_) => Ok(DownloadSource::Remote),
        Err(_) => {
            // Fall back to local
            if local_path.exists() {
                std::fs::copy(local_path, output)?;
                Ok(DownloadSource::LocalFallback)
            } else {
                Err(anyhow!("Remote unavailable and no local fallback"))
            }
        }
    }
}
```

## Build & Distribution

### Cross-Compilation Targets

```bash
# Windows
rustup target add x86_64-pc-windows-msvc

# macOS (from macOS)
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build all
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Output Binaries

```
target/
├── x86_64-pc-windows-msvc/release/code-assist.exe
├── x86_64-apple-darwin/release/code-assist
└── aarch64-apple-darwin/release/code-assist
```

## Error Handling

All errors should be user-friendly:

```
Error: VS Code is not installed

Please install VS Code via Software Center (Windows) or Self-Service (macOS)
before running this installer.
```

## Future Extensibility

To add a new tool:

1. Create `src/tools/new_tool.rs` implementing the `Tool` trait
2. Register in `src/tools/registry.rs`
3. Add config files to `local/WIN/` and `local/MACOS/`
4. Add any VSIX extensions to `local/VSIX/`
