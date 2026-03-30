// Cost calculation engine

use crate::error::CostResult;
use crate::pricing::{ModelPricing, PricingDatabase};
use crate::token_counter::{TokenCountBreakdown, TokenCounter};
use serde::{Deserialize, Serialize};

/// Cost calculation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCalculationRequest {
    pub model: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl CostCalculationRequest {
    /// Create a new cost calculation request
    pub fn new(model: String, input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            model,
            input_tokens,
            output_tokens,
        }
    }

    /// Create from token count breakdown
    pub fn from_breakdown(model: String, breakdown: TokenCountBreakdown) -> Self {
        Self {
            model,
            input_tokens: breakdown.prompt_tokens,
            output_tokens: breakdown.completion_tokens,
        }
    }
}

/// Cost calculation result with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCalculation {
    pub model: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub input_cost_usd: f64,
    pub output_cost_usd: f64,
    pub total_cost_usd: f64,
    pub input_price_per_mtok: f64,
    pub output_price_per_mtok: f64,
}

impl CostCalculation {
    /// Create new cost calculation
    pub fn new(
        model: String,
        input_tokens: usize,
        output_tokens: usize,
        input_cost_usd: f64,
        output_cost_usd: f64,
        input_price_per_mtok: f64,
        output_price_per_mtok: f64,
    ) -> Self {
        let total_cost_usd = input_cost_usd + output_cost_usd;
        Self {
            model,
            input_tokens,
            output_tokens,
            input_cost_usd,
            output_cost_usd,
            total_cost_usd,
            input_price_per_mtok,
            output_price_per_mtok,
        }
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> usize {
        self.input_tokens + self.output_tokens
    }

    /// Get cost per 1000 tokens
    pub fn cost_per_1k_tokens(&self) -> f64 {
        if self.total_tokens() == 0 {
            0.0
        } else {
            (self.total_cost_usd / self.total_tokens() as f64) * 1000.0
        }
    }

    /// Get breakdown as percentage
    pub fn cost_ratio(&self) -> (f64, f64) {
        if self.total_cost_usd == 0.0 {
            (0.0, 0.0)
        } else {
            let input_ratio = (self.input_cost_usd / self.total_cost_usd) * 100.0;
            let output_ratio = (self.output_cost_usd / self.total_cost_usd) * 100.0;
            (input_ratio, output_ratio)
        }
    }

    /// Format cost in microdollars for precision
    pub fn cost_microdollars(&self) -> u64 {
        (self.total_cost_usd * 1_000_000.0).round() as u64
    }
}

/// Main cost calculator
#[derive(Debug, Clone)]
pub struct CostCalculator {
    pricing_db: PricingDatabase,
}

impl CostCalculator {
    /// Create a new cost calculator with default pricing
    pub fn new() -> Self {
        Self {
            pricing_db: PricingDatabase::new(),
        }
    }

    /// Create with custom pricing database
    pub fn with_pricing(pricing_db: PricingDatabase) -> Self {
        Self { pricing_db }
    }

    /// Calculate cost for a request
    pub fn calculate(&self, request: &CostCalculationRequest) -> CostResult<CostCalculation> {
        let pricing = if self.pricing_db.is_supported(&request.model) {
            self.pricing_db.get_pricing(&request.model)?
        } else {
            self.pricing_db.get_pricing_or_default(&request.model)
        };

        let input_cost = pricing.calculate_cost(request.input_tokens, 0);
        let output_cost = pricing.calculate_cost(0, request.output_tokens);

        Ok(CostCalculation::new(
            request.model.clone(),
            request.input_tokens,
            request.output_tokens,
            input_cost,
            output_cost,
            pricing.input_per_mtok,
            pricing.output_per_mtok,
        ))
    }

    /// Calculate cost from text and max_tokens
    pub fn calculate_from_text(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: Option<usize>,
    ) -> CostResult<CostCalculation> {
        let input_tokens = TokenCounter::estimate_input_tokens(prompt);
        let output_tokens = TokenCounter::estimate_output_tokens(max_tokens);

        let request = CostCalculationRequest::new(model.to_string(), input_tokens, output_tokens);
        self.calculate(&request)
    }

    /// Calculate batch cost (multiple requests)
    pub fn calculate_batch(&self, requests: &[CostCalculationRequest]) -> CostResult<BatchCost> {
        let mut calculations = Vec::new();
        let mut total_cost = 0.0;
        let mut total_input_tokens = 0;
        let mut total_output_tokens = 0;

        for request in requests {
            let calc = self.calculate(request)?;
            total_cost += calc.total_cost_usd;
            total_input_tokens += calc.input_tokens;
            total_output_tokens += calc.output_tokens;
            calculations.push(calc);
        }

        Ok(BatchCost {
            calculations,
            total_cost_usd: total_cost,
            total_input_tokens,
            total_output_tokens,
            request_count: requests.len(),
        })
    }

    /// Get pricing for a model
    pub fn get_pricing(&self, model: &str) -> CostResult<ModelPricing> {
        self.pricing_db.get_pricing(model)
    }

    /// Get list of supported models
    pub fn supported_models(&self) -> Vec<String> {
        self.pricing_db.supported_models()
    }

    /// Check if model is supported
    pub fn is_supported(&self, model: &str) -> bool {
        self.pricing_db.is_supported(model)
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch cost calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCost {
    pub calculations: Vec<CostCalculation>,
    pub total_cost_usd: f64,
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub request_count: usize,
}

impl BatchCost {
    /// Get average cost per request
    pub fn average_cost_per_request(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.total_cost_usd / self.request_count as f64
        }
    }

    /// Get total tokens
    pub fn total_tokens(&self) -> usize {
        self.total_input_tokens + self.total_output_tokens
    }

    /// Get cost per 1000 tokens
    pub fn cost_per_1k_tokens(&self) -> f64 {
        if self.total_tokens() == 0 {
            0.0
        } else {
            (self.total_cost_usd / self.total_tokens() as f64) * 1000.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = CostCalculator::new();
        assert!(calc.is_supported("gpt-4"));
        assert!(calc.is_supported("claude-opus"));
    }

    #[test]
    fn test_cost_calculator_calculate() {
        let calc = CostCalculator::new();
        let req = CostCalculationRequest::new("gpt-4o".to_string(), 1000, 500);
        let result = calc.calculate(&req).unwrap();
        assert_eq!(result.model, "gpt-4o");
        assert!(result.total_cost_usd > 0.0);
    }

    #[test]
    fn test_cost_calculator_calculate_from_text() {
        let calc = CostCalculator::new();
        let result = calc.calculate_from_text("gpt-4", "Hello world", Some(100));
        assert!(result.is_ok());
    }
}
