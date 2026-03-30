---
audience: [developers, agents]
---

# Frontmatter Schema

Every documentation file uses structured YAML frontmatter for metadata, filtering, and cross-referencing.

## Required Fields

```yaml
---
title: "Human-readable title"
audience: [developers, agents, pms, sdk]
---
```

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Display title |
| `audience` | string[] | Module filter targets: `agents`, `developers`, `pms`, `sdk` |

## Optional Fields

```yaml
---
type: idea | research | prd | adr | spec | guide | reference
status: draft | active | published | archived
layer: 0 | 1 | 2 | 3 | 4
date: 2026-02-28
author: agent | human | "name"
relates_to: ["path/to/other.md"]
traces_to: ["FR-001", "ADR-023", "WP01"]
---
```

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Document type for taxonomy |
| `status` | string | Lifecycle state |
| `layer` | number | PhenoDocs layer (0–4) |
| `date` | string | Creation or last-modified date |
| `author` | string | Who created this document |
| `relates_to` | string[] | Bidirectional links to related docs |
| `traces_to` | string[] | Traceability links (FRs, ADRs, WPs) |

## Audience Values

| Value | Who | What They See |
|-------|-----|---------------|
| `agents` | AI coding agents | Prompt formats, constraints, harness integration |
| `developers` | Human developers | Contributing, architecture, testing, extending |
| `pms` | Project managers | Roadmap, retrospectives, status, planning |
| `sdk` | SDK/API consumers | Port traits, gRPC API, MCP tools |

## Filtering Behavior

The sidebar module switcher uses `audience` to filter pages:

- **All Docs** — shows everything
- **For Agents** — shows pages with `agents` in audience
- **For Developers** — shows pages with `developers` in audience
- **For PMs** — shows pages with `pms` in audience
- **SDK / API** — shows pages with `sdk` in audience
- **Show all** toggle — overrides filtering

Pages without `audience` frontmatter are always visible.

## Status Badges

Use in markdown:

```html
<span class="status-badge status-draft">Draft</span>
<span class="status-badge status-active">Active</span>
<span class="status-badge status-published">Published</span>
<span class="status-badge status-archived">Archived</span>
```

Renders as: <span class="status-badge status-active">Active</span>

## Layer Badges

```html
<span class="layer-badge layer-0">Layer 0</span>
<span class="layer-badge layer-2">Layer 2</span>
```

See [Documentation Layers](/doc-system/layers) for the full taxonomy.
