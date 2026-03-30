# AgilePlus Specification Outlines — Three New Forks

**Date**: 2026-03-30  
**Status**: Prepared (Not Yet Implemented)  
**Ready for**: AgilePlus spec entry creation via CLI

---

## Overview

Three comprehensive AgilePlus specification outlines have been prepared for new forked projects in the Phenotype ecosystem. Each fork is a complete, self-contained epic with full detail for implementation planning and agent activation.

---

## The Three Forks

### 1. forgecode-fork: Custom Providers + Subagents

**Epic ID**: `eco-fork-001`  
**Timeline**: Weeks 1-6  
**Effort**: 18-24 parallel subagents, 72-99 tool calls, ~2,170 LOC  
**Summary**: Extends AgilePlus with pluggable provider abstraction and subagent spawning.

**Vision**: Transform AgilePlus into a composable agent orchestration platform where providers (Claude, Grok, local models) and agent types can be independently registered, versioned, and routed.

**Key Deliverables**:
- forgecode-providers crate (Provider trait, registry, 3 implementations)
- agileplus-provider-cli integration (provider + spawn-agent subcommands)
- forgecode-agent-dispatch gRPC service
- Provider audit trail with hash-chain verification

**Work Packages** (6 total):
- WP01: Provider Trait & Registry Foundation
- WP02: Claude Provider Implementation
- WP03: Local Provider + Ollama Integration
- WP04: Provider-Aware Subagent Spawning
- WP05: Capability Discovery & Routing
- WP06: Provider Performance Metrics & Feedback Loop

---

### 2. phenotype-router-monitor: Consolidated API Monitoring & Routing

**Epic ID**: `eco-fork-002`  
**Timeline**: Weeks 1-4  
**Effort**: 15-18 parallel subagents, 54-66 tool calls, ~1,850 LOC  
**Summary**: Unified API gateway with observability, failure recovery, and load balancing.

**Vision**: Single entry point for all Phenotype services with transparent observability, failure recovery, and gradual rollout support.

**Key Deliverables**:
- phenotype-router-core (routing engine, path-based + load balancing)
- phenotype-router-health (health checker aggregation)
- phenotype-router-limiter (rate limiting + circuit breaker)
- phenotype-router-metrics (Prometheus exporter)
- phenotype-router-cli integration (route management + status)

**Work Packages** (5 total):
- WP01: Router Core & Routing Engine
- WP02: Health Checking & Aggregation
- WP03: Rate Limiting & Circuit Breaker
- WP04: Metrics Collection & Prometheus Export
- WP05: Router CLI & Status Dashboard

---

### 3. bifrost-routing: LLM Routing Infrastructure

**Epic ID**: `eco-fork-003`  
**Timeline**: Weeks 1-6  
**Effort**: 16-20 parallel subagents, 72-90 tool calls, ~2,010 LOC  
**Summary**: Specialized router for LLM inference with intelligent model selection and cost optimization.

**Vision**: Transform LLM selection from manual ("use Opus for everything") into data-driven ("route to Haiku/Sonnet/Opus based on cost/latency/quality").

**Key Deliverables**:
- bifrost-routing-core (routing engine, request classifier, SLA enforcement)
- bifrost-routing-models (LLM model definitions, cost/latency metadata)
- bifrost-routing-analytics (cost tracking, A/B metrics, performance analysis)
- bifrost-routing-cli integration (routing management, SLA tuning)

**Work Packages** (6 total):
- WP01: Request Classifier & Workload Inference
- WP02: LLM Model Registry & Capability Metadata
- WP03: Token-Aware & Workload-Based Model Selection
- WP04: SLA Enforcement & Latency Timeout
- WP05: Cost Tracking & Budget Enforcement
- WP06: A/B Testing & Shadow Routing

---

## Cross-Fork Dependencies

```
forgecode-fork (eco-fork-001)
  ↓ provides Provider abstraction
bifrost-routing (eco-fork-003)
  ↓ routes to providers
phenotype-router-monitor (eco-fork-002)
  ↓ monitors bifrost + other services
AgilePlus integrates all three
```

### Execution Phases

**Phase 1A (Weeks 1-2)**: Foundational work across all forks
- forgecode-fork: WP01 (Provider trait)
- phenotype-router-monitor: WP01 (Routing engine)
- bifrost-routing: WP01-WP02 (Classification + model registry)

**Phase 1B (Weeks 2-3)**: Core implementations
- forgecode-fork: WP02-WP04 (Provider impls, spawning, discovery)
- phenotype-router-monitor: WP02-WP04 (Health, rate limiting, metrics)
- bifrost-routing: WP03-WP05 (Selection, SLA, cost tracking)

**Phase 2 (Weeks 4-6)**: Advanced features + integration
- forgecode-fork: WP05-WP06 (Routing + feedback loop)
- phenotype-router-monitor: WP05 (CLI + dashboard)
- bifrost-routing: WP06 (A/B testing + shadow routing)

---

## Effort Summary

| Metric | Total |
|--------|-------|
| Parallel Subagents | 12-15 (Phase 1A), 18-20 (Phase 1B), 15-18 (Phase 2) |
| Tool Calls | 198-255 |
| Lines of Code | ~6,030 |
| Wall-Clock Time | 4-6 weeks |
| Quality Coverage | ≥85% test coverage per fork |

---

## Deliverables

### Full Specification Outline

**File**: `agileplus_spec_outlines.md`  
**Size**: 786 lines, ~33 KB  
**Format**: Markdown with tables, code blocks, DAG diagrams

**Contents**:
- Complete epic overview for each fork (vision, goals, deliverables)
- 5-6 work packages (WP01-WP06) with detailed subtasks
- 8-12 functional requirements (FRs) per epic
- Effort estimates in tool calls and parallel subagents
- Acceptance criteria for each WP and task
- Integration/dependency graph

### Summary Document

**File**: `SPEC_OUTLINES_SUMMARY.txt`  
**Size**: ~7 KB  
**Format**: Plain text with structured sections

**Contents**:
- Executive summary
- Per-fork overview (vision, key deliverables, WPs)
- Cross-fork dependencies
- Effort summary
- Next steps
- Design patterns & constraints
- Validation criteria

---

## Quality Gates

All spec outlines satisfy:
- Epic overview complete (vision, goals, deliverables)
- All WPs have dependencies (no cycles)
- All WPs have subtasks (T001-T048+)
- All WPs have acceptance criteria (specific, measurable)
- All FRs traced to ≥1 WP
- Effort estimates realistic (12-16 tool calls per WP)
- No WP >500 LOC (chunked appropriately)
- Test coverage built-in (integration tests in every WP)
- Non-goals explicit (scope control)
- Dependencies clear (execution order unambiguous)

---

## Next Steps (When User Approves)

1. **Create AgilePlus specs** for each fork
   ```bash
   agileplus specify --title "Custom Providers & Subagents" --epic eco-fork-001
   agileplus specify --title "Consolidated API Monitoring" --epic eco-fork-002
   agileplus specify --title "LLM Routing Infrastructure" --epic eco-fork-003
   ```

2. **Create work package directories** with task templates
   - Create WP01-WP06 directories for each fork
   - Pre-populate subtask templates (T001-T048)

3. **Activate subagents** for Phase 1A
   - 12-15 parallel agents for Week 1-2 work
   - Assign agents to forks: 4-5 per fork

4. **Track progress** in AgilePlus dashboard
   - Monitor burn-down charts
   - Verify test coverage as commits land
   - Update FR traceability

5. **Weekly rollup reports** for user review
   - Metrics: LOC added, tests passing, coverage %
   - Risks/blockers
   - Next phase readiness

---

## Files

- `agileplus_spec_outlines.md` — Full specification outlines (786 lines)
- `SPEC_OUTLINES_SUMMARY.txt` — Summary document (plain text)
- `README_SPEC_OUTLINES.md` — This file (overview & navigation)

---

## Design Patterns & Constraints

### forgecode-fork
- Provider trait minimal (invoke, stream, capabilities)
- Registry thread-safe (Arc + RwLock)
- No inter-provider dependencies
- All providers support fallback + error handling
- Audit trail hash-chained for integrity

### phenotype-router-monitor
- Single-threaded router core (tokio async)
- In-memory rate limiting (no distributed coordination Phase 1)
- TOML configuration (schema validation at startup)
- Health checks independent per backend
- Prometheus metrics format (UTF-8, standard names)

### bifrost-routing
- Classifier heuristic-based (no ML Phase 1)
- Token counting approximate (length-based)
- Workload types fixed: code, analysis, writing, retrieval
- SLA per workload (not per user Phase 1)
- Cost tracking granular (per request, queryable)

---

## Contact & Questions

For questions about:
- **forgecode-fork** scope or design: See epic WP01 foundations
- **phenotype-router-monitor** architecture: See epic integration with phenotype-infrakit
- **bifrost-routing** LLM routing: See epic model registry + SLA enforcement

All specs are self-contained and ready for independent implementation.

---

**Ready to launch**: All three forks can begin Phase 1A simultaneously with 12-15 parallel subagents.
