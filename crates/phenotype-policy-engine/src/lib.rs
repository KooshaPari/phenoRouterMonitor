//! Generic policy evaluation engine for Phenotype.
//!
//! This crate provides a flexible, domain-agnostic policy engine that can evaluate
//! policies against evaluation contexts. It supports multiple rule types (Allow, Deny, Require)
//! and pattern matching via regular expressions.
//!
//! # Core Concepts
//!
//! - **Policy**: A named set of evaluation rules with a description.
//! - **Rule**: A constraint that can be evaluated against a context (Allow, Deny, Require).
//! - **EvaluationContext**: A key-value map of facts used for policy evaluation.
//! - **PolicyResult**: The outcome of policy evaluation, including any violations.
//! - **PolicyEngine**: Orchestrator that evaluates contexts against a set of policies.

pub mod context;
pub mod engine;
pub mod error;
pub mod loader;
pub mod policy;
pub mod result;
pub mod rule;

pub use context::EvaluationContext;
pub use engine::PolicyEngine;
pub use error::PolicyEngineError;
pub use policy::Policy;
pub use result::{PolicyResult, Severity, Violation};
pub use rule::{Rule, RuleType};

/// Re-export commonly used types for convenience.
pub mod prelude {
    pub use crate::{
        context::EvaluationContext,
        engine::PolicyEngine,
        error::PolicyEngineError,
        policy::Policy,
        result::{PolicyResult, Severity, Violation},
        rule::{Rule, RuleType},
    };
}
