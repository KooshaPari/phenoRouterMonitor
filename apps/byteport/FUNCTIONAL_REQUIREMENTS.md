# Byteport — Functional Requirements

**Version:** 1.0  
**Date:** 2026-03-29  
**Status:** Active Development

---

## Requirement ID Convention

Format: `FR-{CATEGORY}-{NNN}`

Categories:
- `LOCAL`: Local container orchestration (Docker/Podman/Lima/OrbStack)
- `FREEMIUM`: Freemium tier providers (Vercel/Netlify/Railway/Fly.io/Supabase)
- `PROD`: Production tier providers (AWS/GCP/Azure)
- `CORE`: Core platform (interfaces, registry, CLI, API)

---

## LOCAL: Local Container Orchestration (FR-LOCAL-001 through FR-LOCAL-005)

### FR-LOCAL-001: Docker & Podman Socket Detection

| Field | Value |
|-------|-------|
| **ID** | FR-LOCAL-001 |
| **Title** | Auto-detect Docker/Podman socket and establish connection |
| **SHALL** | SHALL detect Docker socket at `/var/run/docker.sock` or Podman socket at `/run/podman/podman.sock` with exponential backoff retry (5 retries, max 30s) |
| **Traces to** | E2.1: Container Runtime Detection & Connection |
| **Priority** | P0 (Critical) |
| **Tests** | `test_docker_socket_detection`, `test_podman_socket_detection`, `test_socket_connection_retry` |

### FR-LOCAL-002: Lima VM & OrbStack Integration

| Field | Value |
|-------|-------|
| **ID** | FR-LOCAL-002 |
| **Title** | Seamless Lima VM and OrbStack container runtime support |
| **SHALL** | SHALL detect Lima socket at `~/.lima/default/sock/docker.sock`, auto-start Lima if inactive, and support OrbStack CLI wrapper (deploy, get-logs, delete) with fallback chain: OrbStack → Lima → Docker → Podman |
| **Traces to** | E2.5: Lima VM & OrbStack Integration |
| **Priority** | P0 (Critical) |
| **Tests** | `test_lima_socket_detection`, `test_lima_autostart`, `test_orbstack_cli_wrapper`, `test_fallback_chain` |

### FR-LOCAL-003: Container Lifecycle (Create, Run, Stop, Remove)

| Field | Value |
|-------|-------|
| **ID** | FR-LOCAL-003 |
| **Title** | Full container lifecycle management with resource limits |
| **SHALL** | SHALL support image pull (with private registry auth), container creation with env vars/mounts/resource limits (CPU, memory), start/stop/remove operations, log streaming (STDOUT + STDERR, 100-line history), and interactive shell exec (docker exec -it) |
| **Traces to** | E2.2: Container Lifecycle (Create, Run, Stop, Remove) |
| **Priority** | P0 (Critical) |
| **Tests** | `test_container_creation`, `test_resource_limits`, `test_log_streaming`, `test_container_exec` |

### FR-LOCAL-004: Port Mapping & Networking

| Field | Value |
|-------|-------|
| **ID** | FR-LOCAL-004 |
| **Title** | Automatic port allocation and conflict detection |
| **SHALL** | SHALL auto-allocate random free ports, map to configured ports, detect port conflicts with user prompt for resolution, support DNS aliases (e.g., `myapp.local`), and manage multi-container bridge networking |
| **Traces to** | E2.3: Port Mapping & Networking |
| **Priority** | P1 (High) |
| **Tests** | `test_port_allocation`, `test_port_conflict_detection`, `test_dns_alias`, `test_bridge_networking` |

### FR-LOCAL-005: Volume Mounts & Persistence

| Field | Value |
|-------|-------|
| **ID** | FR-LOCAL-005 |
| **Title** | Bind mounts and named volumes with garbage collection |
| **SHALL** | SHALL support bind mounts (code → container), named volumes (data → host), uid/gid mapping for permissions, and garbage collection of unused volumes after 7 days |
| **Traces to** | E2.4: Volume Mounts & Persistence |
| **Priority** | P1 (High) |
| **Tests** | `test_bind_mount`, `test_named_volume`, `test_uid_gid_mapping`, `test_volume_gc` |

---

## FREEMIUM: Freemium Tier Providers (FR-FREEMIUM-001 through FR-FREEMIUM-006)

### FR-FREEMIUM-001: Vercel Provider Integration

| Field | Value |
|-------|-------|
| **ID** | FR-FREEMIUM-001 |
| **Title** | Deploy to Vercel with token auth and environment management |
| **SHALL** | SHALL authenticate with Vercel API token, deploy Next.js/static projects, retrieve deployment logs via GetLogs API, track metrics (requests/sec, latency) via GetMetrics API, and query deployment status |
| **Traces to** | E5.2: Unified CLI, E6.1: Push-to-Deploy |
| **Priority** | P0 (Critical) |
| **Tests** | `test_vercel_deploy`, `test_vercel_get_logs`, `test_vercel_get_metrics`, `test_vercel_status` |

### FR-FREEMIUM-002: Railway Provider Integration

| Field | Value |
|-------|-------|
| **ID** | FR-FREEMIUM-002 |
| **Title** | Deploy to Railway with project/service management |
| **SHALL** | SHALL authenticate with Railway API token, deploy containerized services, retrieve logs (tail + follow), query metrics (CPU, memory, bandwidth) via Railway GraphQL, and manage service lifecycle (deploy, rollback, delete) |
| **Traces to** | E5.2: Unified CLI, E6.1: Push-to-Deploy |
| **Priority** | P0 (Critical) |
| **Tests** | `test_railway_deploy`, `test_railway_get_logs`, `test_railway_get_metrics`, `test_railway_lifecycle` |

### FR-FREEMIUM-003: Fly.io Provider Integration

| Field | Value |
|-------|-------|
| **ID** | FR-FREEMIUM-003 |
| **Title** | Deploy to Fly.io with app management and monitoring |
| **SHALL** | SHALL authenticate with Fly.io API token, deploy Docker apps, retrieve logs via `fly logs` API, query metrics (connections, requests, latency) via Machines API, and support scale/regions configuration |
| **Traces to** | E5.2: Unified CLI, E6.1: Push-to-Deploy |
| **Priority** | P0 (Critical) |
| **Tests** | `test_flyio_deploy`, `test_flyio_get_logs`, `test_flyio_get_metrics`, `test_flyio_scale` |

### FR-FREEMIUM-004: Cost Estimation & Free Tier Detection

| Field | Value |
|-------|-------|
| **ID** | FR-FREEMIUM-004 |
| **Title** | Per-provider cost estimation with free tier warnings |
| **SHALL** | SHALL estimate monthly costs (per-compute, per-bandwidth, per-storage) with ±10% accuracy, warn if deployment exceeds free tier limits, provide overage estimates and optimization suggestions (e.g., "Enable gzip to save 3 GB/mo"), and track pricing models independently |
| **Traces to** | E3: Cost Estimation |
| **Priority** | P1 (High) |
| **Tests** | `test_cost_estimation`, `test_free_tier_detection`, `test_overage_estimation`, `test_optimization_suggestions` |

### FR-FREEMIUM-005: Deployment Logs & Metrics Aggregation

| Field | Value |
|-------|-------|
| **ID** | FR-FREEMIUM-005 |
| **Title** | Real-time log streaming and metrics aggregation across providers |
| **SHALL** | SHALL stream logs (STDOUT + STDERR, real-time follow), maintain 30-day log retention, aggregate metrics from all freemium providers (requests/sec, latency, error rate, bandwidth), and support log search/filter by timestamp, level (INFO/WARN/ERROR), and keyword |
| **Traces to** | E5.3: Metrics & Observability, E5.4: Log Streaming & Search |
| **Priority** | P1 (High) |
| **Tests** | `test_log_streaming`, `test_log_search`, `test_metrics_aggregation`, `test_log_retention` |

### FR-FREEMIUM-006: Cost Alerts & Anomaly Detection

| Field | Value |
|-------|-------|
| **ID** | FR-FREEMIUM-006 |
| **Title** | Cost spike detection with alerts and notifications |
| **SHALL** | SHALL detect anomalies when daily cost exceeds 20% of 7-day average, send Slack/email alerts with cost breakdown, calculate cost-per-request efficiency metric, and maintain daily cost ledger with provider breakdowns |
| **Traces to** | E3.3: Cost Alerts & Anomaly Detection |
| **Priority** | P2 (Medium) |
| **Tests** | `test_cost_anomaly_detection`, `test_alert_notifications`, `test_cost_per_request_metric` |

---

## PROD: Production Tier Providers (FR-PROD-001 through FR-PROD-007)

### FR-PROD-001: AWS ECS/Lambda Mock Implementation

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-001 |
| **Title** | Mock AWS ECS and Lambda deployment support |
| **SHALL** | SHALL provide mock implementations of ECS task registration, task launch in cluster, Lambda function deployment, CloudWatch logs retrieval, and CloudWatch metrics queries (CPU, memory, invocations) with stateful in-memory store for testing (no real AWS calls) |
| **Traces to** | E1.1: Provider Registry, E5.2: Unified CLI |
| **Priority** | P1 (High) |
| **Tests** | `test_aws_ecs_mock_deploy`, `test_aws_lambda_mock_deploy`, `test_aws_cloudwatch_mock_logs`, `test_aws_cloudwatch_mock_metrics` |

### FR-PROD-002: GCP Cloud Run & Compute Engine Mock

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-002 |
| **Title** | Mock GCP Cloud Run and Compute Engine deployment |
| **SHALL** | SHALL provide mock implementations of Cloud Run service creation, image deployment, GCP Monitoring metrics (CPU, memory, requests) retrieval, Cloud Logging log queries, and instance management with stateful in-memory store (no real GCP calls) |
| **Traces to** | E1.1: Provider Registry, E5.2: Unified CLI |
| **Priority** | P1 (High) |
| **Tests** | `test_gcp_cloudrun_mock_deploy`, `test_gcp_monitoring_mock_metrics`, `test_gcp_logging_mock_logs` |

### FR-PROD-003: Azure Container Instances Mock

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-003 |
| **Title** | Mock Azure Container Instances deployment |
| **SHALL** | SHALL provide mock implementations of ACI container group creation, container image deployment, Azure Monitor metrics retrieval (CPU, memory, network), Azure Monitor Logs queries, and container lifecycle management with stateful in-memory store (no real Azure calls) |
| **Traces to** | E1.1: Provider Registry, E5.2: Unified CLI |
| **Priority** | P1 (High) |
| **Tests** | `test_azure_aci_mock_deploy`, `test_azure_monitor_mock_metrics`, `test_azure_logs_mock_query` |

### FR-PROD-004: Credential Management & IAM Role Generation

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-004 |
| **Title** | Secure credential storage and auto-generated IAM roles |
| **SHALL** | SHALL store cloud credentials (AWS access keys, GCP service account keys, Azure AD tokens) in OS keychain or encrypted SQLite, auto-generate least-privilege IAM roles per provider (ECS task role, Lambda execution role, etc.), and support credential rotation with expiry tracking |
| **Traces to** | E4.1: API Token Management |
| **Priority** | P1 (High) |
| **Tests** | `test_credential_storage`, `test_iam_role_generation`, `test_credential_rotation` |

### FR-PROD-005: Region Selection & Latency Optimization

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-005 |
| **Title** | Intelligent region selection based on latency |
| **SHALL** | SHALL support manual region selection and auto-selection via latency measurement (ping probes to each region), calculate estimated latency from user location, recommend lowest-latency region, and track latency trends over time for optimization |
| **Traces to** | E1.1: Provider Registry (region parameter) |
| **Priority** | P2 (Medium) |
| **Tests** | `test_region_selection`, `test_latency_measurement`, `test_latency_trend_tracking` |

### FR-PROD-006: VPC & Network Configuration

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-006 |
| **Title** | VPC provisioning and network security configuration |
| **SHALL** | SHALL support new VPC creation (with configurable CIDR blocks, subnets, route tables) and existing VPC selection, configure security groups (ingress/egress rules), manage NAT gateways for private subnet outbound, and support public/private subnet topology |
| **Traces to** | E1.1: Provider Registry (network config) |
| **Priority** | P2 (Medium) |
| **Tests** | `test_vpc_creation`, `test_security_group_config`, `test_nat_gateway_setup`, `test_subnet_topology` |

### FR-PROD-007: Cost Alerts for Production Workloads

| Field | Value |
|-------|-------|
| **ID** | FR-PROD-007 |
| **Title** | Production cost alerts with budget caps |
| **SHALL** | SHALL track production costs via provider billing APIs (CloudWatch, GCP Monitoring, Azure Monitor), set monthly budget caps with warning thresholds (80%, 100%), send alerts on threshold breach, and provide cost-per-transaction metrics for ROI analysis |
| **Traces to** | E3.3: Cost Alerts & Anomaly Detection |
| **Priority** | P2 (Medium) |
| **Tests** | `test_production_cost_tracking`, `test_budget_cap_alerts`, `test_cost_per_transaction_metric` |

---

## CORE: Core Platform (FR-CORE-001 through FR-CORE-008)

### FR-CORE-001: Provider Registry & Interface

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-001 |
| **Title** | Unified provider abstraction with Go interfaces |
| **SHALL** | SHALL define Go interfaces: `Deployer` (Deploy, Status, Rollback, Delete), `Monitor` (GetLogs, GetMetrics), `Scaler` (Scale, GetScalingPolicy), `CostEstimator` (EstimateCost, GetPricingModel), and `Authenticator` (Auth, RefreshToken, RevokeToken); registry SHALL support CRUD operations (Add, List, Remove, GetProvider) with zero-downtime hot-swap of providers |
| **Traces to** | E1.1: Provider Registry |
| **Priority** | P0 (Critical) |
| **Tests** | `test_provider_interface`, `test_registry_crud`, `test_provider_hotswap` |

### FR-CORE-002: Resource Lifecycle State Machine

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-002 |
| **Title** | Consistent resource state machine across all providers |
| **SHALL** | SHALL enforce state machine: `PENDING → RUNNING → STOPPING → STOPPED`, support lifecycle hooks (onCreate, onStart, onMetrics, onStop, onDelete), implement health checks (liveness + readiness probes with configurable intervals), and enforce graceful shutdown (SIGTERM then SIGKILL after 30s timeout) |
| **Traces to** | E1.2: Resource Lifecycle Management |
| **Priority** | P0 (Critical) |
| **Tests** | `test_state_machine`, `test_lifecycle_hooks`, `test_health_checks`, `test_graceful_shutdown` |

### FR-CORE-003: Deployment History & Ledger

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-003 |
| **Title** | Immutable deployment ledger with rollback support |
| **SHALL** | SHALL maintain immutable ledger (timestamp, version hash, provider, manifest JSON, deployment status, duration) for each deployment, support one-click rollback to previous version (redeploying previous manifest), show manifest diffs between deployments, and retain 30 deployments or 90 days (whichever is longer) |
| **Traces to** | E1.3: Deployment History & Rollback |
| **Priority** | P1 (High) |
| **Tests** | `test_deployment_ledger`, `test_rollback`, `test_manifest_diff`, `test_ledger_retention` |

### FR-CORE-004: Log Streaming & Aggregation

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-004 |
| **Title** | Real-time log streaming with search and retention |
| **SHALL** | SHALL stream logs in real-time (STDOUT + STDERR, with --follow flag), support log search by timestamp/level/keyword/regex, support exclude and include filters, export logs as CSV/JSON, and maintain retention: 7 days (local), 30 days (freemium), 90 days (production) |
| **Traces to** | E5.4: Log Streaming & Search |
| **Priority** | P1 (High) |
| **Tests** | `test_log_streaming`, `test_log_search`, `test_log_filters`, `test_log_export`, `test_log_retention` |

### FR-CORE-005: Metrics Aggregation & Dashboarding

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-005 |
| **Title** | Multi-source metrics aggregation with time-series storage |
| **SHALL** | SHALL aggregate metrics from Docker stats (local), provider APIs (freemium/prod): CPU %, memory MB, requests/sec, error rate, latency p50/p99, bandwidth MB/s; store time-series with 1-min granularity for 7 days, 1-hour granularity for 90 days; support dashboard widgets (line, heatmap, gauge) and custom metric scraping via Prometheus/StatsD |
| **Traces to** | E5.3: Metrics & Observability |
| **Priority** | P1 (High) |
| **Tests** | `test_metrics_aggregation`, `test_timeseries_storage`, `test_dashboard_widgets`, `test_custom_metrics` |

### FR-CORE-006: Cloudflare Tunnel SDK Wrapper

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-006 |
| **Title** | CF Tunnel integration for secure access without VPN |
| **SHALL** | SHALL wrap Cloudflare Tunnel SDK, auto-create tunnel on deployment, retrieve tunnel token and share public URL, manage tunnel lifecycle (create, update, delete), support CF Access service token auth, and implement OIDC token refresh with retry logic (exponential backoff, max 5 retries) |
| **Traces to** | E4.3: Cloudflare Access Integration |
| **Priority** | P2 (Medium) |
| **Tests** | `test_cf_tunnel_creation`, `test_cf_access_auth`, `test_oidc_token_refresh`, `test_tunnel_lifecycle` |

### FR-CORE-007: CLI Command Suite & UX

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-007 |
| **Title** | Unified CLI with comprehensive command set |
| **SHALL** | SHALL implement commands: `init`, `deploy`, `status`, `logs`, `metrics`, `delete`, `config` (list/set/unset), `estimate`, `rollback`, `team` (CRUD + access grants), `audit` (list access/token events); support flags: `--tier`, `--provider`, `--follow`, `--tail`, `--duration`, `--since`, `--json` output; provide helpful error messages and progress indicators (spinners, progress bars) |
| **Traces to** | E5.2: Unified CLI |
| **Priority** | P1 (High) |
| **Tests** | All CLI command tests: `test_cli_deploy`, `test_cli_status`, etc. |

### FR-CORE-008: API Versioning & Backwards Compatibility

| Field | Value |
|-------|-------|
| **ID** | FR-CORE-008 |
| **Title** | REST API with versioning and deprecation path |
| **SHALL** | SHALL expose `/api/v1/` endpoints for deployments, providers, metrics, logs, history; support content negotiation (JSON/gRPC in future); implement graceful deprecation (v1 → v2 with 6-month overlap), document all endpoints with OpenAPI 3.0, and enforce authentication via API token (OAuth2 in future) |
| **Traces to** | E5.1: Deployment Dashboard (Web) |
| **Priority** | P2 (Medium) |
| **Tests** | `test_api_v1_endpoints`, `test_api_versioning`, `test_api_auth`, `test_openapi_spec` |

---

## Traceability Summary

| Epic | Associated FRs | Count |
|------|---|---|
| E1: Multi-Cloud Provider Abstraction | FR-CORE-001, FR-CORE-002, FR-CORE-003, FR-PROD-001, FR-PROD-002, FR-PROD-003, FR-PROD-004 | 7 |
| E2: Local Container Orchestration | FR-LOCAL-001, FR-LOCAL-002, FR-LOCAL-003, FR-LOCAL-004, FR-LOCAL-005 | 5 |
| E3: Cost Estimation | FR-FREEMIUM-004, FR-FREEMIUM-006, FR-PROD-007 | 3 |
| E4: Auth & Access Control | FR-CORE-006 (Cloudflare), plus token mgmt in CLI | 2 |
| E5: Dashboard & Unified CLI | FR-CORE-004, FR-CORE-005, FR-CORE-007, FR-CORE-008 | 4 |
| E6: Git Integration & Auto-Deployment | (Webhook handler outside FR scope, uses FR-CORE-007 CLI) | — |
| Local-Specific | FR-LOCAL-001 through FR-LOCAL-005 | 5 |
| Freemium-Specific | FR-FREEMIUM-001 through FR-FREEMIUM-006 | 6 |

**Total FRs:** 26+

---

## Priority Levels

| Level | Definition | Count |
|-------|-----------|-------|
| **P0 (Critical)** | Must ship in Phase 1 | 5 |
| **P1 (High)** | Should ship in Phase 1–2 | 13 |
| **P2 (Medium)** | Phase 2–3 or deferred | 8 |
| **P3 (Low)** | Nice-to-have, future | — |

---

## Implementation Notes

1. **Mocking Strategy:** All prod tier providers (AWS/GCP/Azure) use stateful in-memory mocks with zero real cloud calls for Phase 1–2. Real implementations in Phase 3+ (deferred per user instructions).

2. **Test Traceability:** Every FR has `@pytest.mark.requirement("FR-XXX-NNN")` decorator in test file. `task quality` command verifies all FRs have tests and all tests reference FRs.

3. **Cost Model Updates:** Pricing models for Vercel, Fly.io, Railway, Supabase are fetched from provider APIs monthly and cached locally; manual override support for accuracy testing.

4. **Backwards Compatibility:** API v1 remains stable throughout Phase 1–2; v2 introduced only in Phase 3 or later with 6-month overlap.

