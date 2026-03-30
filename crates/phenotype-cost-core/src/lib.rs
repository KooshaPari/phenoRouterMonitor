//! # phenotype-cost-core
//!
//! Cost calculation, pricing models, and budget enforcement for LLM operations.
//!
//! ## Features
//!
//! - Accurate cost calculation with support for 30+ LLM models
//! - March 2026 pricing data for all major providers (Claude, GPT, Gemini, Mistral, etc.)
//! - Token counting utilities with heuristic and accurate estimates
//! - Budget management with daily/monthly limits and per-request caps
//! - Budget enforcement with overflow prevention
//! - Batch cost calculations for multiple requests
//!
//! ## Quick Start
//!
//! ```rust
//! use phenotype_cost_core::{CostCalculator, BudgetManager, BudgetLimits};
//!
//! // Create calculator and manager
//! let calculator = CostCalculator::new();
//! let limits = BudgetLimits::small_team().unwrap();
//! let budget = BudgetManager::new(limits);
//!
//! // Calculate a single request
//! let cost = calculator.calculate_from_text(
//!     "gpt-4o",
//!     "What is the meaning of life?",
//!     Some(256),
//! ).unwrap();
//!
//! println!("Cost: ${:.4}", cost.total_cost_usd);
//! println!("Total tokens: {}", cost.total_tokens());
//!
//! // Check and record cost in budget
//! if budget.can_afford(cost.total_cost_usd).is_ok() {
//!     budget.add_cost(cost.total_cost_usd);
//! }
//! ```
//!
//! ## Supported Models (March 2026)
//!
//! - **Anthropic**: claude-opus, claude-sonnet, claude-haiku
//! - **OpenAI**: gpt-4o, gpt-4-turbo, gpt-4, gpt-3.5-turbo
//! - **Google**: gemini-1.5-pro, gemini-1.5-flash, gemini-1.0-pro
//! - **Meta**: llama-2-70b, llama-2-13b, llama-3-70b, llama-3-8b
//! - **Mistral**: mistral-large, mistral-medium, mistral-small
//! - **Cohere**: command-r-plus, command-r, command
//! - **And 10+ more providers**

pub mod budget;
pub mod calculator;
pub mod error;
pub mod pricing;
pub mod token_counter;

// Re-export commonly used types
pub use budget::{BudgetInfo, BudgetLimits, BudgetManager, BudgetStatus, BudgetUsage};
pub use calculator::{BatchCost, CostCalculation, CostCalculationRequest, CostCalculator};
pub use error::{CostError, CostResult};
pub use pricing::{ModelPricing, PricingDatabase};
pub use token_counter::{Message, TokenCountBreakdown, TokenCounter};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod integration_tests {
    use crate::*;

    #[test]
    fn test_end_to_end_cost_calculation() {
        let calculator = CostCalculator::new();

        let cost = calculator
            .calculate_from_text("claude-opus", "Hello world", Some(100))
            .unwrap();

        assert!(cost.total_cost_usd > 0.0);
        assert_eq!(cost.model, "claude-opus");
        assert!(cost.input_tokens > 0);
    }

    #[test]
    fn test_end_to_end_budget_management() {
        let limits = BudgetLimits::new(100.0, 1000.0, 50.0).unwrap();
        let manager = BudgetManager::new(limits);

        assert!(manager.record_cost(25.0).is_ok());
        assert!(manager.record_cost(25.0).is_ok());
        assert!(manager.record_cost(25.0).is_ok());

        let info = manager.info();
        assert_eq!(info.daily_spent, 75.0);
        assert_eq!(info.daily_remaining, 25.0);
    }

    #[test]
    fn test_batch_calculation_workflow() {
        let calculator = CostCalculator::new();

        let requests = vec![
            CostCalculationRequest::new("gpt-4o".to_string(), 100, 50),
            CostCalculationRequest::new("claude-opus".to_string(), 200, 100),
            CostCalculationRequest::new("gpt-3.5-turbo".to_string(), 50, 25),
        ];

        let batch = calculator.calculate_batch(&requests).unwrap();
        assert_eq!(batch.request_count, 3);
        assert!(batch.total_cost_usd > 0.0);
        assert_eq!(batch.total_input_tokens, 350);
        assert_eq!(batch.total_output_tokens, 175);
    }

    #[test]
    fn test_multiple_providers() {
        let calculator = CostCalculator::new();

        let providers = vec!["gpt-4o", "claude-opus", "gemini-1.5-pro"];

        for provider in providers {
            let req =
                CostCalculationRequest::new(provider.to_string(), 1000, 500);
            let calc = calculator.calculate(&req).unwrap();
            assert_eq!(calc.model, provider);
            assert!(calc.total_cost_usd > 0.0);
        }
    }
}
