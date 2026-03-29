# Session gaps — engineering follow-ons (2026-03-29)

**Category:** PLAN  
**Status:** active  
**Priority:** P0–P1  

This file ties **session-level findings** (transcripts, duplication audits, consolidation research) to **concrete implementation tracks** under `docs/worklogs/Plans/` and ecosystem research.

---

## P0 — Do first

| Track | Plan | Summary |
|-------|------|---------|
| Edition alignment | `Plans/EditionMigration.md` | Move `libs/` and shared crates to **edition 2024** where the workspace expects it. |
| Error consolidation | `Plans/ErrorCoreExtraction.md` | Introduce shared error surface (`error-core` / `phenotype-errors` direction) and replace scattered `*Error` enums. |
| Duplication execution | `Plans/ImplementationPlanDuplication.md` | Operationalize `DUPLICATION.md` + `MasterDuplicationAudit20260329.md` findings. |
| Nested crate cleanup | _(no plan file yet — add under `Plans/` )_ | Remove duplicate inner `crates/*/phenotype-*/` package trees; single `src` root per member (**infrakit**). |
| Session export hardening | `SessionTranscriptAudit.md` **T13** | Redaction + `meta.repo_head` before any JSON leaves the org. |
| Policy / casbin path | `EXTERNAL_DEPENDENCIES.md` + policy engine | If adopting **casbin**, centralize RBAC in `phenotype-policy-engine` instead of ad hoc checks in siblings. |

---

## P1 — Near term

| Track | Plan | Summary |
|-------|------|---------|
| Config | `Plans/ConfigCoreActivation.md` | Activate `config-core` patterns across services that duplicate loaders. |
| LOC / decomposition | `Plans/LocReductionDecomposition.md` | Structured reduction and module boundaries. |
| Event sourcing consistency | `Plans/MasterDuplicationAudit.md` (research refs) | Ensure **one** `phenotype-event-sourcing` implementation path; inner vs outer crate must not diverge. |
| Port error taxonomy | `phenotype-contracts` ports | Merge inbound/outbound `Error` shape with engine-level `thiserror` types where semantically identical. |
| Worktree hygiene | `INACTIVE_FOLDERS.md` | Finish deletion / PR workflow for orphaned `.worktrees/*` copies called out in **`WorkLog.md`**. |
| Transcript tooling | `SessionTranscriptAudit.md` **T2** | Checked-in exporter + documented args (no ad hoc Python). |

---

## P2 — Backlog / research

| Initiative | Source | Next action |
|------------|--------|---------------|
| **eventually** / ES patterns | `RESEARCH.md`, master audit | Spike: wrap vs fork for aggregate roots in Rust services. |
| **figment** multi-source config | `DUPLICATION.md` | Prototype behind feature flag in one service before workspace-wide roll. |
| **temporal-sdk** workflows | External deps worklog | Map long-running agent jobs → workflow boundaries; avoid duplicating schedulers. |
| **thegent hooks** error zoo | Shelf grep (`thegent-work`) | Catalog ~10 `*Error` enums; decide subset for `phenotype-errors` vs local crate. |
| **heliosCLI** harness errors | `heliosCLI-wtrees` | Same as above — `VerifyError`, `OrchestratorError`, etc. share `Serialization` / `NotFound` shapes. |
| **moka / cache** unification | `consolidation-audit` | Confirm `phenotype-cache-adapter` is the only cache façade; retire inline moka usage elsewhere. |
| **Starred-repo radar** | `RESEARCH.md` | Quarterly pass: new OSS replacing hand-rolled parsers, OTEL, policy. |

---

## `phenotype-infrakit` (this git root) — concrete gaps

_Verified against root `Cargo.toml` + `crates/` layout._

| ID | Gap | Owner | Evidence |
|----|-----|-------|----------|
| IK-1 | **Edition 2021** lock-in for all five members | Platform | `[workspace.package] edition = "2021"` |
| IK-2 | **Double `Cargo.toml`** trees for `event-sourcing` + `contracts` | Rust | Nested `phenotype-*/phenotype-*/Cargo.toml` |
| IK-3 | **Error type sprawl** inside small workspace | Rust | `PolicyEngineError`, `EventSourcingError`, `EventStoreError`, `HashError`, port `Error` |
| IK-4 | **Docs / audit mismatch** | Docs | Master audit still cites `agileplus-*` paths — label as **sibling shelf**, not this workspace (see **Appendix A** in master audit). |

---

## Cross-ecosystem context

- **`docs/research/consolidation-audit-2026-03-29.md`** — repo merge targets, shared modules, wrap/fork list.  
- **`docs/worklogs/DUPLICATION.md`** — extended duplication inventory (AgilePlus + shelf).  
- **`docs/worklogs/SessionTranscriptAudit.md`** — transcript methodology, **G1–G14**, **T1–T14**, infrakit verification table.  
- **`docs/governance/ADR-001-external-package-adoption.md`** — adoption criteria for wrap/fork (thegent lane).  
- **`docs/worklogs/EXTERNAL_DEPENDENCIES.md`** — fork/wrap candidates and priority.  

---

## Research queue (agents)

| # | Question | Suggested method |
|---|----------|------------------|
| R1 | How many distinct `NotFound` / `Serialization` variants exist **shelf-wide**? | `rg 'NotFound\(|Serialization'` on `CodeProjects/Phenotype/repos` (exclude `target/`, `.git`). |
| R2 | Which crates already use **thiserror** 2.x vs 1.x? | `rg 'thiserror'` + `Cargo.toml` version pins. |
| R3 | Are **inner** nested crates referenced by any CI job? | Search workflows for path `phenotype-event-sourcing/phenotype-event-sourcing`. |
| R4 | Duplicate **Repository** trait definitions across repos? | `rg 'trait Repository'` limited to `libs/` + `crates/`. |

---

## References

- **Master duplication audit:** `docs/worklogs/MasterDuplicationAudit20260329.md`  
- **Wave tracking:** `docs/worklogs/WorkLog.md`  
- **Session JSON:** `docs/worklogs/data/phenotype_session_extract_2026-03-26_2026-03-29.json`  

---

_Last updated: 2026-03-29_
