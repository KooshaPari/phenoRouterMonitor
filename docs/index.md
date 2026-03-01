---
layout: home
hero:
  name: AgilePlus
  text: Spec-driven development engine
  tagline: From specification to shipped feature — governed, auditable, agent-ready. 51 pages of docs for agents, developers, PMs, and SDK consumers.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Quick Start (5 min)
      link: /guide/quick-start
    - theme: alt
      text: GitHub
      link: https://github.com/KooshaPari/AgilePlus
features:
  - title: Specify → Ship
    details: 9-phase governed pipeline from specification to merge. Every state transition is auditable and immutable.
    link: /workflow/specify
  - title: Multi-Audience Docs
    details: Module switcher filters docs by role — Agents, Developers, PMs, SDK consumers. See only what's relevant.
    link: /doc-system/frontmatter
  - title: Agent-First
    details: Structured prompts, governance constraints, harness integration. Dispatch to Claude Code, Cursor, or custom agents.
    link: /agents/prompt-format
  - title: Clean Architecture
    details: Port-based domain core. Swap storage, VCS, or agent adapters without touching business logic.
    link: /architecture/overview
  - title: Full Workflow Coverage
    details: Specify, clarify, research, plan, tasks, implement, review, accept, merge — every phase documented.
    link: /concepts/feature-lifecycle
  - title: PhenoDocs Framework
    details: 5-layer documentation taxonomy, federation-ready frontmatter, cross-repo linking via the Phenotype ecosystem.
    link: /doc-system/layers
---

<div class="pipeline">
  <span class="stage">specify</span>
  <span class="arrow">→</span>
  <span class="stage">clarify</span>
  <span class="arrow">→</span>
  <span class="stage">research</span>
  <span class="arrow">→</span>
  <span class="stage">plan</span>
  <span class="arrow">→</span>
  <span class="stage">implement</span>
  <span class="arrow">→</span>
  <span class="stage">review</span>
  <span class="arrow">→</span>
  <span class="stage">accept</span>
  <span class="arrow">→</span>
  <span class="stage">merge</span>
</div>

<div class="quick-start">

### Quick Start

```bash
cargo install agileplus

agileplus init my-project
agileplus specify "Add OAuth2 login"
agileplus plan 001
agileplus implement WP01 --agent claude-code
agileplus review WP01
agileplus merge 001
```

</div>
