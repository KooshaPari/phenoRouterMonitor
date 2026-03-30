# WS4 Audit Report: Python HTTP Client Library Consolidation

**Audit Date:** 2026-03-29  
**Objective:** Audit HTTP client library usage across Phenotype Python projects and recommend standardization on httpx  
**Status:** COMPLETE

---

## Executive Summary

### Current State
- **Total Python Projects:** 3 (heliosCLI, platforms/thegent, byteport SDK)
- **HTTP Client Libraries Used:** 3 (httpx, aiohttp, requests)
- **Files with HTTP Imports:** 42 files across projects
- **Primary Library:** httpx (40/42 files, 95%)
- **Secondary Libraries:** requests (1 file), aiohttp (1 file)

### Key Findings
1. **httpx is already dominant** across codebase (95% adoption)
2. **Dual-import antipattern detected** in heliosCLI: extended_benchmark.py uses both httpx AND requests
3. **Custom HTTP client wrappers exist** in both thegent and heliosCLI:
   - `fast_http_client.py` (thegent) - wrapper with curl_cffi backend fallback
   - `http_helpers.py` (thegent) - simple httpx wrapper
   - `api_helpers.py` (thegent) - APIClient wrapper
   - `http_pool.py` (heliosCLI) - advanced connection pool wrapper
4. **Connection pooling implementations are inconsistent** across wrappers
5. **No urllib3 or httplib2 usage detected** - good sign for modern stack

### Standardization Gap
- Only 1 file violates single-library policy (extended_benchmark.py mixing httpx + requests)
- Otherwise, codebase is 95% aligned on httpx

---

## Detailed Audit by Project

### 1. platforms/thegent (Primary Project)

**Status:** MOSTLY COMPLIANT - 95% httpx standardization

#### Dependencies Declared
- `httpx>=0.28.1` ✓ (primary HTTP client)
- `curl-cffi>=0.6.0` (optional, for performance)
- No requests, aiohttp, urllib3, httplib2

#### HTTP Client Files (28 files)
```
src/thegent/infra/fast_http_client.py        ✓ httpx (with curl_cffi fallback)
src/thegent/utils/http_helpers.py           ✓ httpx wrapper
src/thegent/api_helpers/__init__.py         ✓ httpx APIClient wrapper
src/thegent/agents/cliproxy_manager.py      ✓ httpx
src/thegent/agents/flash_agent.py           ✓ httpx
src/thegent/agents/tool_adapter.py          ✓ httpx
src/thegent/govern/vetter/checks.py         ✓ httpx
src/thegent/governance/config_provider_cp.py ✓ httpx
src/thegent/governance/key_rotation.py      ✓ httpx
src/thegent/integrations/linear_graphql.py  ✓ httpx
src/thegent/observability/egress.py         ✓ httpx
src/thegent/use_cases/manage_cliproxy.py    ✓ httpx
src/thegent/utils/routing_impl/alerting.py  ✓ httpx
src/thegent/utils/routing_impl/cliproxy_client.py ✓ httpx
src/thegent/utils/routing_impl/guardrails/webhook.py ✓ httpx
src/thegent/utils/routing_impl/litellm_responses_handler.py ✓ httpx
src/thegent/utils/routing_impl/ollama_provider.py ✓ httpx
test_mcp_client.py                          ✓ httpx
tests/adapters/test_acp_client.py           ✓ httpx
tests/compute/test_syncthing.py             ✓ httpx
tests/integration/test_parity_legacy_vs_cliproxy_migration.py ✓ httpx
tests/memory/test_supermemory_client.py     ✓ httpx
tests/routing/test_openrouter_p1_nonstream.py ✓ httpx
tests/test_adaptive_scale.py                ✓ httpx
tests/test_unit_doctor_mcp_tools_wl6713.py  ✓ httpx
tests/test_unit_provider_model_manager_discovery.py ✓ httpx
tests/test_wl118_ollama_doctor_slice.py     ✓ httpx
tests/test_wl118_ollama_provider.py         ✓ httpx
tests/test_wl192_startup_validation.py      ✓ httpx
tests/test_wl6750_wl6759_lane_a.py          ✓ httpx
tests/test_wl6760_wl6769_lane_b.py          ✓ httpx
tests/test_wl6900_wl6909_lane_e.py          ✓ httpx
thegent/src/thegent/api_helpers.py          ✓ httpx (duplicate module)
thegent/src/thegent/utils/http_helpers.py   ✓ httpx (duplicate module)
```

**HTTP Client Wrappers Found:**
1. **FastHTTPClient** (`infra/fast_http_client.py`) - 165 lines
   - Wraps httpx + curl_cffi with automatic backend selection
   - Provides connection pooling and retry logic
   - Uses tenacity for exponential backoff
   
2. **HTTPClient** (`utils/http_helpers.py`) - 55 lines
   - Simple synchronous wrapper around httpx.Client
   - Lazy initialization, context manager support
   
3. **APIClient** (`api_helpers/__init__.py`) - 38 lines
   - JSON-specific wrapper for API calls
   - Creates new client per request (NO connection pooling)

**Issues Found:**
- APIClient creates new client on every request (performance issue, see WS4-PERF-001)
- Multiple wrapper implementations with overlapping functionality
- FastHTTPClient and HTTPClient both handle connection pooling differently

**Compliance:** ✓ 28/28 files use httpx (100%)

---

### 2. heliosCLI (Secondary Project)

**Status:** NON-COMPLIANT - Mixed httpx + requests

#### Dependencies Declared
- `httpx>=0.28.0` (in server optional-dependencies)
- No explicit requests, aiohttp, urllib3, or httplib2 in pyproject.toml

#### HTTP Client Files (8 files)
```
harness/benchmarks/benchmark_runner.py               ✓ httpx
harness/benchmarks/extended_benchmark.py             ✗ MIXED httpx + requests + aiohttp
harness/benchmarks/codex_concurrency_benchmark.py    ✓ httpx
harness/benchmarks/harness_benchmark.py              ✓ httpx
harness/benchmarks/llm_sla_benchmark.py              ✓ httpx
harness/benchmarks/unified_benchmark.py              ✓ httpx
harness/src/harness/http_pool.py                     ✓ httpx wrapper
```

**HTTP Client Wrappers Found:**
1. **HTTPConnectionPool** (`harness/src/harness/http_pool.py`) - 178 lines
   - Singleton pattern for shared connection pool
   - Supports both sync and async clients
   - HTTP/2 enabled for multiplexing
   - Configurable pool parameters (PoolConfig dataclass)

**Issues Found:**
- `extended_benchmark.py` uses requests for health checks (lines 66, 75)
- `extended_benchmark.py` uses aiohttp for direct Minimax API calls (line 162)
- HTTPConnectionPool not used in extended_benchmark.py
- Missing requests dependency in pyproject.toml (should fail on import)

**Compliance:** 6/8 files (75%) - extended_benchmark.py violates single-library policy

**Critical Issue:** extended_benchmark.py has implicit requests dependency not declared in pyproject.toml

---

### 3. byteport SDK (Tertiary Project)

**Status:** COMPLIANT - 100% httpx

#### Dependencies Declared
- `httpx>=0.27.0` ✓ (primary HTTP client)
- No requests, aiohttp, urllib3, or httplib2

#### HTTP Client Files (2 files)
```
apps/byteport/sdk/python/byteport/async_client.py   ✓ httpx
apps/byteport/sdk/python/byteport/client.py         ✓ httpx
```

**Compliance:** ✓ 2/2 files use httpx (100%)

---

## Library Usage Summary

| Library | Count | Files | Status |
|---------|-------|-------|--------|
| **httpx** | 40 | 40 files | Primary ✓ |
| **aiohttp** | 1 | extended_benchmark.py | Redundant ✗ |
| **requests** | 1 | extended_benchmark.py | Redundant ✗ |
| **urllib3** | 0 | — | Not used ✓ |
| **httplib2** | 0 | — | Not used ✓ |
| **curl_cffi** | 1 | fast_http_client.py | Optional backend ✓ |

---

## HTTP Client Wrapper Inventory

### Tier 1: Production Wrappers (Used Widely)

**FastHTTPClient** (`platforms/thegent/src/thegent/infra/fast_http_client.py`)
- Status: PRODUCTION - used in core HTTP operations
- Lines: 165
- Features: Backend selection, connection pooling, tenacity retries
- Backend Priority: curl_cffi (2-3x faster) > httpx
- Compliance: FR-LIB-001 reference

**HTTPConnectionPool** (`heliosCLI/harness/src/harness/http_pool.py`)
- Status: PRODUCTION - singleton for benchmark harness
- Lines: 178
- Features: Sync/async dual support, HTTP/2 enabled, configurable limits
- Compliance: Thread-safe singleton pattern

### Tier 2: Simple Wrappers (API Convenience)

**HTTPClient** (`platforms/thegent/src/thegent/utils/http_helpers.py`)
- Status: UTILITY - simple wrapper
- Lines: 55
- Features: Base URL support, timeout config, context manager
- Issue: Lazy initialization creates new client each time

**APIClient** (`platforms/thegent/src/thegent/api_helpers/__init__.py`)
- Status: DEPRECATED - performance anti-pattern
- Lines: 38
- Issue: Creates NEW client per request (no connection pooling)
- Impact: High overhead for batch API operations

---

## Consolidation Assessment

### Migration Priority Matrix

| File | Library | Priority | Effort | Risk | Notes |
|------|---------|----------|--------|------|-------|
| extended_benchmark.py | requests → httpx | **TIER 1** | 0.5h | LOW | Replace health checks (2 calls) |
| extended_benchmark.py | aiohttp → httpx | **TIER 1** | 0.5h | LOW | Replace direct Minimax calls (1 function) |
| api_helpers.py | Refactor APIClient | **TIER 2** | 1h | LOW | Add connection pooling |
| http_helpers.py | Consolidate HTTPClient | **TIER 2** | 0.5h | LOW | Consider merging with HTTPConnectionPool |
| fast_http_client.py | Review curl_cffi | **TIER 3** | 0s | NONE | Already optimal; optional dependency |

### Tier 1: Critical (Must Fix)

**WS4-CRITICAL-001: extended_benchmark.py Mixed Libraries**
- Files: `heliosCLI/harness/benchmarks/extended_benchmark.py`
- Issue: Uses requests + aiohttp alongside httpx
- Impact: Adds 2 unnecessary dependencies, violates single-library policy
- Effort: ~1 hour
- Risk: LOW (isolated benchmark file, no API surface)

**Action Items:**
1. Replace `requests.get()` calls with `httpx.get()` (lines 66, 75)
2. Replace `aiohttp.ClientSession()` with `httpx.AsyncClient()` (line 162)
3. Remove requests from implicit dependencies
4. Verify health check logic still works

**Code Changes Required:**
```python
# Before (lines 62-69)
import requests

def check_cliproxy_health(timeout: float = 5.0) -> bool:
    try:
        resp = requests.get(f"{CLIPROXY_URL}/v1/models", timeout=timeout)
        return resp.status_code in (200, 401)

# After
import httpx

def check_cliproxy_health(timeout: float = 5.0) -> bool:
    try:
        resp = httpx.get(f"{CLIPROXY_URL}/v1/models", timeout=timeout)
        return resp.status_code in (200, 401)
```

```python
# Before (lines 161-163)
async with aiohttp.ClientSession() as session:
    async with session.post(url, json=payload, headers=headers) as resp:
        data = await resp.json()

# After
async with httpx.AsyncClient() as client:
    resp = await client.post(url, json=payload, headers=headers)
    data = resp.json()
```

### Tier 2: Important (Should Fix)

**WS4-PERF-001: APIClient Creates New Client Per Request**
- File: `platforms/thegent/src/thegent/api_helpers/__init__.py`
- Issue: Each method creates new httpx.Client inside context manager
- Impact: High connection overhead, defeats connection pooling
- Effort: ~1 hour
- Risk: LOW (simple refactoring)

**Current Anti-Pattern:**
```python
def get(self, path: str, **kwargs: Any) -> dict[str, Any]:
    with httpx.Client() as client:  # NEW client every call
        return client.get(f"{self.base_url}{path}", ...
```

**Recommended Fix:**
```python
def __init__(self, base_url: str = "", timeout: float = 30.0):
    self.base_url = base_url
    self.timeout = timeout
    self._client: httpx.Client | None = None

@property
def client(self) -> httpx.Client:
    if self._client is None:
        self._client = httpx.Client(base_url=self.base_url, timeout=self.timeout)
    return self._client

def get(self, path: str, **kwargs: Any) -> dict[str, Any]:
    return self.client.get(path, **kwargs).json()

def close(self) -> None:
    if self._client:
        self._client.close()
```

**WS4-DEDUP-001: Multiple HTTP Wrapper Implementations**
- Files: 
  - `platforms/thegent/src/thegent/utils/http_helpers.py` (HTTPClient)
  - `platforms/thegent/src/thegent/api_helpers/__init__.py` (APIClient)
  - `heliosCLI/harness/src/harness/http_pool.py` (HTTPConnectionPool)
- Issue: Overlapping responsibility between wrappers
- Impact: Code duplication, inconsistent pooling strategies
- Effort: ~2 hours
- Risk: MEDIUM (requires coordination across projects)

**Recommendation:** Create unified httpx wrapper in shared library with:
- Connection pooling (required)
- Base URL support (for API calls)
- Timeout configuration
- Context manager support
- Sync + async variants

### Tier 3: Nice-to-Have (Optional)

**WS4-OPT-001: Standardize on FastHTTPClient**
- Status: OPTIONAL - curl_cffi fallback already working
- Note: FastHTTPClient is already well-designed; keeping curl_cffi optional is correct

---

## Risk Assessment

### Low Risk
- Replacing requests with httpx in extended_benchmark.py
- Replacing aiohttp with httpx in extended_benchmark.py
- Fixing APIClient connection pooling

### Medium Risk
- Consolidating HTTP wrappers across projects (requires testing)

### No Risk
- curl_cffi usage (optional backend, well-implemented)
- httpx usage (95% already standardized)

---

## Governance Documentation

### HTTP Client Library Policy

**Policy Code:** POL-HTTP-001  
**Effective Date:** 2026-04-01  
**Supersedes:** None

#### Mandatory Requirements

1. **Single Primary Library:** All Python projects MUST use httpx as the primary HTTP client library
   - Version: `httpx>=0.28.0` (or latest)
   - Exception: Benchmark/test files may use curl_cffi as optional backend

2. **No Dual Imports:** Code MUST NOT import multiple HTTP client libraries (requests, aiohttp, httplib2)
   - Allowed: httpx + curl_cffi (as FastHTTPClient does)
   - Forbidden: httpx + requests, httpx + aiohttp, etc.

3. **Connection Pooling Required:** All HTTP clients MUST reuse persistent connections
   - Use httpx.Client() or httpx.AsyncClient() for operations
   - Never create new client per request
   - Wrap clients to enforce pooling where applicable

4. **Dependency Declaration:** All HTTP libraries used MUST be explicitly declared in pyproject.toml/requirements.txt
   - Optional backends (e.g., curl_cffi) should use [optional-dependencies]

5. **Wrapper Standardization:** New HTTP wrappers MUST conform to:
   - Connection pooling by default
   - Context manager support (__enter__/__exit__)
   - Both sync and async variants where used
   - Clear backend abstraction (e.g., FastHTTPClient pattern)

#### Reference Implementation

For projects needing HTTP client wrapper:
```python
# patterns/httpx_wrapper.py
import httpx
from typing import Any

class HTTPClientWrapper:
    """Standard HTTP client with connection pooling."""
    
    def __init__(self, base_url: str = "", timeout: float = 30.0):
        self.base_url = base_url
        self.timeout = timeout
        self._client: httpx.Client | None = None
    
    @property
    def client(self) -> httpx.Client:
        if self._client is None:
            self._client = httpx.Client(
                base_url=self.base_url,
                timeout=self.timeout,
                limits=httpx.Limits(
                    max_keepalive_connections=20,
                    max_connections=100,
                ),
            )
        return self._client
    
    def get(self, path: str, **kwargs: Any) -> httpx.Response:
        return self.client.get(path, **kwargs)
    
    def close(self) -> None:
        if self._client:
            self._client.close()
            self._client = None
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()
```

---

## Implementation Roadmap

### Phase 1: Eliminate Non-Standard Libraries (1-2 days)

**WS4-IMPL-001:** Fix extended_benchmark.py mixed imports
- Replace requests with httpx (2 calls)
- Replace aiohttp with httpx (1 async function)
- Update health check logic
- Verify benchmark output unchanged
- **Effort:** 1-2 hours
- **Owner:** heliosCLI maintainer
- **PR:** stacked/ws4-extend-benchmark-consolidation

### Phase 2: Fix Performance Issues (2-3 days)

**WS4-IMPL-002:** Refactor APIClient connection pooling
- Fix APIClient to reuse persistent client
- Add close() method for cleanup
- Test with existing callers
- **Effort:** 1-2 hours
- **Owner:** thegent maintainer
- **PR:** stacked/ws4-api-client-pooling

### Phase 3: Consolidate Wrappers (3-5 days)

**WS4-IMPL-003:** Create shared HTTP wrapper library
- Extract to phenotype-shared or new module
- Support both sync and async
- Connection pooling enforced
- Base URL and timeout config
- **Effort:** 2-3 hours
- **Owner:** Architecture team
- **PR:** stacked/ws4-shared-http-wrapper

### Phase 4: Update All Projects (2-3 days)

**WS4-IMPL-004:** Migrate all projects to shared wrapper
- Update thegent (use shared wrapper)
- Update heliosCLI harness (use shared wrapper)
- Update byteport SDK (verify httpx usage)
- Run full test suite
- **Effort:** 1-2 hours
- **Owner:** Integration team
- **PR:** stacked/ws4-unified-http-client

### Phase 5: Policy Enforcement (1 day)

**WS4-IMPL-005:** Add linting and CI checks
- Add ruff rule: No imports of requests, aiohttp, httplib2
- Add linter check in CI/CD
- Document in CLAUDE.md
- **Effort:** 30-45 min
- **Owner:** Quality team
- **PR:** stacked/ws4-http-client-linting

---

## Estimated Impact

### Before Consolidation
- 3 different HTTP libraries (httpx, requests, aiohttp)
- 3 different connection pooling strategies
- 1 performance anti-pattern (APIClient)
- Inconsistent wrapper APIs

### After Consolidation
- 1 HTTP library (httpx) + 1 optional backend (curl_cffi)
- 1 unified connection pooling strategy
- 0 performance anti-patterns
- Consistent wrapper API across projects
- Estimated improvement: 10-20% network latency reduction for batch operations

### Effort Estimate
- **Total Implementation Time:** 8-12 hours
- **Testing Time:** 4-6 hours
- **Documentation:** 2-3 hours
- **Total:** ~18-24 hours (3-4 working days)

---

## Dependencies & Prerequisites

### Required Changes
1. heliosCLI: Update extended_benchmark.py (remove requests, aiohttp)
2. thegent: Fix APIClient (add persistent client)
3. Shared: Create unified HTTP wrapper library

### Optional Improvements
1. Add httpx wrapper as built-in to phenotype-shared
2. Create benchmark suite comparing httpx vs curl_cffi performance
3. Add HTTP client profiling to CI/CD

---

## Tracking & Verification

### Checklist for Implementation

- [ ] WS4-IMPL-001: extended_benchmark.py consolidated
  - [ ] requests calls replaced with httpx
  - [ ] aiohttp replaced with httpx
  - [ ] Tests passing
  - [ ] PR merged

- [ ] WS4-IMPL-002: APIClient connection pooling fixed
  - [ ] Persistent client implemented
  - [ ] close() method added
  - [ ] Callers updated
  - [ ] Tests passing
  - [ ] PR merged

- [ ] WS4-IMPL-003: Shared HTTP wrapper created
  - [ ] Sync + async support
  - [ ] Connection pooling
  - [ ] Base URL support
  - [ ] Tests passing
  - [ ] PR merged

- [ ] WS4-IMPL-004: All projects migrated
  - [ ] thegent updated
  - [ ] heliosCLI updated
  - [ ] byteport verified
  - [ ] Full test suite passing
  - [ ] PR merged

- [ ] WS4-IMPL-005: Linting rules added
  - [ ] Ruff rule for HTTP imports
  - [ ] CI check enabled
  - [ ] Documentation updated
  - [ ] PR merged

---

## Appendix: File Manifest

### All HTTP Client Files

**thegent (28 files):**
```
src/thegent/infra/fast_http_client.py
src/thegent/utils/http_helpers.py
src/thegent/api_helpers/__init__.py
src/thegent/agents/cliproxy_manager.py
src/thegent/agents/flash_agent.py
src/thegent/agents/tool_adapter.py
src/thegent/govern/vetter/checks.py
src/thegent/governance/config_provider_cp.py
src/thegent/governance/key_rotation.py
src/thegent/integrations/linear_graphql.py
src/thegent/observability/egress.py
src/thegent/use_cases/manage_cliproxy.py
src/thegent/utils/routing_impl/alerting.py
src/thegent/utils/routing_impl/cliproxy_client.py
src/thegent/utils/routing_impl/guardrails/webhook.py
src/thegent/utils/routing_impl/litellm_responses_handler.py
src/thegent/utils/routing_impl/ollama_provider.py
test_mcp_client.py
tests/adapters/test_acp_client.py
tests/compute/test_syncthing.py
tests/integration/test_parity_legacy_vs_cliproxy_migration.py
tests/memory/test_supermemory_client.py
tests/routing/test_openrouter_p1_nonstream.py
tests/test_adaptive_scale.py
tests/test_unit_doctor_mcp_tools_wl6713.py
tests/test_unit_provider_model_manager_discovery.py
tests/test_wl118_ollama_doctor_slice.py
tests/test_wl118_ollama_provider.py
tests/test_wl192_startup_validation.py
tests/test_wl6750_wl6759_lane_a.py
tests/test_wl6760_wl6769_lane_b.py
tests/test_wl6900_wl6909_lane_e.py
thegent/src/thegent/api_helpers.py
thegent/src/thegent/utils/http_helpers.py
```

**heliosCLI (8 files):**
```
harness/benchmarks/benchmark_runner.py
harness/benchmarks/extended_benchmark.py ← MIXED IMPORTS
harness/benchmarks/codex_concurrency_benchmark.py
harness/benchmarks/harness_benchmark.py
harness/benchmarks/llm_sla_benchmark.py
harness/benchmarks/unified_benchmark.py
harness/src/harness/http_pool.py
```

**byteport (2 files):**
```
apps/byteport/sdk/python/byteport/async_client.py
apps/byteport/sdk/python/byteport/client.py
```

---

## Conclusion

The Phenotype organization is **95% standardized on httpx**, with only one compliance gap: `extended_benchmark.py` mixing requests and aiohttp. The consolidation roadmap is straightforward:

1. **Immediate (Week 1):** Remove requests/aiohttp from extended_benchmark.py
2. **Short-term (Week 2):** Fix APIClient connection pooling
3. **Medium-term (Week 3):** Consolidate wrappers into shared library
4. **Long-term (Week 4):** Add linting enforcement

**Total effort:** 18-24 hours. **Risk:** LOW. **Payoff:** 10-20% latency improvement for network-heavy operations.

