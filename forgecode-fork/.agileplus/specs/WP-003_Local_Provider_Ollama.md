# WP-003: Local Provider + Ollama Integration

**Work Package ID**: WP-003
**Epic**: eco-fork-001 (Custom Providers & Subagent Management)
**Phase**: 1
**Status**: Pending
**Priority**: High
**Created**: 2026-03-30

---

## Overview

Implement `LocalProvider` for Ollama-compatible inference servers, with dynamic model discovery, health checking, and graceful degradation.

## Description

Enable offline-first inference via local Ollama servers. This provider supports dynamic model discovery, health checks with graceful degradation, and zero-cost operation for local models.

---

## Objectives

- Implement `LocalProvider` struct for Ollama HTTP API
- Support dynamic model discovery from running Ollama instance
- Add health check endpoint monitoring with exponential backoff
- Implement graceful degradation (warn if unavailable, continue)
- Cost model: zero cost for local inference

---

## Acceptance Criteria

1. **Provider Implementation**:
   - `LocalProvider` implements Provider trait cleanly
   - Compiles with 0 errors, 0 warnings

2. **Functionality**:
   - `agileplus invoke --provider local --model llama2 --prompt "test"` succeeds
   - Dynamic model discovery works for Ollama model list
   - Automatic model list refresh on each call

3. **Health Checking**:
   - Health check detects unavailable Ollama and warns user
   - Graceful degradation: continue with warning if down
   - Exponential backoff on consecutive failures (max retry 30s)

4. **Cost Tracking**:
   - Audit table shows zero cost for local invocations
   - Cost delta vs Claude tracked for comparison

5. **Testing**:
   - `cargo test -p forgecode-providers` all pass
   - Mock Ollama server tests
   - Graceful degradation tests

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| LocalProvider struct | Ollama HTTP client | Trait impl, no warnings |
| Model discovery | Dynamic /api/tags polling | Reflects current models |
| Health checking | /api/health endpoint monitor | Backoff strategy tested |
| Graceful degradation | Warn if unavailable | Message logged, flow continues |
| Cost tracking | Zero cost model | Audit table correct |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-001 (Provider Registry)

**Blocks**:
- WP-004 (Subagent Spawning)
- WP-05 (Capability Discovery)

---

## Effort Estimate

- **Estimated LOC**: 280
- **Estimated Tool Calls**: 8-10
- **Estimated Duration**: 2-3 days

---

## Technical Details

### Key Components

```rust
pub struct LocalProvider {
    ollama_url: String,
    client: reqwest::Client,
    health_check_interval: Duration,
    last_health_check: Arc<Mutex<Instant>>,
}

impl Provider for LocalProvider {
    fn invoke(&self, prompt: String) -> Result<String> {
        // Check health, get models, invoke generate API
    }

    fn capabilities(&self) -> ProviderCapabilities {
        // Return dynamic model list from Ollama
    }
}
```

### Ollama API Integration

```
GET /api/tags              → List available models
POST /api/generate         → Invoke model with prompt
GET /api/health            → Health check
POST /api/pull             → Download model (optional)
```

---

## Subtasks

- [ ] T012: Create `forgecode-providers/src/local.rs` with `LocalProvider`
- [ ] T013: Implement Ollama API client with model discovery via /api/tags
- [ ] T014: Add health check endpoint (`/api/health`) monitoring
- [ ] T015: Implement graceful degradation (continue with warning if unavailable)
- [ ] T016: Unit tests with mock Ollama server
- [ ] T017: Cost model verification (local = free in audit table)

---

## Testing Strategy

1. **Unit Tests**:
   - Provider initialization
   - Model list parsing
   - Health check state transitions
   - Graceful degradation behavior

2. **Integration Tests**:
   - Mock Ollama server (docker-compose)
   - Full invoke flow with model discovery
   - Unavailable server handling
   - Concurrent requests

3. **Manual Testing**:
   - Real Ollama instance (docker run ollama/ollama)
   - Model download and inference
   - Health check accuracy

---

## Configuration

```toml
[local]
url = "http://localhost:11434"
health_check_interval_ms = 30000
backoff_multiplier = 2.0
max_backoff_ms = 30000
default_model = "llama2"
```

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| Model Discovery | <100ms | /api/tags response time |
| Health Check | 30s backoff | Exponential backoff test |
| Graceful Degradation | 100% | Down server handled |
| Cost Accuracy | 0.00 | Audit table shows zero cost |
| Test Coverage | ≥85% | `cargo tarpaulin` |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Ollama unavailable | Medium | Low | Graceful degradation + warning |
| Model list changes | Low | Low | Refresh on each call |
| Network latency | Medium | Low | Async requests, timeout |

---

## Traceability

**Functional Requirements**:
- FR-PROV05: Local provider for Ollama-compatible inference
- FR-PROV06: Provider health checking and graceful degradation

---

## Notes

- Ollama runs at http://localhost:11434 by default
- Support for remote Ollama instances (configurable URL)
- Future: Support model pull/download via Ollama API

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
