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

---

## P1 — Near term

| Track | Plan | Summary |
|-------|------|---------|
| Config | `Plans/ConfigCoreActivation.md` | Activate `config-core` patterns across services that duplicate loaders. |
| LOC / decomposition | `Plans/LocReductionDecomposition.md` | Structured reduction and module boundaries. |

---

## Cross-ecosystem context

- **`docs/research/consolidation-audit-2026-03-29.md`** — repo merge targets, shared modules, wrap/fork list.  
- **`docs/worklogs/DUPLICATION.md`** — extended duplication inventory (AgilePlus + shelf).  
- **`docs/worklogs/SessionTranscriptAudit.md`** — transcript methodology, **G1–G7**, **T1–T7**.  

---

## References

- **Master duplication audit:** `docs/worklogs/MasterDuplicationAudit20260329.md`  
- **Wave tracking:** `docs/worklogs/WorkLog.md`  
- **Session JSON:** `docs/worklogs/data/phenotype_session_extract_2026-03-26_2026-03-29.json`  

---

_Last updated: 2026-03-29_
