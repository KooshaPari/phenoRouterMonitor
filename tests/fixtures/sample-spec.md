# Sample Feature Specification

**Feature**: sample-feature
**Status**: specified
**Target branch**: main

## Overview

A minimal but valid specification for testing the AgilePlus plan command's
work-package generation. This spec contains enough functional requirements to
produce 2-3 work packages.

## Functional Requirements

### FR-001 — Core data model

The system shall maintain a feature registry backed by SQLite.
Each feature entry shall include slug, friendly_name, state, spec_hash,
target_branch, created_at, and updated_at fields.

**Acceptance criteria**:
- A feature can be created with a valid slug.
- The slug must be unique across all features.
- spec_hash is derived from the SHA-256 digest of the specification content.

### FR-002 — State machine transitions

The system shall enforce a linear state machine:
`created -> specified -> researched -> planned -> implementing -> validated -> shipped -> retrospected`.

**Acceptance criteria**:
- Forward transitions to the immediate next state are always allowed.
- Backward transitions are rejected with an `InvalidTransition` error.
- Skip-forward transitions produce a warning but are allowed.
- The `retrospected` state is terminal — no further transitions are permitted.

### FR-003 — Audit trail

The system shall record every state transition as an immutable audit entry.
Each entry shall include a SHA-256 hash that chains to the previous entry,
forming a tamper-evident log.

**Acceptance criteria**:
- Every state transition creates one audit entry.
- The audit chain can be verified by recomputing all entry hashes.
- Tampering with any entry causes chain verification to fail.

## Non-Functional Requirements

- **Performance**: The SQLite adapter must handle 1 000 features without
  degrading read latency below 10 ms.
- **Security**: API keys must be validated on every HTTP request.
- **Observability**: All commands emit OpenTelemetry spans.
