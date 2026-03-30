# Provider Documentation

## Overview

The `forgecode-providers` crate provides a unified, type-safe interface for interacting with multiple LLM API providers. It abstracts provider-specific implementation details through a common trait system and offers a builder pattern for easy configuration.

**Supported Providers:**
- OpenRouter (openrouter.ai)
- Together AI (together.ai)
- Anthropic (Claude models via anthropic.com)

## Architecture

### Core Components

```
┌──────────────────────────────────────────────────────────┐
│                  ProviderBuilder                         │
│        (Fluent API for provider configuration)           │
└─────────────────────┬──────────────────────────────────┘
                      │ builds
                      ▼
┌──────────────────────────────────────────────────────────┐
│              Provider (Factory Enum)                     │
│      Encapsulates OpenRouter, Together, Anthropic       │
└─────────────────────┬──────────────────────────────────┘
                      │ delegates to
                      ▼
┌──────────────────────────────────────────────────────────┐
│                 LlmProvider Trait                        │
│   Abstract interface for completion & credential checks  │
└──────────────────────────────────────────────────────────┘
           ▲                    ▲                    ▲
           │                    │                    │
     implements            implements          implements
           │                    │                    │
    ┌──────┴────┐         ┌────┴──────┐       ┌───┴───────┐
    │OpenRouter  │         │ Together   │       │Anthropic  │
    │Provider    │         │ Provider   │       │Provider   │
    └────────────┘         └────────────┘       └───────────┘
```

### Type System

```rust
// Configuration types
ProviderType::OpenRouter      // enum variant for provider type
ProviderType::Together
ProviderType::Anthropic

// Request/Response types
CompletionRequest {           // unified request structure
  prompt: String,
  temperature: f32,
  max_tokens: u32,
  top_p: f32,
}

CompletionResponse {          // unified response structure
  text: String,
  prompt_tokens: u32,
  completion_tokens: u32,
  total_tokens: u32,
}

// Provider types
Provider::OpenRouter(...)     // factory enum
Provider::Together(...)
Provider::Anthropic(...)
```

## Usage

### Basic Example: Build a Provider

```rust
use forgecode_providers::{ProviderBuilder, ProviderType};

// Build an OpenRouter provider with fluent API
let provider = ProviderBuilder::new()
    .provider(ProviderType::OpenRouter)
    .api_key("sk-or-xxxxx")
    .model("gpt-4")
    .temperature(0.7)
    .max_tokens(2048)
    .top_p(0.9)
    .build()
    .expect("valid configuration");

// Get provider metadata
println!("Provider: {}", provider.provider_type());
println!("Model: {}", provider.model());
```

### Advanced Example: Complete Request

```rust
use forgecode_providers::{ProviderBuilder, ProviderType, CompletionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build Together AI provider
    let provider = ProviderBuilder::new()
        .provider(ProviderType::Together)
        .api_key("sk-together-xxxxx")
        .model("meta-llama/Llama-2-70b")
        .temperature(0.5)
        .max_tokens(512)
        .build()?;

    // Verify credentials before making requests
    let is_valid = provider.verify_credentials().await?;
    if !is_valid {
        return Err("Invalid API key".into());
    }

    // Create a completion request
    let request = CompletionRequest {
        prompt: "Explain quantum computing in 100 words".to_string(),
        temperature: 0.5,
        max_tokens: 512,
        top_p: 0.9,
    };

    // Send request to provider
    let response = provider.complete(request).await?;

    println!("Response: {}", response.text);
    println!("Tokens used: {}", response.total_tokens);

    Ok(())
}
```

### Custom Base URL

```rust
use forgecode_providers::{ProviderBuilder, ProviderType};

// For self-hosted or proxy endpoints
let provider = ProviderBuilder::new()
    .provider(ProviderType::OpenRouter)
    .api_key("sk-local-key")
    .model("gpt-4")
    .base_url("https://local-proxy.example.com/v1")
    .build()
    .expect("valid configuration");
```

### Build Configuration Only

```rust
use forgecode_providers::ProviderBuilder;

// Build configuration without creating provider (useful for serialization)
let config = ProviderBuilder::new()
    .provider(ProviderType::Anthropic)
    .api_key("sk-ant-xxxxx")
    .model("claude-3-opus-20240229")
    .temperature(0.7)
    .max_tokens(4096)
    .build_config()
    .expect("valid config");

// Serialize to JSON
let json = serde_json::to_string(&config)?;
```

## Provider Details

### OpenRouter

**Endpoint:** `https://openrouter.ai/api/v1`

**Features:**
- Supports 200+ models via single API
- Model switching without code changes
- Fallback routing (automatic model substitution)
- Usage reporting per model
- Header: `Authorization: Bearer {api_key}`

**Supported Models:**
- GPT-4 series: `gpt-4`, `gpt-4-turbo-preview`, `gpt-4-32k`
- GPT-3.5: `gpt-3.5-turbo`
- Claude: `claude-2`, `claude-instant`
- Open source: `meta-llama/Llama-2-70b`, `mistral-7b`

**Example:**
```rust
let provider = ProviderBuilder::new()
    .provider(ProviderType::OpenRouter)
    .api_key(std::env::var("OPENROUTER_API_KEY")?)
    .model("gpt-4")
    .temperature(0.7)
    .build()?;
```

### Together AI

**Endpoint:** `https://api.together.xyz/v1`

**Features:**
- Open-source model focus
- Fast inference with optimized infrastructure
- Competitive pricing
- Support for fine-tuning
- Header: `Authorization: Bearer {api_key}`

**Supported Models:**
- Meta Llama: `meta-llama/Llama-2-7b`, `meta-llama/Llama-2-70b`
- Mistral: `mistralai/Mistral-7B-Instruct-v0.1`
- Code models: `codellama/CodeLlama-7b-Python`
- Custom fine-tuned models

**Example:**
```rust
let provider = ProviderBuilder::new()
    .provider(ProviderType::Together)
    .api_key(std::env::var("TOGETHER_API_KEY")?)
    .model("meta-llama/Llama-2-70b-chat")
    .temperature(0.8)
    .build()?;
```

### Anthropic (Claude)

**Endpoint:** `https://api.anthropic.com/v1`

**Features:**
- Claude models only (no model switching)
- Extended context windows
- Streaming support (via streaming response protocol)
- Special headers: `x-api-key`, `anthropic-version`
- Header: `x-api-key: {api_key}`, `anthropic-version: 2023-06-01`

**Supported Models:**
- Claude 3 Opus: `claude-3-opus-20240229`
- Claude 3 Sonnet: `claude-3-sonnet-20240229`
- Claude 3 Haiku: `claude-3-haiku-20240307`
- Claude 2.1: `claude-2.1`
- Claude 2: `claude-2`
- Claude Instant: `claude-instant-1.2`

**Example:**
```rust
let provider = ProviderBuilder::new()
    .provider(ProviderType::Anthropic)
    .api_key(std::env::var("ANTHROPIC_API_KEY")?)
    .model("claude-3-opus-20240229")
    .max_tokens(4096)
    .build()?;
```

## Configuration Parameters

### Builder Methods

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `provider` | ProviderType | **required** | - | Which provider to use |
| `api_key` | String | **required** | - | Authentication token |
| `model` | String | **required** | - | Model identifier |
| `temperature` | f32 | 0.7 | 0.0 - 2.0 | Response randomness (lower = deterministic) |
| `max_tokens` | u32 | 2048 | 1+ | Maximum response length |
| `top_p` | f32 | 0.9 | 0.0 - 1.0 | Nucleus sampling parameter |
| `timeout` | u64 | 30 | 1+ | Request timeout in seconds |
| `base_url` | String | Provider default | - | Custom API endpoint |

### Validation

The builder validates all parameters during `.build()`:

```rust
// Temperature must be 0.0 - 2.0
Err(Error::InvalidConfig {
    field: "temperature",
    reason: "must be between 0.0 and 2.0"
})

// max_tokens must be > 0
Err(Error::InvalidConfig {
    field: "max_tokens",
    reason: "must be greater than 0"
})

// api_key and model are required
Err(Error::MissingConfig {
    field: "api_key"
})
```

## Error Handling

The crate uses the `Error` enum for comprehensive error reporting:

```rust
pub enum Error {
    MissingConfig { field: String },
    InvalidConfig { field: String, reason: String },
    AuthenticationFailed { reason: String },
    RequestFailed { reason: String },
    InvalidResponse { reason: String },
    UnsupportedProvider { provider: String },
    RateLimited { retry_after: Option<u64> },
    Serialization(String),
    Network(String),
}
```

### Common Error Patterns

**Missing Configuration:**
```rust
if let Err(e) = provider_result {
    eprintln!("Setup failed: {}", e);  // "Missing required configuration: api_key"
}
```

**Network Issues:**
```rust
match provider.complete(request).await {
    Ok(response) => { /* use response */ },
    Err(Error::Network(reason)) => eprintln!("Network error: {}", reason),
    Err(Error::RequestFailed { reason }) => eprintln!("API error: {}", reason),
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Rate Limiting:**
```rust
match provider.complete(request).await {
    Err(Error::RateLimited { retry_after: Some(secs) }) => {
        println!("Rate limited; retry after {} seconds", secs);
    },
    _ => {},
}
```

## Testing

The crate includes 42+ unit tests covering:

**Configuration Tests (12):**
- Provider instantiation
- Validation of temperature, max_tokens, top_p
- Custom base URLs
- Builder methods

**Error Tests (6):**
- Missing configuration
- Invalid values
- Provider-specific errors
- Rate limiting

**Provider Tests (12):**
- OpenRouter, Together, Anthropic instantiation
- Authorization headers
- Base URL overrides
- Provider type identification

**Builder Tests (12):**
- Full provider creation
- Configuration validation
- Error cases
- Config-only mode

### Running Tests

```bash
# Run all tests
cargo test --lib

# Run tests for specific module
cargo test config::tests
cargo test providers::openrouter::tests
cargo test builder::tests

# Run with output
cargo test -- --nocapture
```

### Example Test

```rust
#[test]
fn test_builder_openrouter_full() {
    let result = ProviderBuilder::new()
        .provider(ProviderType::OpenRouter)
        .api_key("sk-openrouter-key")
        .model("gpt-4")
        .temperature(0.8)
        .max_tokens(4096)
        .top_p(0.95)
        .timeout(60)
        .build();

    assert!(result.is_ok());
    let provider = result.unwrap();
    assert_eq!(provider.provider_type(), ProviderType::OpenRouter);
    assert_eq!(provider.model(), "gpt-4");
}
```

## Environment Variables

Best practice: store API keys in environment variables

```rust
// OpenRouter
let provider = ProviderBuilder::new()
    .provider(ProviderType::OpenRouter)
    .api_key(std::env::var("OPENROUTER_API_KEY")?)
    .model("gpt-4")
    .build()?;

// Together
let provider = ProviderBuilder::new()
    .provider(ProviderType::Together)
    .api_key(std::env::var("TOGETHER_API_KEY")?)
    .model("meta-llama/Llama-2-70b")
    .build()?;

// Anthropic
let provider = ProviderBuilder::new()
    .provider(ProviderType::Anthropic)
    .api_key(std::env::var("ANTHROPIC_API_KEY")?)
    .model("claude-3-opus-20240229")
    .build()?;
```

## Credentials Verification

Before making production requests, verify API credentials:

```rust
if !provider.verify_credentials().await? {
    eprintln!("Invalid API key or credentials");
    std::process::exit(1);
}
```

**Note:** Credential verification makes a lightweight API call and may incur minimal charges depending on the provider.

## Performance Considerations

### Connection Pooling
The crate uses `reqwest::Client` which automatically maintains connection pools. Create providers once and reuse them:

```rust
// Good: reuse across multiple requests
let provider = ProviderBuilder::new()
    .provider(ProviderType::OpenRouter)
    .api_key("key")
    .model("gpt-4")
    .build()?;

for prompt in prompts {
    let response = provider.complete(prompt).await?;
    // ...
}

// Avoid: creating new providers repeatedly
for prompt in prompts {
    let provider = ProviderBuilder::new()
        .provider(ProviderType::OpenRouter)
        .api_key("key")
        .model("gpt-4")
        .build()?;
    let response = provider.complete(prompt).await?;
}
```

### Timeout Configuration
Adjust timeout based on model size and response length:

```rust
// Fast models (gpt-3.5-turbo): 10-15s
.timeout(10)

// Large models (gpt-4, claude-3-opus): 30-60s
.timeout(60)

// Long generations: 120s+
.timeout(120)
```

## Compatibility

- **Rust Edition:** 2021
- **Minimum Rust Version:** 1.75
- **Async Runtime:** Tokio (required for `.await`)
- **Serialization:** Serde (with serde_json)

## Future Enhancements

Planned features for future releases:

1. **Streaming responses** - Real-time token streaming from providers
2. **Retry policies** - Built-in exponential backoff for transient failures
3. **Request batching** - Combine multiple requests into single API call
4. **Usage tracking** - Automatic cost calculation per request
5. **Provider router** - Smart routing based on latency, cost, or model availability
6. **Local model support** - Llama.cpp, Ollama integration
7. **Function calling** - Tool/function use across providers

## Support & Issues

For bugs, feature requests, or questions:
- GitHub Issues: https://github.com/KooshaPari/phenotype-infrakit/issues
- Documentation: See inline code comments and examples
- Tests: Reference unit tests for usage patterns

## License

MIT License - see repository for details
