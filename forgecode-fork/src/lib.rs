//! Forgecode Fork - Subagent System with YAML Configuration
//!
//! This library provides a zero-code agent definition system that allows agents to be loaded
//! from YAML configuration files without requiring Rust code changes.
//!
//! # Architecture
//!
//! The system consists of three main components:
//! 1. **SubagentConfig Trait**: Defines the interface for agent configurations
//! 2. **YAML Parser**: Loads agent definitions from YAML files
//! 3. **Agent Discovery**: Automatically finds and registers agents from the filesystem
//! 4. **Agent Registry**: Central registry for agent lookup and management

pub mod config;
pub mod discovery;
pub mod error;
pub mod parser;
pub mod registry;

pub use config::{AgentConfig, SubagentConfig};
pub use discovery::AgentDiscovery;
pub use error::{ForgeError, Result};
pub use registry::AgentRegistry;

/// Agent execution context
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub agent_id: String,
    pub agent_name: String,
    pub input: serde_json::Value,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
