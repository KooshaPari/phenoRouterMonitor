# AgilePlus Crate LOC Audit & Libification Plan

**Category:** AGILEPLUS | **Updated:** 2026-04-03

---

## 2026-04-03 - AgilePlus Crate Inventory

**Project:** [crates/]
**Category:** audit, libification
**Status:** in_progress
**Priority:** P0

### Crate Summary

| Crate | LOC | Status | Priority | Dependencies |
|-------|-----|--------|----------|--------------|
| agileplus-domain | ~3,500 | Active | P0 | tokio, serde |
| agileplus-graph | ~2,800 | Active | P0 | async-graphql |
| phenotype-event-sourcing | ~1,200 | Active | P0 | serde, tokio |
| phenotype-cache-adapter | ~1,100 | Active | P0 | lru, dashmap, moka |
| phenotype-policy-engine | ~800 | Active | P1 | regex |
| phenotype-state-machine | ~600 | Active | P1 | - |
| phenotype-contracts | ~2,500 | Active | P0 | async-trait |
| phenotype-config-core | ~1,200 | Active | P1 | toml, serde |
| phenotype-error-core | ~800 | Active | P1 | thiserror |
| phenotype-health | ~500 | Active | P2 | - |
| phenotype-async-traits | ~300 | Active | P2 | async-trait |

**Total Active LOC:** ~15,300

---

### 2026-04-03 - Detailed LOC Breakdown

#### 1. agileplus-domain (~3,500 LOC)

```
src/
├── lib.rs                    # 150 - exports
├── credentials/              # 800 - credential management
│   ├── mod.rs
│   ├── memory.rs            # 47 - duplicate in-memory
│   └── ...
├── services/                # 1,200 - domain services
├── models/                  # 800 - domain models
└── ports/                   # 500 - (DUPLICATE - see below)
```

**Issues:**
- Ports duplicated from phenotype-contracts (~500 LOC)
- InMemoryCredentialStore duplicate (~47 LOC)

**Savings:** ~550 LOC via deduplication

---

#### 2. agileplus-graph (~2,800 LOC)

```
src/
├── lib.rs                    # 80 - exports
├── query/                   # 600 - GraphQL query
├── store/                   # 900 - (DUPLICATES)
│   ├── mod.rs
│   ├── memory.rs           # 203 - duplicate in-memory
│   └── ...
├── schema/                  # 500 - GraphQL schema
└── health.rs               # 90 - (DUPLICATE - see below)
```

**Issues:**
- InMemoryGraphBackend duplicate (~203 LOC)
- GraphHealth duplicate (~90 LOC)

**Savings:** ~300 LOC via deduplication

---

#### 3. phenotype-event-sourcing (~1,200 LOC)

```
src/
├── lib.rs                   # 20 - exports
├── error.rs                # 46 - (DUPLICATE error)
├── hash.rs                 # 195 - content hash (DUPLICATE)
├── event.rs                # 98 - event envelope
├── snapshot.rs             # 92 - snapshot logic
├── store.rs                # 64 - event store trait
├── memory.rs               # 266 - in-memory (DUPLICATE)
└── retry.rs                # 62 - (DUPLICATE retry)
```

**Issues:**
- InMemoryEventStore duplicate (~266 LOC)
- ContentHash duplicate from evidence-ledger (~195 LOC)
- Retry logic duplicate (~62 LOC)

**Savings:** ~520 LOC via deduplication

---

#### 4. phenotype-cache-adapter (~1,100 LOC)

```
src/
├── lib.rs                   # 40 - exports
├── tier.rs                 # 400 - two-tier cache
├── l1.rs                   # 150 - L1 LRU
├── l2.rs                   # 150 - L2 dashmap
├── metrics.rs             # 80 - metrics hook
└── health.rs              # 28 - (DUPLICATE health)
```

**Issues:**
- CacheHealth duplicate (~28 LOC)

**Savings:** ~30 LOC

---

#### 5. phenotype-policy-engine (~800 LOC)

```
src/
├── lib.rs                   # 30 - exports
├── engine.rs               # 300 - policy engine
├── rules.rs                # 200 - rule definitions
├── context.rs             # 100 - evaluation context
├── loader.rs              # 100 - TOML loader
└── error.rs               # 70 - (DUPLICATE error)
```

**Issues:**
- PolicyError duplicate (~70 LOC)

**Savings:** ~70 LOC

---

#### 6. phenotype-contracts (~2,500 LOC)

```
src/
├── lib.rs                   # 50 - exports
├── ports/
│   ├── mod.rs
│   ├── inbound/            # 400 - inbound ports
│   └── outbound/           # 600 - outbound ports (SHOULD BE CANONICAL)
├── models/
│   ├── mod.rs
│   ├── entity.rs          # 150 - Entity trait
│   ├── value_object.rs    # 100 - ValueObject trait
│   └── aggregate.rs       # 200 - AggregateRoot trait
└── errors.rs              # 50 - ContractError
```

**Issues:**
- Should be canonical for all port definitions
- Currently duplicated in agileplus-domain/ports

**Savings:** ~500 LOC (by making canonical)

---

### 2026-04-03 - Cross-Crate Duplication Analysis

| Pattern | Occurrences | Total LOC | Library Candidate |
|---------|-------------|-----------|-------------------|
| In-Memory Stores | 4 | 476 | libs/test-stores |
| Health Enums | 6 | 180 | libs/health-core |
| Retry Logic | 4 | 186 | backoff crate |
| Error Types | 8 | 200 | libs/error-core |
| Repository Traits | 6 | 300 | libs/hexagonal-rs |
| Content Hash | 2 | 195 | libs/content-hash |
| Serialization | 10+ | 353 | libs/serde-adapters |

**Total Duplicate LOC:** ~1,890 LOC

---

### 2026-04-03 - Aggressive LOC Reduction Targets

| Target | Current | After | Reduction | Method |
|--------|---------|-------|-----------|--------|
| Remove all in-memory duplicates | 476 | 100 | **376** | libs/test-stores |
| Unify health checks | 180 | 60 | **120** | libs/health-core |
| Adopt backoff crate | 186 | 30 | **156** | backoff crate |
| Consolidate errors | 200 | 50 | **150** | libs/error-core |
| Fix port duplication | 500 | 0 | **500** | Use phenotype-contracts |
| Content hash lib | 195 | 30 | **165** | libs/content-hash |
| Serialization adapters | 353 | 80 | **273** | libs/serde-adapters |
| Remove nested duplicates | 500 | 0 | **500** | Delete nested |

**Total Aggressive Reduction:** **2,240 LOC**

---

### 2026-04-03 - Implementation Phases

#### Phase 1: Immediate (Week 1)
- [ ] Delete nested duplicate in agentapi-plusplus
- [ ] Migrate logrus → slog in cliproxyapi-plusplus
- [ ] Adopt backoff crate

#### Phase 2: High Priority (Week 2)
- [ ] Viper → Koanf migration
- [ ] Create phenotype-go-middleware
- [ ] Make phenotype-contracts canonical

#### Phase 3: Medium Priority (Week 3-4)
- [ ] Create libs/test-stores
- [ ] Create libs/health-core
- [ ] Create libs/error-core
- [ ] Create libs/content-hash
- [ ] Create libs/serde-adapters

#### Phase 4: Evaluation (Week 5)
- [ ] Evaluate casbin-rs
- [ ] Evaluate LLM frameworks
- [ ] Plan database modernization

---

### 2026-04-03 - External Dependencies Audit

| Current | Recommendation | Action |
|---------|----------------|--------|
| spf13/viper | knadh/koanf/v2 | Migrate |
| sirupsen/logrus | log/slog | Migrate |
| custom retry | backoff crate | Adopt |
| no circuit breaker | gobreaker/v2 | Add |
| custom middleware | go-chi/* | Adopt |
| rusqlite | sqlx | Evaluate |
| raw serde | serde + derive | Standardize |

---

### 2026-04-03 - Additional Opportunities Beyond Crates

Looking at wider codebase (~500K LOC across all repos):

| Area | Current | Opportunity | Savings |
|------|---------|-------------|---------|
| agentapi-plusplus | 65K | Viper → Koanf, middleware | ~600 LOC |
| cliproxyapi-plusplus | 394K | Logrus → Slog, config | ~900 LOC |
| thegent | ~50K | Custom loops → frameworks | ~400 LOC |
| heliosCLI | ~30K | LLM routing → LiteLLM | ~200 LOC |
| Python SDKs | ~20K | Validation → Pydantic | ~150 LOC |

**Additional Savings:** ~2,250 LOC

---

### Summary: Total LOC Reduction Opportunity

| Category | LOC Savings |
|----------|-------------|
| AgilePlus Crate Deduplication | 2,240 |
| Go Repos Modernization | 1,500 |
| Python/TS Standardization | 350 |
| **TOTAL** | **4,090 LOC** |

---

_Last updated: 2026-04-03_