# AI Code Platforms Visual Guide (2026)

Quick visual reference for decision-making, presentations, and team discussions.

---

## The Landscape: 5 Platforms at a Glance

```
┌──────────────────────────────────────────────────────────────────┐
│                    COST vs. SPEED MATRIX                         │
│                                                                   │
│  HIGH COST                                                       │
│     ^                                                             │
│     │     Cursor                                                 │
│     │     (IDE-native)                                           │
│     │                                                             │
│     │               OpenAI (GPT-4)                               │
│     │               (Deprecated!)                                │
│     │                                                             │
│     │     Kilo                                                   │
│     │     (Event-driven)                                         │
│     │                                                             │
│     │                Claude                                      │
│     │                (Batch + Cache)                             │
│     │                                                             │
│     │                                    Groq + CrewAI           │
│     │                                    (Real-time loops)       │
│     │                                                             │
│     +──────────────────────────────────────────────────────────> SPEED
│  LOW              Low Latency              High Speed (1K+/sec)
│
│  SWEET SPOT FOR DIFFERENT WORKLOADS:                             │
│  • Small batch       → Claude (cost efficiency)                  │
│  • Interactive IDE   → Cursor (UX)                               │
│  • Event-driven      → Kilo (webhooks)                           │
│  • Real-time loops   → Groq (latency)                            │
└──────────────────────────────────────────────────────────────────┘
```

---

## Platform Selection Flowchart

```
                        START: Choose Platform
                               │
                      ┌────────┴────────┐
                      │                 │
            Do you code interactively   No
            in IDE daily?               │
                │                       │
              Yes                    Continue
                │                       │
            CURSOR         ┌────────────┴────────────┐
            (Best IDE UX)  │                         │
                      Do you need    No
                      real-time      │
                      latency?   Continue
                           │           │
                         Yes       ┌───┴──────────────────┐
                           │       │                      │
                        GROQ    Is cost your    No     Continue
                     (1000+/sec) primary concern?   │
                                   │               │
                                 Yes           ┌───┴─────────────────┐
                                   │           │                     │
                            CLAUDE            Do you need   No    Continue
                          Batch + Cache    event-driven   │
                         (95% savings)      automation?   │
                                               │          │
                                             Yes       ┌──┴────────┐
                                               │       │           │
                                            KILO    Use CLAUDE SDK
                                        (Webhooks)  (Most flexible)

                    DEFAULT: CLAUDE AGENT SDK
```

---

## Pricing Comparison: Real-World Examples

```
┌────────────────────────────────────────────────────────────────────┐
│            COST COMPARISON: 100 CODE REVIEWS (2K LOC)              │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Claude (batch + cache):    ████ $1.95                             │
│  Groq (if integrated):      █████ $2.40                            │
│  OpenAI GPT-4:              ██████████████████████ $45             │
│  Cursor:                    ████████████████████████████ $28.90    │
│  Kilo:                      ██████████████████████████████ $30–50  │
│                                                                     │
│  Winner: Claude (15–35x cheaper) ✅                               │
│                                                                     │
│  Note: Cursor includes IDE features; not apples-to-apples         │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│       COST COMPARISON: 1000+ TASKS/DAY (HIGH VOLUME)               │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Groq Llama 70B:            ████ $1.05/day                         │
│  Claude Haiku:              ███████ $4.50/day                      │
│  Claude Haiku (batch):      ████ $2.25/day                         │
│  OpenAI GPT-4:              ██████████ $8–12/day                   │
│  Cursor:                    ███ Not recommended (interactive tool) │
│                                                                     │
│  Winner: Groq (3–5x cheaper at scale) ✅                          │
│                                                                     │
│  Monthly:  Groq $31.50  |  Claude $67.50  |  OpenAI $240+         │
└────────────────────────────────────────────────────────────────────┘
```

---

## Feature Heatmap

```
┌────────────┬────────┬────────┬────────┬──────┬────────┐
│ Feature    │ Claude │ OpenAI │ Cursor │ Kilo │ Groq   │
├────────────┼────────┼────────┼────────┼──────┼────────┤
│ Read files │   ✅   │   ✅   │   ✅   │  ✅  │   ❌   │
│ Write code │   ✅   │   ✅   │   ✅   │  ✅  │   ❌   │
│ Shell exec │   ✅   │  ⚠️*   │   ✅   │  ✅  │   ❌   │
│ Auto PR    │  ⚠️**  │   ❌   │   ✅   │  ✅  │   ❌   │
│ Parallel   │  ⚠️    │   ❌   │   ✅   │  ❌  │  ⚠️    │
│ Caching    │   ✅   │   ❌   │   ❌   │  ❌  │   ❌   │
│ Webhooks   │   ❌   │   ❌   │  ⚠️    │  ✅  │   ❌   │
│ IDE native │   ❌   │  ⚠️    │   ✅   │  ✅  │   ❌   │
│ Fast (<1s) │   ❌   │   ⚠️   │  ⚠️    │  ⚠️  │   ✅   │
└────────────┴────────┴────────┴────────┴──────┴────────┘

Legend:
  ✅ Full support
  ⚠️ Partial/requires workaround
  ❌ Not supported

* OpenAI: Python sandbox only (limited)
** Claude: Via MCP (requires setup)
```

---

## Strengths & Weaknesses at a Glance

```
CLAUDE AGENT SDK
├─ ✅ Best: Cost optimization (batch 50% off, caching 90% off)
├─ ✅ Best: Long context (1M tokens)
├─ ✅ Best: Reasoning (extended thinking)
├─ 🔴 Worst: Model lock-in (Claude-only)
└─ 🔴 Worst: No cloud VMs

OPENAI GPT-4 / RESPONSES API
├─ ✅ Best: Mature ecosystem (until Aug 26, 2026)
├─ ✅ Best: Multi-modal support
├─ 🔴 Worst: Assistants sunset (Aug 26, 2026 — URGENT)
├─ 🔴 Worst: Expensive ($2.50–30/1M tokens)
└─ 🔴 Worst: No GitHub integration

CURSOR CLOUD AGENTS
├─ ✅ Best: IDE integration (native VSCode, JetBrains)
├─ ✅ Best: Parallel execution (20 agents)
├─ ✅ Best: Auto PR creation + video evidence
├─ 🔴 Worst: Expensive for automation ($20–60/mo)
└─ 🔴 Worst: IDE lock-in (not headless-friendly)

KILO CODE
├─ ✅ Best: Transparent pricing (zero markup)
├─ ✅ Best: Webhook-driven automation
├─ ✅ Best: GitHub App integration
├─ 🔴 Worst: Single agent (no parallelism)
└─ 🔴 Worst: KiloClaw costs $49/mo (free period ended)

GROQ + CREWAI
├─ ✅ Best: Speed (1,200+ tokens/sec; <100ms latency)
├─ ✅ Best: Cost at scale (1000+ tasks/day)
├─ ✅ Best: Burst-friendly (no hard limits)
├─ 🔴 Worst: No agent orchestration built-in
└─ 🔴 Worst: No GitHub integration
```

---

## Decision Flowchart (Text Version)

```
                    Which Platform for My Use Case?
                              │
                    ┌─────────┴─────────┐
                    │                   │
             Interactive    OR      Automated
             IDE Coding          Batch Work
                    │                   │
                    │         ┌─────────┴──────────┐
                    │         │                    │
                    │    Is cost    YES        Is latency
                  CURSOR      critical?        critical?
                  (Best IDE)       │               │
                    │          CLAUDE          GROQ
                    │        (Batch +         (Speed)
                    │         Cache)
                    │
                    └──────────────┘
                          │
                    Use multiple:
                    • Cursor for IDE
                    • Claude for automation
                    • Groq for real-time loops
```

---

## Migration Timeline

```
2026 TIMELINE
═════════════════════════════════════════════════════════════════

NOW                                          AUG 26, 2026
│                                                    │
├─────────────────────────────────────────────────┤
│     OPENAI ASSISTANTS SUNSET                    │
│     ⚠️ HARD DEADLINE                             │
│                                                  │
│     ✅ If using Assistants: Start migration NOW │
│     ✅ Test Responses API in parallel            │
│     ✅ Export all data before Aug 26              │
│                                                  │
├─────────────────────────────────────────────────┤
│     CURSOR SELF-HOSTED GA                       │
│     ✅ Available now (generally available)       │
│     ✅ Enterprise-ready                          │
│                                                  │
├─────────────────────────────────────────────────┤
│     GROQ COMPOUND GA                            │
│     ✅ Agentic AI system available               │
│     ✅ Documentation improving                   │
│                                                  │
├─────────────────────────────────────────────────┤
│     CLAUDE AGENT SDK STABLE                     │
│     ✅ v0.1.48+ Python, v0.2.71+ TypeScript      │
│     ✅ Hooks + MCP mature                        │
│                                                  │
├─────────────────────────────────────────────────┤
│     KILO CLOUD AGENTS PAID                      │
│     ⚠️ Free period ended (Mar 23, 2026)          │
│     ⚠️ Now costs $49/mo                          │
│                                                  │
└──────────────────────────────────────────────────
```

---

## Recommendation by Company Size

```
STARTUP (5–20 people, budget <$500/mo)
┌────────────────────────────────────────────┐
│ Primary:  Claude Agent SDK                 │
│           Cost: $5–20/mo                   │
│           Reason: Best cost-efficiency     │
│                                             │
│ Secondary: Cursor (team IDE)               │
│           Cost: $20/mo (shared)            │
│           Reason: Developer productivity   │
└────────────────────────────────────────────┘

SMB (20–100 people, budget $500–5K/mo)
┌────────────────────────────────────────────┐
│ Primary:  Claude Agent SDK + Cursor        │
│           Cost: $40–60/mo                  │
│           Reason: Hybrid (automation+IDE)  │
│                                             │
│ Secondary: Kilo (GitHub automation)        │
│           Cost: $49–199/mo                 │
│           Reason: Event-driven PRs         │
│                                             │
│ Backup:   Groq (if 1K+ tasks/day)          │
│           Cost: $50–100/mo                 │
│           Reason: Cost at scale            │
└────────────────────────────────────────────┘

ENTERPRISE (100+ people, budget $5K+/mo)
┌────────────────────────────────────────────┐
│ Primary:  Groq + CrewAI (real-time)        │
│           Cost: $150–500/mo                │
│           Reason: Cost + latency at scale  │
│                                             │
│ Secondary: Claude (reasoning tasks)        │
│           Cost: $200–300/mo                │
│           Reason: Extended thinking + RAG  │
│                                             │
│ Tertiary: Cursor + self-hosted agents      │
│           Cost: $200/mo (Enterprise seat)  │
│           Reason: Team IDE + automation    │
│                                             │
│ Fallback: Kilo (GitHub-first)              │
│           Cost: $200+/mo                   │
│           Reason: Webhook orchestration    │
└────────────────────────────────────────────┘
```

---

## Cost Breakdown Example: Mid-Market Setup

```
HYBRID PLATFORM STRATEGY
═══════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────┐
│ CLAUDE AGENT SDK (Background automation, RAG, reasoning)│
│ ┌───────────────────────────────────────────────────────┤
│ │ Input:  5M tokens/mo × $3/1M  = $15                   │
│ │ Output: 1M tokens/mo × $15/1M = $15                   │
│ │ Batch discount (50%): (-$15)                          │
│ │ Cache hits (80%): (-$12)                              │
│ │                                                        │
│ │ Subtotal: $3/month                                    │
│ └───────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ CURSOR PRO (Team IDE, 30 engineers @ $20/user)         │
│ ┌───────────────────────────────────────────────────────┤
│ │ Subscription: 30 × $20/mo = $600/month               │
│ │ Cloud agent overheads: ~$50/mo                        │
│ │                                                        │
│ │ Subtotal: $650/month                                  │
│ └───────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ KILO CODE (GitHub event automation)                    │
│ ┌───────────────────────────────────────────────────────┤
│ │ KiloClaw: $49/mo                                      │
│ │ Subscription (Pro): $49/mo                            │
│ │                                                        │
│ │ Subtotal: $98/month                                   │
│ └───────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ INFRASTRUCTURE (Lambda, DynamoDB, Vector DB)            │
│ ┌───────────────────────────────────────────────────────┤
│ │ Serverless compute: $20/mo                            │
│ │ Data storage (DynamoDB, Milvus): $30/mo              │
│ │ Monitoring & observability: $20/mo                    │
│ │                                                        │
│ │ Subtotal: $70/month                                   │
│ └───────────────────────────────────────────────────────┘

╔═════════════════════════════════════════════════════════╗
║              TOTAL: ~$821/month                         ║
║                                                         ║
║  Breakdown:                                             ║
║    IDE (Cursor):     79% ($650)                         ║
║    GitHub (Kilo):    12% ($98)                          ║
║    Infrastructure:    9% ($70)                          ║
║    API (Claude):    <1% ($3)                            ║
║                                                         ║
║  Key Insight:                                           ║
║  IDE costs dwarf API costs; infrastructure matters!     ║
╚═════════════════════════════════════════════════════════╝
```

---

## Comparison Cheat Sheet

```
WHEN TO USE EACH PLATFORM

Claude Agent SDK
├─ ✅ Cost-critical projects (<$50/mo budget)
├─ ✅ Long-context RAG pipelines (200K+ tokens)
├─ ✅ Complex reasoning (extended thinking)
├─ ✅ Batch processing (non-time-sensitive)
└─ ✅ Custom orchestration (hooks + MCP)

Cursor
├─ ✅ Team coding environment (30+ engineers)
├─ ✅ Time-sensitive features (need 2–10x speedup)
├─ ✅ Multi-module refactoring (parallel execution)
├─ ✅ Interactive debugging (IDE-native)
└─ ✅ Self-hosted requirements (GA available)

Kilo Code
├─ ✅ Event-driven PR automation
├─ ✅ GitHub-first workflows
├─ ✅ Transparent pricing requirement
├─ ✅ Webhook integration (external triggers)
└─ ✅ Scheduled code review automation

Groq + CrewAI
├─ ✅ Real-time agent loops (<500ms latency)
├─ ✅ High-concurrency systems (1000+ QPS)
├─ ✅ Cost-critical at scale (1000+ tasks/day)
├─ ✅ Open-source model preference
└─ ✅ Interactive agent systems

OpenAI
├─ ⚠️ Existing ecosystem (until Aug 26, 2026)
├─ ⚠️ Multi-modal requirements (vision, images)
├─ 🔴 NOT RECOMMENDED for new projects
└─ 🔴 Migrate existing Assistants NOW
```

---

## Key Numbers to Remember

```
TOKEN PRICING (Per 1M Tokens, 2026)
═════════════════════════════════════
Claude Haiku:      $1 input, $5 output       (cheapest)
Claude Sonnet:     $3 input, $15 output      (balanced)
Claude Opus:       $5 input, $25 output      (best)
GPT-5.4:           $2.50 input, $15 output  (good)
GPT-4 (legacy):    $30 input, $60 output     (avoid)
Groq Llama 70B:    $0.30 input, $0.90 out   (fastest)

COST MULTIPLIERS
═════════════════════════════════════
Claude batch API:           50% discount
Claude caching:             90% savings on cache hits
Cursor cloud agent:         ~$0.04 per call
Kilo zero markup:           Exact token rates
OpenAI Code Interpreter:    $0.03 per session

LATENCY (Approximate)
═════════════════════════════════════
Claude (P99):               2–5 seconds
OpenAI (P99):               1–3 seconds
Cursor (interactive):       <500ms (IDE cached)
Groq (TTFT):                <100ms (deterministic)
Groq (per token):           50–100ms for 1K tokens

CONTEXT WINDOWS
═════════════════════════════════════
Claude Haiku:               200K tokens
Claude Sonnet/Opus:         200K–1M tokens
GPT-4:                      128K tokens
Groq Llama 70B:             8K–100K tokens (varies)
Cursor (model-dependent):   Same as underlying model
```

---

## Questions This Guide Answers

✅ Which platform should I choose for my use case?
✅ What will it cost?
✅ How do I migrate from OpenAI Assistants?
✅ Can I use multiple platforms together?
✅ What are the pitfalls to avoid?
✅ How fast/cheap is each option?
✅ Which has the best IDE integration?
✅ Which scales best?

---

## Use This Guide For

- [ ] Team presentations (use flowcharts, scorecards)
- [ ] Budget planning (use cost examples)
- [ ] Vendor evaluations (use feature heatmap)
- [ ] Platform selection (use decision tree)
- [ ] Migration planning (use timeline, migration checklist)
- [ ] Training new engineers (use recommendation matrix)

---

**Generated:** March 30, 2026
**Status:** ✅ Complete

For detailed analysis, see the companion documents:
- `RESEARCH_SUMMARY.md` — 5 key findings
- `AI_CODE_PLATFORMS_COMPARISON_2026.md` — Full technical comparison
- `AI_CODE_PLATFORMS_COST_ANALYSIS.md` — Detailed pricing
- `CODE_PLATFORM_GAPS_AND_WORKAROUNDS.md` — Problem solving
- `AI_PLATFORMS_QUICK_REFERENCE.md` — Lookup tables

