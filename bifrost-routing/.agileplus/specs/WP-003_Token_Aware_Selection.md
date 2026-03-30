# WP-003: Token-Aware & Workload-Based Model Selection

**Work Package ID**: WP-003
**Epic**: eco-fork-003 (LLM Routing Infrastructure)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Implement intelligent model selection based on prompt token count and workload type.

## Description

Route requests to optimal model: Haiku for small/simple tasks, Opus for large/complex tasks.

---

## Objectives

- Implement ModelSelector with token-aware routing
- Define routing rules matrix (workload × token-range → model)
- Implement cost-aware fallback
- Add TOML configuration for routing rules

---

## Acceptance Criteria

1. **Routing**:
   - <500 tokens → Haiku
   - 500-2000 tokens → Sonnet
   - >2000 tokens → Opus
   - Code task → prefer Haiku/Sonnet
   - Analysis task → prefer Opus

2. **Cost Awareness**:
   - If cost exceeds cap, use cheaper model
   - `/select-model` returns model name + estimated cost

3. **Testing**:
   - All combinations of workload + token range tested
   - Selection logic tests pass

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| ModelSelector | Selection logic | Trait impl |
| Token counter | Approximate token counting | Accurate |
| Rules matrix | Routing rules | Defined |
| /select-model API | HTTP endpoint | JSON output |
| Config | TOML routing rules | Loads correctly |
| Tests | Unit + integration | ≥90% coverage |

---

## Dependencies

**Depends On**:
- WP-001 (Request Classifier)
- WP-002 (Model Registry)

**Blocks**:
- WP-004 (SLA Enforcement)
- WP-005 (Cost Tracking)

---

## Effort Estimate

- **Estimated LOC**: 400
- **Estimated Tool Calls**: 12-14
- **Estimated Duration**: 4 days

---

## Subtasks

- [ ] T015: Create bifrost-routing-core/src/selector.rs
- [ ] T016: Implement token counting from prompt
- [ ] T017: Define routing rules matrix
- [ ] T018: Implement cost heuristic
- [ ] T019: Add TOML config schema
- [ ] T020: Create HTTP endpoint POST /select-model
- [ ] T021: Unit tests for selection logic
- [ ] T022: Integration test

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
