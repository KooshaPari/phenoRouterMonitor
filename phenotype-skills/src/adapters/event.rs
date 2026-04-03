//! Event adapter using tracing

use crate::domain::SkillEvent;
use crate::ports::EventPort;
use crate::SkillsError;
use tracing::{info, debug, warn};

/// Event port implementation using tracing
#[derive(Debug)]
pub struct TracingEventPort;

impl TracingEventPort {
    pub fn new() -> Self {
        Self
    }
}

impl EventPort for TracingEventPort {
    fn emit(&self, event: SkillEvent) -> Result<(), SkillsError> {
        match &event {
            SkillEvent::Registered { skill_id, manifest, .. } => {
                info!("Skill registered: {} ({})", manifest.name, skill_id);
            }
            SkillEvent::Unregistered { skill_id, .. } => {
                info!("Skill unregistered: {}", skill_id);
            }
            SkillEvent::Activated { skill_id, .. } => {
                debug!("Skill activated: {}", skill_id);
            }
            SkillEvent::Deactivated { skill_id, .. } => {
                debug!("Skill deactivated: {}", skill_id);
            }
            SkillEvent::Updated { skill_id, old_manifest, new_manifest, .. } => {
                info!("Skill updated: {} ({} -> {})", 
                    new_manifest.name, 
                    old_manifest.version, 
                    new_manifest.version
                );
            }
            SkillEvent::DependencyResolved { skill_id, dependency_name, resolved_version, .. } => {
                debug!("Dependency resolved: {} -> {}@{}", 
                    skill_id, dependency_name, resolved_version);
            }
            SkillEvent::ExecutionStarted { skill_id, execution_id, .. } => {
                debug!("Execution started: {} ({})", skill_id, execution_id);
            }
            SkillEvent::ExecutionCompleted { skill_id, execution_id, success, .. } => {
                if *success {
                    debug!("Execution completed: {} ({}) - success", skill_id, execution_id);
                } else {
                    warn!("Execution completed: {} ({}) - failed", skill_id, execution_id);
                }
            }
        }
        
        Ok(())
    }
}

impl Default for TracingEventPort {
    fn default() -> Self {
        Self::new()
    }
}
