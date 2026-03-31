use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    #[serde(default = "default_agent")]
    pub default_agent: String,
    #[serde(default = "default_max_subagents")]
    pub max_subagents: u32,
    #[serde(default = "default_max_review_cycles")]
    pub max_review_cycles: u32,
    #[serde(default = "default_review_poll_interval")]
    pub review_poll_interval_secs: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_agent: default_agent(),
            max_subagents: default_max_subagents(),
            max_review_cycles: default_max_review_cycles(),
            review_poll_interval_secs: default_review_poll_interval(),
        }
    }
}

fn default_agent() -> String {
    "claude-code".to_string()
}

fn default_max_subagents() -> u32 {
    3
}

fn default_max_review_cycles() -> u32 {
    5
}

fn default_review_poll_interval() -> u64 {
    30
}
