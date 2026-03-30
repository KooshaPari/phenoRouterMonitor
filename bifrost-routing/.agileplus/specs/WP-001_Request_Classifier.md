# WP-001: Request Classifier & Workload Inference

**Work Package ID**: WP-001
**Epic**: eco-fork-003 (LLM Routing Infrastructure)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Implement request classifier to infer workload type (code, analysis, writing, retrieval) from request content.

## Description

Heuristic-based classification system that categorizes LLM requests by workload type for intelligent model routing.

---

## Objectives

- Implement `RequestClassifier` for workload type inference
- Support 4 workload types: code, analysis, writing, retrieval
- Implement keyword matching and complexity heuristics
- Add SQLite schema for classification audit trail
- Expose `/classify` API endpoint

---

## Acceptance Criteria

1. **Classification**:
   - Correctly labels "write a function" as code
   - Correctly labels "analyze sales data" as analysis
   - Classification latency <10ms

2. **Audit Trail**:
   - Classification stored in SQLite
   - Queryable for analysis

3. **API**:
   - `/classify` endpoint returns JSON with workload label + confidence
   - Confidence score 0.0-1.0

4. **Testing**:
   - `cargo test -p bifrost-routing` all pass
   - Mock prompts test accuracy

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| Classifier | RequestClassifier struct | Trait impl |
| Rules engine | Keyword-based classification | Rules defined |
| SQLite schema | Classification audit trail | Queries work |
| /classify API | HTTP endpoint | JSON output |
| Tests | Unit tests | ≥90% coverage |

---

## Dependencies

**Depends On**: None (foundational)

**Blocks**:
- WP-003 (Model Selection)
- WP-004 (SLA Enforcement)

---

## Effort Estimate

- **Estimated LOC**: 380
- **Estimated Tool Calls**: 11-13
- **Estimated Duration**: 3-4 days

---

## Subtasks

- [ ] T001: Create bifrost-routing-models/src/workload.rs
- [ ] T002: Create bifrost-routing-core/src/classifier.rs
- [ ] T003: Implement rule engine (keyword weights, length, complexity)
- [ ] T004: Add SQLite schema for classification audit
- [ ] T005: Create HTTP endpoint POST /classify
- [ ] T006: Unit tests for classifier accuracy
- [ ] T007: Edge case tests

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
