//! # phenotype-policy-engine
//!
//! Generic policy evaluation engine supporting configurable evaluation modes.
//!
//! Policies are evaluated against a context type `Ctx` and produce a
//! [`PolicyResult`]. A [`PolicySet`] aggregates multiple policies and applies
//! an [`EvaluationMode`] to derive a final decision. [`PolicyEngine`] wraps a
//! policy set with tracing instrumentation and timing.

use std::fmt;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by the policy engine.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// A policy evaluation failed unexpectedly.
    #[error("policy evaluation failed: {0}")]
    EvaluationFailed(String),

    /// The policy set contains no policies.
    #[error("policy set is empty")]
    EmptyPolicySet,
}

// ---------------------------------------------------------------------------
// PolicyResult
// ---------------------------------------------------------------------------

/// Outcome of a single policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyResult {
    /// The policy explicitly allows the action.
    Allow,
    /// The policy explicitly denies the action.
    Deny {
        /// Human-readable reason for denial.
        reason: String,
    },
    /// The policy has no opinion.
    Abstain,
}

impl fmt::Display for PolicyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "Allow"),
            Self::Deny { reason } => write!(f, "Deny({reason})"),
            Self::Abstain => write!(f, "Abstain"),
        }
    }
}

// ---------------------------------------------------------------------------
// Policy trait
// ---------------------------------------------------------------------------

/// A single policy that evaluates a context and returns a [`PolicyResult`].
pub trait Policy<Ctx>: Send + Sync {
    /// Human-readable name of this policy.
    fn name(&self) -> &str;

    /// Evaluate the policy against the given context.
    fn evaluate(&self, context: &Ctx) -> PolicyResult;
}

// ---------------------------------------------------------------------------
// Built-in policies
// ---------------------------------------------------------------------------

/// A policy that always allows.
pub struct AlwaysAllow;

impl<Ctx> Policy<Ctx> for AlwaysAllow {
    fn name(&self) -> &str {
        "AlwaysAllow"
    }

    fn evaluate(&self, _context: &Ctx) -> PolicyResult {
        PolicyResult::Allow
    }
}

/// A policy that always denies with a fixed reason.
pub struct AlwaysDeny {
    /// Reason surfaced in the deny result.
    pub reason: String,
}

impl AlwaysDeny {
    /// Create a new `AlwaysDeny` with the given reason.
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl<Ctx> Policy<Ctx> for AlwaysDeny {
    fn name(&self) -> &str {
        "AlwaysDeny"
    }

    fn evaluate(&self, _context: &Ctx) -> PolicyResult {
        PolicyResult::Deny {
            reason: self.reason.clone(),
        }
    }
}

/// A policy built from a closure.
pub struct ClosurePolicy<F> {
    name: String,
    f: F,
}

impl<F> ClosurePolicy<F> {
    /// Create a new closure-based policy.
    pub fn new(name: impl Into<String>, f: F) -> Self {
        Self {
            name: name.into(),
            f,
        }
    }
}

impl<Ctx, F> Policy<Ctx> for ClosurePolicy<F>
where
    F: Fn(&Ctx) -> PolicyResult + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&self, context: &Ctx) -> PolicyResult {
        (self.f)(context)
    }
}

// ---------------------------------------------------------------------------
// EvaluationMode
// ---------------------------------------------------------------------------

/// Strategy for combining individual policy results into a final decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationMode {
    /// Every non-abstaining policy must allow. A single deny causes denial.
    AllMustAllow,
    /// At least one policy must allow. If none allow, the action is denied.
    AnyMustAllow,
    /// A strict majority of non-abstaining policies must allow.
    MajorityAllow,
}

// ---------------------------------------------------------------------------
// PolicyDecision
// ---------------------------------------------------------------------------

/// The final decision produced by [`PolicyEngine::evaluate`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Whether the action is allowed.
    pub allowed: bool,
    /// Number of policies that were evaluated.
    pub evaluated_policies: usize,
    /// Collected reasons (from denials, or from the mode summary).
    pub reasons: Vec<String>,
    /// Wall-clock duration of the evaluation in milliseconds.
    #[serde(with = "duration_millis")]
    pub duration: std::time::Duration,
    /// Timestamp when the decision was made.
    pub timestamp: DateTime<Utc>,
}

/// Serde helper: serialize/deserialize [`std::time::Duration`] as milliseconds.
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u128(d.as_millis())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let millis = u64::deserialize(d)?;
        Ok(Duration::from_millis(millis))
    }
}

// ---------------------------------------------------------------------------
// PolicySet
// ---------------------------------------------------------------------------

/// An ordered collection of policies with an evaluation mode.
pub struct PolicySet<Ctx> {
    policies: Vec<Box<dyn Policy<Ctx>>>,
    mode: EvaluationMode,
}

impl<Ctx> PolicySet<Ctx> {
    /// Create a new empty policy set with the given evaluation mode.
    pub fn new(mode: EvaluationMode) -> Self {
        Self {
            policies: Vec::new(),
            mode,
        }
    }

    /// Add a policy to the set.
    pub fn add(&mut self, policy: impl Policy<Ctx> + 'static) {
        self.policies.push(Box::new(policy));
    }

    /// Builder-style: add a policy and return self.
    pub fn with(mut self, policy: impl Policy<Ctx> + 'static) -> Self {
        self.add(policy);
        self
    }

    /// Number of registered policies.
    pub fn len(&self) -> usize {
        self.policies.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }

    /// The configured evaluation mode.
    pub fn mode(&self) -> EvaluationMode {
        self.mode
    }

    /// Evaluate all policies and produce a decision.
    pub fn evaluate(&self, context: &Ctx) -> Result<PolicyDecision, PolicyError> {
        if self.policies.is_empty() {
            return Err(PolicyError::EmptyPolicySet);
        }

        let start = Instant::now();
        let mut allows: usize = 0;
        let mut denies: usize = 0;
        let mut reasons: Vec<String> = Vec::new();

        for policy in &self.policies {
            let result = policy.evaluate(context);
            debug!(policy = policy.name(), %result, "policy evaluated");

            match result {
                PolicyResult::Allow => allows += 1,
                PolicyResult::Deny { reason } => {
                    denies += 1;
                    reasons.push(format!("{}: {}", policy.name(), reason));
                }
                PolicyResult::Abstain => {}
            }
        }

        let voted = allows + denies;
        let allowed = match self.mode {
            EvaluationMode::AllMustAllow => denies == 0 && allows > 0,
            EvaluationMode::AnyMustAllow => allows > 0,
            EvaluationMode::MajorityAllow => voted > 0 && allows > voted / 2,
        };

        if !allowed && reasons.is_empty() {
            reasons.push("no policy allowed the action".to_string());
        }

        let duration = start.elapsed();

        Ok(PolicyDecision {
            allowed,
            evaluated_policies: self.policies.len(),
            reasons,
            duration,
            timestamp: Utc::now(),
        })
    }
}

// ---------------------------------------------------------------------------
// PolicyEngine
// ---------------------------------------------------------------------------

/// Instrumented wrapper around [`PolicySet`] that adds tracing and logging.
pub struct PolicyEngine<Ctx> {
    policy_set: PolicySet<Ctx>,
    name: String,
}

impl<Ctx> PolicyEngine<Ctx> {
    /// Create a new engine wrapping the given policy set.
    pub fn new(name: impl Into<String>, policy_set: PolicySet<Ctx>) -> Self {
        Self {
            policy_set,
            name: name.into(),
        }
    }

    /// Evaluate the context against all registered policies.
    #[instrument(skip_all, fields(engine = %self.name, mode = ?self.policy_set.mode(), policy_count = self.policy_set.len()))]
    pub fn evaluate(&self, context: &Ctx) -> Result<PolicyDecision, PolicyError> {
        info!(engine = %self.name, "starting policy evaluation");

        let decision = self.policy_set.evaluate(context)?;

        if decision.allowed {
            info!(
                engine = %self.name,
                evaluated = decision.evaluated_policies,
                duration_ms = decision.duration.as_millis() as u64,
                "policy evaluation: ALLOWED"
            );
        } else {
            warn!(
                engine = %self.name,
                evaluated = decision.evaluated_policies,
                reasons = ?decision.reasons,
                duration_ms = decision.duration.as_millis() as u64,
                "policy evaluation: DENIED"
            );
        }

        Ok(decision)
    }

    /// Reference to the underlying policy set.
    pub fn policy_set(&self) -> &PolicySet<Ctx> {
        &self.policy_set
    }

    /// Engine name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_allow_returns_allow() {
        let result = AlwaysAllow.evaluate(&());
        assert_eq!(result, PolicyResult::Allow);
    }

    #[test]
    fn always_deny_returns_deny() {
        let policy = AlwaysDeny::new("nope");
        let result = policy.evaluate(&());
        assert_eq!(
            result,
            PolicyResult::Deny {
                reason: "nope".into()
            }
        );
    }

    #[test]
    fn closure_policy_works() {
        let policy = ClosurePolicy::new("even_only", |n: &i32| {
            if n % 2 == 0 {
                PolicyResult::Allow
            } else {
                PolicyResult::Deny {
                    reason: "odd number".into(),
                }
            }
        });

        assert_eq!(policy.evaluate(&4), PolicyResult::Allow);
        assert!(matches!(policy.evaluate(&3), PolicyResult::Deny { .. }));
    }

    #[test]
    fn all_must_allow_mode() {
        let set = PolicySet::new(EvaluationMode::AllMustAllow)
            .with(AlwaysAllow)
            .with(AlwaysAllow);

        let decision = set.evaluate(&()).unwrap();
        assert!(decision.allowed);
        assert_eq!(decision.evaluated_policies, 2);
    }

    #[test]
    fn all_must_allow_denied_by_one() {
        let set = PolicySet::new(EvaluationMode::AllMustAllow)
            .with(AlwaysAllow)
            .with(AlwaysDeny::new("blocked"));

        let decision = set.evaluate(&()).unwrap();
        assert!(!decision.allowed);
        assert!(!decision.reasons.is_empty());
    }

    #[test]
    fn any_must_allow_with_one_allow() {
        let set = PolicySet::new(EvaluationMode::AnyMustAllow)
            .with(AlwaysDeny::new("no"))
            .with(AlwaysAllow);

        let decision = set.evaluate(&()).unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn any_must_allow_all_deny() {
        let set = PolicySet::new(EvaluationMode::AnyMustAllow)
            .with(AlwaysDeny::new("a"))
            .with(AlwaysDeny::new("b"));

        let decision = set.evaluate(&()).unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn majority_allow_mode() {
        let set = PolicySet::new(EvaluationMode::MajorityAllow)
            .with(AlwaysAllow)
            .with(AlwaysAllow)
            .with(AlwaysDeny::new("dissent"));

        let decision = set.evaluate(&()).unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn majority_deny_mode() {
        let set = PolicySet::new(EvaluationMode::MajorityAllow)
            .with(AlwaysAllow)
            .with(AlwaysDeny::new("a"))
            .with(AlwaysDeny::new("b"));

        let decision = set.evaluate(&()).unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn empty_policy_set_errors() {
        let set = PolicySet::<()>::new(EvaluationMode::AllMustAllow);
        assert!(set.evaluate(&()).is_err());
    }

    #[test]
    fn engine_evaluate_traces() {
        let set = PolicySet::new(EvaluationMode::AllMustAllow).with(AlwaysAllow);
        let engine = PolicyEngine::new("test-engine", set);
        let decision = engine.evaluate(&()).unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn policy_decision_serializes() {
        let set = PolicySet::new(EvaluationMode::AllMustAllow).with(AlwaysAllow);
        let decision = set.evaluate(&()).unwrap();
        let json = serde_json::to_string(&decision).unwrap();
        assert!(json.contains("\"allowed\":true"));
    }

    #[test]
    fn abstain_does_not_count_as_allow() {
        let set = PolicySet::new(EvaluationMode::AllMustAllow).with(ClosurePolicy::new(
            "abstainer",
            |_: &()| PolicyResult::Abstain,
        ));
        let decision = set.evaluate(&()).unwrap();
        assert!(!decision.allowed);
    }
}
