# Expanded Libification & LOC Reduction Tasks (2x Target)

**Category:** LIBIFICATION | **Updated:** 2026-04-03

**Target:** ~7,000+ LOC Reduction (2x previous target)

---

## Phase 1: P0 - Immediate Actions (~2,000 LOC)

### Task 1.1: Remove Nested Duplicate in agentapi-plusplus
- **Location:** `agentapi-plusplus/agentapi-plusplus/`
- **Issue:** Complete nested copy (~23K)
- **Savings:** ~500 LOC
- **Status:** ⬜ Pending

### Task 1.2: Logrus → Slog Migration in cliproxyapi-plusplus
- **Location:** `cliproxyapi-plusplus/`
- **Issue:** Deprecated logrus (security issues)
- **Savings:** ~400 LOC
- **Status:** ⬜ Pending

### Task 1.3: Adopt backoff Crate for Retry
- **Location:** 4 Rust crates
- **Issue:** 4 separate retry implementations
- **Savings:** ~163 LOC
- **Status:** ⬜ Pending

### Task 1.4: Remove phenotype-event-sourcing Nested Duplicate
- **Location:** `crates/phenotype-event-sourcing/phenotype-event-sourcing/`
- **Issue:** Nested copy of source
- **Savings:** ~200 LOC

### Task 1.5: Clean Up Empty/Nested Directories
- **Issue:** Multiple empty or nested duplicates across repos
- **Savings:** ~150 LOC
- **Targets:**
  - [ ] phenotype-infrakit/phenotype-infrakit/
  - [ ] AgilePlus/.agileplus/ (if empty)
  - [ ] Various .git folders cleaned

---

## Phase 2: P1 - High Priority Actions (~2,000 LOC)

### Task 2.1: Viper → Koanf Migration
- **Locations:** agentapi-plusplus, cliproxyapi-plusplus
- **Issue:** Deprecated Viper
- **Savings:** ~470 LOC

### Task 2.2: HTTP Resilience (retryablehttp + gobreaker)
- **Issue:** No circuit breaker, custom retry
- **Savings:** ~270 LOC

### Task 2.3: Middleware Consolidation
- **Issue:** 2+ duplicate middleware implementations
- **Savings:** ~305 LOC

### Task 2.4: Health Check Unification
- **Issue:** 6 different health enums
- **Savings:** ~90 LOC

### Task 2.5: Make phenotype-contracts Canonical
- **Issue:** Ports duplicated in agileplus-domain
- **Savings:** ~500 LOC

---

## Phase 3: P2 - Medium Priority (~1,500 LOC)

### Task 3.1: Repository Trait Consolidation
- **Issue:** 6+ duplicate Store/Repository traits
- **Savings:** ~200 LOC

### Task 3.2: In-Memory Store Extraction
- **Issue:** 4 duplicate Arc<Mutex<HashMap>> implementations
- **Savings:** ~320 LOC

### Task 3.3: Serialization Adapter Library
- **Issue:** Duplicate serde boilerplate
- **Savings:** ~273 LOC

### Task 3.4: Error Core Library
- **Issue:** 8+ custom error types
- **Savings:** ~150 LOC

### Task 3.5: Content Hash Library
- **Issue:** SHA-256 chain duplicated in event-sourcing, evidence-ledger
- **Savings:** ~165 LOC

### Task 3.6: Test Fixture Consolidation
- **Issue:** Duplicate test utilities
- **Savings:** ~250 LOC

---

## Phase 4: P2 - Extended Opportunities (~1,500 LOC)

### Task 4.1: Evaluate casbin-rs for Policy Engine
- **Current:** Custom policy-engine (~500 LOC)
- **Alternative:** Wrap casbin-rs
- **Potential Savings:** ~400 LOC

### Task 4.2: Evaluate LLM Orchestration Frameworks
- **Current:** Custom agent loops
- **Alternatives:** Mastra, rig-core, CrewAI
- **Potential Savings:** ~200 LOC

### Task 4.3: Database Modernization
- **Current:** rusqlite (sync)
- **Target:** sqlx (async)
- **Potential Savings:** ~400 LOC

### Task 4.4: Python SDK Consolidation
- **Issue:** Multiple Python SDKs with duplicated patterns
- **Savings:** ~150 LOC

### Task 4.5: TypeScript Client Standardization
- **Issue:** Multiple TS clients with different patterns
- **Savings:** ~200 LOC

---

## Phase 5: P3 - Deep Audit Opportunities (~500 LOC)

### Task 5.1: Dead Code Removal
- **Issue:** Unused functions, imports, files across repos
- **Savings:** ~150 LOC

### Task 5.2: Import Consolidation
- **Issue:** Multiple paths to same module
- **Savings:** ~50 LOC

### Task 5.3: Constants/Config Duplication
- **Issue:** Duplicate constants across crates
- **Savings:** ~100 LOC

### Task 5.4: Documentation Boilerplate
- **Issue:** Redundant doc comments
- **Savings:** ~50 LOC

### Task 5.5: Comment-Only Files
- **Issue:** Files with only comments or empty
- **Savings:** ~50 LOC

---

## 2026-04-03 - Deep Audit: Additional Patterns Found

### Additional 1: Message Handler Consolidation
- **Pattern:** 20+ similar message type dispatches
- **Before:** Match statements duplicated
- **After:** Registry pattern
- **Savings:** ~100 LOC

### Additional 2: Builder Pattern Consolidation
- **Pattern:** 12+ builder implementations
- **Before:** Manual implementation each
- **After:** Macro-based or shared builder
- **Savings:** ~200 LOC

### Additional 3: Query/Filter Duplication
- **Pattern:** Similar query builders in multiple crates
- **Savings:** ~150 LOC

### Additional 4: Config Loading Patterns
- **Pattern:** Various config loading methods
- **Savings:** ~100 LOC

### Additional 5: Logging Initialization
- **Pattern:** Duplicate logger setup
- **Savings:** ~80 LOC

### Additional 6: Test Utilities
- **Pattern:** Duplicate assert helpers, mock utilities
- **Savings:** ~120 LOC

---

## Summary: Total LOC Reduction

| Phase | Tasks | Target LOC |
|-------|-------|------------|
| P0 | 5 tasks | ~1,563 |
| P1 | 5 tasks | ~1,635 |
| P2 | 6 tasks | ~1,308 |
| P3 | 5 tasks | ~500 |
| Extended | 5 tasks | ~1,100 |
| Deep Audit Extras | 6 tasks | ~750 |
| **TOTAL** | **32 tasks** | **~6,856 LOC** |

---

## Progress Tracking

- [ ] Phase 1: 0/5 complete
- [ ] Phase 2: 0/5 complete
- [ ] Phase 3: 0/6 complete
- [ ] Phase 4: 0/5 complete
- [ ] Phase 5: 0/5 complete
- [ ] Deep Audit: 0/6 complete

---

_Last updated: 2026-04-03_