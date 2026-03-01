---
layout: home
hero:
  name: AgilePlus
  text: Spec-driven development engine
  tagline: From specification to shipped feature — governed, auditable, agent-ready.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/KooshaPari/AgilePlus
features:
  - title: Specify → Ship
    details: 7-stage governed pipeline with full audit trail. Every state transition is SHA-256 hashed and immutable.
  - title: Multi-Agent
    details: Dispatch work to Claude Code, Cursor, Codex, or Copilot. Built-in review loops with Coderabbit fallback.
  - title: Clean Architecture
    details: Port-based domain core. Swap storage, VCS, or agent adapters without touching business logic.
  - title: Triage & Sync
    details: Rule-based intent classification. Bidirectional sync with Plane.so and GitHub Issues.
---

<div class="pipeline">
  <span class="stage">specify</span>
  <span class="arrow">→</span>
  <span class="stage">research</span>
  <span class="arrow">→</span>
  <span class="stage">plan</span>
  <span class="arrow">→</span>
  <span class="stage">implement</span>
  <span class="arrow">→</span>
  <span class="stage">validate</span>
  <span class="arrow">→</span>
  <span class="stage">ship</span>
  <span class="arrow">→</span>
  <span class="stage">retrospective</span>
</div>

<div class="quick-start">

### Quick Start

```bash
cargo install --path crates/agileplus-cli

agileplus init
agileplus specify --title "Auth Flow" --description "Add OAuth2 login"
agileplus plan auth-flow
agileplus implement auth-flow --wp WP01
agileplus ship auth-flow
```

</div>
