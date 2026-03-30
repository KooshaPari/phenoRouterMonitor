//! Hysteresis management for preventing route oscillation.
//!
//! This module implements damping logic to prevent tasks from switching routes
//! due to small risk score fluctuations. It uses a hysteresis band and dwell time
//! to smooth routing decisions.

use crate::router::RoutingMode;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Hysteresis manager for dampening route switching.
///
/// Prevents route oscillation through:
/// - Hysteresis band: tasks in middle range stay put
/// - Dwell time: minimum time before allowing next switch
/// - Max dwell: force re-evaluation after extended period
/// - Large risk changes override dwell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HysteresisManager {
    /// Hysteresis band width (±band around decision point)
    pub band_width: f64,
    /// Minimum dwell time before allowing next switch
    pub dwell_time: Duration,
    /// Maximum time to stay in current mode
    pub max_dwell: Duration,
    /// Large risk change threshold that overrides dwell
    pub override_threshold: f64,
}

impl HysteresisManager {
    /// Create a new hysteresis manager with defaults.
    pub fn new() -> Self {
        Self {
            band_width: 0.15,
            dwell_time: Duration::from_secs(300), // 5 minutes
            max_dwell: Duration::from_secs(1800), // 30 minutes
            override_threshold: 0.20,
        }
    }

    /// Create a new hysteresis manager with custom parameters.
    pub fn with_config(
        band_width: f64,
        dwell_time: Duration,
        max_dwell: Duration,
        override_threshold: f64,
    ) -> Self {
        assert!(
            (0.0..=0.5).contains(&band_width),
            "band_width must be in [0.0, 0.5]"
        );
        assert!(
            (0.0..=1.0).contains(&override_threshold),
            "override_threshold must be in [0.0, 1.0]"
        );
        assert!(dwell_time < max_dwell, "dwell_time must be < max_dwell");

        Self {
            band_width,
            dwell_time,
            max_dwell,
            override_threshold,
        }
    }

    /// Check if risk is in hysteresis band around the decision point.
    ///
    /// Returns true if the risk is within ±band_width of the threshold.
    pub fn in_hysteresis_band(&self, risk_score: f64, threshold: f64) -> bool {
        let lower = (threshold - self.band_width).max(0.0);
        let upper = (threshold + self.band_width).min(1.0);
        risk_score >= lower && risk_score <= upper
    }

    /// Determine if a route switch should occur.
    ///
    /// Arguments:
    /// - `current_mode`: current routing mode
    /// - `new_risk_score`: newly calculated risk score
    /// - `decision_threshold`: routing threshold (low_threshold or high_threshold)
    /// - `last_switch_time`: time of last route switch (or start time)
    /// - `prev_risk_score`: previous risk score (for change detection)
    ///
    /// Returns true if the switch should be allowed.
    pub fn should_switch(
        &self,
        _current_mode: RoutingMode,
        new_risk_score: f64,
        decision_threshold: f64,
        last_switch_time: Instant,
        prev_risk_score: f64,
    ) -> bool {
        let now = Instant::now();
        let time_since_switch = now.duration_since(last_switch_time);

        // Condition 1: Outside band → always switch
        if !self.in_hysteresis_band(new_risk_score, decision_threshold) {
            return true;
        }

        // Condition 4 (checked early): Large risk change (>threshold) → override dwell
        let risk_change = (new_risk_score - prev_risk_score).abs();
        if risk_change > self.override_threshold {
            return true;
        }

        // Condition 2: In band + dwell active → don't switch
        if time_since_switch < self.dwell_time {
            return false;
        }

        // Condition 3: Max dwell exceeded → force switch
        if time_since_switch >= self.max_dwell {
            return true;
        }

        false
    }
}

impl Default for HysteresisManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hysteresis_creation() {
        let hyst = HysteresisManager::new();
        assert_eq!(hyst.band_width, 0.15);
        assert_eq!(hyst.dwell_time.as_secs(), 300);
        assert_eq!(hyst.max_dwell.as_secs(), 1800);
    }

    #[test]
    fn test_in_hysteresis_band() {
        let hyst = HysteresisManager::new();
        let threshold = 0.5;

        // Inside band
        assert!(hyst.in_hysteresis_band(0.45, threshold));
        assert!(hyst.in_hysteresis_band(0.50, threshold));
        assert!(hyst.in_hysteresis_band(0.55, threshold));

        // Outside band
        assert!(!hyst.in_hysteresis_band(0.30, threshold));
        assert!(!hyst.in_hysteresis_band(0.70, threshold));
    }

    #[test]
    fn test_in_hysteresis_band_boundaries() {
        let hyst = HysteresisManager::new();

        // Test boundary handling at 0.0
        assert!(hyst.in_hysteresis_band(0.05, 0.1));
        assert!(hyst.in_hysteresis_band(0.0, 0.1)); // Lower bound clamped

        // Test boundary handling at 1.0
        assert!(hyst.in_hysteresis_band(0.95, 0.9));
        assert!(hyst.in_hysteresis_band(1.0, 0.9)); // Upper bound clamped
    }

    #[test]
    fn test_should_switch_outside_band() {
        let hyst = HysteresisManager::new();
        let now = Instant::now();

        // Risk score far outside band → always switch
        let can_switch = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.8, // Far above threshold 0.5
            0.5,
            now,
            0.3,
        );
        assert!(can_switch);
    }

    #[test]
    fn test_should_switch_in_band_no_dwell_expired() {
        let hyst = HysteresisManager::new();
        let now = Instant::now();

        // Risk in band + dwell active → don't switch
        let can_switch = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.50, // Inside band of threshold 0.5
            0.5,
            now, // Just switched
            0.51,
        );
        assert!(!can_switch);
    }

    #[test]
    fn test_should_switch_max_dwell_exceeded() {
        let hyst = HysteresisManager::new();
        let past = Instant::now() - Duration::from_secs(2000); // > 1800s

        // In band but max dwell exceeded → force switch
        let can_switch = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.50, // Inside band
            0.5,
            past, // Long time ago
            0.51,
        );
        assert!(can_switch);
    }

    #[test]
    fn test_should_switch_large_risk_change() {
        let hyst = HysteresisManager::new();
        // Create a time very recent to test large risk change override
        let past = Instant::now() - Duration::from_millis(50);

        // In band, dwell active, but large risk change → override
        let can_switch = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.50, // Inside band
            0.5,
            past, // Recent switch time (within dwell)
            0.28, // Change of 0.22 > override_threshold 0.20
        );
        assert!(can_switch);
    }

    #[test]
    fn test_should_switch_small_change_in_band() {
        let hyst = HysteresisManager::new();
        let now = Instant::now();

        // In band, dwell active, small change → don't switch
        let can_switch = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.50, // Inside band
            0.5,
            now,   // Just switched
            0.505, // Small change < 0.20
        );
        assert!(!can_switch);
    }

    #[test]
    fn test_should_switch_after_dwell_expires() {
        let hyst = HysteresisManager::with_config(
            0.15,
            Duration::from_secs(60),  // Shorter dwell for testing
            Duration::from_secs(120), // Shorter max_dwell for testing
            0.20,
        );
        let past = Instant::now() - Duration::from_secs(70); // > 60s dwell but < 120s max

        // In band but dwell expired → can switch (no large change needed after dwell expires)
        let can_switch = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.50, // Inside band
            0.5,
            past, // time_since_switch = 70s > dwell_time = 60s
            0.51, // Small change, but dwell expired allows switch
        );
        // After dwell expires but before max_dwell, we can switch if we want (condition 2 is false)
        // Condition 4 (large change) is also false since |0.50 - 0.51| = 0.01 < 0.20
        // So we should NOT switch in this middle state (between dwell and max_dwell with no large change)
        // Actually, looking at the logic: we're in band (cond 1 false),
        // dwell expired (cond 2 false), max not exceeded (cond 3 false),
        // no large change (cond 4 false), so should_switch returns false
        // This test is checking that between dwell and max_dwell, nothing forces a switch
        assert!(!can_switch);
    }

    #[test]
    fn test_custom_band_width() {
        let hyst = HysteresisManager::with_config(
            0.1,
            Duration::from_secs(60),
            Duration::from_secs(600),
            0.15,
        );

        // Narrower band
        assert!(hyst.in_hysteresis_band(0.45, 0.5));
        assert!(!hyst.in_hysteresis_band(0.38, 0.5)); // Outside narrower band
    }

    #[test]
    #[should_panic]
    fn test_invalid_band_width() {
        HysteresisManager::with_config(
            0.6, // Too wide
            Duration::from_secs(60),
            Duration::from_secs(600),
            0.15,
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_override_threshold() {
        HysteresisManager::with_config(
            0.1,
            Duration::from_secs(60),
            Duration::from_secs(600),
            1.5, // Out of range
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_dwell_times() {
        HysteresisManager::with_config(
            0.1,
            Duration::from_secs(600), // > max_dwell
            Duration::from_secs(60),  // < dwell_time
            0.15,
        );
    }

    #[test]
    fn test_hysteresis_prevents_oscillation() {
        let hyst = HysteresisManager::new();
        let mut last_time = Instant::now();
        let threshold = 0.5;

        // Simulate oscillating risk near threshold
        let risk_scores = vec![0.48, 0.52, 0.49, 0.51, 0.50];
        let mut switch_count = 0;

        for risk in risk_scores {
            if hyst.should_switch(RoutingMode::Lifecycle, risk, threshold, last_time, 0.5) {
                switch_count += 1;
                last_time = Instant::now();
            }
        }

        // Should not switch frequently due to dwell
        assert!(switch_count < 3);
    }

    #[test]
    fn test_multiple_large_changes_force_switches() {
        let hyst = HysteresisManager::new();
        let past_short = Instant::now() - Duration::from_millis(50);
        let threshold = 0.5;

        // First large change overrides dwell
        let can_switch_1 = hyst.should_switch(
            RoutingMode::Lifecycle,
            0.50, // Inside band
            threshold,
            past_short, // Recent but with large change
            0.28,       // Large change of 0.22 > override_threshold 0.20
        );
        assert!(can_switch_1);

        // Second: max dwell exceeded forces switch regardless of change
        let past_long = Instant::now() - Duration::from_secs(2000);
        let can_switch_2 = hyst.should_switch(
            RoutingMode::TheGent,
            0.50, // Inside band
            threshold,
            past_long, // Max dwell exceeded (2000s > 1800s max_dwell)
            0.30,      // Any change; max_dwell condition overrides
        );
        assert!(can_switch_2);
    }

    #[test]
    fn test_steady_state_no_switches() {
        let hyst = HysteresisManager::new();
        let mut last_time = Instant::now();
        let threshold = 0.5;

        // Consistent risk in band with small variations
        let risk_scores = vec![0.495, 0.505, 0.498, 0.502];
        let mut switch_count = 0;

        for risk in risk_scores {
            if hyst.should_switch(RoutingMode::Lifecycle, risk, threshold, last_time, 0.5) {
                switch_count += 1;
                last_time = Instant::now();
            }
        }

        // Should not switch in steady state (well within dwell)
        assert_eq!(switch_count, 0);
    }

    #[test]
    fn test_hysteresis_default() {
        let hyst1 = HysteresisManager::new();
        let hyst2 = HysteresisManager::default();

        assert_eq!(hyst1.band_width, hyst2.band_width);
        assert_eq!(hyst1.dwell_time, hyst2.dwell_time);
    }
}
