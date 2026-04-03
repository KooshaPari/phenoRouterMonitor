# Session: Tooling Stack, VPS Research, and Lightweight Agent Alternatives
**Date**: 2026-03-31
**Session ID**: phenotype-20260331-tooling-vps-agents
**Mac Hardware**: 16GB RAM, 10-core Apple Silicon (MacBook Pro)

---

## Context: What Was Discussed

This session covered three major topics:
1. **Complete tooling stack** evaluation for Phase 2 (CI/CD, coverage, error tracking, SAST)
2. **VPS/Cloud research** for $20/mo self-hosted infrastructure
3. **Lightweight agent alternatives** for running 100+ instances on local Mac

---

## Part 1: Tooling Stack Recommendations

### CI/CD Alternatives (GitHub Actions billing exhausted)

**Problem**: GitHub Actions billing exhausted ($863/mo need, $450 free tier, 65 agents exhausting instantly)

| Tool | Robustness | Power | Cost | Best For |
|------|-------------|-------|------|----------|
| **CircleCI** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 6K min/mo free, $0.003/min over | **GitHub repos** |
| **Woodpecker CI** | ⭐⭐⭐⭐ | ⭐⭐⭐ | Free (self-hosted) | **GitHub + GitLab + Bitbucket** |
| **Forgejo Actions** | ⭐⭐⭐ | ⭐⭐⭐ | Free | **Pure self-hosted** |
| **GitHub Actions + Local Runners** | ⭐⭐⭐⭐ | ⭐⭐⭐ | Free (hardware) | **Bridge solution** |

**Recommendation**: CircleCI for GitHub repos + Woodpecker for GitLab/Bitbucket

### Code Coverage

| Tool | Robustness | Power | Cost | Best For |
|------|-------------|-------|------|----------|
| **SonarCloud** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Free for OSS | **Most powerful** |
| **Coveralls** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Free for OSS | **Coverage-focused** |
| **GitHub Native** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Free | **Most robust** |

**Recommendation**: SonarCloud for deep analysis (GitHub App one-click install)

### SAST (Already Deployed)
- Semgrep + CodeQL working across 30 repos
- Snyk partially deployed (token issues resolved)

### Error Tracking
- Sentry: Token regenerated and working (organization: stealth-startup-3u)

---

## Part 2: VPS Research ($20/Month)

### Free Tier Comparison

| Provider | Free Tier | Specs | Persistent? | Suitable for CI? |
|----------|-----------|-------|-------------|-----------------|
| **Oracle Cloud** | ✅ Forever | 4 cores, 24GB RAM, 200GB SSD | ✅ Yes | ✅ **Yes — best free** |
| **AWS** | ⚠️ 1 year only | t2.micro (1GB RAM) | ✅ Yes | ⚠️ Too slow |
| **GCP** | ⚠️ e2-micro forever | 0.25 vCPU, 1GB RAM | ✅ Yes | ⚠️ Too slow |
| **Vercel** | ✅ Forever | No Docker/SSH | ❌ Sleeps | ❌ No |
| **Netlify** | ✅ Forever | No Docker/SSH | ❌ Sleeps | ❌ No |
| **Render** | ⚠️ 90 days | No SSH | ❌ Spins down | ❌ No |

### $20/Month VPS Rankings

| Rank | Provider | Plan | Price | CPU | RAM | Storage | Type |
|------|----------|------|-------|-----|-----|---------|------|
| 🥇 | **Hetzner** | CX41 | ~$17 | 4 vCPU | 16GB | 160GB NVMe | KVM |
| 🥈 | **Contabo** | M | ~$10 | 6 vCPU | 16GB | 300GB HDD | KVM |
| 🥉 | **BuyVM** | Tier 3 | $12 | 4 vCPU | 6GB | 160GB NVMe | KVM |
| 4 | **ScalaHosting** | Managed | $20 | 4 vCPU | 8GB | 100GB NVMe | Managed |
| 5 | **DigitalOcean** | Basic | $20 | 2 vCPU | 4GB | 80GB SSD | Premium |

### Optimal Architecture

**Oracle Cloud (free) + Hetzner CX31 (~$10)** = $10/mo total
- Oracle: SonarQube + artifact storage + PostgreSQL
- Hetzner: Woodpecker CI + 2-3 agents

**Scales to**: 100-200 CI jobs/day

---

## Part 3: Agent Resource Usage (Your Mac)

### Current Agent Usage (from Activity Monitor data)

| Agent | CPU Time | CPU% | RAM (Active) | RAM (GPU) | Threads | Est. 100x RAM |
|-------|----------|------|---------------|-----------|---------|----------------|
| **claude** | 30:11 | 31% | 308 MB | 0 MB | 104 | ~31 GB |
| **opencode** | 1:21:19 | 22% | 1.30 GB | 0 MB | 1,389 | ~130 GB |
| **.kilo** | 22:17 | 19% | 512 MB | 0 MB | 2,831 | ~51 GB |
| **forge** | 4.88s | 0.2% | 13 MB | 0 MB | 1,985 | ~1.3 GB |

### Your Mac: 16GB RAM, 10-core Apple Silicon

### Instance Capacity Estimates

| Agent Type | RAM/Instance | 100 Instances | Feasibility |
|------------|--------------|---------------|-------------|
| **forge** (Rust) | ~15 MB | ~1.5 GB | ✅ **100+ easily** |
| **claude** (heavy) | ~350 MB | ~35 GB | ⚠️ 45 max |
| **.kilo** (medium) | ~550 MB | ~55 GB | ⚠️ 29 max |
| **opencode** (very heavy) | ~1.3 GB | ~130 GB | ❌ 12 max |

**Insight**: forge (Rust-based) can handle 100+ instances. Heavy agents (claude, opencode, .kilo) are memory-constrained.

---

## Part 4: Action Items

### Completed
- [x] Sentry token regenerated and verified working
- [x] GitHub Secrets configured for 3 Phase 1 repos (AgilePlus, phenotype-infrakit, heliosCLI)
- [x] VPS research documented
- [x] Agent resource usage analyzed

### Pending
- [ ] Complete Sentry project creation (API issue — POST not allowed)
- [ ] SonarQube installation check
- [ ] Deploy Woodpecker CI on Hetzner
- [ ] Research lightweight agent alternatives

### User Decisions Needed
1. Sentry project creation — manual or alternative approach?
2. SonarQube — install now on Oracle free tier?
3. Hetzner — sign up and deploy?

---

## Key Findings

1. **Vercel/Netlify/Railway are NOT alternatives** for self-hosted CI — they don't support Docker/SSH
2. **Oracle Cloud is the best free tier** — 4-core ARM, 24GB RAM, 200GB SSD forever
3. **Hetzner is the best value** at ~$10-17/mo for VPS with NVMe
4. **forge (Rust agent) is ideal** for 100+ instances — only ~15MB RAM per instance
5. **GitHub Actions must be bypassed** for CI due to billing exhaustion

---

*Session created: 2026-03-31*
