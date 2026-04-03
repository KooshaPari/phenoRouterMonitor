//! Sandbox adapters for three-tier isolation

use crate::domain::{Skill, ExecutionMode};
use crate::ports::SandboxPort;
use crate::SkillsError;

/// WASM sandbox adapter (Tier 1)
#[derive(Debug)]
pub struct WasmSandbox;

impl WasmSandbox {
    pub fn new() -> Self {
        Self
    }
}

impl SandboxPort for WasmSandbox {
    fn execute(&self, skill: &Skill, input: serde_json::Value) -> Result<serde_json::Value, SkillsError> {
        // Placeholder for WASM execution
        // Would use wasmtime here
        Ok(serde_json::json!({
            "status": "success",
            "skill": skill.name(),
            "sandbox": "wasm",
            "input": input
        }))
    }
    
    fn is_available(&self) -> bool {
        // Check if wasmtime is available
        true // Placeholder
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// gVisor sandbox adapter (Tier 2)
#[derive(Debug)]
pub struct GVisorSandbox;

impl GVisorSandbox {
    pub fn new() -> Self {
        Self
    }
}

impl SandboxPort for GVisorSandbox {
    fn execute(&self, skill: &Skill, input: serde_json::Value) -> Result<serde_json::Value, SkillsError> {
        // Placeholder for gVisor execution
        Ok(serde_json::json!({
            "status": "success",
            "skill": skill.name(),
            "sandbox": "gvisor",
            "input": input
        }))
    }
    
    fn is_available(&self) -> bool {
        // Check if runsc is available
        std::process::Command::new("runsc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for GVisorSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Firecracker sandbox adapter (Tier 3)
#[derive(Debug)]
pub struct FirecrackerSandbox;

impl FirecrackerSandbox {
    pub fn new() -> Self {
        Self
    }
}

impl SandboxPort for FirecrackerSandbox {
    fn execute(&self, skill: &Skill, input: serde_json::Value) -> Result<serde_json::Value, SkillsError> {
        // Placeholder for Firecracker execution
        Ok(serde_json::json!({
            "status": "success",
            "skill": skill.name(),
            "sandbox": "firecracker",
            "input": input
        }))
    }
    
    fn is_available(&self) -> bool {
        // Check if firecracker is available
        std::process::Command::new("firecracker")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for FirecrackerSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Sandbox selector based on execution mode
pub fn select_sandbox(mode: ExecutionMode) -> Box<dyn SandboxPort> {
    match mode {
        ExecutionMode::WASM => Box::new(WasmSandbox::new()),
        ExecutionMode::GVisor => Box::new(GVisorSandbox::new()),
        ExecutionMode::Firecracker => Box::new(FirecrackerSandbox::new()),
        ExecutionMode::InProcess => Box::new(WasmSandbox::new()), // Fallback
    }
}
