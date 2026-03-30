// Pricing database for major LLM models (March 2026)

use crate::error::{CostError, CostResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pricing for a specific model (per 1M tokens)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Input cost in USD per 1M tokens
    pub input_per_mtok: f64,
    /// Output cost in USD per 1M tokens
    pub output_per_mtok: f64,
}

impl ModelPricing {
    /// Create new pricing
    pub fn new(input_per_mtok: f64, output_per_mtok: f64) -> Self {
        Self {
            input_per_mtok,
            output_per_mtok,
        }
    }

    /// Calculate cost for given token counts
    pub fn calculate_cost(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        let input_cost = (input_tokens as f64 * self.input_per_mtok) / 1_000_000.0;
        let output_cost = (output_tokens as f64 * self.output_per_mtok) / 1_000_000.0;
        input_cost + output_cost
    }
}

/// Pricing database for all supported models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingDatabase {
    models: HashMap<String, ModelPricing>,
}

impl PricingDatabase {
    /// Create a new pricing database with March 2026 pricing data
    pub fn new() -> Self {
        let mut models = HashMap::new();

        // === ANTHROPIC CLAUDE ===
        models.insert("claude-opus".to_string(), ModelPricing::new(15.0, 75.0));
        models.insert("claude-sonnet".to_string(), ModelPricing::new(3.0, 15.0));
        models.insert("claude-haiku".to_string(), ModelPricing::new(0.80, 4.0));

        // === OPENAI GPT ===
        models.insert("gpt-4o".to_string(), ModelPricing::new(5.0, 15.0));
        models.insert("gpt-4-turbo".to_string(), ModelPricing::new(10.0, 30.0));
        models.insert("gpt-4".to_string(), ModelPricing::new(15.0, 45.0));
        models.insert("gpt-3.5-turbo".to_string(), ModelPricing::new(0.5, 1.5));

        // === GOOGLE GEMINI ===
        models.insert("gemini-1.5-pro".to_string(), ModelPricing::new(7.0, 21.0));
        models.insert("gemini-1.5-flash".to_string(), ModelPricing::new(0.075, 0.30));
        models.insert("gemini-1.0-pro".to_string(), ModelPricing::new(0.50, 1.50));

        // === META LLAMA ===
        models.insert("llama-2-70b".to_string(), ModelPricing::new(0.90, 1.20));
        models.insert("llama-2-13b".to_string(), ModelPricing::new(0.22, 0.30));
        models.insert("llama-3-70b".to_string(), ModelPricing::new(0.90, 1.20));
        models.insert("llama-3-8b".to_string(), ModelPricing::new(0.20, 0.30));

        // === MISTRAL ===
        models.insert("mistral-large".to_string(), ModelPricing::new(8.0, 24.0));
        models.insert("mistral-medium".to_string(), ModelPricing::new(2.7, 8.1));
        models.insert("mistral-small".to_string(), ModelPricing::new(0.14, 0.42));

        // === COHERE COMMAND ===
        models.insert("command-r-plus".to_string(), ModelPricing::new(3.0, 15.0));
        models.insert("command-r".to_string(), ModelPricing::new(0.50, 1.50));

        // === FALLBACK ===
        models.insert("__default__".to_string(), ModelPricing::new(1.0, 3.0));

        Self { models }
    }

    /// Get pricing for a specific model
    pub fn get_pricing(&self, model: &str) -> CostResult<ModelPricing> {
        self.models
            .get(model)
            .copied()
            .ok_or_else(|| CostError::UnknownModel(model.to_string()))
    }

    /// Get pricing with fallback to default if model not found
    pub fn get_pricing_or_default(&self, model: &str) -> ModelPricing {
        self.models
            .get(model)
            .copied()
            .unwrap_or_else(|| self.models.get("__default__").copied().unwrap())
    }

    /// Check if model is supported
    pub fn is_supported(&self, model: &str) -> bool {
        self.models.contains_key(model)
    }

    /// Get list of all supported models
    pub fn supported_models(&self) -> Vec<String> {
        self.models
            .keys()
            .filter(|k| *k != &"__default__".to_string())
            .cloned()
            .collect()
    }
}

impl Default for PricingDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_database_creation() {
        let db = PricingDatabase::new();
        assert!(!db.supported_models().is_empty());
    }

    #[test]
    fn test_get_anthropic_pricing() {
        let db = PricingDatabase::new();
        let pricing = db.get_pricing("claude-opus").unwrap();
        assert_eq!(pricing.input_per_mtok, 15.0);
        assert_eq!(pricing.output_per_mtok, 75.0);
    }

    #[test]
    fn test_unknown_model_error() {
        let db = PricingDatabase::new();
        let result = db.get_pricing("nonexistent-model-xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_fallback_pricing() {
        let db = PricingDatabase::new();
        let pricing = db.get_pricing_or_default("unknown-model");
        assert_eq!(pricing.input_per_mtok, 1.0);
        assert_eq!(pricing.output_per_mtok, 3.0);
    }

    #[test]
    fn test_model_pricing_calculation() {
        let pricing = ModelPricing::new(10.0, 30.0);
        let cost = pricing.calculate_cost(1000, 500);
        let expected = (1000.0 * 10.0 + 500.0 * 30.0) / 1_000_000.0;
        assert!((cost - expected).abs() < 0.0001);
    }
}
