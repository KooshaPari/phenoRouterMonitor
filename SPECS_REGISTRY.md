# Specs Registry — Single Source of Truth (SSOT)

**Version:** 2.1
**Status:** Active
**Updated:** 2026-03-31
**Branch:** `specs/main` (canonical)

---

## Overview

This registry is the **authoritative index** of all specifications across the Phenotype polyrepo ecosystem. It tracks:
- Functional Requirements (FRs) — `FUNCTIONAL_REQUIREMENTS.md`
- Architecture Decision Records (ADRs) — `ADR.md`
- Implementation Plans — `PLAN.md`
- User Journeys — `USER_JOURNEYS.md`

**Single Source of Truth Principle:**
- One canonical version per spec type per repository
- Version numbers use semantic versioning (e.g., v2.1, v1.0)
- All specs must live on the `specs/main` branch
- Changes tracked via `Spec-Traces: FR-XXX-NNN` in commits

---

## Central Registry

### Canonical Specs (Deployed on specs/main)

#### phenotype-infrakit

| Spec Type | File | Version | Status | FRs Covered | Last Updated |
|-----------|------|---------|--------|------------|--------------|
| **Functional Requirements** | `FUNCTIONAL_REQUIREMENTS.md` | 3.0 | ✅ Deployed | FR-INFRA-001 to FR-INFRA-007 | 2026-03-30 |
| **Architecture Decisions** | `ADR.md` | 1.2 | ✅ Deployed | ADR-001 to ADR-008 | 2026-03-27 |
| **Implementation Plan** | `PLAN.md` | 2.0 | ✅ Deployed | Phases 1-4 | 2026-03-30 |
| **User Journeys** | `USER_JOURNEYS.md` | 1.0 | ✅ Deployed | UJ-001 to UJ-010 | 2026-03-30 |

**Health:** ✅ 100% (all 4 spec types complete)

#### AgilePlus

| Spec Type | File | Version | Status | Items | Last Updated |
|-----------|------|---------|--------|-------|--------------|
| **Functional Requirements** | `FUNCTIONAL_REQUIREMENTS.md` | 2.3 | ✅ Deployed | 24 FRs | 2026-03-30 |
| **Architecture Decisions** | `ADR.md` | 1.1 | ✅ Deployed | 5 ADRs | 2026-03-28 |
| **Implementation Plan** | `PLAN.md` | 1.5 | ✅ Deployed | 3 phases | 2026-03-29 |
| **User Journeys** | `USER_JOURNEYS.md` | 0.9 | ⏳ Draft | 6 journeys | 2026-03-25 |

**Health:** ⚠️ 75% (UJ incomplete; ADR needs consolidation)

#### platforms/thegent

| Spec Type | File | Version | Status | Items | Last Updated |
|-----------|------|---------|--------|-------|--------------|
| **Functional Requirements** | `FUNCTIONAL_REQUIREMENTS.md` | 2.8 | ✅ Deployed | 31 FRs | 2026-03-30 |
| **Architecture Decisions** | `ADR.md` | 2.2 | ✅ Deployed | 8 ADRs | 2026-03-27 |
| **Implementation Plan** | `PLAN.md` | 2.1 | ✅ Deployed | 4 phases | 2026-03-29 |
| **User Journeys** | `USER_JOURNEYS.md` | 1.1 | ✅ Deployed | 12 journeys | 2026-03-30 |

**Health:** ✅ 100% (all 4 spec types complete)

#### heliosCLI

| Spec Type | File | Version | Status | Items | Last Updated |
|-----------|------|---------|--------|-------|--------------|
| **Functional Requirements** | `FUNCTIONAL_REQUIREMENTS.md` | 1.9 | ✅ Deployed | 18 FRs | 2026-03-29 |
| **Architecture Decisions** | `ADR.md` | 0.8 | ⏳ Draft | 4 ADRs | 2026-03-25 |
| **Implementation Plan** | `PLAN.md` | 2.3 | ✅ Deployed | 5 phases | 2026-03-30 |
| **User Journeys** | `USER_JOURNEYS.md` | 1.2 | ✅ Deployed | 8 journeys | 2026-03-30 |

**Health:** ⚠️ 75% (ADR draft needs finalization)

---

## Spec Versions & Approval Status

### Version Legend

| Status | Symbol | Meaning |
|--------|--------|---------|
| Deployed | ✅ | Live on specs/main, all validation passing |
| Review | ⏳ | In PR to specs/main, awaiting approval |
| Draft | 🔧 | Incomplete, not yet submitted |
| Deprecated | ❌ | No longer maintained, marked for archival |
| Superseded | ↪️ | Replaced by newer version |

### Master Version Table

```markdown
| Repo | FR | ADR | PLAN | UJ | Overall Health | Last Updated |
|------|----|----|------|-----|--------|--------------|
| phenotype-infrakit | ✅ v3.0 | ✅ v1.2 | ✅ v2.0 | ✅ v1.0 | 100% ✅ | 2026-03-30 |
| AgilePlus | ✅ v2.3 | ⏳ v1.1 | ✅ v1.5 | 🔧 v0.9 | 75% ⚠️ | 2026-03-30 |
| platforms/thegent | ✅ v2.8 | ✅ v2.2 | ✅ v2.1 | ✅ v1.1 | 100% ✅ | 2026-03-30 |
| heliosCLI | ✅ v1.9 | 🔧 v0.8 | ✅ v2.3 | ✅ v1.2 | 75% ⚠️ | 2026-03-29 |
```

---

## Spec Synchronization Schedule

### Automatic Merges (specs/agent-* → specs/main)

**Frequency:** Every 5 minutes (batched)

**Process:**
1. Agent pushes to `specs/agent-<name>-<task>` branch
2. CI runs validation (fr-coverage, structure, traceability)
3. If passing → Auto-merge within 5 minutes
4. If failing → Validation error comment on PR

**Success Rate Target:** 95%+ (conflicts are rare)

### Manual Reviews

**Frequency:** Daily at 10:00 UTC

**Scope:**
- Review all merged specs in past 24h
- Verify traceability (FR↔Test)
- Check for spec drift (specs vs. code)
- Approve version bumps

**Approval Role:** `specs-admin` (2+ approvals required for version bump)

### Version Updates

**Frequency:** Weekly on Mondays at 11:00 UTC

**Actions:**
- Bump version numbers (semantic: major.minor)
- Generate changelog entry
- Tag release (e.g., `specs-v2.1`)
- Update SPECS_REGISTRY.md

**Criteria for Version Bump:**
- ✅ All FRs traced to tests (100%)
- ✅ All tests passing
- ✅ No open spec validation issues
- ✅ Documentation complete

---

## Sync Status (Real-Time)

### Current Sync State (as of 2026-03-31 14:30 UTC)

**Overall Health:** 87.5% (Target: 100%)

| Repo | Specs Complete | FR Coverage | Test Traceability | Merge Health |
|------|----------------|-------------|------------------|--------------|
| phenotype-infrakit | 4/4 (100%) | 7 FRs | 100% | ✅ Clean |
| AgilePlus | 3/4 (75%) | 24 FRs | 92% | ⚠️ 2 pending |
| platforms/thegent | 4/4 (100%) | 31 FRs | 98% | ✅ Clean |
| heliosCLI | 3/4 (75%) | 18 FRs | 85% | ⚠️ 1 pending |

**Pending Actions:**
- [ ] AgilePlus: Complete UJ.md (6 journeys drafted, needs approval)
- [ ] heliosCLI: Finalize ADR.md (4 ADRs reviewed, needs approval)
- [ ] All repos: Target 100% FR test coverage by 2026-04-11

---

## Spec Creation & Maintenance

### Adding a New Functional Requirement (FR)

**Steps:**

1. **Create feature branch:**
   ```bash
   git checkout -b specs/agent-<name>-fr-<number>
   ```

2. **Edit FUNCTIONAL_REQUIREMENTS.md:**
   ```markdown
   #### FR-REPO-NNN: Feature Name
   **Requirement:** System SHALL ...
   **Traces To:** Epic ID (e.g., E3.1)
   **Code Location:** `path/to/implementation.rs`
   **Repository:** repo-name
   **Status:** Active
   **Test Traces:** Tests in `tests/test_feature.rs`
   ```

3. **Create test:**
   ```rust
   #[test]
   fn test_feature_requirement() {
       // Traces to: FR-REPO-NNN
       assert!(/* requirement validation */);
   }
   ```

4. **Commit with trace:**
   ```bash
   git commit -am "specs: add FR-REPO-NNN

   Spec-Traces: FR-REPO-NNN
   Co-Authored-By: agent-name <agent@phenotype.local>"
   ```

5. **Push and auto-merge:**
   ```bash
   git push origin specs/agent-<name>-fr-<number>
   # → CI validates → Auto-merges to specs/main within 5 min
   ```

### Updating an Existing FR

**Process:** Same as adding, but use `Spec-Traces: FR-REPO-NNN` (existing ID)

### Deprecating an FR

**Steps:**

1. Update status to "Deprecated" in FUNCTIONAL_REQUIREMENTS.md
2. Reference replacement FR (if applicable)
3. Commit with reason:
   ```bash
   git commit -am "specs: deprecate FR-REPO-NNN

   Reason: Replaced by FR-REPO-MMM
   Spec-Traces: FR-REPO-NNN
   Related-Issues: #123"
   ```

---

## Registry Schema

**File:** `.specs/REGISTRY_SCHEMA.json`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Spec Registry Entry",
  "type": "object",
  "properties": {
    "repo": {
      "type": "string",
      "description": "Repository name (e.g., phenotype-infrakit)"
    },
    "spec_type": {
      "enum": ["FR", "ADR", "PLAN", "UJ"],
      "description": "Type of specification"
    },
    "file": {
      "type": "string",
      "description": "Path to spec file (e.g., FUNCTIONAL_REQUIREMENTS.md)"
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+(\\.\\d+)?$",
      "description": "Semantic version (e.g., 2.1, 1.0.0)"
    },
    "status": {
      "enum": ["draft", "review", "deployed", "deprecated"],
      "description": "Spec status"
    },
    "items_covered": {
      "type": "array",
      "items": { "type": "string" },
      "description": "IDs covered (e.g., [FR-INFRA-001, FR-INFRA-002])"
    },
    "fr_coverage": {
      "type": "number",
      "minimum": 0,
      "maximum": 100,
      "description": "Percentage of FRs with tests"
    },
    "last_updated": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp"
    },
    "owner": {
      "type": "string",
      "description": "Responsible team/role"
    }
  },
  "required": ["repo", "spec_type", "file", "version", "status"]
}
```

---

## Cross-Repo Spec Dependencies

### Traceable Dependencies

```
heliosCLI (FR-HELIOS-*)
  ├─→ depends on: phenotype-infrakit (FR-INFRA-*)
  └─→ depends on: platforms/thegent (FR-THEGENT-*)

AgilePlus (FR-AGILE-*)
  ├─→ depends on: phenotype-infrakit (FR-INFRA-*)
  └─→ optional: platforms/thegent (FR-THEGENT-*)

platforms/thegent (FR-THEGENT-*)
  ├─→ depends on: phenotype-infrakit (FR-INFRA-*)
  └─→ depends on: heliosCLI (FR-HELIOS-*)
```

**Validation:** CI checks that all cross-repo FR references are valid and bidirectional.

---

## Monitoring & Alerts

### Health Score Calculation

**Formula:**

```
Health Score = (Specs Completeness × 0.25)
             + (FR Test Coverage × 0.25)
             + (Merge Success Rate × 0.20)
             + (Agent Adoption × 0.20)
             + (Documentation × 0.10)
```

**Current Score:** 87.5/100

**Target (Phase 1):** 65/100 → 100/100 (by 2026-04-11)

### Alerts

**Low Health (<50):**
- Automated alert: Create GitHub issue "⚠️ SSOT Health Below 50"
- Notify specs-admin team

**Merge Failures (>5% in 24h):**
- Alert: "⚠️ High merge conflict rate: X%"
- Review conflict patterns

**Test Coverage Drop (<95%):**
- Alert: "⚠️ FR test coverage below 95%"
- List orphan FRs

---

## Related Documentation

| Document | Purpose |
|----------|---------|
| `FUNCTIONAL_REQUIREMENTS.md` | Master FR index (per-repo) |
| `ADR.md` | Architecture decision records |
| `PLAN.md` | Multi-phase implementation plans |
| `USER_JOURNEYS.md` | User workflow definitions |
| `.github/BRANCH_PROTECTION_SPECS_MAIN.md` | Branch protection rules |
| `docs/reference/SSOT_PHASE1_IMPLEMENTATION_PLAN.md` | Phase 1 execution roadmap |
| `docs/reference/SSOT_PHASE1_AGENT_WORKFLOW.md` | Agent branching procedures |

---

## FAQ

**Q: How do I add a new FR?**
A: Create branch `specs/agent-<name>-fr-<number>`, edit FUNCTIONAL_REQUIREMENTS.md, commit with `Spec-Traces: FR-REPO-NNN`, push. Auto-merges within 5 min.

**Q: What if my spec merge conflicts?**
A: Resolve conflict in your branch, push updated commit, wait for auto-merge. If unresolvable, issue created for manual review.

**Q: How do I verify my spec traces?**
A: CI automatically validates `Spec-Traces:` field. Use `python3 scripts/validate-spec-structure.py` locally before pushing.

**Q: When should I bump version numbers?**
A: When all FRs pass tests and no validation issues exist. Version bump happens weekly on Mondays.

**Q: Can I force-push to specs/main?**
A: No. Only SSOT service can force-push (emergency only). Use conflict resolution branch for your changes.

---

**Registry Owner:** Platform Architect
**Last Updated:** 2026-03-31 14:30 UTC
**Next Update:** Daily at 09:00 UTC (auto-generated)
