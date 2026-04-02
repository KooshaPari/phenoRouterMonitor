# Plane Fork / Shared PM Substrate — Session Overview

**Date:** 2026-03-27
**Spec:** `.agileplus/specs/008-plane-shared-pm-substrate/`
**Workstream:** G037

## Decision

**Fork Plane** (plane.so, Apache 2.0) as the shared PM substrate for the organization. Keep AgilePlus as the custom orchestration/control-plane layer. Keep TracerTM custom.

## Rationale

- Plane already covers canonical PM primitives: workspaces, projects, work items, cycles, modules, pages, comments, mentions, notifications, permissions, search, API/SDK surface, webhooks, import/export.
- Building a parallel PM dashboard would maintain two systems indefinitely; a fork absorbs ongoing Plane development.
- AgilePlus adds unique orchestration value (agent activity panels, service health/restart controls, local process supervision, device sync, evidence bundles) that Plane doesn't cover.
- TracerTM is a requirements-traceability/governance system with a fundamentally different data model, not a classic PM board.
- TheGent is an agent-control plane, not a PM surface.

## What Goes Into the Fork

- Workspaces, projects, work items, cycles, modules, pages
- Comments, mentions, notifications, activity and audit trails
- Permissions, multi-tenant boundaries, search and filtering
- API/SDK surface for automation, webhooks, import/export

## What Stays Custom

- Agent activity panels
- Service health/restart/toggle controls
- Local process supervision and device sync
- Evidence bundles, trace artifacts, media artifacts
- Spec/ADR/contract coverage and gap analysis
- Governance or orchestration UI outside PM scope

## Current Status

Pending — implementation not started. Requires:
1. Fork Plane repository into the org's GitHub
2. Define the AgilePlus control-plane boundary adapter layer
3. Migrate or quarantine duplicate dashboard code
