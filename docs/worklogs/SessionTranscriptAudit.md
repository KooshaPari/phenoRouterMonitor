# Session transcript audit (Claude Code + Cursor)

**Date:** 2026-03-29  
**Category:** GOVERNANCE + RESEARCH  
**Status:** in_progress  
**Priority:** P1  

---

## How to use this doc

1. Read `docs/worklogs/README.md` (index and templates).  
2. For broad audits, read `docs/worklogs/AgentMasterAuditPrompt.md`.  
3. **Canonical export:** `docs/worklogs/data/phenotype_session_extract_2026-03-26_2026-03-29.json`  
4. **Engineering follow-ons:** `docs/worklogs/SessionGaps20260329.md`  
5. **Duplication master:** `docs/worklogs/MasterDuplicationAudit20260329.md`  

---

## Resume

- Export includes Claude Code JSONL under `~/.claude/projects/*Phenotype*` plus a **CWD pass** over other Claude project dirs (lines kept when `cwd` is under `.../CodeProjects/Phenotype/`), and Cursor `agent-transcripts` where present under `~/.cursor/projects/*Phenotype*`.  
- **Window:** on or after **2026-03-26** (see `meta` in JSON).  
- **Counts:** see `meta.counts` in the JSON (raw vs substantive vs plan-scored assistant blocks).  
- **Relocation:** large JSON lives under `docs/worklogs/data/`; `docs/reports/README.md` redirects old paths.  

---

## Gaps (deeper audit)

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | **Cursor coverage** | P1 | Only Cursor trees with `agent-transcripts` are ingested; naming must match glob. |
| G2 | **Claude CWD pass** | P2 | May yield zero rows if no sessions used a Phenotype `cwd` in-window. |
| G3 | **Noise in prompts** | P2 | Slash-commands, skills, and large pastes skew “substantive” heuristics. |
| G4 | **No spec / FR linkage** | P1 | Prompts are not mapped to AgilePlus feature IDs or `FR-*` tags. |
| G5 | **Reproducibility** | P2 | Check in `scripts/export_phenotype_session_artifacts.py` (see **T2**). |
| G6 | **Git / LFS** | P3 | JSON may grow large; consider LFS or policy. |
| G7 | **Stale chat paths** | P1 | Historical text may cite removed `docs/reports/*` paths — verify against tree. |

---

## Task backlog

| ID | Task | Priority |
|----|------|----------|
| T1 | Extend Cursor scan when new transcript roots appear; record them in JSON `meta`. | P1 |
| T2 | Add **`scripts/export_phenotype_session_artifacts.py`** (or Taskfile target) + README note. | P2 |
| T3 | Map high-value prompts → AgilePlus specs / WPs. | P1 |
| T4 | Tighten substantive filter; document before/after counts here. | P2 |
| T5 | Execute **`SessionGaps20260329.md`** initiatives per `docs/worklogs/Plans/*`. | P0 |
| T6 | Restore any still-needed artifacts referenced only in old chat logs under `docs/worklogs/data/` or `docs/research/`. | P2 |
| T7 | Keep cross-links aligned with on-disk names (`WorkLog.md`, this file, gaps + master audit). | P2 |

---

## Related

- `docs/worklogs/WorkLog.md` — **Wave 92** tracks hygiene + pending export/ingest tasks  
- `docs/research/consolidation-audit-2026-03-29.md`  

_Last updated: 2026-03-29_
