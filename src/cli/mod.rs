use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "code-assist")]
#[command(author, version, about = "Cross-platform CLI for installing AI coding assistants")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Skip confirmation prompts
    #[arg(short, long, global = true)]
    pub yes: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check prerequisites (VS Code, Git)
    Check,

    /// Install a tool and configure environment
    Install {
        /// Tool to install (e.g., claude-code)
        #[arg(short, long)]
        tool: String,
    },

    /// Uninstall a tool and remove configuration
    Uninstall {
        /// Tool to uninstall
        #[arg(short, long)]
        tool: String,
    },

    /// Apply/update configuration without reinstalling
    Configure {
        /// Tool to configure
        #[arg(short, long)]
        tool: String,
    },

    /// List available tools and their installation status
    List,
}
