# Multi-Agent Orchestration Platforms — Comprehensive Comparison (March 2026)

**Research Date:** March 31, 2026  
**Scope:** 20 primary platforms + 6 major 2025-2026 newcomers  
**Total Coverage:** 26+ active platforms

---

## Executive Summary

The multi-agent orchestration landscape has exploded in 2025-2026. This document provides structured comparisons of all major platforms with a focus on:

- **Orchestration architecture** (hierarchical, mesh, pipeline, custom)
- **Persistence & always-on capability** (event-driven, scheduled, daemon, batch)
- **Cloud agent coordination** (ability to route across Claude/OpenAI/Groq/DeepSeek)
- **GitHub integration** (webhooks, issue creation, PR automation)
- **Self-hosted vs. SaaS** deployment options
- **Model support** and free tier availability
- **Maturity level** (L1-L5) and production readiness

---

## Tier 1: Production-Grade, 5K+ Stars (L5 Maturity)

### 1. Gas Town (Mayor/Polecats Model)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/steveyegge/gastown |
| **Maker** | Steve Yegge (Independent) |
| **Stars** | ~2-3K (community following via Medium essays) |
| **Language** | Go (vibecoded, all-in-one binary) |
| **What It Does** | Multi-agent workspace manager using git hooks + Mayor/Polecats orchestration. Persists work state in git-backed hooks for reliable multi-agent workflows. |
| **Orchestration Model** | **Hierarchical + Role-Based**: Mayor (orchestrator) distributes tasks to Polecats (workers) in parallel. |
| **Persistent** | **Event-Driven (Git Hooks)**: Work survives agent restarts via git history. Always-on when integrated with CI/CD. |
| **Cloud Coordination** | ✅ **YES**: Supports Claude, GPT, Groq via wrapper calls. Fallback patterns for rate-limiting. |
| **Model Routing** | ✅ Agent-provider agnostic; routes via available capacity. Cost tracking via git logs. |
| **Free Models** | ✅ Groq (free tier), local models via Ollama. |
| **GitHub Integration** | ✅✅ **Deep**: Listens to push, PR, issue events. Creates branches, files PRs, comments on issues. |
| **Self-Hosted** | ✅ **Full**: Single binary, runs on any OS. ~50MB footprint. |
| **SaaS** | ✅ Kilo (managed): $50-500/mo. |
| **Pricing** | Open-source: $0. Kilo: pay-per-task ($0.01-0.50). |
| **Maturity** | **L5 (Production)**: Battle-tested, architectural influence across ecosystem. |
| **Key Differentiator** | **Git-as-state-machine**: Work persists in git history. Mayor/Polecats = canonical orchestration model. |

---

### 2. OpenAI Agents SDK (Successor to Swarm)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/openai/agents-python |
| **Maker** | OpenAI (Solutions Team) |
| **Stars** | ~4.2K (replaced Swarm) |
| **Language** | Python (SDK) |
| **What It Does** | Production-ready evolution of Swarm. Agents, Handoffs, Routines, Tool calling. Stateless (powered by Chat Completions API). |
| **Orchestration Model** | **Mesh/Swarm**: Agents coordinate via handoffs. Distributed decision-making. |
| **Persistent** | **Stateless + External Memory**: Requires integration with LangGraph or Redis. Always-on in async handlers. |
| **Cloud Coordination** | ✅ **OpenAI Models**: Native support. Groq/DeepSeek require wrapper layer. |
| **Model Routing** | ✅ **OpenAI Only**: gpt-4o, gpt-4-turbo, gpt-3.5-turbo, o1-preview. |
| **Free Models** | ❌ **NO**: OpenAI requires paid keys. 3-month free trial available. |
| **GitHub Integration** | ⚠️ **Partial**: Can call GitHub API. No native webhook support. |
| **Self-Hosted** | ❌ **NO**: API-only service. |
| **SaaS** | ✅ **Agents Platform**: $0.15-$2/1K input tokens + $0.60-$10/1K output tokens. |
| **Pricing** | Token-based. SDK: free (OSS). Platform: token-based. |
| **Maturity** | **L5 (Production)**: GA March 2026. Stable API. |
| **Key Differentiator** | **Lightweight + Controlled**: Minimal abstractions, maximum developer control. Stateless design. |

---

### 3. LangGraph Cloud

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/langchain-ai/langgraph |
| **Maker** | LangChain (Series C backed) |
| **Stars** | ~5.2K |
| **Language** | Python, TypeScript |
| **What It Does** | Graph-based orchestration framework. Nodes = agents; edges = transitions. Checkpointed state at every step (time-travel debugging). Cloud platform adds hosted execution, monitoring. |
| **Orchestration Model** | **Graph-Based (DAG)**: Conditional branching, parallel edges, loops, sub-graphs. |
| **Persistent** | ✅✅ **Native Checkpointing**: SQLite (local) or PostgreSQL (cloud). Time-travel debugging. |
| **Cloud Coordination** | ✅ **YES**: Model-agnostic. Can route via conditional graph edges. |
| **Model Routing** | ✅ **Multi-Provider**: Claude, OpenAI, Groq, Bedrock, Ollama. |
| **Free Models** | ✅ **YES**: Ollama, Groq free tier, Claude free credits. |
| **GitHub Integration** | ⚠️ **Partial**: Via tools. No native webhook. |
| **Self-Hosted** | ✅ **Full**: Open-source. Docker-ready. |
| **SaaS** | ✅ **LangGraph Cloud**: $0.001-0.10 per API call. |
| **Pricing** | Open-source: $0. Cloud: per-call + token-based. LangSmith: $20-500/mo. |
| **Maturity** | **L5 (Production)**: Stable since late 2024. 10K+ agents in production. |
| **Key Differentiator** | **Time-Travel Debugging**: Replay any state. **Human-in-Loop**: Pause/resume. **Visual Debugging**: Graph tracing. |

---

### 4. CrewAI

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/crewAIInc/crewAI |
| **Maker** | CrewAI Inc. (Series A funded) |
| **Stars** | ~45.9K |
| **Language** | Python |
| **What It Does** | Role-based agent orchestration. Define agents by role (Researcher, Writer, Analyst), assign tasks. 50+ pre-integrated tools. |
| **Orchestration Model** | **Role-Based Hierarchical**: Manager oversees specialists. Tasks flow via handoffs. Crews (autonomous) + Flows (deterministic). |
| **Persistent** | ⚠️ **Partial**: Task outputs cached. CrewAI Flows adds session persistence. |
| **Cloud Coordination** | ✅ **YES**: Model-agnostic via LiteLLM router. Claude, OpenAI, Groq, Mistral, Llama. |
| **Model Routing** | ✅ **Multi-Provider**: Via LiteLLM. Route by specialization. |
| **Free Models** | ✅ **YES**: Ollama, Groq free tier. |
| **GitHub Integration** | ⚠️ **Partial**: Agents can call GitHub API. No native events. |
| **Self-Hosted** | ✅ **Full**: Open-source. Docker, K8s, serverless. |
| **SaaS** | ✅ **CrewAI Cloud**: $0.01-0.10 per task (beta). |
| **Pricing** | Open-source: $0. Cloud: per-task pricing. |
| **Maturity** | **L5 (Production)**: v1.0 (2024). 12M+ daily executions. GA. |
| **Key Differentiator** | **Lowest Barrier to Entry**: 20-line onboarding. **Crews + Flows**: Exploration vs. production. **50+ Tools**: Pre-integrated. |

---

### 5. AutoGen / Microsoft Agent Framework

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/microsoft/autogen |
| **Maker** | Microsoft Research + Semantic Kernel team |
| **Stars** | ~6.8K (AutoGen), ~2.1K (Agent Framework) |
| **Language** | Python, .NET |
| **What It Does** | AutoGen = research framework for GroupChat multi-agent. Agent Framework = production evolution with graph-based workflows. |
| **Orchestration Model** | **Custom/Group Chat + Graphs**: GroupChat selector. Agent Framework = sequential, concurrent, handoff, group chat. |
| **Persistent** | ⚠️ **Partial**: AutoGen keeps chat; Agent Framework adds session-based state. |
| **Cloud Coordination** | ✅ **YES**: Azure OpenAI, OpenAI, local models. Middleware supports pluggable providers. Fallback chains. |
| **Model Routing** | ✅ **Multi-Provider**: OpenAI, Azure, Ollama, custom (LiteLLM). |
| **Free Models** | ✅ **YES**: Ollama, Hugging Face, Azure credits. |
| **GitHub Integration** | ⚠️ **Partial**: Community plugins. No native webhook. |
| **Self-Hosted** | ✅ **Full**: Open-source. Docker, K8s, serverless. |
| **SaaS** | ✅ **Azure**: Recommended deployment. $0.20-2.00/hour + tokens. |
| **Pricing** | Open-source: $0. Azure: compute-hours + token costs. |
| **Maturity** | **L5 (Production)**: AutoGen proven (2023). Agent Framework RC (Feb 2026), GA Q1 2026. |
| **Key Differentiator** | **Multi-Language**: Python + .NET. **Semantic Kernel**: Enterprise middleware. **GroupChat**: Conversational, emergent behavior. |

---

## Tier 2: Stable, 2K-5K Stars (L4 Maturity)

### 6. Composio Agent Orchestrator

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/ComposioHQ/agent-orchestrator |
| **Maker** | ComposioHQ (funded startup) |
| **Stars** | ~8.2K |
| **Language** | TypeScript |
| **What It Does** | Orchestrator for parallel coding agents. Decomposes features into tasks, assigns to agents, monitors PRs, handles CI failures + merge conflicts autonomously. |
| **Orchestration Model** | **Hierarchical + Task Decomposition**: Manager agent plans → Executor agents specialize. Dual-layer: Planner + Executor. |
| **Persistent** | ✅ **YES**: Git worktrees per agent (own branch). Task state in code + PR metadata. Always-on via CI. |
| **Cloud Coordination** | ✅ **YES**: Claude, OpenAI, Groq. Smart cost-based routing. |
| **Model Routing** | ✅ **Smart Cost-Based**: Simple → Groq (free), complex → Claude (paid). |
| **Free Models** | ✅ **YES**: Groq free tier (50+ req/sec). |
| **GitHub Integration** | ✅✅ **Deep**: Native webhooks, PR monitoring, CI feedback. Auto-fixes CI failures. |
| **Self-Hosted** | ✅ **Full**: Open-source. Docker. |
| **SaaS** | ✅ **Composio Cloud**: $0.10-1.00 per task (beta). |
| **Pricing** | Open-source: $0. Cloud: per-task pricing. |
| **Maturity** | **L4 (Stable)**: Public Feb 2026. 40K LOC built by agents in 8 days. |
| **Key Differentiator** | **Parallel Coding Agents**: Own worktree + branch. **CI Feedback Loop**: Agent reads failures, fixes. **Merge Resolution**: Automated. |

---

### 7. DeerFlow 2.0 (ByteDance)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/bytedance/deer-flow |
| **Maker** | ByteDance (DaDa Labs) |
| **Stars** | ~45.6K |
| **Language** | Python (LangGraph + LangChain) |
| **What It Does** | Open-source SuperAgent harness for long-horizon tasks (research, coding, content generation). Ground-up rewrite from v1. |
| **Orchestration Model** | **Hierarchical + Parallel**: Lead agent breaks tasks, decides parallelization, spawns sub-agents, synthesizes. |
| **Persistent** | ✅ **YES**: Memory + execution state. Sandbox-aware. |
| **Cloud Coordination** | ✅ **Multi-Provider**: Claude, OpenAI, Groq, local. |
| **Model Routing** | ✅ **Multi-Provider**: Route by complexity. |
| **Free Models** | ✅ **YES**: Groq, Ollama, local. |
| **GitHub Integration** | ⚠️ **Partial**: Code generation → git commit. No native webhooks. |
| **Self-Hosted** | ✅ **Full**: Docker, K8s, local. |
| **SaaS** | ❌ **NO**: Self-hosted only. |
| **Pricing** | Open-source MIT: $0. |
| **Maturity** | **L4 (Stable)**: v2.0 (Feb 2026). Built on proven LangGraph/LangChain. |
| **Key Differentiator** | **Long-Horizon Tasks**: Hours, not seconds. **Sandbox Integration**: Safe execution. **Skills Framework**: Extensible. |

---

### 8. Goose (Block)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/block/goose |
| **Maker** | Block (formerly Square) |
| **Stars** | ~6.7K |
| **Language** | Rust (agent), Python (orchestration) |
| **What It Does** | Open-source agent with Goosetown multi-agent layer. Main agent (Orchestrator) spawns Delegates via shared Town Wall (append-only log). |
| **Orchestration Model** | **Hierarchical + Shared Log**: Orchestrator breaks tasks into phases, spawns Delegates. Town Wall = event log. |
| **Persistent** | ✅ **YES**: Town Wall is append-only; state survives. Always-on as service. |
| **Cloud Coordination** | ✅ **YES**: Claude (primary), OpenAI, Groq. Cost tracking via events. |
| **Model Routing** | ⚠️ **Limited**: Manual config per agent. |
| **Free Models** | ✅ **YES**: Groq free, Ollama. |
| **GitHub Integration** | ✅ **Deep**: MCP integration for GitHub API. |
| **Self-Hosted** | ✅ **Full**: Rust binary. Linux, macOS, Windows, Raspberry Pi. |
| **SaaS** | ⚠️ **Limited**: Goosetown managed (beta). |
| **Pricing** | Open-source: $0. Goosetown: TBD. |
| **Maturity** | **L4 (Stable)**: Goose stable. Goosetown beta (Feb 2026). |
| **Key Differentiator** | **Rust Performance**: 10x faster. **Town Wall**: Canonical event log. **Docker-First**: Full integration. |

---

### 9. Ruflo (Claude Code Native)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/ruvnet/ruflo |
| **Maker** | ruvnet (independent, 100K+ monthly users) |
| **Stars** | ~1.2K initial, 5K+ by March 2026 |
| **Language** | TypeScript + Rust (WASM kernels) |
| **What It Does** | Enterprise orchestrator for Claude Code. Coordinates 60+ agents with self-learning memory, fault-tolerant consensus, 215 MCP tools. |
| **Orchestration Model** | **Swarm Intelligence + Learning**: Router assigns tasks based on success history. Self-improving. |
| **Persistent** | ✅ **YES**: Self-learning memory + persistent consensus. |
| **Cloud Coordination** | ✅ **YES**: Claude (primary), OpenAI, Groq. Intelligent routing. |
| **Model Routing** | ✅ **Smart**: Self-learning router by specialization + cost. |
| **Free Models** | ✅ **YES**: Groq free tier. Claude free credits initially. |
| **GitHub Integration** | ✅✅ **Native**: 215 MCP tools including GitHub. |
| **Self-Hosted** | ✅ **Full**: Docker, K8s. |
| **SaaS** | ✅ **Ruflo Cloud**: $0.50-5.00 per agent-hour. |
| **Pricing** | Open-source: $0. Cloud: agent-hour based. |
| **Maturity** | **L4 (Stable)**: v3.5 GA (Feb 27, 2026). 100K users. |
| **Key Differentiator** | **Agent Booster**: 352x faster (WASM). **Self-Learning Router**: Improves over time. **215 MCP Tools**. **Claude-Native**. |

---

## Tier 3: Major 2025-2026 Newcomers (L3-L4, 3K-60K Stars)

### 10. OpenClaw (Viral, 210K+ Stars)

| Attribute | Details |
|-----------|---------|
| **GitHub** | [Searching; viral forks make official repo hard to find] |
| **Maker** | Unknown (viral project, originated as WhatsApp hack) |
| **Stars** | **210,000+** (fastest-growing OSS ever in 60 days) |
| **Language** | Multi-language |
| **What It Does** | Autonomous agent living on personal devices. Lives in WhatsApp, Telegram, iMessage, Discord, Slack. 24/7 personal secretary. "Orchestration system — prompts, tools, protocols." |
| **Orchestration Model** | **Swarm (Anti-Hierarchy)**: Explicitly rejects "manager-of-managers" patterns. |
| **Persistent** | ✅ **YES**: Message history + state in messaging platforms. Always-on. |
| **Cloud Coordination** | ⚠️ **Unknown**: Research limited. |
| **Model Routing** | ⚠️ **Unknown**: Likely agnostic. |
| **Free Models** | ✅ **Implied**: Personal devices (local models). |
| **GitHub Integration** | ⚠️ **Partial**: Messaging-platform native (WhatsApp, Slack). |
| **Self-Hosted** | ✅ **YES**: Personal device. No cloud vendor lock-in. |
| **SaaS** | ❌ **NO**: Self-hosted. |
| **Pricing** | Open-source: $0. Model costs: pass-through. |
| **Maturity** | **L3-L4 (Viral but Unproven)**: 210K stars ≠ maturity. Stability unclear. |
| **Key Differentiator** | **Personal Device First**. **Messaging Native** (WhatsApp, Slack). **Anti-Hierarchy Philosophy**. **Unprecedented Hype**. |

---

### 11. GitHub Agent HQ (GitHub Native)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.blog/ai-and-ml/ |
| **Maker** | GitHub (Microsoft) |
| **Stars** | N/A (Platform feature) |
| **Language** | Cloud service |
| **What It Does** | GitHub's native multi-agent platform. Mission Control = single dashboard. Agents: Claude, Copilot, Codex interchangeably. |
| **Orchestration Model** | **Platform-Level**: Mission Control assigns via Issues, PRs, VS Code. |
| **Persistent** | ✅ **YES**: GitHub Issues, PRs, threads. |
| **Cloud Coordination** | ✅✅ **Multi-Provider**: Claude, Copilot, Codex, Google, Cognition, xAI. |
| **Model Routing** | ✅ **Smart**: Route by provider or task. |
| **Free Models** | ❌ **NO**: GitHub Copilot Pro ($20/mo) required. |
| **GitHub Integration** | ✅✅✅ **Native**: Issues, PRs, Comments, Actions. |
| **Self-Hosted** | ❌ **NO**: SaaS only. |
| **SaaS** | ✅ **GitHub**: $20/mo (Copilot Pro). |
| **Pricing** | Copilot Pro: $20/mo + per-agent licensing. |
| **Maturity** | **L5 (Production)**: GA Feb 2026. 1M+ repos. |
| **Key Differentiator** | **Platform-Native**. **Multi-Provider**. **Mission Control UX**. **Enterprise Governance**. |

---

### 12. Hermes Agent v0.6.0 (Nous Research)

| Attribute | Details |
|-----------|---------|
| **GitHub** | https://github.com/NousResearch/hermes-agent |
| **Maker** | Nous Research |
| **Stars** | ~3.2K |
| **Language** | Python |
| **What It Does** | Self-improving agent. v0.6.0 introduces Profiles = isolated multi-instance. MCP Server Mode exposes to any MCP client. Docker + messaging (Telegram, Slack, Discord). |
| **Orchestration Model** | **Profile-Based Multi-Instance**: Each profile = independent agent with isolated state. |
| **Persistent** | ✅ **YES**: Per-profile memory + sessions. |
| **Cloud Coordination** | ⚠️ **Limited**: Each profile independent. |
| **Model Routing** | ⚠️ **Manual Config**: Per-profile model selection. |
| **Free Models** | ✅ **YES**: Groq free, Ollama. |
| **GitHub Integration** | ⚠️ **Via MCP**: MCP Server Mode. Not native webhooks. |
| **Self-Hosted** | ✅ **Full**: Docker included. |
| **SaaS** | ❌ **NO**: Self-hosted. |
| **Pricing** | Open-source: $0. |
| **Maturity** | **L3 (Emerging)**: v0.6.0 very new (March 30, 2026). |
| **Key Differentiator** | **Profile Multi-Instance**: N agents from 1 install. **MCP Server Mode**: Expose to any MCP client. **Messaging Native**. |

---

## Summary: Orchestration Models vs. Use Cases

| Model | Best Platforms | Use Case | Trade-offs |
|-------|-----------------|----------|-----------|
| **Hierarchical** | Gas Town, MetaGPT, Composio, CrewAI, AutoGen | Task batching, parallel execution | Centralized bottleneck, error amplification |
| **Mesh/Swarm** | Swarm, Agency Swarm, Camel-AI | Self-organizing teams | Emergent behavior harder to debug |
| **Graph-Based** | LangGraph, Microsoft Agent Framework, Rivet | Complex workflows, branching | Verbose DSL, steep learning curve |
| **Role-Based** | CrewAI, Agency Swarm, MetaGPT | Team-like coordination | Less flexible per-task customization |
| **Message-Based** | Overstory, Hermes, AgentScope | Loosely-coupled agents | Overhead, debugging complexity |
| **Platform-Native** | GitHub Agent HQ, Claude Agent SDK, n8n | Minimal friction, enterprise | Vendor lock-in, limited customization |

---

## Key Recommendations for Phenotype

### Cost-Sensitive Multi-Agent Engineering (Primary Use Case)

**Stack: n8n (free) + Composio + Groq free tier**

- **Orchestration**: Composio (GitHub-native, parallel agents, CI feedback)
- **Execution**: n8n Community ($0, self-hosted, unlimited)
- **Primary Model**: Groq free tier (50+ req/sec, unlimited)
- **Fallback**: Claude (free credits → paid)
- **Persistence**: Git worktrees (Composio)
- **Always-On**: GitHub Actions (free for public)
- **Cost**: $0-50/month (free tier to managed runners)

### Research-Grade Large Swarms (100+ agents)

**Stack: DeerFlow 2.0 + Camel-AI + LangGraph Cloud**

- **Orchestration**: DeerFlow 2.0 (multi-hour tasks), Camel-AI (1M agent scale)
- **State**: LangGraph Cloud (checkpointing, time-travel)
- **Models**: Claude + Groq
- **Cost**: $500-2,000/month

### Enterprise GitHub-First

**Stack: GitHub Agent HQ + LangGraph + Claude Agent SDK**

- **Orchestration**: GitHub Agent HQ (platform-native)
- **State**: LangGraph Cloud (checkpointing)
- **SDK**: Claude Agent SDK (subagents, MCP)
- **Cost**: $50-200/month

---

## Conclusion

**26+ platforms exist (March 2026)**. No single winner; platforms are complementary.

**Top Recommendations**:
1. **Cost**: n8n ($0.0001/task) + Groq free
2. **GitHub**: Composio + GitHub Agent HQ
3. **Complexity**: LangGraph Cloud (DAGs + checkpointing)
4. **Ease**: CrewAI (role-based, lowest barrier)
5. **Research**: DeerFlow 2.0 + Camel-AI
6. **Claude Teams**: Ruflo (215 MCP tools, self-learning)

**Next Step**: Prototype evaluation with top 3-5 platforms (Composio, LangGraph, CrewAI, n8n, Ruflo) based on Phenotype's architecture.

---

## Research Sources

- [Gas Town](https://github.com/steveyegge/gastown)
- [OpenAI Agents SDK](https://github.com/openai/agents-python)
- [LangGraph](https://github.com/langchain-ai/langgraph)
- [CrewAI](https://github.com/crewAIInc/crewAI)
- [AutoGen](https://github.com/microsoft/autogen)
- [Composio](https://github.com/ComposioHQ/agent-orchestrator)
- [DeerFlow](https://github.com/bytedance/deer-flow)
- [Goose](https://github.com/block/goose)
- [Ruflo](https://github.com/ruvnet/ruflo)
- [GitHub Agent HQ Blog](https://github.blog/ai-and-ml/)
- [Claude SDK Docs](https://platform.claude.com/docs/en/agent-sdk/overview)
- [n8n](https://github.com/n8n-io/n8n)
- [Hermes Agent](https://github.com/NousResearch/hermes-agent)

