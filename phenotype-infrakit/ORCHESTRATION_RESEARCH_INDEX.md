# Multi-Agent Orchestration Research — Quick Index (March 2026)

**Full Document**: `/Users/kooshapari/CodeProjects/Phenotype/repos/MULTI_AGENT_ORCHESTRATION_COMPARISON_2026.md` (383 lines)

---

## Quick Navigation

### Tier 1 Platforms (L5, Production-Grade, 5K+ Stars)
1. **Gas Town** — Git-backed, Mayor/Polecats model, $0 open-source
2. **OpenAI Agents SDK** — Swarm-based handoffs, stateless, paid-only
3. **LangGraph Cloud** — Graph DAGs, time-travel debugging, best for complexity
4. **CrewAI** — Role-based, 20-line onboarding, easiest to learn
5. **AutoGen / Microsoft Agent Framework** — GroupChat + graphs, multi-language

### Tier 2 Platforms (L4, Stable, 2K-5K Stars)
6. **Composio** — GitHub-native, parallel agents, CI feedback loops (RECOMMENDED FOR PHENOTYPE)
7. **DeerFlow 2.0** — Long-horizon tasks (hours), sandbox-safe, LangGraph-based
8. **Goose** — Rust performance, Town Wall event log, MCP integration
9. **Ruflo** — Claude Code native, 215 MCP tools, self-learning router
10. **Overstory** — tmux + SQLite mail, 11 runtime adapters, honest about risks

### Tier 3 Platforms (L3, Emerging, <2K Stars)
11. **Agency Swarm** — OpenAI Assistants API, org chart DSL
12. **ChatDev 2.0** — Zero-code visual canvas, DevAll platform
13. **Camel-AI** — Research-grade, 1M agent simulations (OWL, OASIS)
14. **AgentScope** — Multi-language, CoPaw workstation, Alibaba backing
15. **TaskWeaver** — Code-first, sandboxed execution, data analytics focus

### Tier 4 Newcomers (L3-L4, 2025-2026, 3K-60K Stars)
16. **OpenClaw** — 210K stars (fastest-growing ever), messaging-native, viral but unproven
17. **DeerFlow 2.0** — 45.6K stars, multi-hour tasks, ByteDance backing
18. **GitHub Agent HQ** — Platform-native, multi-provider (Claude, GPT, Codex), $20/mo
19. **Hermes Agent v0.6.0** — Profiles (multi-instance), MCP Server Mode, March 2026 release
20. **n8n AI Agent Node** — $0.0001/task execution pricing, unlimited self-hosted, workflow integration
21. **AWS Agent Squad** — AWS-native, Bedrock integration, intent routing

### Additional Platforms
22. **Rivet (Ironclad)** — Visual prompt chaining, AI Graph Creator, thinking models
23. **Claude Agent SDK** — Native Anthropic solution, Agent Teams (experimental), MCP tools
24. **oh-my-claudecode** — Claude Code native, zero-config, 3-5x speedup (emerging)
25. **MetaGPT** — SOP materialization, ICLR 2025 paper, end-to-end generation
26. **Swarms (Kyegomez)** — Enterprise-grade, 10+ swarm patterns, 6.1K stars

---

## Quick Decision Matrix

### Best For Cost-Sensitive Teams
**Recommendation**: **Composio + n8n + Groq free tier**
- Composio: GitHub-native orchestration ($0 OSS)
- n8n: $0.0001 per task execution (free self-hosted)
- Groq: 50+ req/sec free tier (unlimited)
- **Total**: $0-50/month (free tier to managed runners)

### Best For GitHub-Heavy Teams
**Recommendation**: **Composio** (native integration, parallel agents, CI feedback)
- Alternative: GitHub Agent HQ (platform-native but $20/mo minimum)

### Best For Learning / Prototyping
**Recommendation**: **CrewAI** (20-line onboarding, role-based, lowest barrier)

### Best For Complex Workflows
**Recommendation**: **LangGraph Cloud** (graph DAGs, time-travel debugging, checkpointing)

### Best For Long-Running Tasks (Multi-Hour)
**Recommendation**: **DeerFlow 2.0** (sandbox-safe, memory + skills framework)

### Best For Enterprise SaaS
**Recommendation**: **GitHub Agent HQ** (governance, multi-provider, platform-native)

### Best For Research / 1M Agent Simulations
**Recommendation**: **Camel-AI** (OWL learning, OASIS simulations)

### Best For Personal Automation (Non-Code)
**Recommendation**: **OpenClaw** (messaging-native: WhatsApp, Slack, Telegram)

---

## Orchestration Model Comparison

| Model | Example Platforms | Pros | Cons |
|-------|-------------------|------|------|
| **Hierarchical** | CrewAI, MetaGPT, Composio, Gas Town | Clear structure, parallelizable | Manager bottleneck, error amplification |
| **Swarm/Mesh** | Swarm (OpenAI), Agency Swarm, Camel-AI | Self-organizing, resilient | Harder to debug, emergent behavior |
| **Graph-Based** | LangGraph, Rivet, Microsoft Agent Framework | Explicit branching, time-travel debugging | Verbose DSL, steep learning curve |
| **Message-Based** | Overstory, Hermes, AgentScope | Loosely-coupled, scalable | Overhead, protocol design |
| **Platform-Native** | GitHub Agent HQ, Claude Agent SDK | Minimal friction, zero setup | Vendor lock-in, limited customization |

---

## Persistence / Always-On Capability

| Approach | Platforms | Details |
|----------|-----------|---------|
| **Native Checkpointing (Time-Travel)** | LangGraph Cloud, TaskWeaver | State at every step, debuggable history |
| **Git-Backed** | Gas Town, Composio | Code as state; versioned, mergeable |
| **Message-Based (Event Log)** | Overstory (SQLite), Goose (Town Wall), Hermes | Append-only events; replay-able |
| **Session-Based** | CrewAI Flows, AutoGen, AgentScope | External DB (SQLite, Postgres, Redis) |
| **Stateless + External** | OpenAI Agents SDK, Swarm | Requires external memory layer |

---

## Cloud Agent Coordination (Multi-Provider Routing)

| Framework | Support | Notes |
|-----------|---------|-------|
| **Native Multi-Provider** | LangGraph (conditional edges), Composio (smart routing), n8n (fallback chains) | Route Claude + OpenAI + Groq dynamically |
| **LiteLLM Router** | CrewAI, Agency Swarm, AutoGen, Camel-AI | Wrapper layer; flexible but adds latency |
| **Manual Config** | Ruflo, Hermes, Overstory | Specify per-agent model; no intelligent routing |
| **Single-Provider** | Claude Agent SDK, GitHub Agent HQ, OpenAI Agents SDK | Primary provider only; custom fallbacks required |
| **Groq Free Tier Preferred** | Composio, n8n, CrewAI | Explicitly route simple tasks to Groq (free), complex to paid |

---

## GitHub Integration Depth

| Level | Platforms | Capability |
|-------|-----------|-----------|
| **Native Webhook + Auto-PR** | Composio, Gas Town, n8n | GitHub Events → Agent → PR (fully automated) |
| **Deep Issue/PR Manipulation** | GitHub Agent HQ, Composio, CrewAI (tools) | Read/write issues, manage workflows, auto-comment |
| **API Tools Only** | LangGraph, AutoGen, ChatDev, TaskWeaver | Agents call GitHub API manually; no webhooks |
| **MCP Integration** | Goose, Claude Agent SDK, Hermes (v0.6.0+) | Expose to MCP clients; not native GitHub integration |
| **None** | Some older frameworks | No native GitHub capability |

---

## Cost Analysis (Estimated Monthly for 10-Agent Engineering Team)

| Stack | Components | Cost |
|-------|-----------|------|
| **Ultra-Low (Free Tier)** | n8n community + Composio + Groq free | $0 (self-hosted, unlimited Groq requests) |
| **Lean (Groq + Free Credits)** | CrewAI + Groq (free) + Claude (free credits 3mo) | $0-50 (free credits exhaust in weeks) |
| **Standard (Groq + Paid)** | Composio + Groq (free) + Claude (paid overflow) | $50-200/month (Claude overages) |
| **Premium (Multi-Provider)** | LangGraph Cloud + Claude + OpenAI + Groq | $200-500/month (checkpointing + tokens) |
| **Enterprise (Platform)** | GitHub Agent HQ + multiple agents | $500-2,000/month (platform fee + tokens) |

---

## Maturity Levels (L1-L5)

| Level | Definition | Examples | Recommendation |
|-------|-----------|----------|-----------------|
| **L5** | GA, 5K+ stars, commercial backing, 1+ year stable API | LangGraph, CrewAI, AutoGen, GitHub Agent HQ | ✅ Safe for production |
| **L4** | Stable, 2K-5K stars, active maintenance, production use | Composio, DeerFlow, Ruflo, Overstory, TaskWeaver | ✅ Production-ready; smaller communities |
| **L3** | Emerging, <2K stars, active dev, growing adoption | ChatDev 2.0, Camel-AI, Agency Swarm, Hermes | ⚠️ Use with caution; API may change |
| **L2** | Early POC, <500 stars, experimental features | Various 2026 launches, oh-my-claudecode | ❌ Research-only; not production-ready |
| **L1** | Concept, <100 stars, proof-of-concept | Academic projects, side projects | ❌ Highly experimental |

**Note**: OpenClaw = 210K stars (L1-L4 hype mismatch); viral ≠ mature. Recommend wait-and-see.

---

## Top 3 Recommendations for Phenotype

### #1: Composio (Best Overall for GitHub-Heavy Engineering)
- GitHub-native orchestration (webhooks, PR creation, CI feedback)
- Parallel agents (own git worktree + branch per agent)
- Smart cost routing (Groq free → Claude paid)
- L4 maturity, strong funding, proven in production
- **Cost**: $0 (self-hosted OSS)

### #2: LangGraph Cloud (Best for Complexity + Debugging)
- Graph-based DAGs with conditional branching
- Time-travel debugging (replay any state)
- Native checkpointing (SQLite/Postgres)
- L5 maturity, largest OSS community (LangChain ecosystem)
- **Cost**: $0.001-0.10 per API call + token costs

### #3: CrewAI (Best for Rapid Prototyping)
- Role-based, intuitive 20-line onboarding
- Crews (autonomous) + Flows (deterministic) duality
- 50+ pre-integrated tools
- L5 maturity, 45.9K GitHub stars
- **Cost**: $0 (self-hosted); $0.01-0.10 per task (cloud beta)

---

## 2025-2026 Newcomers to Watch

| Platform | Hype | Reality | Verdict |
|----------|------|---------|---------|
| **OpenClaw** | 210K stars (viral) | Uncertain; messaging-native angle novel | Wait-and-see; hype may not equal quality |
| **DeerFlow 2.0** | 45.6K stars, ByteDance backing | Solid LangGraph foundation; multi-hour focus | ✅ Recommend; proven tech stack |
| **GitHub Agent HQ** | Enterprise hype | GA Feb 2026; deep GitHub integration; expensive | ✅ Recommend for GitHub-heavy enterprises |
| **Ruflo v3.5** | 100K users, strong adoption | Self-learning, 215 MCP tools, Claude-native | ✅ Recommend for Claude teams |
| **Hermes v0.6.0** | Profiles feature new | Multi-instance orchestration, emerging use case | ⚠️ Watch; good alternative to Gas Town |

---

## Phenotype Integration Roadmap

### Phase 1 (Weeks 1-2): Proof-of-Concept
- Implement minimal Composio setup (2-3 agents, GitHub integration)
- Compare execution time + cost vs. manual development
- Validate CI/CD feedback loop

### Phase 2 (Weeks 3-4): Production Pilot
- Add LangGraph Cloud for state persistence (checkpointing)
- Implement multi-provider routing (Groq free + Claude paid)
- Deploy to private repo (GitHub Actions runners)

### Phase 3 (Weeks 5-8): Full Deployment
- Scale to 10-20 parallel agents
- Integrate with AgilePlus spec tracking
- Monitor cost, accuracy, merge conflict rate

### Phase 4 (Ongoing): Optimization
- Route complex tasks to Claude, simple to Groq
- Tune agent prompts based on execution history
- Migrate to Ruflo if self-learning benefits appear

---

## Research Artifacts

- **Full Comparison Document**: 383 lines, 12 tiers, 26+ platforms
- **Decision Matrix**: Cost, GitHub integration, complexity, maturity
- **Recommendation Stack**: Composio + LangGraph + CrewAI for different use cases
- **2026 Newcomer Analysis**: OpenClaw, DeerFlow 2.0, GitHub Agent HQ, Ruflo, Hermes
- **Cost Breakdown**: Estimated monthly costs for typical engineering teams

---

## Key Sources

- GitHub repositories (verified stars, last commit dates, March 2026)
- Official documentation + blogs
- Research papers (ICLR 2025, arXiv agent papers)
- Community adoption (HN, Twitter, GitHub discussions)
- Commercial announcements (funding, releases, product launches)

