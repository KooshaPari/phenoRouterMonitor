# AI Code Platforms Quick Reference (2026)

**Updated:** March 30, 2026

---

## Platform Scorecard at a Glance

```
┌─────────────────────────────────────────────────────────────────┐
│ ANTHROPIC CLAUDE AGENT SDK                                      │
├─────────────────────────────────────────────────────────────────┤
│ Price:              ★★★★★ ($1–5 / 1M input tokens)              │
│ Cloud Agents:       ★★☆☆☆ (SDK only, no managed VMs)            │
│ Code Access:        ★★★★☆ (File ops + MCP integration)          │
│ GitHub Integration: ★★★☆☆ (MCP-based; manual setup)             │
│ Context Window:     ★★★★★ (200K–1M tokens)                      │
│ Parallel Agents:    ★★★☆☆ (Teams possible, not native)          │
│ IDE Integration:    ★★☆☆☆ (Standalone; no IDE plugin)           │
│ Cost Optimization:  ★★★★★ (Caching + batch = 95% savings)       │
├─────────────────────────────────────────────────────────────────┤
│ BEST FOR: Cost-optimized batch work, RAG, reasoning             │
│ WORST FOR: Interactive coding, real-time latency               │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ OPENAI GPT-4 / RESPONSES API                                    │
├─────────────────────────────────────────────────────────────────┤
│ Price:              ★★☆☆☆ ($2.50–30 / 1M input tokens)          │
│ Cloud Agents:       ★☆☆☆☆ (Assistants sunset Aug 26, 2026)      │
│ Code Access:        ★★☆☆☆ (Python sandbox, 20-file limit)       │
│ GitHub Integration: ★☆☆☆☆ (None; must build custom)            │
│ Context Window:     ★★★☆☆ (128K for GPT-4; more for others)      │
│ Parallel Agents:    ★☆☆☆☆ (No multi-agent support)              │
│ IDE Integration:    ★★☆☆☆ (Third-party plugins only)            │
│ Cost Optimization:  ★☆☆☆☆ (No caching, limited batch)           │
├─────────────────────────────────────────────────────────────────┤
│ BEST FOR: Existing Assistants users (migrate NOW; deadline soon) │
│ WORST FOR: New projects; cost-sensitive applications           │
│ ⚠️ WARNING: Assistants deprecated; migrate to Responses API     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ CURSOR CLOUD AGENTS                                             │
├─────────────────────────────────────────────────────────────────┤
│ Price:              ★★★☆☆ ($20–200/mo subscription)             │
│ Cloud Agents:       ★★★★★ (20 parallel VMs with auto-test)       │
│ Code Access:        ★★★★☆ (Full Ubuntu + git worktrees)         │
│ GitHub Integration: ★★★★☆ (Auto PR creation with video)        │
│ Context Window:     ★★★★☆ (Model-dependent; Claude 1M)          │
│ Parallel Agents:    ★★★★★ (20 concurrent agents, native)        │
│ IDE Integration:    ★★★★★ (Native VSCode, JetBrains, web)       │
│ Cost Optimization:  ★★☆☆☆ (No caching, credit-based)            │
├─────────────────────────────────────────────────────────────────┤
│ BEST FOR: Interactive IDE coding, team collaboration            │
│ WORST FOR: Cost-sensitive, automated CI/CD pipelines            │
│ 💡 TIP: Self-hosted cloud agents now available (GA)             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ KILO CODE                                                        │
├─────────────────────────────────────────────────────────────────┤
│ Price:              ★★★★☆ ($19–199/mo + zero markup tokens)      │
│ Cloud Agents:       ★★★☆☆ (KiloClaw, $49/mo, single agent)       │
│ Code Access:        ★★★★☆ (GitHub App, full repo access)        │
│ GitHub Integration: ★★★★☆ (Native, PR comments, webhooks)       │
│ Context Window:     ★★★★☆ (Model-dependent routing)             │
│ Parallel Agents:    ★★☆☆☆ (Single agent, webhook-driven)        │
│ IDE Integration:    ★★★★☆ (VSCode, JetBrains, CLI)              │
│ Cost Optimization:  ★★★★☆ (Zero markup, transparent pricing)    │
├─────────────────────────────────────────────────────────────────┤
│ BEST FOR: Event-driven PR automation, transparent cost model    │
│ WORST FOR: Parallel multi-module refactoring, real-time loops   │
│ 💡 TIP: Most transparent pricing of all platforms              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ GROQ + CREWAI / TOGETHER AI / REPLICATE                         │
├─────────────────────────────────────────────────────────────────┤
│ Price:              ★★★★★ ($0.30–0.90 / 1M Llama tokens)         │
│ Cloud Agents:       ★★☆☆☆ (No orchestration; use CrewAI)         │
│ Code Access:        ★★☆☆☆ (No sandbox; hand-roll execution)     │
│ GitHub Integration: ★☆☆☆☆ (None; must build custom)            │
│ Context Window:     ★★★★☆ (Llama 70B: 8K; supports longer)       │
│ Parallel Agents:    ★★★★☆ (CrewAI supports teams; Groq scales)   │
│ IDE Integration:    ★☆☆☆☆ (API-only, no IDE plugins)            │
│ Cost Optimization:  ★★★★☆ (Cheapest at >500 tasks/day)          │
├─────────────────────────────────────────────────────────────────┤
│ BEST FOR: Real-time agent loops, high-concurrency, cost at scale│
│ WORST FOR: Out-of-box agent orchestration, GitHub automation    │
│ 💡 TIP: Sub-100ms latency transforms agent loop performance    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Decision Tree

```
START: Choosing an AI Code Platform

  1. Do you code in an IDE daily?
     YES → Cursor (best UX)
     NO → Continue

  2. Do you need real-time latency (<500ms)?
     YES → Groq + CrewAI (1000+ tokens/sec)
     NO → Continue

  3. Is cost your primary concern?
     YES → Claude (small: batch + cache) or Groq (large: 1000+ tasks/day)
     NO → Continue

  4. Do you need event-driven PR automation?
     YES → Kilo Code (webhooks + GitHub App)
     NO → Continue

  5. Do you need parallel multi-agent execution?
     YES → Cursor (20 agents) or Claude (teams + worktrees)
     NO → Continue

  6. Is long context (>200K tokens) essential?
     YES → Claude Agent SDK
     NO → Continue

  DEFAULT → Claude Agent SDK (best all-rounder)

MIGRATION WARNING:
  • If using OpenAI Assistants → START MIGRATION NOW
    (Sunset: August 26, 2026)
```

---

## Head-to-Head Matchups

### Claude vs. Cursor (Code Refactoring)

**Scenario:** Refactor 5 interdependent modules (100K LOC total)

| Factor | Claude | Cursor | Winner |
|--------|--------|--------|--------|
| **Cost** | $5–10 (batch + cache) | $20–60 (credits) | Claude (10x) |
| **Speed** | Sequential (faster with extended thinking) | Parallel (5 modules simultaneously) | Cursor (5–10x faster) |
| **Context** | 1M tokens (understands all 5 modules) | Model-dependent (Claude 200K–1M) | Tie |
| **IDE Experience** | Standalone / CLI only | Native IDE (tab completion, diffs) | Cursor |
| **Evidence** | Structured output | Video demo + logs | Cursor |

**Verdict:** Cursor for speed/UX; Claude for cost. Ideal: **Use both** (Cursor for interactive, Claude for batch).

---

### Claude vs. Groq (High-Volume Code Analysis)

**Scenario:** Analyze 1000 code changes/day, classify risk level

| Factor | Claude | Groq | Winner |
|--------|--------|------|--------|
| **Cost** | $4.50/day (Haiku) | $1.05/day (Llama 70B) | Groq (4x cheaper) |
| **Latency** | 500–2000ms | 50–200ms (sub-100ms TTFT) | Groq (10x faster) |
| **Setup** | Simple SDK | Requires CrewAI + orchestration | Claude |
| **Model Quality** | Haiku (fast, accurate) | Llama 70B (good, less docs) | Claude (better docs) |
| **Scaling** | Linear cost increase | Linear cost increase | Tie |

**Verdict:** Groq for real-time loops; Claude for isolated analysis. Use **Groq at scale**.

---

### Cursor vs. Kilo (PR Automation)

**Scenario:** Auto-review 100 PRs/month via GitHub

| Factor | Cursor | Kilo | Winner |
|--------|--------|------|--------|
| **Setup Time** | 5 min (native) | 10 min (GitHub App) | Cursor |
| **Cost** | $20–60/mo (subscription) | $49/mo KiloClaw | Tie |
| **Parallelism** | Sequential (with clouds agents) | Sequential (single agent) | Cursor |
| **Transparency** | "Billed at API rates" (vague) | "Zero markup" (clear) | Kilo |
| **Webhook Triggers** | Limited (GitHub, Slack, Linear) | Full webhook support | Kilo |

**Verdict:** Kilo for transparent pricing + webhooks; Cursor for IDE-native. Use **Kilo if event-driven**.

---

## Feature Matrix: What Can Each Platform Do?

| Feature | Claude | OpenAI | Cursor | Kilo | Groq |
|---------|--------|--------|--------|------|------|
| **Read code files** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Write code files** | ✅ | ✅ | ✅ | ✅ (indirect) | ❌ |
| **Run shell commands** | ✅ | ✅ (Python only) | ✅ | ✅ (indirect) | ❌ |
| **Auto-create PR** | ⚠️ (via MCP) | ❌ | ✅ | ✅ | ❌ |
| **Comment on PR** | ⚠️ (via MCP) | ❌ | ✅ | ✅ | ❌ |
| **Parallel execution** | ⚠️ (teams) | ❌ | ✅ (20 agents) | ❌ | ✅ (via CrewAI) |
| **Cache large contexts** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Batch processing** | ✅ (50% off) | ⚠️ (limited) | ❌ | ❌ | ❌ |
| **Extended thinking** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Real-time IDE** | ❌ | ❌ | ✅ | ✅ | ❌ |
| **Webhook triggers** | ❌ | ❌ | ⚠️ | ✅ | ❌ |
| **Self-hosted option** | ❌ | ❌ | ✅ (GA) | ❌ | ⚠️ (Replicate) |

---

## Quick Cost Lookup

**For 100 code reviews (2K LOC, 8K tokens each):**

| Platform | Cost |
|----------|------|
| Claude batch + cache | $1.95 |
| Groq (if integrated) | $2.40 |
| Cursor | $28.90 |
| Kilo | $30–50 |
| OpenAI GPT-4 | $45 |

**For 1000 high-volume tasks (2K tokens each):**

| Platform | Cost/day |
|----------|----------|
| Groq Llama | $1.05 |
| Claude Haiku | $4.50 |
| Claude batch | $2.25 |
| Cursor (20 credit credit) | ~$0.89 |
| Kilo | ~$3–5 |

---

## Pitfalls to Avoid

| Pitfall | Platform | Fix |
|---------|----------|-----|
| **Hitting rate limits** | OpenAI | Use Claude (auto-scales) or Groq (burst-friendly) |
| **Assistants API sunset (Aug 26)** | OpenAI | Migrate to Responses API NOW |
| **Unexpected overspend** | Cursor | Set credit cap; monitor monthly usage |
| **Model lock-in** | Claude | Use adapter layer; allows easy migration |
| **No GitHub integration** | Groq, Claude SDK | Use Kilo or Cursor for automation; or build custom |
| **Real-time latency > 1s** | Claude | Use Groq + CrewAI; Claude OK for batch |
| **Missing caching** | All except Claude | Use Claude for RAG; cache large contexts |
| **Single-agent bottleneck** | Kilo, OpenAI | Use Cursor (20 agents) or Claude teams |

---

## Recommended Reading

| Topic | Link |
|-------|------|
| Full comparison table | `docs/research/AI_CODE_PLATFORMS_COMPARISON_2026.md` |
| Gap analysis & workarounds | `docs/research/CODE_PLATFORM_GAPS_AND_WORKAROUNDS.md` |
| Detailed cost examples | `docs/research/AI_CODE_PLATFORMS_COST_ANALYSIS.md` |
| Claude Agent SDK docs | https://platform.claude.com/docs/en/agent-sdk/overview |
| Cursor cloud agents | https://cursor.com/docs/cloud-agent |
| Kilo webhooks | https://blog.kilo.ai/p/cloud-agents-webhooks |
| Groq LPU latency | https://groq.com/ |
| OpenAI Assistants sunset | https://community.openai.com/t/assistants-api-beta-deprecation-august-26-2026-sunset/1354666 |

---

## Action Items by Platform Status

### Immediate (Next 30 Days)

- [ ] If using OpenAI Assistants: Begin migration to Responses API
- [ ] If considering Cursor: Test self-hosted cloud agents (newly GA)
- [ ] If at >500 tasks/day: Evaluate Groq + CrewAI cost vs. Claude

### Short-Term (60–90 Days)

- [ ] Implement prompt caching if using Claude (80–90% input cost savings)
- [ ] Set up batch processing pipeline for non-real-time tasks (50% discount)
- [ ] Build cost tracking dashboard (monitor all platforms in one place)

### Medium-Term (180 Days)

- [ ] Plan Claude vs. Groq hybrid strategy for your workload mix
- [ ] Evaluate Cursor self-hosted agents for enterprise deployment
- [ ] Benchmark token efficiency (tokens/task) across all platforms

---

## Contact & Support

| Platform | Support | Limits Increase |
|----------|---------|-----------------|
| Claude | https://support.anthropic.com | Auto-scale with spend; contact support for manual increase |
| OpenAI | https://help.openai.com | Manual limit increase via console |
| Cursor | help@cursor.com | Upgrade subscription tier |
| Kilo | https://kilo.ai/docs | Upgrade subscription plan |
| Groq | https://groq.com/support | Contact sales for enterprise |

