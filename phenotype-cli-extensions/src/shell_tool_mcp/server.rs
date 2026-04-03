//! MCP Server implementation for shell command execution

use std::process::{Command, Stdio};

/// MCP Server handle
pub struct McpServer;

impl McpServer {
    /// Create new MCP server instance
    pub fn new() -> Self {
        Self
    }
    
    /// Execute a shell command
    pub fn execute(&self, cmd: &str, args: &[&str]) -> Result<String, std::io::Error> {
        let output = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}
