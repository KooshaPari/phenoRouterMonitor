# WP-002: Claude Provider Implementation

**Work Package ID**: WP-002
**Epic**: eco-fork-001 (Custom Providers & Subagent Management)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Implement the `ClaudeProvider` adapter for the Anthropic SDK, supporting multi-model routing (Opus, Sonnet, Haiku) with fallback strategies and streaming support.

## Description

Build a production-ready Claude provider implementation that handles model selection, error recovery, token counting, and audit trail recording. This provider serves as the reference implementation for custom provider development.

---

## Objectives

- Implement `ClaudeProvider` struct wrapping anthropic-sdk client
- Support multi-model routing (Opus, Sonnet, Haiku)
- Implement fallback to Sonnet on Opus failure
- Add streaming support via async iterators
- Record all calls with token counts to audit trail

---

## Acceptance Criteria

1. **Provider Implementation**:
   - `ClaudeProvider` implements Provider trait cleanly
   - Compiles with 0 errors, 0 warnings
   - All models (Opus, Sonnet, Haiku) functional

2. **Functionality**:
   - `agileplus invoke --provider claude --model opus --prompt "test"` succeeds
   - Fallback to Sonnet on network error or rate limit
   - Stream mode produces correct output for long-form requests
   - Token counting accurate within ±5% (vs actual API response)

3. **Audit Trail**:
   - Every call recorded to provider_calls table
   - Input/output tokens captured
   - Latency measured in milliseconds
   - Hash chain verification passes

4. **Testing**:
   - `cargo test -p forgecode-providers` all pass
   - Integration tests with mock Claude API
   - Fallback retry logic tested

5. **Error Handling**:
   - Network errors handled gracefully with retry
   - Rate limits honored with backoff
   - API key missing produces clear error message

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| ClaudeProvider struct | Core implementation | Trait impl, no warnings |
| Multi-model support | Opus, Sonnet, Haiku routing | All models work |
| Streaming support | Async streaming output | Correct output, type-safe |
| Token counting | Input/output token tracking | Within ±5% of API |
| Error handling | Network, rate limit, auth errors | Logged, retriable |
| Tests & docs | Unit + integration tests | ≥90% coverage |

---

## Dependencies

**Depends On**:
- WP-001 (Provider Trait Registry)

**Blocks**:
- WP-004 (Subagent Spawning)
- WP-05 (Capability Discovery)
- WP-06 (Performance Metrics)

---

## Effort Estimate

- **Estimated LOC**: 350
- **Estimated Tool Calls**: 10-12
- **Estimated Duration**: 3-4 days
- **Requires**: Anthropic API key for testing

---

## Technical Details

### Key Components

```rust
pub struct ClaudeProvider {
    client: Arc<anthropic::Client>,
    model_strategy: ModelStrategy,
    fallback_enabled: bool,
}

impl Provider for ClaudeProvider {
    fn invoke(&self, prompt: String) -> Result<String> {
        // Route to appropriate model, with fallback
    }

    fn stream(&self, prompt: String) -> Result<ProviderStream> {
        // Return async stream of tokens
    }

    fn capabilities(&self) -> ProviderCapabilities {
        // Return models, costs, latencies
    }
}
```

### Model Configuration

```toml
[[claude.models]]
name = "opus"
cost_per_1m_input = 15.0
cost_per_1m_output = 75.0
latency_p95_ms = 500

[[claude.models]]
name = "sonnet"
cost_per_1m_input = 3.0
cost_per_1m_output = 15.0
latency_p95_ms = 300

[[claude.models]]
name = "haiku"
cost_per_1m_input = 0.80
cost_per_1m_output = 4.0
latency_p95_ms = 100
```

---

## Subtasks

- [ ] T006: Create `forgecode-providers/src/claude.rs` with `ClaudeProvider` struct
- [ ] T007: Implement `Provider::invoke()` with error handling and retry logic
- [ ] T008: Add streaming support via `Provider::stream()` (AsyncIterator)
- [ ] T009: Wire token counting and audit trail recording
- [ ] T010: Write integration tests with mock Claude API
- [ ] T011: Document model selection strategy and fallback behavior

---

## Testing Strategy

1. **Unit Tests**:
   - Provider initialization
   - Model selection logic
   - Token counting accuracy
   - Error handling (network, auth, rate limit)

2. **Integration Tests**:
   - Mock Claude API server
   - Full invoke() → audit trail flow
   - Streaming output correctness
   - Fallback retry logic

3. **Manual Testing**:
   - Real API key testing (against staging/test account)
   - Token count verification
   - Latency baseline measurement

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| Token Accuracy | ±5% | Compare with actual API response |
| Latency p95 | <1s | Benchmark real API |
| Test Coverage | ≥90% | `cargo tarpaulin` |
| Build Success | 0 warnings | `cargo clippy` |
| Fallback Success | 100% | Fallback test passes |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| API changes | Low | High | Use stable API version, vendor SDK |
| Token counting mismatch | Medium | Medium | Add test suite validating counts |
| Rate limiting | Low | Medium | Implement exponential backoff |

---

## Traceability

**Functional Requirements**:
- FR-PROV03: Claude provider with multi-model support
- FR-PROV04: Provider fallback and error handling

**Related Documents**:
- agileplus_spec_outlines.md (Lines 86-112)

---

## Notes

- Use anthropic-sdk v0.7+ for streaming support
- Consider caching model metadata in SQLite
- Fallback strategy: Opus → Sonnet → Haiku
- Future: Support additional Claude models as they release

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
