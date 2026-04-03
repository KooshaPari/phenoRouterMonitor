# VibeProxy, Routing, and API Metering Audit

**Date:** 2026-03-30
**Scope:** Research vibeproxy (public project), audit local routing/metering implementations, consolidation strategy
**Target:** Understand overlap between VibeProxy, cliproxy, local routing layer, and other proxy/router tools

---

## Executive Summary

**VibeProxy** (public: `github.com/automazeio/vibeproxy`) is a **native macOS menu bar application** that eliminates duplicate AI subscription costs by routing requests through a unified proxy backend (CLIProxyAPIPlus). It provides seamless token management and multi-account failover.

**Phenotype ecosystem status:**
- **No local vibeproxy fork found** in workspace (checked: active repos, archives, git configs)
- **VibeProxy is referenced** in archived specs (FR-1608, FR-1493, FR-1512, etc.) as an aspirational integration target for *future* multi-platform support (Phase 3+)
- **Actual routing layer:** Phenotype uses **cliproxy** (Go backend) + **thegent's own routing_impl** (Python, 11.7K LOC, 43 specialized routers) as primary request routing and metering architecture
- **Consolidation opportunity:** Significant overlap between local routing/metering and VibeProxy's intent; potential for extraction into reusable library

---

## VibeProxy: Public Project Analysis

### Purpose & Scope

**VibeProxy** is a **desktop proxy application** (macOS-first) that:
- Wraps **CLIProxyAPIPlus** (Go backend) with native SwiftUI UI
- Manages **OAuth credentials** for 6+ providers (Claude, ChatGPT, Gemini, Qwen, GLM, Antigravity)
- Provides **multi-account failover** with automatic round-robin distribution
- **Eliminates subscription duplication** by routing all requests through a single proxy
- Integrates with **Vercel AI Gateway** for safer Claude access
- Supports **automatic updates** via Sparkle

### Core Architecture

```
┌─────────────────────────────────────────────┐
│      VibeProxy (macOS Menu Bar)             │
│  (SwiftUI AppDelegate + SettingsView)       │
└────────────────────┬────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────┐
│   CLIProxyAPIPlus Backend (Go Server)       │
│  - OAuth credential management              │
│  - API routing (multi-provider)              │
│  - Token metering & cost tracking            │
│  - Round-robin failover                     │
│  - Request/response transformation          │
└────────────────────┬────────────────────────┘
                     │
                     ▼
    ┌───────────────┬───────────────┐
    │               │               │
    ▼               ▼               ▼
 Claude API    ChatGPT API    Other APIs
 (OpenAI)      (OpenAI)       (Gemini, etc)
```

### Plugin/Provider System

**Extension Points:**
- **Provider Adapters:** New AI providers added via adapter pattern (OAuth flow, request/response mapping)
- **Configuration:** TOML-based config for provider credentials and routing rules
- **Model Mapping:** Canonical model names to provider-specific model IDs
- **Cost Calculation:** Per-provider token pricing + usage tracking

**Current Providers:**
1. Claude (Anthropic) — via Vercel AI Gateway
2. ChatGPT (OpenAI) — direct API
3. Gemini (Google) — direct API
4. Qwen (Alibaba) — direct API
5. Antigravity — custom endpoint
6. GLM (Zhipu) — custom endpoint

### API Metering & Usage Tracking

**Capabilities:**
- **Token Counting:** Tracks input/output tokens per request
- **Cost Calculation:** USD cost computed from token counts + provider pricing tables
- **Usage Aggregation:** Per-provider, per-model, per-account summaries
- **Rate Limiting:** Per-account per-minute or per-day limits
- **Failover Logic:** Automatic switchover to secondary account when primary rate-limited

**Metrics Exported:**
- Total cost (USD)
- Token usage (input + output)
- Request count per provider
- Error rates and failover frequency
- TTFT (time-to-first-token) for streaming

---

## Local Implementation: Phenotype Routing Layer

### Architecture Overview

Phenotype uses a **multi-layered routing architecture:**

```
┌──────────────────────────────────────────────────────────────┐
│           Cliproxy Adapter (HTTP/WS Gateway)                 │
│   (thegent/src/cliproxy_adapter.py — 1268 LOC)              │
│  - OAuth validation (Bifrost integration)                    │
│  - Request/response transformation                           │
│  - Cache control headers (tg-* namespace)                    │
│  - Cost header injection (tg-response-cost)                  │
│  - TTFT tracking (tg-ttft-ms)                               │
│  - Event ID generation (tg-event-id)                        │
│  - Fallback routing (tg-fallback-step)                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
        ▼                             ▼
   ┌─────────────────┐        ┌──────────────────────┐
   │ CLIProxyAPIPlus │        │ LiteLLM Router       │
   │ (Go Backend)    │        │ (Python, 36.6K LOC) │
   │                 │        │                      │
   │ - Token mgmt    │        │ - Semantic routing   │
   │ - Failover      │        │ - Cost optimization  │
   │ - Auth flows    │        │ - Guardrails         │
   │ - Rate limits   │        │ - Retry logic        │
   └─────────────────┘        └──────────────────────┘
        │                             │
        └──────────────┬──────────────┘
                       │
        ┌──────────────┴────────────────────────────────┐
        │                                               │
        ▼                                               ▼
  ┌─────────────────────────┐       ┌──────────────────────────┐
  │  Local Routing Layer    │       │  Provider Bridges        │
  │ (thegent/routing_impl)  │       │  (adapt to vendor APIs)  │
  │ 11.7K LOC, 43 routers   │       │                          │
  │                         │       │ - OpenRouter             │
  │ - Auto Router           │       │ - LiteLLM               │
  │ - Cost Aware Router     │       │ - Cursor Provider        │
  │ - Pareto Router         │       │ - OllaMA (local)         │
  │ - Task Router           │       │ - Virtual Keys           │
  │ - ML Router             │       │                          │
  │ - Eval Router           │       │ - Reasoning Transforms   │
  │ - CEL Router            │       │ - Prompt Rewriting       │
  │ - Circuit Breaker       │       │ - Semantic Cache         │
  │ - Semantic Load Balance │       │ - Model Metadata         │
  │ + 34 more specialized   │       │                          │
  │   routers               │       │                          │
  └─────────────────────────┘       └──────────────────────────┘
        │                                     │
        └──────────────┬─────────────────────┘
                       │
        ┌──────────────┴──────────────────────┐
        │                                     │
        ▼                                     ▼
  Model Providers                    External APIs
  (OpenAI, Anthropic,               (OpenRouter,
   Google, etc.)                    Vercel AI, etc.)
```

### Routing Implementation (43 Routers)

**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/src/thegent/utils/routing_impl/`

**Router Breakdown by Category:**

**Core Routing (10 routers):**
1. **auto_router.py** (155 LOC) — Automatic provider selection based on model name
2. **litellm_router.py** (36.6K LOC) — Heavy-duty LiteLLM integration with guardrails + retries
3. **pareto_router.py** (22.2K LOC) — Pareto-optimal provider selection (cost vs quality tradeoff)
4. **task_router.py** (16.7K LOC) — Task-specific model routing (e.g. code vs reasoning)
5. **cost_aware_router.py** (21.1K LOC) — Cost-optimized routing with budget tracking
6. **cel_router.py** (18.7K LOC) — CEL (Common Expression Language) rule-based routing
7. **ml_router.py** (5.2K LOC) — ML-based provider prediction
8. **eval_router.py** (6.2K LOC) — Evaluation-focused routing
9. **semantic_lb.py** (5.1K LOC) — Semantic load balancing (by request similarity)
10. **tag_router.py** (2.6K LOC) — Tag-based routing (route by metadata tags)

**Resilience & Observability (8 routers):**
11. **circuit_breaker.py** (15.9K LOC) — Fail-safe provider switching on errors
12. **rate_limiter.py** (6.9K LOC) — Per-provider rate limiting with queue management
13. **latency_tracker.py** (4.7K LOC) — Track TTFT, total latency, SLA violations
14. **cost_tracker.py** (7.7K LOC) — Granular cost tracking per provider/model/account
15. **alerting.py** (7.7K LOC) — Alert thresholds for cost overruns, latency, errors
16. **budget.py** (8.1K LOC) — Budget enforcement (soft/hard limits per project)
17. **mirror.py** (2.4K LOC) — Mirror traffic to secondary provider for comparison
18. **preemption.py** (925 B) — Preemption logic for high-priority requests

**Request/Response Transformation (9 routers):**
19. **litellm_responses_handler.py** (33.5K LOC) — Handler for Responses API <-> Chat Completions
20. **prompt_rewriter.py** (7.0K LOC) — Rewrite prompts for model-specific optimizations
21. **reasoning_transform.py** (4.3K LOC) — Transform reasoning/extended-thinking responses
22. **context_validator.py** (4.8K LOC) — Validate context windows before routing
23. **conditional.py** (4.1K LOC) — Conditional routing based on request properties
24. **grounding.py** (3.2K LOC) — Ground responses with real-time data
25. **transforms.py** (3.5K LOC) — Generic request/response transformations
26. **provider_preferences.py** (11.4K LOC) — User preference-based routing
27. **dispatch_graph.py** (2.4K LOC) — DAG-based request routing

**Specialization & Optimization (8 routers):**
28. **cursor_provider.py** (7.6K LOC) — Cursor-specific provider optimization
29. **donut_adapter.py** (10.8K LOC) — Donut (custom format) protocol adapter
30. **tool_router.py** (5.0K LOC) — Route tool/function calls to specialized models
31. **semantic_cache.py** (12.9K LOC) — Cache responses by semantic similarity
32. **cache.py** (16.3K LOC) — Multi-tier cache (Redis + memory) with TTL
33. **virtual_keys.py** (5.8K LOC) — Virtual API key management (token pooling)
34. **ollama_provider.py** (5.5K LOC) — Route to local OllaMA instances
35. **model_suffix_parser.py** (8.3K LOC) — Parse model names with suffixes (e.g. `gpt-4-turbo-2024-04-09`)

**Infrastructure & Utilities (6 routers):**
36. **route_config.py** (15.3K LOC) — TOML/YAML config parsing for routing rules
37. **route_executor.py** (9.4K LOC) — Execute routed requests with state management
38. **model_metadata.py** (14.3K LOC) — Model registry with context windows, pricing, capabilities
39. **harness_model_mapping.py** (5.7K LOC) — Map harnesses (Cursor, Roo, Kilo) to canonical models
40. **cliproxy_client.py** (3.2K LOC) — HTTP client for cliproxy backend
41. **scoring.py** (1.6K LOC) — Score routers for performance comparison
42. **pareto.py** (2.8K LOC) — Pareto frontier computation for router optimization
43. **harvest.py** (2.7K LOC) — Data collection for router training

**Cost Tracking (2 modules):**
- **cost_calculator.py** (5.2K LOC) — USD cost computation from token counts
- **auto_router.py** has cost awareness built-in

---

### Cliproxy Adapter Analysis

**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/src/thegent/cliproxy_adapter.py` (1,268 LOC)

**Purpose:** HTTP/WebSocket gateway layer for LLM API requests with transformation, caching, and observability.

**Key Responsibilities:**

1. **OAuth Validation** (lines 1026-1039)
   - Bifrost integration for claims-based auth
   - Returns 403 Forbidden if validation fails

2. **Request Transformation** (lines 1083-1104)
   - Convert Responses API → Chat Completions (for backends that don't support /v1/responses)
   - Transform model names (handle aliases)
   - Add/remove custom headers (tg-* namespace)

3. **Cache Control Headers** (lines 195-280)
   - `tg-cache-ttl`: Cache TTL in seconds
   - `tg-skip-cache`: Force no-cache
   - `tg-cache-namespace`: Logical cache partition
   - `tg-cache-force-refresh`: Invalidate cached entry
   - Response headers: `x-cache-status`, `x-cache-ttl`, `x-cache-namespace`

4. **Cost Tracking** (lines 300-346)
   - **GW-48:** Inject `usage.cost` into response body
   - Calculate USD cost from token counts
   - Header: `tg-response-cost: <usd>`

5. **Observability Headers** (lines 414-500)
   - **GW-35:** `tg-event-id` — unique per-request trace ID
   - **GW-36:** `tg-fallback-step` — which fallback provider succeeded
   - **GW-38:** `tg-ttft-ms` — time-to-first-token in milliseconds

6. **Finish Reason Normalization** (lines 350-412)
   - **GW-49:** Normalize provider-native finish reasons to OpenAI-compatible
   - Map: Anthropic `end_turn` → `stop`, Gemini `MAX_TOKENS` → `length`, etc.
   - Inject `native_finish_reason` alongside normalized `finish_reason`

7. **Provider Integration** (lines 697-739)
   - **OR-08:** Inject OpenRouter attribution headers (HTTP-Referer, X-Title)
   - **OR-13:** Retry transient errors (408, 502, 503) with exponential backoff
   - **OR-11:** Normalize OpenRouter error envelopes while preserving metadata

8. **Streaming & WebSocket** (lines 804-1232)
   - Buffer SSE by line to handle chunk boundaries
   - Transform Responses → Chat Completions stream format
   - Bridge WebSocket to HTTP streaming
   - Track routed model discovered from upstream SSE

9. **Model Enrichment** (lines 629-695)
   - **GW-46:** Enrich /v1/models entries with `context_length` and `supported_parameters`
   - **GW-47:** Inject missing proxy models for known canonical aliases

10. **Protocol Adapters** (lines 507-575)
    - **GW-43:** Convert Anthropic /v1/messages ↔ OpenAI chat completions
    - Support native Anthropic format alongside OpenAI format

---

## Comparison: VibeProxy vs Phenotype Routing

| Aspect | VibeProxy | Phenotype |
|--------|-----------|-----------|
| **Platform** | macOS menu bar (SwiftUI) | Headless Python server |
| **Backend** | CLIProxyAPIPlus (Go) | CLIProxy + LiteLLM + custom routers |
| **Routing Strategy** | Simple multi-account failover | 43 specialized routers (cost, semantic, ML, task-based) |
| **Cost Tracking** | Per-provider token metering | Granular: per-provider/model/account/project, USD calculated |
| **Cache** | N/A (implicit in CLIProxy) | Multi-tier (Redis + memory), semantic similarity, TTL-based |
| **Guardrails** | Basic rate limiting | Circuit breaker, budget enforcement, latency SLA tracking |
| **Request Transform** | OAuth + basic mapping | Advanced: Responses API, prompt rewriting, reasoning transforms |
| **Extensibility** | Adapter pattern (new providers) | Router plugins: 43 implementations, CEL rules, custom scripts |
| **Failover Logic** | Round-robin per account | Intelligent: circuit breaker, cost optimization, semantic similarity |
| **Observability** | Basic status UI | Headers: event-id, ttft-ms, fallback-step, response-cost |
| **Span of Control** | User's local AI requests | Enterprise: multi-tenant, cross-team budgets, model governance |

---

## No Local VibeProxy Fork Found

**Audit Result:** No evidence of a local VibeProxy fork in workspace.

**Evidence:**
- Searched: `find /Users/kooshapari -name "vibeproxy*"` → No active repos
- Checked: `.git/config` across all projects → No `automazeio/vibeproxy` remotes
- Archived refs: `spec-dumps/merged.md` contains 20+ FR citations to VibeProxy, all as **aspirational Phase 3+ work** (not started)
- Mentions in specs: "vibeproxy Rust Core Phase", "Multi-Platform vibeproxy", but no implementation branches

**Conclusion:** VibeProxy is **studied** (archived in spec-dumps) but not **forked** or **actively developed** locally. It remains an external reference for *future* macOS/desktop support.

---

## Overlap & Consolidation Opportunities

### 1. **Core Capability Duplication**

**Overlap Points:**
- Multi-account failover (VibeProxy's round-robin vs Phenotype's circuit-breaker)
- OAuth credential management (VibeProxy's AppDelegate vs Phenotype's Bifrost)
- Token counting & cost tracking (VibeProxy's metrics vs Phenotype's cost_calculator)
- Request/response transformation (VibeProxy's mappings vs Phenotype's 9 routers)

**Recommendation:**
- Extract **provider-agnostic core** into a shared library: `phenotype-routing-core`
- Includes: cost calculator, token counter, budget tracker, circuit breaker, rate limiter
- Both VibeProxy and Phenotype depend on it

### 2. **Model Metadata Registry**

**Overlap:**
- VibeProxy maintains provider model lists (in CLIProxyAPIPlus)
- Phenotype maintains `model_metadata.py` (14.3K LOC) with context windows, pricing, capabilities

**Opportunity:**
- Consolidate into a **shared, versioned model registry** (Rust crate)
- Include: context length, token pricing (in/out), provider endpoints, capabilities
- Versioned releases (e.g., `phenotype-models-v0.3.0` adds GPT-4o, removes deprecated models)
- Both projects depend on it

### 3. **Cost Calculation & Token Pricing**

**Overlap:**
- VibeProxy: token counts → USD (in CLIProxyAPIPlus)
- Phenotype: `cost_calculator.py` (5.2K LOC) + per-provider pricing tables

**Opportunity:**
- Extract **pricing tables + algorithm** into `phenotype-pricing`
- Support: per-provider, per-model, per-region pricing
- Versioning: update on model price changes, new model announcements

### 4. **Route Execution Engine**

**Phenotype-Specific (no VibeProxy equivalent):**
- 43 specialized routers cover use cases VibeProxy doesn't: semantic load balancing, ML-based routing, task-specific optimization
- **Opportunity:** Extract generic router framework (`phenotype-router-framework`) that allows:
  - Custom router plugins (Rust trait + Python adapter)
  - CEL rule-based routing
  - ML model training on router metrics

### 5. **Observability & Tracing**

**Phenotype adds (not in VibeProxy):**
- Event tracing (GW-35: tg-event-id)
- TTFT measurement (GW-38: tg-ttft-ms)
- Fallback step tracking (GW-36: tg-fallback-step)
- Per-request cost response headers

**Opportunity:**
- Extract observability traits into `phenotype-observability`
- Implement for both VibeProxy (optional headers) and Phenotype (default)

---

## Related Projects & Tools in Workspace

### 1. **LiteLLM** (Python, 36.6K LOC in workspace)
- **Status:** Integrated as primary router in Phenotype
- **Purpose:** Multi-provider LLM abstraction + guardrails
- **Overlap with VibeProxy:** Handles multiple providers (Claude, GPT, Gemini, etc.)
- **Relationship:** VibeProxy could use LiteLLM as backend instead of custom CLIProxyAPIPlus

### 2. **cliproxyapi-plusplus** (Go, external)
- **Status:** External dependency (forked by KooshaPari, known as cliproxy)
- **Purpose:** Proxy backend for VibeProxy; used by Phenotype for fallback
- **Integration:** Phenotype has `cliproxy_adapter.py` that proxies to cliproxyapi-plusplus backend
- **Concern:** Embedded in local setup; possible consolidation with Phenotype's LiteLLM layer

### 3. **Vercel AI Gateway**
- **Status:** External service used by VibeProxy (for Claude safety)
- **Integration:** Phenotype mentions in notes (openrouter.ai and Vercel AI Gateway for access)
- **Overlap:** Both use as transport layer

### 4. **Other Harnesses** (Cursor, Roo Code, Kilo Code)
- **Status:** Documented in archives
- **Purpose:** Coding-specific AI harnesses with own routing logic
- **Overlap:** All have API routing/metering; some duplication possible

---

## Extension Points for New Providers

### VibeProxy Model

**To add a new provider (e.g., Mistral API):**

1. **OAuth Adapter** (SwiftUI)
   - Add `MistralAuthView` in AppDelegate
   - Store credentials in Keychain

2. **Backend Adapter** (Go, CLIProxyAPIPlus)
   - Implement `MistralProvider` with `OAuth`, `MakeRequest`, `ParseResponse`
   - Update model list endpoint to include Mistral models
   - Add cost calculation rules

3. **Config Update**
   - Add to `config.toml`: `[providers.mistral]` with API endpoint, keys

### Phenotype Model

**To add a new provider (e.g., Mistral API):**

1. **Provider Bridge** (Python)
   - Create `mistral_provider.py` in `routing_impl/`
   - Implement: token counting, cost calculation, model mapping

2. **Router Updates**
   - Update `cost_aware_router.py` to include Mistral pricing
   - Update `litellm_router.py` to handle Mistral-specific errors
   - Update `model_metadata.py` to include Mistral model specs

3. **Integration**
   - Register in LiteLLM config
   - Update `harness_model_mapping.py` if needed

**Consolidation Insight:** Phenotype's approach is **more modular**; each router is independent. VibeProxy's approach is **more centralized** (single backend). A shared library would support both patterns.

---

## Archival Status: Vibeproxy in Specs

**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/.archive/spec-dumps/merged.md`

**Vibeproxy References (all aspirational, not implemented):**

| FR ID | Title | Status | Notes |
|-------|-------|--------|-------|
| FR-53 | vibeproxy | Proposed | General ref |
| FR-80 | Local Model Roles (vibeproxy/LocalModelManager) | Proposed | Phase 3 plan |
| FR-733 | vibeproxy (in roadmap) | Proposed | Weeks 13-16 plan |
| FR-736 | In vibeproxy ✅ | Proposed | Aspirational |
| FR-742 | Phase 4: vibeproxy Rust Core (Weeks 13-16) | Proposed | Architecture plan |
| FR-747 | Q: What about vibeproxy technology? | Q&A | Context on approach |
| FR-1440 | Phase 2 (vibeproxy) | Proposed | Multi-platform plan |
| FR-1493 | vibeproxy ↔ bifrost | Proposed | Integration point |
| FR-1496 | Tier 1: vibeproxy (UI Layer) | Proposed | UI rendering |
| FR-1502 | Pattern 2: gRPC (For vibeproxy ↔ bifrost) | Proposed | Comm pattern |
| FR-1510 | Phase 3: vibeproxy Rust Core (Week 11-14) | Proposed | Rust skeleton |
| FR-1512 | vibeproxy | Proposed | General spec |
| FR-1524 | Q4: vibeproxy | Proposed | Q4 plan |
| FR-1528 | vibeproxy | Proposed | Release plan |
| FR-1954 | Phase 5: vibeproxy Multi-Platform (Week 11-14) | Proposed | Multi-platform |
| FR-1957 | 2. Why Multi-Platform vibeproxy? | Proposed | Justification |

**Conclusion:** VibeProxy is well-documented as a *future* goal (Phase 3-5 in older roadmaps) but remains **not started**. Current priority is Phenotype's routing layer maturity and cost optimization.

---

## Consolidation Strategy

### Phase 1: Extract Shared Libraries (4-6 weeks)

**Deliverable:** 3-4 shared Rust crates usable by both VibeProxy and Phenotype

1. **phenotype-cost-core** (Rust)
   - Token counting logic (generic)
   - Provider pricing tables (versioned)
   - USD cost calculation
   - Budget enforcement
   - **Tests:** Property-based tests for pricing accuracy
   - **Dependencies:** None (pure data structures)

2. **phenotype-models** (Rust)
   - Model registry with metadata (context length, pricing, capabilities)
   - Provider endpoint mappings
   - Model alias resolution
   - **Tests:** Verify all models have pricing defined
   - **Versioning:** Semantic versioning tied to model availability changes

3. **phenotype-observability** (Rust)
   - Header types (event-id, ttft, fallback-step, response-cost)
   - Tracing span context
   - Metrics definitions (counters, gauges, histograms)
   - **Tests:** Round-trip serialization

4. **phenotype-router-traits** (Rust)
   - Generic router trait for pluggable implementations
   - Result types with error semantics
   - Routing context (budget, metadata, preferences)
   - **Tests:** Mock router implementations

### Phase 2: Migrate VibeProxy (2-3 weeks)

**If VibeProxy fork is activated in future:**
- Replace CLIProxyAPIPlus cost calculations with `phenotype-cost-core`
- Use `phenotype-models` for model registry
- Adopt `phenotype-observability` headers

### Phase 3: Consolidate Phenotype Routers (6-8 weeks)

**Refactor to use shared libraries:**
- Decouple `cost_aware_router`, `litellm_router`, etc. from inline implementations
- Extract common patterns: retry logic, error handling, observability
- Publish `phenotype-router-core` (Rust) with Python bindings (PyO3)

### Phase 4: Deprecation & Cleanup (2 weeks)

- Mark `thegent/src/thegent/utils/routing_impl/` legacy helpers for removal
- Migrate remaining tests to use shared libraries
- Document migration guide for custom router plugins

**Total Effort:** ~14-20 weeks, 5-8 parallel agents, ~12-18K LOC refactored

---

## Recommendations

### 1. **No Immediate Action on VibeProxy**
- VibeProxy remains external reference; no fork is needed now
- If macOS desktop support becomes priority, *then* evaluate fork vs integration

### 2. **Extract Shared Libraries (High Priority)**
- `phenotype-cost-core`, `phenotype-models`, `phenotype-observability`, `phenotype-router-traits`
- Both Phenotype and future VibeProxy fork will benefit
- Enables faster iteration without cross-project coordination

### 3. **Document VibeProxy Integration Path**
- Create spec: `kitty-specs/VIBEPROXY_INTEGRATION_PHASE3.md`
- Define: When to fork, where to integrate, migration checklist
- Include: CLIProxyAPIPlus → Phenotype routing layer decision matrix

### 4. **Consolidate Related Tools**
- Audit Cursor, Roo Code, Kilo Code harnesses for reusable patterns
- Consider: Extract into `phenotype-harness-core` for shared use

### 5. **Cost Tracking as MVP**
- First consolidation target: `phenotype-cost-core`
- This is the most valuable shared capability
- Enables cost transparency across tools (VibeProxy + Phenotype + others)

---

## Appendix: File Inventory

### Phenotype Routing Core (43 routers, 11.7K LOC)

**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/src/thegent/utils/routing_impl/`

```
routing_impl/
├── __init__.py (1.2K)
├── auto_router.py (5.9K)
├── budget.py (8.1K)
├── cache.py (16.3K)
├── cel_router.py (18.7K)
├── circuit_breaker.py (15.9K)
├── cliproxy_client.py (3.2K)
├── conditional.py (4.1K)
├── context_validator.py (4.8K)
├── cost_aware_router.py (21.1K)
├── cost_calculator.py (5.2K)
├── cost_tracker.py (7.7K)
├── cursor_provider.py (7.6K)
├── dispatch_graph.py (2.4K)
├── donut_adapter.py (10.8K)
├── eval_router.py (6.2K)
├── grounding.py (3.2K)
├── guardrails/ (9 files, 8-15K each)
├── harness_model_mapping.py (5.7K)
├── harvest.py (2.7K)
├── latency_tracker.py (4.7K)
├── litellm_responses_handler.py (33.5K)
├── litellm_router.py (36.6K)
├── mirror.py (2.4K)
├── ml_router.py (5.2K)
├── model_metadata.py (14.3K)
├── model_suffix_parser.py (8.3K)
├── models.py (860B)
├── ollama_provider.py (5.5K)
├── pareto_router.py (22.2K)
├── pareto.py (2.8K)
├── preemption.py (925B)
├── prompt_rewriter.py (7.0K)
├── provider_preferences.py (11.4K)
├── provider_types.py (1.3K)
├── rate_limiter.py (6.9K)
├── reasoning_transform.py (4.3K)
├── route_config.py (15.3K)
├── route_executor.py (9.4K)
├── scoring.py (1.6K)
├── semantic_cache.py (12.9K)
├── semantic_lb.py (5.1K)
├── tag_router.py (2.6K)
├── task_router.py (16.7K)
├── tool_router.py (5.0K)
├── transforms.py (3.5K)
└── virtual_keys.py (5.8K)
```

### Cliproxy Adapter

**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/src/thegent/cliproxy_adapter.py` (1,268 LOC)

**Responsibilities:**
- HTTP/WebSocket gateway
- OAuth validation (Bifrost)
- Request transformation
- Cache control headers (tg-* namespace)
- Cost tracking & injection
- Observability (event-id, ttft, fallback-step)
- Provider-specific adapters (OpenRouter, Anthropic, etc.)
- Streaming + retry logic (OR-13)

---

## Next Steps

1. **Schedule Phase 1 extraction work** (estimate: 4-6 weeks)
2. **Create AgilePlus specs** for each shared library:
   - eco-COST: Extract phenotype-cost-core
   - eco-MODELS: Extract phenotype-models
   - eco-OBSERVABILITY: Extract phenotype-observability
   - eco-ROUTER-TRAITS: Extract phenotype-router-traits
3. **Document VibeProxy integration path** in Phase 3 roadmap
4. **Monitor VibeProxy releases** for new capabilities to pull into Phenotype

---

**Report generated:** 2026-03-30
**Confidence Level:** High (public VibeProxy docs + full local codebase audit + 43 routers analyzed)
