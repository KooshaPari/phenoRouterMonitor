# Plane Fork — Research Notes

## Plane.so Current Capabilities

Source: https://docs.plane.so/

### Core Features (available, in scope for fork)
- Workspaces and multi-tenancy
- Projects, work items, cycles, modules
- Pages/wiki
- Comments, mentions, notifications
- Activity and audit trails
- Permissions and role-based access control
- Search and filtering across workspaces
- API and SDK surface for automation
- Webhooks
- Import/export adapters
- Board, list, and calendar views

### Observability / Integration Layer (available, may overlap)
- Slack and GitHub integrations
- Custom webhook triggers
- API endpoints for external tooling

## Alternative Evaluation

### Linear
- Too opinionated; limited customization for agent/evidence workflows
- SaaS-only model; no self-hosted option that fits org constraints
- Not a candidate for fork

### Height
- Project management only; no PM substrate depth
- Not a candidate

### OpenProject
- Self-hosted but heavier; less active development than Plane
- Not a candidate

### Plane
- Best fit: open source, active development, covers full PM primitive surface, API-first design
- Fork is the right call

## Org-Specific Surfaces NOT Covered by Plane

These are already custom and must stay outside the fork:
- Agent activity panels and telemetry
- Service health/restart/toggle controls
- Local process supervision (Dragonfly, PostgreSQL, NATS, MinIO)
- Device sync and peer discovery
- Evidence bundle management (trace artifacts, media artifacts)
- Spec/ADR/contract coverage gap analysis
- Governance and orchestration UI outside PM scope

## Migration Considerations

- Plane uses PostgreSQL; the existing PostgreSQL dependency in `process-compose.yml` aligns well.
- Plane's API is REST + WebSockets; AgilePlus can build an adapter layer around it.
- Plane's UI is React/TypeScript; no special tooling requirements.
- The fork should track Plane upstream and rebase regularly to avoid drift.
