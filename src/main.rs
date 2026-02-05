use anyhow::Result;
use clap::Parser;
use console::style;
use tracing_subscriber::EnvFilter;

mod cli;
mod config;
mod download;
mod platform;
mod prerequisites;
mod tools;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Check platform support - warn on Linux but allow for development
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        eprintln!(
            "{} Warning: This platform is not officially supported. Some features may not work.",
            style("!").yellow().bold()
        );
    }

    match cli.command {
        Commands::Check => cmd_check(),
        Commands::Install { tool } => cmd_install(&tool, cli.yes),
        Commands::Uninstall { tool } => cmd_uninstall(&tool, cli.yes),
        Commands::Configure { tool } => cmd_configure(&tool),
        Commands::List => cmd_list(),
    }
}

fn cmd_check() -> Result<()> {
    println!(
        "{} Checking prerequisites...\n",
        style("→").cyan().bold()
    );

    let vscode_ok = prerequisites::check_vscode();
    let git_ok = prerequisites::check_git();

    println!();

    if !vscode_ok || !git_ok {
        println!(
            "{} Some prerequisites are missing.\n",
            style("✗").red().bold()
        );
        platform::print_install_instructions();
        std::process::exit(1);
    }

    println!(
        "{} All prerequisites satisfied!",
        style("✓").green().bold()
    );
    Ok(())
}

fn cmd_install(tool_name: &str, skip_confirm: bool) -> Result<()> {
    // First check prerequisites
    println!(
        "{} Checking prerequisites...",
        style("→").cyan().bold()
    );

    let vscode_ok = prerequisites::check_vscode();
    let git_ok = prerequisites::check_git();

    if !vscode_ok || !git_ok {
        println!(
            "\n{} Prerequisites not met.\n",
            style("✗").red().bold()
        );
        platform::print_install_instructions();
        std::process::exit(1);
    }

    println!(
        "{} Prerequisites satisfied.\n",
        style("✓").green().bold()
    );

    // Get the tool
    let tool = tools::get_tool(tool_name)?;

    if !skip_confirm {
        println!(
            "This will install {} and configure your environment.",
            style(tool.display_name()).cyan()
        );
        print!("Continue? [Y/n] ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if !input.is_empty() && input != "y" && input != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!();
    tool.install()?;

    println!(
        "\n{} {} installed successfully!",
        style("✓").green().bold(),
        tool.display_name()
    );

    Ok(())
}

fn cmd_uninstall(tool_name: &str, skip_confirm: bool) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;

    if !skip_confirm {
        println!(
            "This will uninstall {} and remove its configuration.",
            style(tool.display_name()).cyan()
        );
        print!("Continue? [Y/n] ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if !input.is_empty() && input != "y" && input != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!();
    tool.uninstall()?;

    println!(
        "\n{} {} uninstalled successfully!",
        style("✓").green().bold(),
        tool.display_name()
    );

    Ok(())
}

fn cmd_configure(tool_name: &str) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;

    println!(
        "{} Configuring {}...\n",
        style("→").cyan().bold(),
        tool.display_name()
    );

    tool.configure()?;

    println!(
        "\n{} Configuration complete!",
        style("✓").green().bold()
    );

    Ok(())
}

fn cmd_list() -> Result<()> {
    println!("{} Available tools:\n", style("→").cyan().bold());

    for tool in tools::list_tools() {
        let status = if tool.is_installed()? {
            style("installed").green()
        } else {
            style("not installed").dim()
        };

        println!("  {} - {} [{}]", tool.name(), tool.display_name(), status);
    }

    Ok(())
}
