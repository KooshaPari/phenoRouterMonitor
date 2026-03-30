# Phase 2 Execution Playbook — Implementation Guide

**Date**: 2026-03-30
**Purpose**: Detailed execution guide for Phase 2 teams
**Scope**: WP-007 through WP-016 (10 work packages, 10 weeks)
**Audience**: Phase 2A, Phase 2B, Phase 2C implementation teams

---

## Quick Start for Phase 2 Teams

### Before You Start

1. **Read** (60 min):
   - `/docs/reports/PHASE_1_COMPLETION_MASTER_SUMMARY_2026-03-30.md` (Executive Summary section only)
   - `/docs/reports/PHASE_1_DELIVERABLES_INDEX.md` (Work Package Artifacts section)

2. **Verify** (15 min):
   - Clone or pull all 3 forks: `forgecode-fork`, `phenotype-router-monitor`, `bifrost-routing`
   - Run test suites: `cargo test --all` in each directory
   - Confirm all 237 tests pass

3. **Understand** (30 min):
   - Review "Phase 2 Roadmap" in master summary
   - Identify your team's WPs
   - Review acceptance criteria for assigned WPs

### Team Assignment

| Team | Phase | WPs | Duration | Lead |
|------|-------|-----|----------|------|
| **Phase 2A** | Stability | 7-10 | 4 weeks | TBD |
| **Phase 2B** | Features | 11-14 | 4 weeks | TBD |
| **Phase 2C** | Release | 15-16 | 2 weeks | TBD |

---

## Phase 2A: Integration & Stability (4 weeks)

### Overview

Phase 2A focuses on ensuring Phase 1 crates work well together, hardening for production, and establishing CI/CD pipelines.

**Goals**:
- ✅ All cross-crate interactions tested
- ✅ Production-ready error handling and logging
- ✅ Automated CI/CD pipelines
- ✅ Complete documentation for operators

**Dependencies**: Phase 1 complete (✅ verified)

---

### WP-007: Cross-Crate Integration Testing

**Objective**: Test interactions between Phase 1 crates

**Acceptance Criteria**:
- [ ] Integration test suite for forgecode-fork ↔ bifrost-routing
- [ ] Integration test suite for phenotype-router-monitor ↔ bifrost-routing
- [ ] End-to-end scenario: agent discovery + LLM invocation
- [ ] All new tests passing (target: 15+ new tests)
- [ ] Documentation: integration guide

**Effort**: 15-20 tool calls, 2-3 days
**Start Date**: Week 1, Day 1

**Execution Steps**:

1. **Setup** (Day 1):
   - Create `tests/integration/` directory in each fork
   - Set up shared test utilities (fixtures, mocks)
   - Reference: `/forgecode-fork/tests/integration_test.rs` pattern

2. **forgecode-fork ↔ bifrost-routing** (Day 1-2):
   - Create agent configurations that invoke LLM providers
   - Test agent discovery loading LLM provider configs
   - Verify hot reload with provider changes
   - Expected: 5-8 new tests

   ```rust
   // Example integration test pattern
   #[tokio::test]
   async fn test_agent_llm_integration() {
       // 1. Discover agents with LLM config
       // 2. Instantiate LLM router
       // 3. Invoke agent with LLM call
       // 4. Verify response
   }
   ```

3. **phenotype-router-monitor ↔ bifrost-routing** (Day 2-3):
   - Create router that delegates to LLM providers
   - Test metrics collection on LLM calls
   - Verify cost tracking across providers
   - Expected: 5-8 new tests

   ```rust
   // Example: router selecting provider based on cost
   #[test]
   fn test_router_cost_aware_llm_selection() {
       // 1. Create router with cost-aware strategy
       // 2. Configure bifrost providers
       // 3. Route requests
       // 4. Verify lowest-cost provider selected
       // 5. Verify metrics recorded
   }
   ```

4. **End-to-End Scenario** (Day 3):
   - Full flow: agent discovery → config → LLM invocation → metrics
   - Test in production-like conditions
   - Expected: 3-5 new tests

5. **Documentation** (Day 3):
   - Create integration guide
   - Document test patterns
   - Add troubleshooting section

**Success Metrics**:
- [ ] 15+ new integration tests
- [ ] All tests passing
- [ ] >80% code coverage on integration paths
- [ ] Integration guide written (300+ lines)

**Testing Commands**:
```bash
# Run integration tests across all forks
cd /path/to/fork
cargo test --test integration

# Run with output
cargo test --test integration -- --nocapture --test-threads=1

# Coverage (if using tarpaulin)
cargo tarpaulin --test integration --out Html
```

**Common Pitfalls**:
- ❌ Not testing error cases (e.g., LLM provider timeout)
- ❌ Flaky tests due to timing assumptions
- ❌ Missing cleanup between test runs
- ✅ Use `#[tokio::test]` and proper async handling
- ✅ Implement test utilities for common setup

---

### WP-008: Production Hardening

**Objective**: Prepare crates for production deployment

**Acceptance Criteria**:
- [ ] Graceful shutdown implemented in all services
- [ ] Structured logging (tracing) integrated
- [ ] Health checks and liveness probes
- [ ] Performance benchmarks established
- [ ] Error recovery strategies documented
- [ ] All tests passing
- [ ] Documentation: operations guide

**Effort**: 20-25 tool calls, 3-4 days
**Start Date**: Week 2, Day 1 (parallel with WP-007)

**Execution Steps**:

1. **Graceful Shutdown** (Day 1):
   - Add shutdown signal handling (SIGTERM)
   - Implement request draining
   - Close resources cleanly
   - Focus: phenotype-router-monitor (has running service)

   ```rust
   // Example pattern
   #[tokio::main]
   async fn main() {
       let (tx, rx) = tokio::sync::broadcast::channel(1);

       // Spawn signal handler
       tokio::spawn(async move {
           tokio::signal::ctrl_c().await.ok();
           let _ = tx.send(());
       });

       // Main loop with shutdown check
       loop {
           tokio::select! {
               _ = rx.recv() => {
                   println!("Shutting down...");
                   break;
               }
               // Main work here
           }
       }
   }
   ```

2. **Structured Logging** (Day 1-2):
   - Add `tracing` crate
   - Replace println! with tracing macros
   - Implement tracing subscriber
   - Add request ID propagation
   - Guidance: Use `tracing-subscriber` with JSON output for logs

   ```rust
   // Example logging pattern
   use tracing::{info, warn, error, debug};

   info!("Starting router on port {}", port);
   warn!("Provider {} unavailable, retrying", provider_name);
   error!("Failed to parse config: {}", err);
   ```

3. **Health Checks** (Day 2):
   - Implement liveness probe (is service running?)
   - Implement readiness probe (is service ready for requests?)
   - Add startup probe (has service completed initialization?)
   - Focus: Kubernetes compatibility

   ```rust
   // Example health endpoints (Router API pattern)
   // GET /live   → 200 if running
   // GET /ready  → 200 if accepting connections
   // GET /startup → 200 if initialized
   ```

4. **Performance Benchmarks** (Day 2-3):
   - Establish baseline metrics:
     - Latency: p50, p95, p99
     - Throughput: requests/sec
     - Memory usage
     - CPU usage
   - Create benchmark suite
   - Document performance targets

   ```bash
   # Run benchmarks (if using criterion)
   cargo bench --all

   # Memory profiling
   valgrind --leak-check=full ./target/release/router-api

   # Load testing (using wrk or similar)
   wrk -t4 -c100 -d30s http://localhost:3030/metrics
   ```

5. **Error Recovery** (Day 3):
   - Document retry strategies
   - Implement circuit breaker patterns (if applicable)
   - Document fallback behaviors
   - Add example configurations

6. **Documentation** (Day 3-4):
   - Create operations guide (500+ lines)
   - Include deployment checklist
   - Add troubleshooting guide
   - Document performance tuning

**Success Metrics**:
- [ ] Graceful shutdown working
- [ ] All logs structured with tracing
- [ ] Health endpoints implemented
- [ ] Benchmarks established
- [ ] Operations guide written (500+ lines)
- [ ] All tests passing

**Operations Guide Should Cover**:
- Deployment requirements
- Configuration reference
- Monitoring and alerting
- Scaling guidelines
- Troubleshooting common issues
- Performance tuning

---

### WP-009: Documentation Expansion

**Objective**: Create comprehensive documentation for operators and developers

**Acceptance Criteria**:
- [ ] System architecture guide (with diagrams)
- [ ] Deployment guides (Docker, Kubernetes, standalone)
- [ ] API reference documentation
- [ ] Configuration reference (all parameters)
- [ ] Troubleshooting runbook
- [ ] Performance tuning guide
- [ ] Example use cases (3+)
- [ ] Migration guide from Phase 1 to Phase 2

**Effort**: 10-15 tool calls, 2-3 days
**Start Date**: Week 2, Day 1 (parallel with WP-007 & 008)

**Execution Steps**:

1. **System Architecture Guide** (Day 1):
   - Create `/docs/architecture/SYSTEM_OVERVIEW.md`
   - Include Mermaid diagrams:
     - Component interaction diagram
     - Data flow diagram
     - Request flow diagram
   - Explain design patterns
   - Expected: 400-500 lines

2. **Deployment Guides** (Day 1-2):
   - Create `/docs/deployment/DOCKER.md` (standalone + docker-compose)
   - Create `/docs/deployment/KUBERNETES.md` (manifests, best practices)
   - Create `/docs/deployment/STANDALONE.md` (binary distribution)
   - Each: 250-350 lines
   - Include environment variables reference

3. **API Reference** (Day 2):
   - Create `/docs/api/REST_API_REFERENCE.md`
   - Document all endpoints with:
     - Method, path, parameters
     - Request/response examples
     - Status codes and errors
     - Curl examples
   - Expected: 300-400 lines

4. **Configuration Reference** (Day 2):
   - Create `/docs/configuration/CONFIGURATION_REFERENCE.md`
   - Document all config parameters:
     - Type, default, required
     - Examples
     - Environment variable mappings
   - Expected: 250-350 lines

5. **Troubleshooting Runbook** (Day 3):
   - Create `/docs/troubleshooting/RUNBOOK.md`
   - Common issues and solutions:
     - Service won't start
     - High latency
     - Memory leaks
     - Provider unavailable
   - Diagnostic procedures
   - Expected: 300-400 lines

6. **Performance Tuning Guide** (Day 3):
   - Create `/docs/operations/PERFORMANCE_TUNING.md`
   - Explain:
     - Load balancing strategies and tradeoffs
     - Caching strategies
     - Connection pooling
     - Metric collection impact
   - Includes benchmarks from WP-008
   - Expected: 250-350 lines

7. **Example Use Cases** (Day 3):
   - Create `/docs/examples/USE_CASES.md`
   - 3+ detailed scenarios:
     - Cost optimization workflow
     - High-availability setup
     - Multi-tenant configuration
   - Include code examples and configurations
   - Expected: 400-500 lines

**Success Metrics**:
- [ ] All guides written (2,500+ total lines)
- [ ] All examples runnable
- [ ] All Mermaid diagrams rendering correctly
- [ ] Zero broken links
- [ ] Vale lint passing (prose quality)

**Vale Configuration**:
```yaml
# .vale.ini
[core]
StylesPath = .vale/styles
Vocab = [Default, Custom]

[[Jinja]]
[[Markdown]]
# Rules to enable
```

---

### WP-010: CI/CD Pipeline Setup

**Objective**: Automate testing, building, and releasing

**Acceptance Criteria**:
- [ ] GitHub Actions workflows for all 3 forks
- [ ] Automated testing on every commit (Unit + Integration)
- [ ] Automated linting (rustfmt, clippy)
- [ ] Security scanning (cargo-audit)
- [ ] Automated release process (git-cliff + cargo-release)
- [ ] Publish crates to crates.io
- [ ] Documentation build and deployment
- [ ] All workflows passing

**Effort**: 10-15 tool calls, 2 days
**Start Date**: Week 3, Day 1

**Execution Steps**:

1. **Test Workflow** (Day 1):
   - Create `.github/workflows/test.yml`
   - Trigger: push to main, PR
   - Jobs:
     - `test-unit`: cargo test --all
     - `test-integration`: cargo test --test integration --all
     - `clippy`: cargo clippy --all -- -D warnings
     - `format`: cargo fmt --check
   - Runs on: Linux (ubuntu-latest)

   ```yaml
   name: Tests
   on: [push, pull_request]
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: dtolnay/rust-toolchain@stable
         - run: cargo test --all
         - run: cargo clippy --all -- -D warnings
   ```

2. **Security Workflow** (Day 1):
   - Create `.github/workflows/security.yml`
   - Jobs:
     - `audit`: cargo-audit check
     - `deny`: cargo-deny check
   - Fail on vulnerabilities

3. **Release Workflow** (Day 1-2):
   - Create `.github/workflows/release.yml`
   - Trigger: Manual (workflow_dispatch) or tag push
   - Jobs:
     - `build-release`: cargo build --release
     - `publish-crates`: cargo publish (all crates)
     - `create-release`: Create GitHub release
   - Reference: git-cliff for changelog generation

   ```bash
   # git-cliff configuration
   git cliff --init
   # Creates cliff.toml in repository root
   ```

4. **Documentation Workflow** (Day 2):
   - Create `.github/workflows/docs.yml`
   - Build docs with: cargo doc --no-deps
   - Deploy to GitHub Pages
   - Trigger: push to main

5. **Lint & Format Workflow** (Day 2):
   - Create `.github/workflows/lint.yml`
   - Tools:
     - rustfmt (formatting)
     - clippy (linting)
     - cargo-audit (security)
     - vale (prose)

**Workflow Template**:
```yaml
name: CI/CD Pipeline
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --verbose
      - run: cargo test --doc

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo clippy --all -- -D warnings

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
```

**Success Metrics**:
- [ ] All workflows created
- [ ] All workflows passing
- [ ] Test workflow running on every commit
- [ ] Security scanning active
- [ ] Release process automated
- [ ] Documentation auto-deploying

---

## Phase 2B: Feature Expansion (4 weeks)

### Overview

Phase 2B adds advanced features to Phase 1 crates, enabling more sophisticated use cases.

**Goals**:
- ✅ Streaming support for LLM providers
- ✅ Advanced monitoring and observability
- ✅ Multi-provider orchestration
- ✅ Caching and performance optimization

**Dependencies**: Phase 2A complete (✅ to be verified)

---

### WP-011: Streaming Support

**Objective**: Add streaming response support to bifrost-routing

**Acceptance Criteria**:
- [ ] Streaming trait defined
- [ ] Streaming implemented for all 4 providers
- [ ] Stream handling in router
- [ ] Integration tests for streaming (5+ tests)
- [ ] Documentation: streaming guide
- [ ] Example: streaming client

**Effort**: 15-20 tool calls, 2-3 days
**Start Date**: Week 5, Day 1

**Key Implementation Points**:
- Use `futures::Stream` for async streaming
- Implement provider-specific streaming:
  - OpenAI: Server-Sent Events (SSE)
  - Anthropic: Streaming protocol
  - OpenRouter: Passthrough streaming
  - Together: Event stream format
- Handle backpressure and cancellation

---

### WP-012: Advanced Monitoring

**Objective**: Add distributed tracing and custom metrics

**Acceptance Criteria**:
- [ ] Distributed tracing integration (OpenTelemetry)
- [ ] Custom metrics exporters (beyond Prometheus)
- [ ] Request tracing across services
- [ ] Dashboard/visualization examples
- [ ] Documentation: observability guide

**Effort**: 15-20 tool calls, 2-3 days
**Start Date**: Week 5, Day 1

**Key Implementation Points**:
- OpenTelemetry Rust SDK
- Jaeger exporter for distributed tracing
- Custom metrics with OTLP protocol
- Grafana dashboard examples

---

### WP-013: Multi-Provider Orchestration

**Objective**: Advanced agent composition and workflow support

**Acceptance Criteria**:
- [ ] Agent composition patterns
- [ ] Workflow/pipeline DSL
- [ ] Conditional routing based on input
- [ ] Agent chaining
- [ ] Integration tests (8+ tests)
- [ ] Documentation: orchestration guide

**Effort**: 20-25 tool calls, 3-4 days
**Start Date**: Week 6, Day 1

**Example Use Case**:
```yaml
# Orchestration DSL example
workflow:
  name: content-analyzer
  steps:
    - agent: classifier
      input: "{{ input }}"
      output: category
    - step: router
      condition: "category == 'technical'"
      then: technical-analyzer
      else: general-analyzer
    - agent: "{{ chosen_analyzer }}"
      input: "{{ input }}"
```

---

### WP-014: Caching & Performance

**Objective**: Add intelligent caching and optimize throughput

**Acceptance Criteria**:
- [ ] Response caching with TTL
- [ ] Request deduplication
- [ ] Memory optimization
- [ ] Performance benchmarks (30%+ improvement target)
- [ ] Cache invalidation strategies
- [ ] Documentation: caching guide

**Effort**: 15-20 tool calls, 2-3 days
**Start Date**: Week 7, Day 1

**Key Implementation Points**:
- Two-tier cache (in-memory + distributed)
- Cache invalidation strategies
- Semantic caching (similar requests)
- Benchmark suite showing before/after

---

## Phase 2C: Production Release (2 weeks)

### Overview

Phase 2C consolidates all Phase 1 and Phase 2 work into a production-ready v0.1.0 release.

**Goals**:
- ✅ Tag and publish v0.1.0
- ✅ Create comprehensive release notes
- ✅ Publish all crates to crates.io
- ✅ Create installation and migration guides

**Dependencies**: Phase 2A & 2B complete

---

### WP-015: v0.1.0 Release

**Objective**: Prepare and publish production release

**Acceptance Criteria**:
- [ ] All code merged to main
- [ ] Version bumped to 0.1.0
- [ ] CHANGELOG.md updated
- [ ] Release notes created (500+ lines)
- [ ] All 3 crates published to crates.io
- [ ] Git tags created
- [ ] GitHub release created
- [ ] Installation guide updated

**Effort**: 10-15 tool calls, 2 days
**Start Date**: Week 9, Day 1

**Release Process**:

1. **Version Management**:
   - Update all Cargo.toml version fields to 0.1.0
   - Update CHANGELOG.md with git-cliff
   - Create annotated git tag: `v0.1.0`

2. **Publishing**:
   - Publish to crates.io:
     ```bash
     cd forgecode-fork && cargo publish
     cd phenotype-router-monitor && cargo publish -p phenotype-router-core
     # (repeat for all crates)
     ```

3. **Release Notes** (500+ lines):
   - What's new
   - Breaking changes
   - Migration guide from Phase 1
   - Known limitations
   - Performance metrics
   - Contributors
   - Roadmap for v0.2.0

4. **GitHub Release**:
   - Create release on GitHub
   - Attach binary artifacts (if applicable)
   - Link to release notes
   - Link to documentation

---

### WP-016: Migration Guide

**Objective**: Help users upgrade to v0.1.0

**Acceptance Criteria**:
- [ ] Migration guide created (300+ lines)
- [ ] Upgrade path documented
- [ ] Compatibility matrix created
- [ ] Breaking changes listed
- [ ] API changes documented
- [ ] Examples for common migrations

**Effort**: 8-10 tool calls, 1-2 days
**Start Date**: Week 9, Day 1

**Migration Guide Contents**:
- Upgrade from v0.0.x to v0.1.0
- API breaking changes
- Configuration changes
- Dependency updates
- Performance improvements
- New features to adopt
- Rollback procedures

---

## Success Metrics for Phase 2

### Phase 2A Metrics

| Metric | Target | Verification |
|--------|--------|---------------|
| Integration tests | 15+ passing | cargo test --test integration |
| Production hardening | Complete | Deployment checklist passed |
| Documentation | 2,500+ lines | Line count, vale lint |
| CI/CD workflows | 5+ workflows | All green on GitHub |
| Code quality | 0 warnings | cargo clippy + cargo fmt |

### Phase 2B Metrics

| Metric | Target | Verification |
|--------|--------|---------------|
| Streaming support | All 4 providers | Provider tests passing |
| Tracing integration | Full coverage | OpenTelemetry data flowing |
| Orchestration | DSL + runtime | Workflow tests passing |
| Performance | 30%+ improvement | Benchmark comparison |

### Phase 2C Metrics

| Metric | Target | Verification |
|--------|--------|---------------|
| Release readiness | 100% | Release checklist |
| Crates.io publish | 6 crates | cargo search phenotype-* |
| Documentation | Complete | All links working, vale passing |
| User adoption | 5+ downloads | crates.io statistics |

---

## Risk Mitigation Strategies

### For Each Phase

**Phase 2A Risks**:
- **Risk**: Integration tests fail due to timing
  - **Mitigation**: Use proper async/await, add retries with backoff
- **Risk**: CI/CD setup is complex
  - **Mitigation**: Start with simple workflows, add incrementally
- **Risk**: Documentation gets out of sync
  - **Mitigation**: Review docs in every PR, add lint checks

**Phase 2B Risks**:
- **Risk**: Streaming implementation takes longer than expected
  - **Mitigation**: Implement basic streaming first, add features later
- **Risk**: Performance optimization breaks existing functionality
  - **Mitigation**: Comprehensive benchmarking and regression tests
- **Risk**: Orchestration DSL is too complex
  - **Mitigation**: Start with basic patterns, extend gradually

**Phase 2C Risks**:
- **Risk**: Publishing to crates.io fails
  - **Mitigation**: Dry-run publish before real attempt
- **Risk**: Breaking changes missed
  - **Mitigation**: Comprehensive changelog review
- **Risk**: Users can't upgrade
  - **Mitigation**: Detailed migration guide with examples

---

## Timeline Overview

```
Week 1-2: Phase 2A (WP-007, 008, 009, 010)
├── Integration Testing (WP-007)
├── Production Hardening (WP-008) [parallel]
├── Documentation Expansion (WP-009) [parallel]
└── CI/CD Setup (WP-010) [parallel]

Week 3-4: Phase 2A Continuation (wrap-up)
└── Final QA and integration testing

Week 5-8: Phase 2B (WP-011, 012, 013, 014)
├── Streaming Support (WP-011)
├── Advanced Monitoring (WP-012) [parallel]
├── Orchestration (WP-013) [parallel]
└── Caching/Performance (WP-014) [parallel]

Week 9-10: Phase 2C (WP-015, 016)
├── v0.1.0 Release (WP-015)
└── Migration Guide (WP-016) [parallel]

Target Release: Mid-May 2026 (v0.1.0)
```

---

## Team Communication Template

### Daily Standup

```markdown
## Phase 2A Standup — 2026-04-01

### WP-007 (Integration Testing)
- **Status**: In Progress
- **Completed**: Forgecode ↔ Bifrost integration tests (5 tests)
- **Today**: Phenotype-Router ↔ Bifrost integration tests
- **Blockers**: None
- **Next Review**: 2026-04-02

### WP-008 (Production Hardening)
- **Status**: In Progress
- **Completed**: Graceful shutdown, logging
- **Today**: Health checks implementation
- **Blockers**: None

### WP-009 (Documentation)
- **Status**: Planned (starting tomorrow)
- **Owner**: @docs-team
```

### Weekly Progress Report

```markdown
## Phase 2A Weekly Report — Week 1

### Work Packages Status
- WP-007 Integration Testing: 40% (3/15 tests)
- WP-008 Production Hardening: 35%
- WP-009 Documentation Expansion: 0% (starts Day 3)
- WP-010 CI/CD Setup: 0% (starts Day 4)

### Key Achievements
- Forgecode ↔ Bifrost integration test suite created
- Graceful shutdown implemented in phenotype-router-api
- Health check endpoints designed

### Risks & Blockers
- None currently

### Next Week Plan
- Complete remaining integration tests (WP-007)
- Finish production hardening (WP-008)
- Begin documentation (WP-009)
- Start CI/CD setup (WP-010)
```

---

## Appendix: Useful Commands

### Testing

```bash
# All tests
cargo test --workspace

# Specific test
cargo test test_name -- --nocapture

# Integration tests only
cargo test --test integration

# With coverage
cargo tarpaulin --workspace --out Html

# Benchmarks
cargo bench --all
```

### Code Quality

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy --all -- -D warnings

# Security audit
cargo audit

# Dependency check
cargo deny check
```

### Documentation

```bash
# Build docs
cargo doc --no-deps --open

# Check links
cargo-doc-check

# Lint prose
vale docs/
```

### Release

```bash
# Dry run publish
cargo publish --dry-run

# Actual publish
cargo publish -p crate-name

# Create changelog
git cliff > CHANGELOG.md

# Tag release
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## Conclusion

Phase 2 is structured to be executed in parallel where possible, with clear dependencies and success metrics. Each team should:

1. Review this playbook and their assigned WPs
2. Identify blockers and dependencies
3. Establish daily standups
4. Maintain clear communication
5. Execute systematically with quality gates

**Target**: v0.1.0 production release by mid-May 2026

---

**Document Version**: 1.0
**Created**: 2026-03-30
**Last Updated**: 2026-03-30
**Status**: Ready for Phase 2 execution
