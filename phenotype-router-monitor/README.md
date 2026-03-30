# phenotype-router-monitor

**Consolidated router and API monitoring infrastructure for the Phenotype ecosystem.**

A unified, production-grade routing and metrics collection framework consolidating domain-agnostic infrastructure from thegent-router, thegent-metrics, and related API monitoring tools.

## Purpose

This project consolidates three related infrastructure concerns:

1. **Pareto-Efficient Routing** — Task distribution, hysteresis-aware decision-making, and audit trail preservation
2. **High-Performance Metrics Collection** — Counter, gauge, histogram abstractions with concurrent-safe implementations
3. **API Metering & Usage Tracking** — Request classification, quota enforcement, and usage analytics

These tools are extracted from thegent (agent orchestration) and made reusable across Phenotype projects including AgilePlus, heliosCLI, and custom integrations.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│            phenotype-router-monitor Unified Layer                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │  Router Engine   │  │  Metrics Core    │  │ Usage Meter  │  │
│  │  ─────────────   │  │  ─────────────   │  │ ────────────  │  │
│  │ • Pareto routing │  │ • Counters      │  │ • Quota      │  │
│  │ • Hysteresis     │  │ • Gauges        │  │ • Rate limit │  │
│  │ • Audit chains   │  │ • Histograms    │  │ • Analytics  │  │
│  │ • Task dispatch  │  │ • JSON export   │  │ • Tracking   │  │
│  └──────────────────┘  └──────────────────┘  └──────────────┘  │
│           │                    │                    │           │
│           └────────────────────┴────────────────────┘           │
│                                │                                │
│                    ┌───────────▼──────────┐                    │
│                    │  Shared Contracts    │                    │
│                    │  & Traits (Port)     │                    │
│                    └──────────────────────┘                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Projects Being Consolidated

### 1. thegent-router (Primary Source)
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-router/`

- **Purpose:** Pareto-efficient routing engine for task distribution
- **Key Modules:**
  - `router.rs` — Core routing logic with priority enforcement
  - `audit.rs` — SHA-256 hash-chain audit trail
  - `hysteresis.rs` — Hysteresis-aware decision making
  - `executor.rs` — Task execution tracking
  - `risk.rs` — Risk assessment and failover
  - `orchestrator.rs` — Multi-router coordination
  - `python.rs` — Python FFI bindings (PyO3)

- **Key Traits/Types:**
  - `Router` — Main routing abstraction
  - `Executor` — Task execution interface
  - `AuditChain` — Immutable audit log with SHA-256 linking
  - `HysteresisState` — State machine for hysteresis logic

- **Dependencies:**
  - `serde/serde_json` for serialization
  - `sha2` for hash chains
  - `uuid` for identifiers
  - `pyo3` for Python bindings (optional)

- **Tests:** 50+ tests covering routing, hysteresis, Python FFI, phase 3 integration

### 2. thegent-metrics (Primary Source)
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-metrics/`

- **Purpose:** High-performance metrics collection library
- **Key Types:**
  - `Counter` — Arc<Mutex<u64>> with inc() method
  - `Gauge` — Arc<Mutex<f64>> with set() method
  - `Histogram` — Percentile calculations (p50, p99)
  - `MetricsRegistry` — Centralized metric storage

- **Key Features:**
  - Lock-free reads via Arc<Mutex<>> pattern
  - JSON serialization support
  - Histogram percentile calculations
  - Thread-safe concurrent access (DashMap backend)

- **Dependencies:**
  - `serde/serde_json`
  - `dashmap` for concurrent hash map

### 3. API Monitoring Tools (Secondary Sources)
**Location:** Various locations in AgilePlus, thegent

- **agileplus-telemetry** — Request/response tracking, usage analytics
- **thegent-utils/bin/monitor.rs** — CLI for real-time monitoring
- **Rate limiting & quota enforcement** — Scattered across API handlers

**Key Patterns to Consolidate:**
- Request classification (by user, endpoint, method)
- Usage quota tracking
- Rate limit enforcement
- API cost tracking (token-based or time-based)

## Crate Structure

```
phenotype-router-monitor/
├── crates/
│   ├── phenotype-router/          # Core routing engine (from thegent-router)
│   ├── phenotype-metrics/         # Metrics collection (from thegent-metrics)
│   ├── phenotype-meter/           # API metering & quota (NEW)
│   ├── phenotype-monitor-cli/     # CLI tooling (NEW)
│   └── phenotype-monitor-api/     # HTTP API for metrics (NEW)
├── tests/
│   ├── integration/
│   └── benchmarks/
├── docs/
│   ├── ARCHITECTURE.md
│   ├── API_METERING.md
│   ├── CONSOLIDATED_PROJECTS.md
│   └── MIGRATION.md
├── Cargo.toml                     # Workspace root
└── .agileplus/                    # Work tracking
```

## Dependencies Strategy

- **Inherit from phenotype-infrakit:** Use workspace-level deps (v0.2.0)
- **Versions:** Latest stable (serde 1.0+, tokio 1.41+, dashmap 6+)
- **Optional Features:** Python bindings (pyo3, feature-gated)
- **No Internal Deps:** Each crate stands alone initially; ports used for trait contracts

## Getting Started

### Build
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-router-monitor
cargo build --release --workspace
```

### Test
```bash
cargo test --workspace
cargo test --workspace -- --nocapture  # verbose output
```

### Benchmark
```bash
cargo bench --workspace
```

### Quality Checks
```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
cargo test --workspace
```

## Next Steps

1. Initialize git repository (preserving history from thegent)
2. Create crate stubs in `crates/` directory
3. Extract code from source projects with git subtree history
4. Implement shared contracts in `phenotype-monitor-contracts`
5. Create integration tests across all three domains
6. Document API metering patterns and migration strategy

See `docs/CONSOLIDATION_ROADMAP.md` for detailed execution plan.

## References

- **Architecture:** `docs/ARCHITECTURE.md`
- **Consolidated Projects:** `docs/CONSOLIDATED_PROJECTS.md`
- **Migration Strategy:** `docs/MIGRATION.md`
- **API Metering Spec:** `docs/API_METERING.md`
