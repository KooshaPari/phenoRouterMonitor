# Bifrost Routing — LLM Integration Guide

## Overview

Bifrost Routing provides a unified interface for integrating multiple LLM providers (OpenAI, Anthropic, OpenRouter, Together) with intelligent routing strategies, automatic failover, cost tracking, and latency monitoring.

## Features

- **Multi-Provider Support**: OpenAI, Anthropic, OpenRouter, Together
- **Routing Strategies**: Round-robin, cost-aware, latency-aware, failover
- **Cost Tracking**: Accurate cost calculation per provider
- **Latency Monitoring**: Min/max/average latency per provider
- **Automatic Failover**: Retry with exponential backoff
- **Type Safety**: Full Rust async/await support

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
bifrost-routing = "0.2.0"
```

### 2. Initialize Providers

```rust
use bifrost_routing::providers::{
    OpenAIProvider, OpenAIConfig,
    AnthropicProvider, AnthropicConfig,
};
use bifrost_routing::router::{Router, RoundRobinStrategy};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create providers
    let openai = Arc::new(OpenAIProvider::new(
        OpenAIConfig::new("sk-...".to_string())
    ));

    let anthropic = Arc::new(AnthropicProvider::new(
        AnthropicConfig::new("sk-ant-...".to_string())
    ));

    // Create router with round-robin strategy
    let router = Router::new(
        vec![openai, anthropic],
        Arc::new(RoundRobinStrategy::new()),
    );

    Ok(())
}
```

### 3. Send Requests

```rust
use bifrost_routing::models::{LLMRequest, Message, MessageRole};

let request = LLMRequest::new(
    "gpt-4".to_string(),
    vec![
        Message {
            role: MessageRole::User,
            content: "What is 2+2?".to_string(),
        }
    ]
)
.with_max_tokens(1024)
.with_temperature(0.7)
.with_timeout(30_000);

let response = router.invoke(&request).await?;
println!("Response: {}", response.content);
println!("Cost: ${:.4}", response.cost_usd);
println!("Latency: {}ms", response.latency_ms);
```

## Provider Configuration

### OpenAI

```rust
use bifrost_routing::providers::{OpenAIProvider, OpenAIConfig};

let config = OpenAIConfig::new("sk-...".to_string())
    .with_base_url("https://api.openai.com/v1".to_string())
    .with_org_id("org-...".to_string());

let provider = OpenAIProvider::new(config);
```

**Supported Models**:
- gpt-4o
- gpt-4-turbo
- gpt-4
- gpt-3.5-turbo

**Pricing** (as of March 2026):
- gpt-4o: $0.005 input, $0.015 output per 1K tokens
- gpt-4-turbo: $0.01 input, $0.03 output per 1K tokens
- gpt-4: $0.03 input, $0.06 output per 1K tokens
- gpt-3.5-turbo: $0.0005 input, $0.0015 output per 1K tokens

### Anthropic

```rust
use bifrost_routing::providers::{AnthropicProvider, AnthropicConfig};

let config = AnthropicConfig::new("sk-ant-...".to_string())
    .with_base_url("https://api.anthropic.com".to_string());

let provider = AnthropicProvider::new(config);
```

**Supported Models**:
- claude-opus
- claude-sonnet
- claude-haiku

**Pricing** (as of March 2026):
- claude-opus: $15 input, $75 output per 1M tokens
- claude-sonnet: $3 input, $15 output per 1M tokens
- claude-haiku: $0.8 input, $4 output per 1M tokens

### OpenRouter

```rust
use bifrost_routing::providers::{OpenRouterProvider, OpenRouterConfig};

let config = OpenRouterConfig::new("sk-or-...".to_string())
    .with_http_referer("https://myapp.com".to_string());

let provider = OpenRouterProvider::new(config);
```

**Supported Models**: All models on OpenRouter (aggregates multiple providers)
- OpenAI models (GPT-4, GPT-3.5, etc.)
- Anthropic models (Claude, etc.)
- Open-source models (Llama 2, Mistral, etc.)

### Together

```rust
use bifrost_routing::providers::{TogetherProvider, TogetherConfig};

let config = TogetherConfig::new("sk-together-...".to_string());
let provider = TogetherProvider::new(config);
```

**Supported Models**: Open-source models
- mistral-7b
- llama-2-70b
- llama-2-13b
- codellama-34b

**Pricing** (as of March 2026):
- mistral-7b: $0.0002 input, $0.0006 output per 1M tokens
- llama-2-70b: $0.0009 input, $0.0012 output per 1M tokens
- General estimate: $0.0003 input, $0.001 output per 1M tokens

## Routing Strategies

### 1. Round-Robin

Cycles through providers in order. Useful for load balancing.

```rust
use bifrost_routing::router::RoundRobinStrategy;

let strategy = Arc::new(RoundRobinStrategy::new());
let router = Router::new(providers, strategy);
```

**Use Case**: Distribute load evenly across all providers

### 2. Cost-Aware

Selects the cheapest provider for each request. Considers model and token count.

```rust
use bifrost_routing::router::CostAwareStrategy;

let strategy = Arc::new(CostAwareStrategy::new());
let router = Router::new(providers, strategy);
```

**Use Case**: Minimize costs while maintaining quality

### 3. Latency-Aware

Selects the provider with the lowest recent latency. Requires historical data.

```rust
use bifrost_routing::router::LatencyAwareStrategy;

let strategy = Arc::new(LatencyAwareStrategy::new());
let router = Router::new(providers, strategy);
```

**Use Case**: Minimize response time for user-facing applications

### 4. Failover

Uses the first available provider. Automatically switches on timeout/failure.

```rust
use bifrost_routing::router::FailoverStrategy;

let strategy = Arc::new(FailoverStrategy::new());
let router = Router::new(providers, strategy)
    .with_max_retries(3);
```

**Use Case**: High reliability with primary/fallback provider pairs

## Request Configuration

### Basic Request

```rust
let request = LLMRequest::new(
    "gpt-4".to_string(),
    vec![Message {
        role: MessageRole::User,
        content: "Hello!".to_string(),
    }]
);
```

### With Parameters

```rust
let request = LLMRequest::new(model, messages)
    .with_max_tokens(2048)              // Max output tokens
    .with_temperature(0.7)               // 0.0-2.0, higher = more random
    .with_stream(false)                  // Enable streaming (future feature)
    .with_timeout(30_000);               // Timeout in milliseconds
```

### Token Estimation

The router automatically estimates tokens using a heuristic (4 characters ≈ 1 token).

```rust
let tokens = request.estimate_tokens(); // Returns estimated token count
```

For accurate token counting, use provider-specific tokenizers.

## Response Handling

```rust
let response = router.invoke(&request).await?;

// Response contents
println!("Content: {}", response.content);
println!("Model: {}", response.model);
println!("Provider: {}", response.provider);
println!("Tokens: {} input, {} output",
    response.prompt_tokens,
    response.completion_tokens
);
println!("Cost: ${:.6}", response.cost_usd);
println!("Latency: {}ms", response.latency_ms);
println!("Timestamp: {}", response.finished_at);
```

## Error Handling

### Error Types

```rust
use bifrost_routing::BifrostError;

match router.invoke(&request).await {
    Ok(response) => { /* handle response */ },
    Err(BifrostError::Timeout { provider, timeout_ms }) => {
        println!("{} timed out after {}ms", provider, timeout_ms);
    },
    Err(BifrostError::RateLimited { provider }) => {
        println!("{} is rate-limited", provider);
    },
    Err(BifrostError::AuthenticationError { provider, reason }) => {
        println!("Auth failed for {}: {}", provider, reason);
    },
    Err(BifrostError::AllProvidersFailed { attempts }) => {
        println!("All {} attempts failed", attempts);
    },
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

### Retryable Errors

Some errors trigger automatic retry with exponential backoff:
- Timeout
- Rate limit
- All providers failed

Non-retryable errors (authentication, invalid request) fail immediately.

## Metrics & Monitoring

### Provider Metrics

```rust
let metadata = provider.metadata();
println!("Provider: {}", metadata.name);
println!("Available: {}", metadata.available);
println!("Requests: {}", metadata.total_requests);
println!("Success Rate: {:.2}%", metadata.success_rate * 100.0);
println!("Avg Latency: {}ms", metadata.latency_ms.unwrap_or(0));
println!("Total Cost: ${:.2}", metadata.total_cost_usd);
```

### Router Metrics

```rust
// List all providers
let providers = router.list_providers();
println!("Available providers: {:?}", providers);

// Get detailed metrics for all providers
for metric in router.provider_metrics() {
    println!("{}", metric);
}
```

### Cost Tracking

```rust
let total_cost = provider.metrics.cost_tracker.total_cost();
let avg_cost = provider.metrics.cost_tracker.average_cost();
let request_count = provider.metrics.cost_tracker.request_count();
```

### Latency Tracking

```rust
let avg_latency = provider.metrics.latency_tracker.average_latency_ms();
let min_latency = provider.metrics.latency_tracker.min_latency_ms();
let max_latency = provider.metrics.latency_tracker.max_latency_ms();
```

## Best Practices

### 1. Choose the Right Strategy

| Strategy | Use Case | When |
|----------|----------|------|
| Round-Robin | Load balancing | Equal distribution needed |
| Cost-Aware | Budget optimization | Cost is primary concern |
| Latency-Aware | Performance | Speed is critical |
| Failover | High reliability | Need redundancy |

### 2. Model Selection

Use model parameters to hint which provider to select:

```rust
// For cost-sensitive tasks, use cheaper models
let request = LLMRequest::new("gpt-3.5-turbo".to_string(), messages);

// For quality-critical tasks, use better models
let request = LLMRequest::new("gpt-4".to_string(), messages);

// For speed-critical tasks, use fastest models
let request = LLMRequest::new("claude-haiku".to_string(), messages);
```

### 3. Timeout Configuration

Set reasonable timeouts:

```rust
// Fast responses (e.g., classifications)
let request = request.with_timeout(5_000);    // 5 seconds

// Normal requests
let request = request.with_timeout(30_000);   // 30 seconds

// Long-running requests
let request = request.with_timeout(120_000);  // 2 minutes
```

### 4. Token Management

Monitor token usage to control costs:

```rust
let estimated_tokens = request.estimate_tokens();
let estimated_cost = provider.estimate_cost(
    &model,
    estimated_tokens,
    max_completion_tokens
);

if estimated_cost > budget {
    // Use cheaper model or reduce token limits
}
```

### 5. Error Recovery

Implement graceful degradation:

```rust
match router.invoke(&request).await {
    Ok(response) => handle_success(response),
    Err(BifrostError::AllProvidersFailed { .. }) => {
        // Use cached response or fallback
        use_fallback_response()
    },
    Err(e) => {
        // Log and alert
        log_error(&e);
        return Err(e);
    }
}
```

## Examples

### Example 1: Cost-Optimized Chat

```rust
use bifrost_routing::{
    providers::{OpenAIProvider, OpenAIConfig, TogetherProvider, TogetherConfig},
    router::{Router, CostAwareStrategy},
    models::{LLMRequest, Message, MessageRole},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Premium provider
    let openai = Arc::new(OpenAIProvider::new(
        OpenAIConfig::new("sk-...".to_string())
    ));

    // Budget provider
    let together = Arc::new(TogetherProvider::new(
        TogetherConfig::new("sk-together-...".to_string())
    ));

    let router = Router::new(
        vec![openai, together],
        Arc::new(CostAwareStrategy::new()),
    );

    let request = LLMRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: MessageRole::User,
            content: "What is Rust?".to_string(),
        }],
    )
    .with_max_tokens(512);

    let response = router.invoke(&request).await?;
    println!("Response from {}: {}", response.provider, response.content);
    println!("Cost: ${:.6}", response.cost_usd);

    Ok(())
}
```

### Example 2: High-Availability Setup

```rust
use bifrost_routing::router::FailoverStrategy;

// Primary + Fallback
let primary = Arc::new(OpenAIProvider::new(config_primary));
let fallback = Arc::new(AnthropicProvider::new(config_fallback));

let router = Router::new(
    vec![primary, fallback],
    Arc::new(FailoverStrategy::new()),
)
.with_max_retries(5);

// Will automatically switch to fallback if primary fails
let response = router.invoke(&request).await?;
```

### Example 3: Monitoring Dashboard

```rust
// Periodic metrics collection
tokio::spawn(async move {
    loop {
        for metric in router.provider_metrics() {
            metrics_db.insert(metric)?;
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

## Testing

Run the test suite:

```bash
cargo test --package bifrost-routing
```

All 53 tests pass:
- 8 model tests
- 4 metrics tests
- 16 provider tests
- 8 router tests
- 17 integration tests

## Troubleshooting

### All Providers Fail

**Symptom**: `AllProvidersFailed` error

**Solution**:
1. Check API keys are valid
2. Verify network connectivity
3. Check provider API status pages
4. Review rate limit headers
5. Increase timeout values

### Unexpected Costs

**Symptom**: Costs higher than expected

**Solution**:
1. Review token estimates (heuristic may overestimate)
2. Use cost-aware routing strategy
3. Switch to cheaper models
4. Reduce max_tokens parameter
5. Monitor latency to detect inefficiencies

### Slow Responses

**Symptom**: High latency on all requests

**Solution**:
1. Use latency-aware routing strategy
2. Check network connectivity
3. Reduce timeout constraints (forces failover)
4. Switch to faster models
5. Monitor provider-specific latencies

## Versioning

- **Current Version**: 0.2.0
- **Supported Rust**: 1.75+
- **License**: MIT

## Future Enhancements

- [ ] Streaming response support
- [ ] Vision/image understanding models
- [ ] Function calling / tool use
- [ ] Batch processing API
- [ ] A/B testing framework
- [ ] Advanced caching strategies
- [ ] Multi-modal request handling

## Support

For issues, feature requests, or questions:
1. Check existing GitHub issues
2. Create new issue with reproducible example
3. Reference error messages and provider logs
4. Include router configuration and request details

## References

- [OpenAI API Docs](https://platform.openai.com/docs)
- [Anthropic API Docs](https://docs.anthropic.com)
- [OpenRouter API Docs](https://openrouter.ai/docs)
- [Together API Docs](https://docs.together.ai)
