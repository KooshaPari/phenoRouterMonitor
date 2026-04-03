# Open Work — DAG and WBS

## Priority DAG

```
[Plane Fork Decision]       ← G037 (gate: org approval)
        ↓
[Spec 008 Created]
        ↓
[portage: Linux/WSL host readiness]   ← portage F0101
        ↓
[portage: pre-run metadata snapshot]  ← portage F0102
        ↓
[portage: quickcheck execution]        ← portage F0103-F0108
        ↓
[portage: evidence capture]            ← portage F0109-F0116
        ↓
[portage: release closeout]           ← portage F0117-F0124
        ↓
[heliosApp: return to main]           ← HAPP-001
        ↓
[heliosApp: native gates + evidence] ← HAPP-002
        ↓
[heliosCLI: return to main]          ← HCLI-001
        ↓
[heliosCLI: docs/runtime lane]       ← HCLI-002
```

## Work Packages by Repo

### AgilePlus (G037)

| WP | Description | Status |
|----|-------------|--------|
| G037-WP1 | Fork Plane repo into org GitHub | pending |
| G037-WP2 | Define AgilePlus → Plane API boundary adapter | pending |
| G037-WP3 | Migrate or quarantine duplicate PM dashboard code | pending |
| G037-WP4 | Wire existing controls into Plane | pending |
| G037-WP5 | Validate co-existence with Plane | pending |
| G037-WP6 | Archive TracerTM and TheGent from PM surface | pending |

### portage

| WP | Description | Status |
|----|-------------|--------|
| portage-F0101 | Confirm Linux/WSL host availability | pending (host-gated) |
| portage-F0102 | Record pre-run evidence snapshot | pending (host-gated) |
| portage-F0103-F0108 | Execute quickcheck and capture summary | pending (host-gated) |
| portage-F0109-F0116 | Follow-on evidence capture | pending (host-gated) |
| portage-F0117-F0124 | Release closeout and handoff | pending (host-gated) |

### heliosApp

| WP | Description | Status |
|----|-------------|--------|
| HAPP-001 | Canonical status reconciliation | pending |
| HAPP-002 | Native gates and evidence refresh | pending |
| HAPP-003 | Reference hardware replay | pending |
| HAPP-004 | Adapter parity audit | pending |
| HAPP-005 | Desktop MVP parity validation | pending |
| HAPP-006 | Dependency canary and rollback | pending |
| HAPP-007 | Cross-repo contract freeze | pending |
| HAPP-008 | Release packet refresh and worktree integration | pending |

### heliosCLI

| WP | Description | Status |
|----|-------------|--------|
| HCLI-001 | Canonical status reconciliation | pending |
| HCLI-002 | Docs/runtime lane quarantine and evidence classification | pending |
| HCLI-003 | Absorb stack allowlist/blocklist completion | pending |
| HCLI-004 | Strictness parity and governance normalization | pending |
| HCLI-005 | Failed evidence target replay and index refresh | pending |
| HCLI-006 | Candidate shortlist and go/no-go freeze | pending |
| HCLI-007 | Harness execution contract promotion | pending |
| HCLI-008 | Release packet refresh and return-to-main integration | pending |
