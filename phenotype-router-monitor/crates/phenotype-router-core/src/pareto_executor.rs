//! Route Executor: dispatches tasks to providers based on RoutingDecision.
//!
//! Integrates routing decisions from ParetoRouter with actual task execution
//! by dispatching to the appropriate provider (Lifecycle or TheGent).

use crate::audit::{AuditLogger, AuditRecord};
use crate::router::{RoutingDecision, RoutingMode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// Outcome of executing a routed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    /// Unique decision ID (echoed from AuditRecord).
    pub decision_id: String,
    /// Which provider/mode was dispatched to.
    pub provider: String,
    /// Model used for execution.
    pub model: String,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Estimated cost in USD.
    pub cost_usd: f64,
    /// Whether execution succeeded.
    pub success: bool,
    /// Error message if execution failed.
    pub error: Option<String>,
}

/// Dispatch target: maps RoutingMode to a concrete provider + model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchTarget {
    /// Provider name (e.g. "lifecycle", "thegent").
    pub provider: String,
    /// Model alias (e.g. "gemini-3-flash", "claude-sonnet-4.6").
    pub model: String,
    /// Estimated cost per 1k tokens in USD.
    pub cost_per_1k: f64,
}

impl DispatchTarget {
    /// Lifecycle default: cheap/fast provider.
    pub fn lifecycle() -> Self {
        Self {
            provider: "lifecycle".to_string(),
            model: "gemini-3-flash".to_string(),
            cost_per_1k: 0.0003,
        }
    }

    /// TheGent default: high-quality provider.
    pub fn thegent() -> Self {
        Self {
            provider: "thegent".to_string(),
            model: "claude-sonnet-4.6".to_string(),
            cost_per_1k: 0.003,
        }
    }

    /// Resolve target from routing mode.
    pub fn from_mode(mode: RoutingMode) -> Self {
        match mode {
            RoutingMode::Lifecycle => Self::lifecycle(),
            RoutingMode::TheGent => Self::thegent(),
        }
    }
}

/// Trait for dispatch backends.
///
/// Implementations execute a task string and return success/failure.
/// The default implementation is a no-op stub used in tests.
pub trait Dispatcher: Send + Sync {
    /// Dispatch a task to the backend.
    ///
    /// Returns Ok(latency_ms) on success, Err(message) on failure.
    fn dispatch(&self, target: &DispatchTarget, task: &str) -> Result<u64, String>;
}

/// Stub dispatcher that always succeeds with zero latency.
/// Used as the default when no real dispatcher is wired in.
pub struct StubDispatcher;

impl Dispatcher for StubDispatcher {
    fn dispatch(&self, _target: &DispatchTarget, _task: &str) -> Result<u64, String> {
        Ok(0)
    }
}

/// Route executor: takes a RoutingDecision and dispatches to the provider.
///
/// Records every dispatch to the AuditLogger.
pub struct RouteExecutor {
    dispatcher: Arc<dyn Dispatcher>,
    audit: Arc<AuditLogger>,
}

impl RouteExecutor {
    /// Create a new executor with the stub dispatcher.
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self {
            dispatcher: Arc::new(StubDispatcher),
            audit,
        }
    }

    /// Create an executor with a custom dispatcher.
    pub fn with_dispatcher(dispatcher: Arc<dyn Dispatcher>, audit: Arc<AuditLogger>) -> Self {
        Self { dispatcher, audit }
    }

    /// Execute a task given a routing decision.
    ///
    /// Dispatches to the appropriate provider and writes an audit record.
    pub fn execute(&self, decision: &RoutingDecision, task: &str) -> ExecutionOutcome {
        let target = DispatchTarget::from_mode(decision.mode);
        let start = Instant::now();

        let (latency_ms, success, error) = match self.dispatcher.dispatch(&target, task) {
            Ok(latency) => (latency, true, None),
            Err(msg) => {
                let elapsed = start.elapsed().as_millis() as u64;
                (elapsed, false, Some(msg))
            }
        };

        // If dispatcher returned 0 latency (stub), use wall-clock elapsed.
        let final_latency = if latency_ms == 0 {
            start.elapsed().as_millis() as u64
        } else {
            latency_ms
        };

        // Estimate cost: latency as proxy for token consumption.
        // 1ms of latency ~ 1 token consumed (rough heuristic for audit).
        let estimated_tokens = (final_latency as f64).max(1.0);
        let cost_usd = (estimated_tokens / 1000.0) * target.cost_per_1k;

        let record = AuditRecord::new(
            target.provider.clone(),
            target.model.clone(),
            final_latency,
            cost_usd,
        );
        let decision_id = record.decision_id.clone();

        // Write audit record (best-effort; executor succeeds even if audit fails).
        let _ = self.audit.append(&record);

        ExecutionOutcome {
            decision_id,
            provider: target.provider,
            model: target.model,
            latency_ms: final_latency,
            cost_usd,
            success,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn make_decision(mode: RoutingMode) -> RoutingDecision {
        RoutingDecision {
            mode,
            risk_score: 0.5,
            rationale: "test".to_string(),
        }
    }

    #[test]
    fn test_executor_lifecycle_dispatch() {
        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        let exec = RouteExecutor::new(audit);

        let decision = make_decision(RoutingMode::Lifecycle);
        let outcome = exec.execute(&decision, "test task");

        assert!(outcome.success);
        assert_eq!(outcome.provider, "lifecycle");
        assert_eq!(outcome.model, "gemini-3-flash");
    }

    #[test]
    fn test_executor_thegent_dispatch() {
        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        let exec = RouteExecutor::new(audit);

        let decision = make_decision(RoutingMode::TheGent);
        let outcome = exec.execute(&decision, "complex task");

        assert!(outcome.success);
        assert_eq!(outcome.provider, "thegent");
        assert_eq!(outcome.model, "claude-sonnet-4.6");
    }

    #[test]
    fn test_executor_writes_audit_record() {
        let dir = tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let audit = Arc::new(AuditLogger::new(audit_path.clone()));
        let exec = RouteExecutor::new(audit);

        let decision = make_decision(RoutingMode::Lifecycle);
        let outcome = exec.execute(&decision, "audit test");

        assert!(!outcome.decision_id.is_empty());
        assert!(audit_path.exists());
    }

    #[test]
    fn test_executor_failing_dispatcher() {
        struct FailingDispatcher;
        impl Dispatcher for FailingDispatcher {
            fn dispatch(&self, _: &DispatchTarget, _: &str) -> Result<u64, String> {
                Err("connection refused".to_string())
            }
        }

        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        let exec = RouteExecutor::with_dispatcher(Arc::new(FailingDispatcher), audit);

        let decision = make_decision(RoutingMode::Lifecycle);
        let outcome = exec.execute(&decision, "failing task");

        assert!(!outcome.success);
        assert_eq!(outcome.error.unwrap(), "connection refused");
    }

    #[test]
    fn test_dispatch_target_from_mode() {
        let lc = DispatchTarget::from_mode(RoutingMode::Lifecycle);
        assert_eq!(lc.provider, "lifecycle");

        let tg = DispatchTarget::from_mode(RoutingMode::TheGent);
        assert_eq!(tg.provider, "thegent");
    }

    #[test]
    fn test_executor_decision_id_unique() {
        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        let exec = RouteExecutor::new(Arc::clone(&audit));

        let decision = make_decision(RoutingMode::Lifecycle);
        let o1 = exec.execute(&decision, "task 1");
        let o2 = exec.execute(&decision, "task 2");

        assert_ne!(o1.decision_id, o2.decision_id);
    }

    #[test]
    fn test_executor_cost_positive() {
        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        let exec = RouteExecutor::new(audit);

        let decision = make_decision(RoutingMode::TheGent);
        let outcome = exec.execute(&decision, "some task");

        assert!(outcome.cost_usd >= 0.0);
    }
}
