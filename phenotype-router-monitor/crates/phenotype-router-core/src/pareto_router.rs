//! Router core logic for task distribution based on risk assessment.
//!
//! Routes tasks between Lifecycle (low-risk, fast) and TheGent (high-risk, thorough)
//! based on a configurable risk threshold.

use crate::hysteresis::HysteresisManager;
use crate::risk::{RiskCalculator, RiskFactors};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Routing mode for task execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingMode {
    Lifecycle,
    TheGent,
}

/// Decision produced by the router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub mode: RoutingMode,
    pub risk_score: f64,
    pub rationale: String,
}

/// Metrics tracked by the router.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouterMetrics {
    pub total_decisions: usize,
    pub lifecycle_count: usize,
    pub thegent_count: usize,
    pub route_changes: usize,
    pub hysteresis_activations: usize,
}

/// Router configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Risk threshold below which to route to Lifecycle.
    pub low_threshold: f64,
    /// Risk threshold above which to route to TheGent.
    pub high_threshold: f64,
    /// Hysteresis band width (±band around decision point)
    pub hysteresis_band: f64,
    /// Minimum dwell time before allowing next switch (seconds)
    pub hysteresis_dwell_s: u64,
    /// Maximum time to stay in current mode (seconds)
    pub hysteresis_max_dwell_s: u64,
    /// Large risk change threshold that overrides dwell
    pub hysteresis_override: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            low_threshold: 0.35,  // 35% - Lifecycle for low risk
            high_threshold: 0.65, // 65% - TheGent for high risk
            hysteresis_band: 0.15,
            hysteresis_dwell_s: 300,
            hysteresis_max_dwell_s: 1800,
            hysteresis_override: 0.20,
        }
    }
}

/// Session state tracking for hysteresis.
#[derive(Debug, Clone)]
struct SessionState {
    current_mode: RoutingMode,
    last_switch_time: Instant,
    last_risk_score: f64,
}

/// Main router for task distribution.
pub struct ParetoRouter {
    config: RouterConfig,
    risk_calculator: RiskCalculator,
    hysteresis: HysteresisManager,
    // Atomic counters for metrics
    total_decisions: Arc<AtomicUsize>,
    lifecycle_count: Arc<AtomicUsize>,
    thegent_count: Arc<AtomicUsize>,
    route_changes: Arc<AtomicUsize>,
    hysteresis_activations: Arc<AtomicUsize>,
    last_mode: std::sync::Mutex<Option<RoutingMode>>,
    // Session state tracking for hysteresis
    session_states: std::sync::Mutex<HashMap<String, SessionState>>,
}

impl ParetoRouter {
    /// Create a new router with default configuration.
    pub fn new() -> Self {
        Self::with_config(RouterConfig::default())
    }

    /// Create a router with custom configuration.
    pub fn with_config(config: RouterConfig) -> Self {
        assert!(
            config.low_threshold < config.high_threshold,
            "low_threshold must be < high_threshold"
        );
        assert!(
            config.low_threshold >= 0.0 && config.low_threshold <= 1.0,
            "low_threshold must be in [0.0, 1.0]"
        );
        assert!(
            config.high_threshold >= 0.0 && config.high_threshold <= 1.0,
            "high_threshold must be in [0.0, 1.0]"
        );

        let hysteresis = HysteresisManager::with_config(
            config.hysteresis_band,
            Duration::from_secs(config.hysteresis_dwell_s),
            Duration::from_secs(config.hysteresis_max_dwell_s),
            config.hysteresis_override,
        );

        Self {
            config,
            risk_calculator: RiskCalculator::new(),
            hysteresis,
            total_decisions: Arc::new(AtomicUsize::new(0)),
            lifecycle_count: Arc::new(AtomicUsize::new(0)),
            thegent_count: Arc::new(AtomicUsize::new(0)),
            route_changes: Arc::new(AtomicUsize::new(0)),
            hysteresis_activations: Arc::new(AtomicUsize::new(0)),
            last_mode: std::sync::Mutex::new(None),
            session_states: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Route a task based on risk assessment.
    ///
    /// Returns a RoutingDecision with mode and rationale.
    pub fn route(&self, factors: &RiskFactors) -> RoutingDecision {
        // Calculate risk
        let risk_score = self.risk_calculator.calculate(factors);

        // Determine routing mode
        let mode = if risk_score < self.config.low_threshold {
            RoutingMode::Lifecycle
        } else if risk_score > self.config.high_threshold {
            RoutingMode::TheGent
        } else {
            // In the middle: default to Lifecycle for lower cost
            RoutingMode::Lifecycle
        };

        // Update metrics in memory
        self.total_decisions.fetch_add(1, Ordering::Relaxed);
        let (lc_inc, tg_inc) = match mode {
            RoutingMode::Lifecycle => {
                self.lifecycle_count.fetch_add(1, Ordering::Relaxed);
                (1, 0)
            }
            RoutingMode::TheGent => {
                self.thegent_count.fetch_add(1, Ordering::Relaxed);
                (0, 1)
            }
        };

        // Track route changes
        let mut last = self.last_mode.lock().unwrap();
        let mut changes_inc = 0;
        if let Some(prev_mode) = *last {
            if prev_mode != mode {
                self.route_changes.fetch_add(1, Ordering::Relaxed);
                changes_inc = 1;
            }
        }
        *last = Some(mode);

        // NEW: Sync to SHM
        let _ = thegent_shm::update_router_metrics(lc_inc, tg_inc, changes_inc, 0);

        let rationale = match mode {
            RoutingMode::Lifecycle => format!(
                "Low risk ({:.2}) - using Lifecycle (threshold: {:.2})",
                risk_score, self.config.low_threshold
            ),
            RoutingMode::TheGent => format!(
                "High risk ({:.2}) - using TheGent (threshold: {:.2})",
                risk_score, self.config.high_threshold
            ),
        };

        RoutingDecision {
            mode,
            risk_score,
            rationale,
        }
    }

    /// Route with session-aware hysteresis.
    ///
    /// This method tracks routing decisions per session and applies hysteresis
    /// to prevent oscillation. The session_id is used to maintain state across
    /// multiple routing decisions.
    pub fn route_with_session(&self, session_id: &str, factors: &RiskFactors) -> RoutingDecision {
        let risk_score = self.risk_calculator.calculate(factors);

        let mut session_states = self.session_states.lock().unwrap();
        let session_state = session_states
            .entry(session_id.to_string())
            .or_insert_with(|| SessionState {
                current_mode: RoutingMode::Lifecycle, // Default for first decision
                last_switch_time: Instant::now(),
                last_risk_score: risk_score,
            });

        // Determine if we should switch based on hysteresis
        let should_switch_to_new = if risk_score < self.config.low_threshold {
            session_state.current_mode != RoutingMode::Lifecycle
                && self.hysteresis.should_switch(
                    session_state.current_mode,
                    risk_score,
                    self.config.low_threshold,
                    session_state.last_switch_time,
                    session_state.last_risk_score,
                )
        } else if risk_score > self.config.high_threshold {
            session_state.current_mode != RoutingMode::TheGent
                && self.hysteresis.should_switch(
                    session_state.current_mode,
                    risk_score,
                    self.config.high_threshold,
                    session_state.last_switch_time,
                    session_state.last_risk_score,
                )
        } else {
            false // Stay in band, don't switch
        };

        // Determine final mode (respecting hysteresis)
        let mode = if should_switch_to_new {
            if risk_score < self.config.low_threshold {
                RoutingMode::Lifecycle
            } else if risk_score > self.config.high_threshold {
                RoutingMode::TheGent
            } else {
                session_state.current_mode
            }
        } else {
            session_state.current_mode
        };

        // Track metrics
        let mode_changed = mode != session_state.current_mode;
        if mode_changed {
            self.hysteresis_activations.fetch_add(1, Ordering::Relaxed);
            session_state.last_switch_time = Instant::now();
        }
        session_state.current_mode = mode;
        session_state.last_risk_score = risk_score;

        self.total_decisions.fetch_add(1, Ordering::Relaxed);
        match mode {
            RoutingMode::Lifecycle => {
                self.lifecycle_count.fetch_add(1, Ordering::Relaxed);
            }
            RoutingMode::TheGent => {
                self.thegent_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Track global route changes
        let mut last = self.last_mode.lock().unwrap();
        if let Some(prev_mode) = *last {
            if prev_mode != mode {
                self.route_changes.fetch_add(1, Ordering::Relaxed);
            }
        }
        *last = Some(mode);

        let rationale = match mode {
            RoutingMode::Lifecycle => {
                if should_switch_to_new {
                    format!(
                        "Low risk ({:.2}) - switched to Lifecycle via hysteresis (threshold: {:.2})",
                        risk_score, self.config.low_threshold
                    )
                } else {
                    format!(
                        "Low risk ({:.2}) - staying in Lifecycle (hysteresis active)",
                        risk_score
                    )
                }
            }
            RoutingMode::TheGent => {
                if should_switch_to_new {
                    format!(
                        "High risk ({:.2}) - switched to TheGent via hysteresis (threshold: {:.2})",
                        risk_score, self.config.high_threshold
                    )
                } else {
                    format!(
                        "High risk ({:.2}) - staying in TheGent (hysteresis active)",
                        risk_score
                    )
                }
            }
        };

        RoutingDecision {
            mode,
            risk_score,
            rationale,
        }
    }

    /// Get current metrics.
    pub fn get_metrics(&self) -> RouterMetrics {
        RouterMetrics {
            total_decisions: self.total_decisions.load(Ordering::Relaxed),
            lifecycle_count: self.lifecycle_count.load(Ordering::Relaxed),
            thegent_count: self.thegent_count.load(Ordering::Relaxed),
            route_changes: self.route_changes.load(Ordering::Relaxed),
            hysteresis_activations: self.hysteresis_activations.load(Ordering::Relaxed),
        }
    }

    /// Calculate Lifecycle percentage.
    pub fn lifecycle_percentage(&self) -> f64 {
        let metrics = self.get_metrics();
        if metrics.total_decisions == 0 {
            return 0.0;
        }
        (metrics.lifecycle_count as f64 / metrics.total_decisions as f64) * 100.0
    }
}

impl Default for ParetoRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::risk::ComplexityLevel;

    #[test]
    fn test_router_creation() {
        let router = ParetoRouter::new();
        assert_eq!(router.config.low_threshold, 0.35);
        assert_eq!(router.config.high_threshold, 0.65);
    }

    #[test]
    fn test_router_custom_config() {
        let config = RouterConfig {
            low_threshold: 0.25,
            high_threshold: 0.75,
            ..RouterConfig::default()
        };
        let router = ParetoRouter::with_config(config);
        assert_eq!(router.config.low_threshold, 0.25);
        assert_eq!(router.config.high_threshold, 0.75);
    }

    #[test]
    #[should_panic]
    fn test_router_invalid_thresholds() {
        let config = RouterConfig {
            low_threshold: 0.75,
            high_threshold: 0.25, // Invalid: low > high
            ..RouterConfig::default()
        };
        ParetoRouter::with_config(config);
    }

    #[test]
    fn test_route_simple_task() {
        let router = ParetoRouter::new();
        let factors = RiskFactors::new(ComplexityLevel::Simple);
        let decision = router.route(&factors);

        assert_eq!(decision.mode, RoutingMode::Lifecycle);
        assert!(decision.risk_score < 0.35);
    }

    #[test]
    fn test_route_very_complex_task() {
        let router = ParetoRouter::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 8000,
            dependency_count: 8,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        let decision = router.route(&factors);

        assert_eq!(decision.mode, RoutingMode::TheGent);
        assert!(decision.risk_score > 0.65);
    }

    #[test]
    fn test_metrics_tracking() {
        let router = ParetoRouter::new();
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let very_complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 8_000,
            dependency_count: 8,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };

        router.route(&simple);
        router.route(&very_complex);

        let metrics = router.get_metrics();
        assert_eq!(metrics.total_decisions, 2);
        assert_eq!(metrics.lifecycle_count, 1);
        assert_eq!(metrics.thegent_count, 1);
    }

    #[test]
    fn test_route_changes_tracking() {
        let router = ParetoRouter::new();
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let very_complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 8_000,
            dependency_count: 8,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };

        router.route(&simple);
        let metrics_1 = router.get_metrics();
        assert_eq!(metrics_1.route_changes, 0); // First decision, no change

        router.route(&very_complex);
        let metrics_2 = router.get_metrics();
        assert_eq!(metrics_2.route_changes, 1); // Changed from Lifecycle to TheGent

        router.route(&simple);
        let metrics_3 = router.get_metrics();
        assert_eq!(metrics_3.route_changes, 2); // Changed back
    }

    #[test]
    fn test_lifecycle_percentage() {
        let router = ParetoRouter::new();
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let very_complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 8_000,
            dependency_count: 8,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };

        router.route(&simple);
        router.route(&simple);
        router.route(&very_complex);

        let percentage = router.lifecycle_percentage();
        assert!((percentage - 66.666_67).abs() < 0.1);
    }

    #[test]
    fn test_no_decisions_percentage() {
        let router = ParetoRouter::new();
        assert_eq!(router.lifecycle_percentage(), 0.0);
    }

    #[test]
    fn test_router_is_threadsafe() {
        let router = Arc::new(ParetoRouter::new());
        let mut handles = vec![];

        for _ in 0..4 {
            let r = Arc::clone(&router);
            let handle = std::thread::spawn(move || {
                let factors = RiskFactors::new(ComplexityLevel::Moderate);
                for _ in 0..25 {
                    r.route(&factors);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let metrics = router.get_metrics();
        assert_eq!(metrics.total_decisions, 100);
    }

    #[test]
    fn test_middle_risk_defaults_to_lifecycle() {
        let router = ParetoRouter::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Moderate,
            cost_cents: 5000,
            dependency_count: 5,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let decision = router.route(&factors);

        // Risk will be around 0.5, which is between thresholds
        // Should default to Lifecycle
        assert_eq!(decision.mode, RoutingMode::Lifecycle);
    }

    #[test]
    fn test_rationale_includes_score() {
        let router = ParetoRouter::new();
        let factors = RiskFactors::new(ComplexityLevel::Simple);
        let decision = router.route(&factors);

        assert!(decision
            .rationale
            .contains(&format!("{:.2}", decision.risk_score)));
    }

    #[test]
    fn test_multiple_routes_accumulate() {
        let router = ParetoRouter::new();
        let simple = RiskFactors::new(ComplexityLevel::Simple);

        for _ in 0..10 {
            router.route(&simple);
        }

        let metrics = router.get_metrics();
        assert_eq!(metrics.total_decisions, 10);
        assert_eq!(metrics.lifecycle_count, 10);
        assert_eq!(metrics.thegent_count, 0);
    }

    #[test]
    fn test_router_config_boundary_values() {
        let config = RouterConfig {
            low_threshold: 0.0,
            high_threshold: 1.0,
            ..RouterConfig::default()
        };
        let router = ParetoRouter::with_config(config);

        let factors = RiskFactors::new(ComplexityLevel::Moderate);
        let decision = router.route(&factors);

        // Should still route correctly
        assert!(decision.mode == RoutingMode::Lifecycle || decision.mode == RoutingMode::TheGent);
    }
}
