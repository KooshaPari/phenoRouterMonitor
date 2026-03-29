# Byteport — Product Requirements Document

**Version:** 1.0  
**Date:** 2026-03-29  
**Status:** Active Development

---

## 1. Vision Statement

**Deploy any project from local development to production with a single CLI command.**

Byteport abstracts multi-cloud deployment complexity, enabling developers to:
- Deploy to local development environments (95% of use cases)
- Seamlessly scale to freemium cloud providers (testing, demos, early users)
- Graduate to production infrastructure (AWS/GCP/Azure) when ready
- Manage cost, performance, and access across all tiers without learning multiple APIs

---

## 2. Problem Statement

Modern developers juggle multiple deployment targets:
- **Local dev:** Docker, Podman, Lima, OrbStack—each with different CLI APIs
- **Freemium staging:** Vercel, Netlify, Railway, Fly.io, Supabase—each with separate tokens, cost models, monitoring
- **Production:** AWS/GCP/Azure—complex IAM, VPC, region selection, cost alerts

**Result:** Hours learning provider APIs, context-switching between CLIs, manual cost tracking, and tribal knowledge about tier selection.

**Solution:** A unified deployment abstraction with cost estimation, access control, and automatic provider selection.

---

## 3. Target Users

| User Persona | Byteport Role | Primary Tier | Pain Point |
|--------------|---------------|--------------|-----------|
| **Indie Developer** | Solo deployments | Local → Freemium | "Which cloud is cheapest for my MVP?" |
| **Startup Team (2–5)** | Shared deployments, cost tracking | Freemium → Production | "Coordinating access across providers without secrets sprawl" |
| **Platform Engineer** | Infrastructure abstraction | Local + Freemium + Prod | "Standardizing dev→prod onboarding for all teams" |
| **Agency** | Client project deployments | Any tier | "Deploying to client-preferred cloud without learning each" |

---

## 4. Deployment Tiers

### Tier 1: Local (95% Use)

**Providers:** Docker, Podman, Lima, OrbStack  
**Use:** Local development, team testing, CI/CD sandbox  
**Cost:** Free (host compute only)  
**Setup:** Socket connection to container runtime  
**SLA:** N/A (dev environment)

### Tier 2: Freemium (5% Use)

**Providers:** Vercel, Netlify, Railway, Fly.io, Supabase  
**Use:** Public demos, early-user testing, staging  
**Cost:** Free tier available; pay-as-you-go overage  
**Setup:** API tokens + provider config  
**SLA:** 99.9% uptime (provider responsibility)

### Tier 3: Production (Target, ~1yr)

**Providers:** AWS, GCP, Azure  
**Use:** Customer-facing workloads, regulated industries  
**Cost:** CapEx + operational cost  
**Setup:** IAM roles, VPC, region selection, secret management  
**SLA:** 99.99% uptime + incident response

---

## 5. Epics & User Stories

### **Epic E1: Multi-Cloud Provider Abstraction**

**Goal:** Unified provider interface enabling provider-agnostic deployments.

#### E1.1: Provider Registry  
**Story:** As a platform engineer, I want to register custom providers so I can extend Byteport to support internal infrastructure.

- Acceptance Criteria:
  - Provider interface (Go interfaces: Deployer, Monitor, Scaler)
  - Registry CRUD: add, list, remove, update providers
  - Plugin discovery (filesystem + registry)
  - Zero-downtime provider hot-swap

#### E1.2: Resource Lifecycle Management  
**Story:** As a developer, I want resources (containers, functions, databases) to follow a consistent lifecycle so I can predict deployment behavior.

- Acceptance Criteria:
  - State machine: PENDING → RUNNING → STOPPING → STOPPED
  - Lifecycle hooks (onCreate, onStart, onMetrics, onStop, onDelete)
  - Health checks (liveness + readiness)
  - Graceful shutdown (SIGTERM, 30s timeout, force SIGKILL)

#### E1.3: Deployment History & Rollback  
**Story:** As a developer, I want to track all deployments and rollback quickly if needed.

- Acceptance Criteria:
  - Immutable deployment ledger (timestamp, version, provider, manifest, status)
  - One-click rollback to previous version
  - Comparison view (manifest diff)
  - Retention: 30 deployments or 90 days

---

### **Epic E2: Local Container Orchestration**

**Goal:** Native support for Docker, Podman, Lima, OrbStack as first-class tier.

#### E2.1: Container Runtime Detection & Connection  
**Story:** As a developer on macOS, I want Byteport to auto-detect my container runtime (OrbStack, Lima, Docker Desktop) without manual config.

- Acceptance Criteria:
  - Socket detection: /var/run/docker.sock, lima socket, OrbStack endpoints
  - Connection test + retry logic (exponential backoff, 5 retries)
  - Error messaging for missing or broken runtimes
  - Fallback chain: OrbStack → Lima → Docker → Podman

#### E2.2: Container Lifecycle (Create, Run, Stop, Remove)  
**Story:** As a developer, I want to deploy containers with resource limits (CPU, memory) and custom networking.

- Acceptance Criteria:
  - Image pull (with auth for private registries)
  - Container creation with env vars, mounts, resource limits
  - Start/stop/remove operations
  - Container logs streaming + history (100 lines)
  - Exec into containers (interactive shell)

#### E2.3: Port Mapping & Networking  
**Story:** As a developer, I want my local service accessible via a consistent URL without port conflicts.

- Acceptance Criteria:
  - Automatic port allocation (random free port → configured port mapping)
  - Port conflict detection + user prompt for resolution
  - DNS alias (e.g., `myapp.local:8080`)
  - Multi-container networking (bridge + overlay simulation)

#### E2.4: Volume Mounts & Persistence  
**Story:** As a developer, I want local code changes to reflect immediately in running containers and preserve data across restarts.

- Acceptance Criteria:
  - Bind mount support (code → container)
  - Named volume support (data → host directory)
  - Volume ownership + permissions handling (uid/gid mapping)
  - Garbage collection (unused volumes after 7 days)

#### E2.5: Lima VM & OrbStack Integration  
**Story:** As a macOS developer, I want seamless integration with Lima/OrbStack for lightweight VM-based containers.

- Acceptance Criteria:
  - Lima socket polling and auto-start (lazy init)
  - OrbStack CLI wrapper (deploy, get-logs, delete)
  - VM resource provisioning (CPU, RAM, disk)
  - Fallback to Docker Desktop if VM unavailable

---

### **Epic E3: Cost Estimation**

**Goal:** Transparent cost calculation before and after deployment.

#### E3.1: Per-Provider Cost Estimation  
**Story:** As a developer, I want to see estimated costs before deploying to a freemium or production tier.

- Acceptance Criteria:
  - Pricing models: per-service (Vercel), per-compute (Fly.io), per-database (Supabase)
  - Input: resource manifest (CPU, memory, bandwidth, storage)
  - Output: monthly cost breakdown by service
  - Accuracy ±10% (provider pricing changes daily)

#### E3.2: Free Tier Detection  
**Story:** As an indie developer, I want Byteport to warn me if my deployment exceeds free tier limits.

- Acceptance Criteria:
  - Free tier comparison (resource limits vs. manifest)
  - Overage estimation (e.g., "10 GB/month exceeds Vercel free limit by $5/mo")
  - Optimization suggestions (e.g., "Enable gzip to save 3 GB/month")

#### E3.3: Cost Alerts & Anomaly Detection  
**Story:** As a startup founder, I want alerts if deployment costs spike unexpectedly.

- Acceptance Criteria:
  - Cost tracking per deployment
  - Anomaly detection (>20% increase from last 7-day average)
  - Slack/email alerts on threshold breach
  - Cost vs. compute metric (e.g., cost per request)

---

### **Epic E4: Auth & Access Control**

**Goal:** Secure provider credential management and team access.

#### E4.1: API Token Management  
**Story:** As a developer, I want to securely store provider tokens without hardcoding them in `.env` files.

- Acceptance Criteria:
  - Token storage: OS keychain (macOS/Linux) or encrypted SQLite
  - Token rotation: set expiry, auto-refresh before expiry
  - Token scoping: read-only, deploy-only, full-access roles per provider
  - Audit log: token created/revoked/used timestamps

#### E4.2: Multi-User Access Control  
**Story:** As a platform engineer, I want to grant team members selective access to providers (e.g., "only Vercel, not production AWS").

- Acceptance Criteria:
  - User roles: admin, operator, viewer
  - Provider access matrix: user × provider × permissions
  - RBAC enforcement in CLI commands (deny with explanation)
  - Group permissions (dev-team, data-team, ops-team)

#### E4.3: Cloudflare Access Integration  
**Story:** As an enterprise admin, I want to gate deployments behind Cloudflare Access for VPN-less security.

- Acceptance Criteria:
  - CF Access service token integration
  - Automatic tunnel creation for protected resources
  - OIDC token refresh + retry logic
  - Audit logging: access requests with user identity

---

### **Epic E5: Dashboard & Unified CLI**

**Goal:** Single pane of glass for deployment status, logs, metrics, and controls.

#### E5.1: Deployment Dashboard (Web)  
**Story:** As a team lead, I want a web dashboard showing all active deployments, their status, and recent changes.

- Acceptance Criteria:
  - Deployments list: name, tier, provider, status, last-updated
  - Deployment details: manifest, logs, metrics (CPU, memory, requests/sec)
  - Real-time status updates (WebSocket)
  - Deployment actions: start, stop, restart, rollback, delete

#### E5.2: Unified CLI  
**Story:** As a developer, I want a single CLI to deploy, check status, and read logs across all tiers.

- Acceptance Criteria:
  - `byteport deploy <service> [--tier local|freemium|prod]`
  - `byteport status [<service>]` — all active deployments
  - `byteport logs <service> [--follow] [--tail N]`
  - `byteport metrics <service> [--duration 24h]`
  - `byteport delete <service>`
  - `byteport config list|set|unset [<key>] [<value>]`

#### E5.3: Metrics & Observability  
**Story:** As an operator, I want to see resource usage (CPU, memory, bandwidth) and custom application metrics.

- Acceptance Criteria:
  - Metrics aggregation: local (Docker stats), freemium (provider APIs), prod (CloudWatch/GCP Monitoring/Azure Monitor)
  - Dashboard widgets: time-series (CPU, memory), heatmap (errors), gauge (requests/sec)
  - Custom metrics: via StatsD/Prometheus scraping
  - Retention: 7 days at 1m granularity, 90 days at 1h granularity

#### E5.4: Log Streaming & Search  
**Story:** As a developer, I want to stream logs in real-time and search historical logs by keyword.

- Acceptance Criteria:
  - Log streaming (STDERR + STDOUT)
  - Search: by timestamp, level (INFO, WARN, ERROR), keyword
  - Filtering: exclude patterns, include patterns
  - Export: CSV/JSON for log aggregation services
  - Retention: 7 days (local), 30 days (freemium/prod)

---

### **Epic E6: Git Integration & Auto-Deployment**

**Goal:** Deploy from git workflows (push, PR, release).

#### E6.1: Push-to-Deploy  
**Story:** As a developer, I want my code to deploy automatically when I push to `main`.

- Acceptance Criteria:
  - GitHub/GitLab webhook listener
  - Branch-to-tier mapping: `main` → Freemium, `release/*` → Prod
  - Automatic build + deploy on push
  - Deployment status in GitHub checks (green/red)
  - Rollback on failure (with user approval)

#### E6.2: Deploy Previews  
**Story:** As a reviewer, I want pull requests to include a live preview URL of the proposed changes.

- Acceptance Criteria:
  - Ephemeral preview deployment on PR open
  - Unique URL per PR (e.g., `pr-123.myapp-preview.dev`)
  - Cleanup on PR close
  - Preview link in PR comment + checks

#### E6.3: Release Automation  
**Story:** As a release manager, I want to trigger production deployments from git tags.

- Acceptance Criteria:
  - Tag-based workflow: `git tag v1.0.0` → production deploy
  - Changelog auto-generation (from commit messages)
  - Pre-release checks: all tests pass, security scan OK
  - Approval gate: require 2 approvals before production deploy

---

## 6. User Journeys

### UJ-1: Local Development Loop

**Persona:** Indie Developer (Python backend + Next.js frontend)

```
┌─────────────────────────────────────────────────────────┐
│ Byteport Local Dev Journey                              │
├─────────────────────────────────────────────────────────┤
│ 1. byteport init --template python-fastapi-nextjs      │
│    → Generates Byteport.yaml + .env                     │
│                                                         │
│ 2. byteport deploy --tier local                         │
│    → Detects OrbStack                                   │
│    → Pulls images, creates containers, wires networking│
│    → Backend: http://localhost:8000                     │
│    → Frontend: http://localhost:3000                    │
│                                                         │
│ 3. Edit code, save → Hot reload (mounted volume)       │
│                                                         │
│ 4. byteport logs -f backend                             │
│    → Streams backend logs in real-time                  │
│                                                         │
│ 5. byteport status                                      │
│    → Shows CPU/memory/requests metrics                  │
│                                                         │
│ 6. git push origin main                                 │
│    → Webhook triggers auto-deploy to Freemium          │
│    → Get Vercel preview URL in PR checks               │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### UJ-2: Scaling Local → Freemium

**Persona:** Startup Founder (ready for user testing)

```
┌─────────────────────────────────────────────────────────┐
│ Byteport Scale-Up Journey                               │
├─────────────────────────────────────────────────────────┤
│ 1. byteport estimate --tier freemium                    │
│    → Costs: Vercel $25/mo + Supabase $5/mo = $30/mo    │
│    → Expected free tier available ✓                     │
│                                                         │
│ 2. byteport deploy --tier freemium --provider vercel   │
│    → Provider detection: Use Vercel token from keychain│
│    → Deployment: 45s                                    │
│    → Public URL: myapp.vercel.app                       │
│                                                         │
│ 3. Share preview link with beta users                   │
│    → Cost alerts enabled (Slack notifications)          │
│                                                         │
│ 4. byteport metrics --tier freemium --duration 7d       │
│    → Monitor: requests, errors, latency, bandwidth      │
│                                                         │
│ 5. Detect cost spike (5x baseline)                      │
│    → Byteport alert: "Vercel overage alert: $200+/mo"  │
│    → Investigate + optimize code                        │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### UJ-3: Freemium → Production (AWS)

**Persona:** Startup CTO (customer growth, need reliability)

```
┌─────────────────────────────────────────────────────────┐
│ Byteport Production Migration Journey                   │
├─────────────────────────────────────────────────────────┤
│ 1. byteport estimate --tier prod --cloud aws            │
│    → CapEx: $500/mo (ECS + RDS)                         │
│    → vs. Vercel: $30/mo (but limited scale)             │
│                                                         │
│ 2. byteport configure --tier prod --cloud aws           │
│    → Region: us-east-1 (lowest latency)                 │
│    → VPC: new or existing?                              │
│    → IAM role: auto-generated with minimal permissions  │
│                                                         │
│ 3. Parallel run: Freemium + Prod (canary 10% traffic)  │
│    → byteport deploy --tier prod --canary 0.1           │
│    → Gradual traffic shift if healthy                   │
│                                                         │
│ 4. Production monitoring enabled                        │
│    → CloudWatch dashboards auto-created                 │
│    → Cost anomalies tracked daily                       │
│                                                         │
│ 5. Decommission Freemium (after 7 days stable)         │
│    → byteport delete --tier freemium --resource vercel │
│    → Savings: $30/mo, Freemium archive saved            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### UJ-4: Team Collaboration & Access Control

**Persona:** Platform Engineer (managing team access)

```
┌─────────────────────────────────────────────────────────┐
│ Byteport Team Access Journey                            │
├─────────────────────────────────────────────────────────┤
│ 1. Create team roles + provider matrix                  │
│    → byteport team create --name dev-platform          │
│    → byteport team add-user --user alice --role op     │
│    → byteport team grant --role op --provider vercel   │
│                                                         │
│ 2. Alice deploys to Freemium (auto-allowed)            │
│    → byteport deploy --tier freemium                    │
│    → Alice's keychain token auto-used                   │
│                                                         │
│ 3. Alice tries to deploy to Prod (denied)              │
│    → byteport deploy --tier prod                        │
│    → Error: "Access denied. Only 'admin' role allows" │
│    → Request approval from platform-eng lead           │
│                                                         │
│ 4. Bob (admin) audits token usage                       │
│    → byteport audit list --since 7d                     │
│    → Shows: who accessed what, when, from where        │
│                                                         │
│ 5. CF Access gate for additional security              │
│    → byteport cf-access enable --provider prod         │
│    → Deployments require VPN/OIDC auth                 │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 7. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first deployment | <2 min | CLI init + deploy |
| Cost transparency accuracy | ±10% | Estimate vs. actual (7d avg) |
| Deployment success rate | >99% | Green deployments / total |
| Provider abstraction coverage | 6+ providers | Docker, Podman, Lima, Vercel, Railway, Fly.io |
| Team adoption | 100% | All teams using unified CLI |
| Cost savings vs. manual | >20% | Aggregate overage reduction |

---

## 8. Timeline & Phasing

| Phase | Duration | Deliverables | Milestone |
|-------|----------|--------------|-----------|
| Phase 1: MVP Local | 3 weeks | Local provider, Docker/Podman, CLI basics | Deploy locally |
| Phase 2: Freemium | 4 weeks | Vercel/Railway/Fly.io, cost estimation, auto-deploy | Deploy to staging |
| Phase 3: Production | 6 weeks | AWS/GCP/Azure stubs, CF tunnel, dashboard | Deploy to production |
| Phase 4: Polish | 2 weeks | Performance, security audit, docs | GA release |

---

## 9. Out of Scope (1yr+)

- Kubernetes cluster management (future: Byteport for K8s)
- Multi-region failover
- Advanced observability (Datadog, New Relic integrations)
- Custom provider plugins (future API)
- Database migration automation

