# Code Assist CLI

A cross-platform CLI tool for installing and configuring AI coding assistants in enterprise environments.

## Supported Platforms

- Windows 11 (x64)
- macOS (ARM64 and Intel x64)

## Installation

### Option 1: Download Pre-built Binary

Download the latest release for your platform from the [Releases](https://github.com/ChristopherGRoge/Code-Assist-CLI/releases) page.

### Option 2: Clone and Build

```bash
git clone https://github.com/ChristopherGRoge/Code-Assist-CLI.git
cd Code-Assist-CLI
cargo build --release
```

The binary will be at `target/release/code-assist` (or `code-assist.exe` on Windows).

## Usage

```bash
# Check prerequisites (VS Code, Git)
./code-assist check

# List available tools
./code-assist list

# Install Claude Code
./code-assist --tool claude-code install

# Install without confirmation prompts
./code-assist -y --tool claude-code install

# Update configuration only (without reinstalling)
./code-assist --tool claude-code configure

# Uninstall
./code-assist --tool claude-code uninstall
```

## Prerequisites

Before installing, ensure you have:

- **VS Code** - Install via Software Center (Windows) or Self-Service (macOS)
- **Git** - Install via Software Center (Windows) or Self-Service (macOS)

The `check` command will verify these are installed.

## What Gets Installed

When you run `./code-assist --tool claude-code install`:

1. **Claude Code binary** - Downloaded from remote (with local fallback)
2. **VS Code extensions** - Custom VSIX files from `local/VSIX/`
3. **Configuration files**:
   - Claude Code settings (`~/.claude/settings.json`)
   - VS Code settings (merged with existing)
   - SSL certificates for Zscaler environments
4. **Environment variables**:
   - `NODE_EXTRA_CA_CERTS` (for SSL certificate)
   - PATH updated to include Claude Code

## Enterprise Configuration

The `local/` directory contains enterprise-specific configurations:

```
local/
├── VSIX/                    # VS Code extensions to install
├── WIN/USER-DIRECTORY/      # Windows config files
│   ├── .claude/settings.json
│   ├── .continue/certs/     # SSL certificates
│   └── AppData/.../settings.json  # VS Code settings
├── MACOS/USER-DIRECTORY/    # macOS config files
│   ├── .claude/settings.json
│   └── certs/               # SSL certificates
└── {version}/               # Fallback binaries (optional)
```

## Building from Source

### Requirements

- Rust 1.70 or later
- Cargo

### Build

```bash
# Debug build
cargo build

# Release build (optimized, smaller)
cargo build --release
```

### Cross-compilation

The GitHub Actions workflow automatically builds for:
- `x86_64-pc-windows-msvc` (Windows x64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS ARM64)

## Adding New Tools

To add support for a new tool:

1. Create `src/tools/new_tool.rs` implementing the `Tool` trait
2. Register in `src/tools/mod.rs`
3. Add config files to `local/WIN/` and `local/MACOS/`
4. Add any VSIX extensions to `local/VSIX/`

## License

MIT
