//! Phenotype MCP Server
pub mod tools;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Config { pub name: String, pub version: String }
impl Default for Config { fn default() -> Self { Self { name: "phenotype".into(), version: env!("CARGO_PKG_VERSION").into() } } }

pub struct Server { config: Config, tools: HashMap<String, String> }
impl Server {
    pub fn new() -> Self { let mut s = Self { config: Config::default(), tools: HashMap::new() }; s.register_default_tools(); s }
    fn register_default_tools(&mut self) { self.tools.insert("agileplus_create_feature".into(), "Create feature specs".into()); self.tools.insert("agileplus_validate".into(), "Validate features".into()); self.tools.insert("agent_dispatch".into(), "Dispatch agent tasks".into()); }
    pub fn info(&self) -> ServerInfo { ServerInfo { name: self.config.name.clone(), version: self.config.version.clone(), tool_count: self.tools.len() } }
    pub fn list_tools(&self) -> Vec<ToolInfo> { self.tools.iter().map(|(n, d)| ToolInfo { name: n.clone(), description: d.clone() }).collect() }
}

#[derive(Debug, Clone, Serialize, Deserialize)] pub struct ServerInfo { pub name: String, pub version: String, pub tool_count: usize }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct ToolInfo { pub name: String, pub description: String }

#[cfg(test)] mod tests { use super::*; #[test] fn test_server() { let s = Server::new(); let info = s.info(); assert_eq!(info.name, "phenotype"); assert!(info.tool_count > 0); } }
