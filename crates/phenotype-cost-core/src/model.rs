use crate::cost::Cost;
use crate::types::Complexity;

/// Trait for types that can calculate costs.
pub trait CostModel: Send + Sync {
    /// Calculates the cost for a given context.
    fn calculate(&self, context: &str) -> Cost;

    /// Returns the complexity class of this cost model.
    fn complexity(&self) -> Complexity {
        Complexity::Constant
    }
}

/// Simple token-based cost model.
#[derive(Debug, Clone)]
pub struct TokenCostModel {
    cost_per_token: u64,
}

impl TokenCostModel {
    /// Creates a new token cost model.
    pub fn new(cost_per_token: u64) -> Self {
        Self { cost_per_token }
    }
}

impl CostModel for TokenCostModel {
    fn calculate(&self, context: &str) -> Cost {
        let tokens = context.len() as u64;
        Cost::from_tokens(tokens.saturating_mul(self.cost_per_token))
    }

    fn complexity(&self) -> Complexity {
        Complexity::Linear
    }
}
