# WP-002: LLM Model Registry & Capability Metadata

**Work Package ID**: WP-002
**Epic**: eco-fork-003 (LLM Routing Infrastructure)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Create model registry with Claude models (Opus, Sonnet, Haiku) and capability metadata.

## Description

Centralized registry of LLM models with cost, latency, accuracy, and training data cutoff information.

---

## Objectives

- Create model registry with Claude models
- Store cost/token, latency SLA, accuracy tier, training cutoff
- Support adding custom models
- Expose model discovery API

---

## Acceptance Criteria

1. **Registry**:
   - Stores ≥3 Claude models with accurate cost/latency metadata
   - `/models` returns JSON list with all models
   - Model lookup by name succeeds

2. **Metadata**:
   - Accurate Claude model costs
   - Latency baselines
   - Training data cutoffs

3. **SQLite**:
   - Metadata versioned in SQLite
   - Historical cost tracking possible

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| ModelRegistry | Central registry | Trait impl |
| ModelSpec | Metadata struct | Fields complete |
| Metadata | Claude model data | Accurate |
| /models API | HTTP endpoint | JSON output |
| SQLite schema | Model metadata versioning | Queries work |

---

## Dependencies

**Depends On**: None (foundational)

**Blocks**:
- WP-003 (Model Selection)
- WP-005 (Cost Tracking)

---

## Effort Estimate

- **Estimated LOC**: 300
- **Estimated Tool Calls**: 8-10
- **Estimated Duration**: 2-3 days

---

## Subtasks

- [ ] T008: Create bifrost-routing-models/src/registry.rs
- [ ] T009: Define ModelSpec struct
- [ ] T010: Hard-code Claude models with costs
- [ ] T011: Add SQLite schema for model metadata
- [ ] T012: Create HTTP endpoint GET /models
- [ ] T013: Unit tests for model selection
- [ ] T014: Edge case tests

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
