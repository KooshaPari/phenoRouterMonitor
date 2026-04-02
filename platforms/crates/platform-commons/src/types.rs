//! Shared type definitions for cross-platform use.

use serde::{Deserialize, Serialize};

/// Platform identifier for routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Phenotype infrastructure kit
    Phenotype,
    /// TheGent agent orchestration
    TheGent,
    /// Future platform (reserved)
    Future,
}

impl Default for Platform {
    fn default() -> Self {
        Self::Phenotype
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Phenotype => write!(f, "phenotype"),
            Self::TheGent => write!(f, "thegent"),
            Self::Future => write!(f, "future"),
        }
    }
}

/// Version information for platform components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl Version {
    /// Creates a new version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
        }
    }

    /// Creates a new pre-release version.
    pub fn pre(major: u32, minor: u32, patch: u32, pre: impl Into<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Some(pre.into()),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pre) = &self.pre {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}
