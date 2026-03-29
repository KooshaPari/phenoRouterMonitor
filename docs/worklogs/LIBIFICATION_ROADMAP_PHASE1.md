# Phase 1 Libification Roadmap — 2026-03-29

**Status:** READY FOR EXECUTION
**Effort:** 2-3 days
**Total LOC Savings:** ~180 LOC (Phase 1)
**Parallel Execution:** YES (3 independent agents)

---

## Overview

This is **Phase 1 (immediate, LOW effort)** of the libification roadmap. Three parallel work streams execute simultaneously with no blocking dependencies.

---

## Work Stream 1: Rust thiserror Formalization

**Agent Task:** `WS1-RUST-FORMALIZE-THISERROR`
**Effort:** LOW (~2-3 hours)
**LOC Savings:** ~100-200
**Projects Affected:** phenotype-infrakit, heliosCLI/codex-rs, thegent-*

### Objective
Formalize `thiserror` usage across all Rust crates. Eliminate hand-rolled `impl Error` blocks; replace with thiserror derive macros.

### Scope
1. **Audit** heliosCLI/codex-rs for `impl std::error::Error` hand-rolled patterns
2. **Audit** phenotype-infrakit for similar patterns
3. **Audit** thegent-* crates for error handling inconsistencies
4. **Replace** hand-rolled patterns with `#[derive(Error)]`
5. **Verify** all error types export properly; run cargo check

### Deliverables
- [ ] Audit report: List all hand-rolled error patterns found
- [ ] PR with thiserror replacements (WS1-RUST-THISERROR)
- [ ] Verify cargo check + cargo clippy pass

### Files to Check
- `/crates/phenotype-infrakit/src/error.rs` (likely good baseline)
- `/crates/phenotype-contract/src/error.rs` (reference)
- `/heliosCLI/codex-rs/core/src/error/` (look for hand-rolled)
- `/platforms/thegent/crates/*/src/error.rs` (audit all)
- Search: `impl From<.*> for.*Error` (find manual impls)

### Success Criteria
- [ ] 0 hand-rolled `impl Error` patterns remaining
- [ ] All errors use `#[derive(Error, Debug)]`
- [ ] `cargo check --all-features` passes
- [ ] `cargo clippy --all-targets` passes

---

## Work Stream 2: Go Logging Middleware

**Agent Task:** `WS2-GO-LOGGING-MIDDLEWARE`
**Effort:** LOW (~2-3 hours)
**LOC Savings:** ~50-100
**Projects Affected:** byteport backend, bifrost-extensions

### Objective
Standardize logging across Go projects. Add `slog` or `zap` middleware to byteport + bifrost; eliminate inconsistent logging patterns.

### Scope
1. **Audit** byteport `go.mod` for logging libraries (if any)
2. **Audit** bifrost-extensions for logging patterns
3. **Select** slog (stdlib) or zap (standard choice for structured logging)
4. **Implement** logging middleware in byteport (gin middleware)
5. **Implement** logging middleware in bifrost
6. **Add** request/response logging with structured fields (method, path, status, latency)
7. **Verify** tests pass, no regressions

### Deliverables
- [ ] Audit report: Current logging state in both projects
- [ ] PR with logging middleware (WS2-GO-LOGGING)
- [ ] Integration tests verifying middleware logs requests
- [ ] go mod tidy passes

### Files to Create/Modify
- `/byteport/internal/middleware/logging.go` (new)
- `/bifrost-extensions/internal/middleware/logging.go` (new)
- Update `/byteport/main.go` to register middleware
- Update `/bifrost-extensions/main.go` to register middleware

### Success Criteria
- [ ] Structured logging middleware active in both projects
- [ ] All HTTP requests logged with method, path, status, latency
- [ ] No error logs for normal operations (clean startup)
- [ ] Tests pass: `go test ./...`

---

## Work Stream 3: TypeScript Validation Audit

**Agent Task:** `WS3-TS-ZOD-AUDIT`
**Effort:** LOW (~1-2 hours)
**LOC Savings:** N/A (audit only, refactor optional)
**Projects Affected:** byteport frontend, heliosApp, AgilePlus dashboard, TraceRTM

### Objective
Verify all TypeScript projects use `zod` for validation. Identify any projects using `yup`, `joi`, or custom validators. Plan migration if needed.

### Scope
1. **Audit** all TS projects: search for validation libraries
   - `grep -r "yup\|joi\|valibot\|arktype" package.json tsconfig.json`
   - Check imports: `import * from "yup"` etc.
2. **List** all validation schemas found and their validation patterns
3. **Report** findings: % of projects using zod vs other validators
4. **Plan** migrations if non-zod validators found
5. **Create** standard zod schema location pattern (`src/schemas/` or `lib/validators/`)

### Deliverables
- [ ] Audit report: Validation library usage across all TS projects
- [ ] PR with zod standardization (if migrations needed) (WS3-TS-VALIDATION)
- [ ] Document schema location standards in project CLAUDE.md files

### Files to Check
- `/byteport/package.json` — dependencies
- `/heliosApp/package.json` — dependencies
- `/AgilePlus/package.json` — dependencies (if applicable)
- Recursively: `grep -r "import.*from.*yup\|joi\|valibot"` in all TS projects

### Success Criteria
- [ ] All TS projects identified and audited
- [ ] Validation library report compiled
- [ ] If non-zod found: PR created to standardize to zod
- [ ] All schema files use consistent location pattern

---

## Parallel Execution Matrix

```
Day 1:
├─ WS1 (Rust thiserror) — Start immediately
├─ WS2 (Go logging) — Start immediately (parallel)
└─ WS3 (TS audit) — Start immediately (parallel)

Day 2:
├─ WS1 — Complete PRs, reviews
├─ WS2 — Complete PRs, reviews
└─ WS3 — Complete PR (if needed), analysis

Day 3:
├─ WS1 — Land PR, verify main
├─ WS2 — Land PR, verify main
└─ WS3 — Land PR (if needed), document standards
```

---

## Success Metrics (Phase 1 Complete)

| Work Stream | LOC Saved | PRs Created | Status |
|-------------|-----------|-------------|--------|
| WS1 (Rust thiserror) | ~100-200 | 1 (WS1-RUST-THISERROR) | PENDING |
| WS2 (Go logging) | ~50-100 | 1 (WS2-GO-LOGGING) | PENDING |
| WS3 (TS audit) | N/A (audit) | 0-1 (if migration needed) | PENDING |
| **TOTAL** | **~180 LOC** | **2-3 PRs** | **READY** |

---

## Phase 2 Preview

After Phase 1 completes (2-3 days):

1. **Python httpx consolidation** (MEDIUM, ~30 LOC, 1-2 days)
2. **Python pydantic-settings** (MEDIUM, ~30-50 LOC, 1-2 days)
3. **Rust config crate** (MEDIUM, ~50-100 LOC, 1-2 days)

**Phase 2 Total:** ~130-250 LOC saved, 3-5 days effort

---

## Notes

- **No breaking changes** in Phase 1 (all LOW effort, safe refactoring)
- **All work isolated** in feature branches per governance
- **Parallel execution** reduces wall-clock time from 9 days → 3 days
- **High-confidence work** (thiserror, logging, validation are straightforward)

---

**Status:** READY FOR AGENT DISPATCH
**Next:** Launch WS1, WS2, WS3 haiku agents in parallel

