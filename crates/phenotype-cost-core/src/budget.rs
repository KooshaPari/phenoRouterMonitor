// Budget management and enforcement

use crate::error::{CostError, CostResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Budget limits
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BudgetLimits {
    /// Daily budget in USD
    pub daily_limit: f64,
    /// Monthly budget in USD
    pub monthly_limit: f64,
    /// Per-request limit in USD (prevents runaway requests)
    pub per_request_limit: f64,
}

impl BudgetLimits {
    /// Create new budget limits
    pub fn new(daily_limit: f64, monthly_limit: f64, per_request_limit: f64) -> CostResult<Self> {
        if daily_limit <= 0.0 || monthly_limit <= 0.0 || per_request_limit <= 0.0 {
            return Err(CostError::InvalidBudgetConfig(
                "All limits must be positive".to_string(),
            ));
        }

        if per_request_limit > daily_limit {
            return Err(CostError::InvalidBudgetConfig(
                "Per-request limit cannot exceed daily limit".to_string(),
            ));
        }

        Ok(Self {
            daily_limit,
            monthly_limit,
            per_request_limit,
        })
    }

    /// Create reasonable defaults for a small team
    pub fn small_team() -> CostResult<Self> {
        Self::new(50.0, 1000.0, 5.0)
    }

    /// Create reasonable defaults for a medium team
    pub fn medium_team() -> CostResult<Self> {
        Self::new(200.0, 5000.0, 20.0)
    }

    /// Create reasonable defaults for an enterprise
    pub fn enterprise() -> CostResult<Self> {
        Self::new(1000.0, 50000.0, 100.0)
    }
}

/// Budget usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUsage {
    /// Daily spending (resets daily)
    pub daily_spent: f64,
    /// Monthly spending (resets monthly)
    pub monthly_spent: f64,
    /// Last reset time for daily budget
    pub daily_reset_at: DateTime<Utc>,
    /// Last reset time for monthly budget
    pub monthly_reset_at: DateTime<Utc>,
}

impl BudgetUsage {
    /// Create new empty budget usage
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            daily_spent: 0.0,
            monthly_spent: 0.0,
            daily_reset_at: now,
            monthly_reset_at: now,
        }
    }

    /// Reset daily budget if needed
    pub fn reset_daily_if_needed(&mut self) {
        let now = Utc::now();
        if now.signed_duration_since(self.daily_reset_at) > Duration::days(1) {
            self.daily_spent = 0.0;
            self.daily_reset_at = now;
        }
    }

    /// Reset monthly budget if needed
    pub fn reset_monthly_if_needed(&mut self) {
        let now = Utc::now();
        if now.signed_duration_since(self.monthly_reset_at) > Duration::days(30) {
            self.monthly_spent = 0.0;
            self.monthly_reset_at = now;
        }
    }

    /// Record a cost
    pub fn add_cost(&mut self, cost: f64) {
        self.reset_daily_if_needed();
        self.reset_monthly_if_needed();
        self.daily_spent += cost;
        self.monthly_spent += cost;
    }

    /// Get remaining daily budget
    pub fn daily_remaining(&self, limit: f64) -> f64 {
        (limit - self.daily_spent).max(0.0)
    }

    /// Get remaining monthly budget
    pub fn monthly_remaining(&self, limit: f64) -> f64 {
        (limit - self.monthly_spent).max(0.0)
    }

    /// Get daily spend percentage
    pub fn daily_percentage(&self, limit: f64) -> f64 {
        if limit == 0.0 {
            0.0
        } else {
            (self.daily_spent / limit) * 100.0
        }
    }

    /// Get monthly spend percentage
    pub fn monthly_percentage(&self, limit: f64) -> f64 {
        if limit == 0.0 {
            0.0
        } else {
            (self.monthly_spent / limit) * 100.0
        }
    }
}

impl Default for BudgetUsage {
    fn default() -> Self {
        Self::new()
    }
}

/// Budget status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetStatus {
    /// Under 50% of limit
    Healthy,
    /// 50-90% of limit
    Warning,
    /// 90-100% of limit
    Critical,
    /// Over limit
    Exceeded,
}

impl BudgetStatus {
    /// Get status from percentage
    pub fn from_percentage(percentage: f64) -> Self {
        match percentage {
            p if p >= 100.0 => Self::Exceeded,
            p if p >= 90.0 => Self::Critical,
            p if p >= 50.0 => Self::Warning,
            _ => Self::Healthy,
        }
    }
}

/// Budget manager
#[derive(Debug, Clone)]
pub struct BudgetManager {
    limits: BudgetLimits,
    usage: Arc<Mutex<BudgetUsage>>,
}

impl BudgetManager {
    /// Create new budget manager
    pub fn new(limits: BudgetLimits) -> Self {
        Self {
            limits,
            usage: Arc::new(Mutex::new(BudgetUsage::new())),
        }
    }

    /// Check if a request cost would exceed budget
    pub fn can_afford(&self, cost: f64) -> CostResult<()> {
        let mut usage = self.usage.lock().unwrap();
        usage.reset_daily_if_needed();
        usage.reset_monthly_if_needed();

        // Check per-request limit
        if cost > self.limits.per_request_limit {
            return Err(CostError::BudgetExceeded {
                limit: format!("${:.2}", self.limits.per_request_limit),
                current: format!("${:.2}", usage.daily_spent),
                requested: format!("${:.2}", cost),
            });
        }

        // Check daily limit
        if usage.daily_spent + cost > self.limits.daily_limit {
            return Err(CostError::BudgetExceeded {
                limit: format!("${:.2}", self.limits.daily_limit),
                current: format!("${:.2}", usage.daily_spent),
                requested: format!("${:.2}", cost),
            });
        }

        // Check monthly limit
        if usage.monthly_spent + cost > self.limits.monthly_limit {
            return Err(CostError::BudgetExceeded {
                limit: format!("${:.2}", self.limits.monthly_limit),
                current: format!("${:.2}", usage.monthly_spent),
                requested: format!("${:.2}", cost),
            });
        }

        Ok(())
    }

    /// Record a cost (call only after verifying with can_afford)
    pub fn add_cost(&self, cost: f64) {
        let mut usage = self.usage.lock().unwrap();
        usage.add_cost(cost);
    }

    /// Record a cost if within budget, returns error if not
    pub fn record_cost(&self, cost: f64) -> CostResult<()> {
        self.can_afford(cost)?;
        self.add_cost(cost);
        Ok(())
    }

    /// Get current budget status
    pub fn status(&self) -> BudgetStatus {
        let usage = self.usage.lock().unwrap();
        let daily_pct = usage.daily_percentage(self.limits.daily_limit);
        let monthly_pct = usage.monthly_percentage(self.limits.monthly_limit);

        // Use the highest percentage (most critical)
        let max_pct = daily_pct.max(monthly_pct);
        BudgetStatus::from_percentage(max_pct)
    }

    /// Get detailed budget info
    pub fn info(&self) -> BudgetInfo {
        let usage = self.usage.lock().unwrap();
        BudgetInfo {
            daily_limit: self.limits.daily_limit,
            daily_spent: usage.daily_spent,
            daily_remaining: usage.daily_remaining(self.limits.daily_limit),
            daily_percentage: usage.daily_percentage(self.limits.daily_limit),
            daily_status: BudgetStatus::from_percentage(
                usage.daily_percentage(self.limits.daily_limit),
            ),
            monthly_limit: self.limits.monthly_limit,
            monthly_spent: usage.monthly_spent,
            monthly_remaining: usage.monthly_remaining(self.limits.monthly_limit),
            monthly_percentage: usage.monthly_percentage(self.limits.monthly_limit),
            monthly_status: BudgetStatus::from_percentage(
                usage.monthly_percentage(self.limits.monthly_limit),
            ),
            per_request_limit: self.limits.per_request_limit,
        }
    }

    /// Reset daily budget (admin only)
    pub fn reset_daily(&self) {
        let mut usage = self.usage.lock().unwrap();
        usage.daily_spent = 0.0;
        usage.daily_reset_at = Utc::now();
    }

    /// Reset monthly budget (admin only)
    pub fn reset_monthly(&self) {
        let mut usage = self.usage.lock().unwrap();
        usage.monthly_spent = 0.0;
        usage.monthly_reset_at = Utc::now();
    }
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new(BudgetLimits::small_team().unwrap())
    }
}

/// Detailed budget information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetInfo {
    pub daily_limit: f64,
    pub daily_spent: f64,
    pub daily_remaining: f64,
    pub daily_percentage: f64,
    pub daily_status: BudgetStatus,
    pub monthly_limit: f64,
    pub monthly_spent: f64,
    pub monthly_remaining: f64,
    pub monthly_percentage: f64,
    pub monthly_status: BudgetStatus,
    pub per_request_limit: f64,
}

impl BudgetInfo {
    /// Check if overall status is healthy
    pub fn is_healthy(&self) -> bool {
        self.daily_status == BudgetStatus::Healthy && self.monthly_status == BudgetStatus::Healthy
    }

    /// Check if any limit is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.daily_status == BudgetStatus::Exceeded || self.monthly_status == BudgetStatus::Exceeded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_manager_can_afford() {
        let limits = BudgetLimits::new(50.0, 1000.0, 10.0).unwrap();
        let manager = BudgetManager::new(limits);

        assert!(manager.can_afford(5.0).is_ok());
        assert!(manager.can_afford(100.0).is_err()); // Over per-request limit
    }

    #[test]
    fn test_budget_manager_record_cost() {
        let limits = BudgetLimits::new(50.0, 1000.0, 10.0).unwrap();
        let manager = BudgetManager::new(limits);

        assert!(manager.record_cost(5.0).is_ok());
        assert!(manager.record_cost(100.0).is_err());
    }

    #[test]
    fn test_budget_status_from_percentage() {
        assert_eq!(BudgetStatus::from_percentage(30.0), BudgetStatus::Healthy);
        assert_eq!(BudgetStatus::from_percentage(70.0), BudgetStatus::Warning);
        assert_eq!(BudgetStatus::from_percentage(95.0), BudgetStatus::Critical);
        assert_eq!(BudgetStatus::from_percentage(100.0), BudgetStatus::Exceeded);
    }
}
