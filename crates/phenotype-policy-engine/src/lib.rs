//! phenotype-policy-engine - Policy evaluation engine with TOML support.

pub mod context;
pub mod engine;
pub mod error;
pub mod loader;
pub mod policy;
pub mod result;
pub mod rule;

pub use context::EvaluationContext;
pub use engine::PolicyEngine;
pub use error::{PolicyEngineError, ErrorKind};
pub use loader::PolicyLoader;
pub use policy::{EvaluablePolicy, Policy};
pub use result::{PolicyResult, Severity, Violation};
pub use rule::{Rule, RuleType};
