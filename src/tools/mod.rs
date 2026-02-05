mod claude_code;

use anyhow::{anyhow, Result};

pub use claude_code::ClaudeCode;

/// Trait for installable tools
pub trait Tool {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn is_installed(&self) -> Result<bool>;
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn configure(&self) -> Result<()>;
}

/// Get a tool by name
pub fn get_tool(name: &str) -> Result<Box<dyn Tool>> {
    match name {
        "claude-code" => Ok(Box::new(ClaudeCode::new())),
        _ => Err(anyhow!(
            "Unknown tool: '{}'. Run 'code-assist list' to see available tools.",
            name
        )),
    }
}

/// List all available tools
pub fn list_tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(ClaudeCode::new())]
}
