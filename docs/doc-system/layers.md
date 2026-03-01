---
audience: [developers, pms]
---

# Documentation Layers

AgilePlus follows the PhenoDocs 5-layer documentation model. Each layer represents a maturity stage for documentation artifacts.

## The Five Layers

```mermaid
graph TD
    L0[Layer 0: Raw/Ephemeral] --> L1[Layer 1: Working]
    L1 --> L2[Layer 2: Formal/Spec]
    L2 --> L3[Layer 3: Delivery/Audit]
    L3 --> L4[Layer 4: Retrospective/Knowledge]

    style L0 fill:#991b1b,color:#fecaca
    style L1 fill:#9a3412,color:#fed7aa
    style L2 fill:#854d0e,color:#fef08a
    style L3 fill:#166534,color:#bbf7d0
    style L4 fill:#1e40af,color:#bfdbfe
```

### <span class="layer-badge layer-0">Layer 0</span> Raw / Ephemeral

Scratch notes, conversation dumps, agent work logs. Not published. Retained for 48–90 days, then promoted or discarded.

| Type | Path | Retention |
|------|------|-----------|
| Conversation dump | `docs/research/CONVERSATION_DUMP_*.md` | 90 days |
| Scratch note | `docs/scratch/YYYYMMDD-*.md` | 48 hours |
| Agent work log | `docs/reference/WORK_STREAM.md` | Permanent |

### <span class="layer-badge layer-1">Layer 1</span> Working / Lab

Ideas, research docs, debug logs. Published under `/lab/`. Working documents that may be promoted to formal specs.

| Type | Path | Promotes To |
|------|------|-------------|
| Idea note | `docs/ideas/YYYY-MM-DD-{slug}.md` | Research doc |
| Research doc | `docs/research/{TOPIC}.md` | Design doc / FR |
| Debug log | `docs/debug/YYYY-MM-DD-{issue}.md` | Incident retro |

### <span class="layer-badge layer-2">Layer 2</span> Formal / Spec

Source-of-truth documents: PRDs, ADRs, functional requirements, architecture docs. Published under `/docs/`.

| Type | Path | ID System |
|------|------|-----------|
| Spec | `kitty-specs/{NNN}-{slug}/spec.md` | Feature number |
| Plan | `kitty-specs/{NNN}-{slug}/plan.md` | Feature number |
| ADR | `docs/adr/ADR-{NNN}-{slug}.md` | ADR-{NNN} |

### <span class="layer-badge layer-3">Layer 3</span> Delivery / Audit

Changelogs, completion reports, sprint plans. Published under `/audit/`. Created automatically from lifecycle events.

| Type | Path | Trigger |
|------|------|---------|
| Changelog | `CHANGELOG.md` | Git tag |
| Completion report | `docs/reports/*-complete.md` | Feature shipped |
| Sprint plan | `docs/sprints/SPRINT-{NN}.md` | Sprint planning |

### <span class="layer-badge layer-4">Layer 4</span> Retrospective / Knowledge

Retrospectives, knowledge extracts, lessons learned. Published under `/kb/`. Created after feature completion.

| Type | Path | Trigger |
|------|------|---------|
| Feature retro | `kitty-specs/{NNN}-{slug}/retrospective.md` | Feature shipped |
| Sprint retro | `docs/retros/SPRINT-{NN}-retro.md` | Sprint end |
| Knowledge extract | `docs/kb/{topic}/{slug}.md` | Semantic indexer |

## Layer Progression

Documents flow upward through layers as they mature:

1. **Scratch note** (L0) → promoted to **idea** (L1)
2. **Research doc** (L1) → becomes **spec** (L2)
3. **Spec** (L2) → produces **completion report** (L3) when shipped
4. **Completion report** (L3) → feeds **retrospective** (L4)
