//! # Phenotype CLI Extensions
//!
//! CLI extensions for the Phenotype ecosystem including:
//! - Kitty graphics protocol support
//! - Model Context Protocol (MCP) shell integration
//! - TypeScript SDK bindings

pub mod kitty;
pub mod shell_tool_mcp;
pub mod sdk_typescript;

pub use kitty::graphics;
pub use shell_tool_mcp::server;
pub use sdk_typescript::bindings;
