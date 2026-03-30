# Open Work — Testing Strategy

## Validation by Repo

### AgilePlus (G037)

- WP1: GitHub fork confirmed, branch visible in org GitHub
- WP2-WP5: `cargo check` on boundary adapter, `process-compose up` for co-existence
- WP6: TracerTM/TheGent no longer appear in PM surface audit

### portage

- Each packet step is validated by running the documented command and capturing output
- Evidence is committed to `libs/portage/docs/sessions/20260303-portage-runtime-followup/`
- F0071 (host-verified quickcheck) is the gate: non-SKIP result required to close F0103

### heliosApp

- HAPP-001: git status confirmed (canonical is detached, needs return-to-main)
- HAPP-002: `bun run gates --json` → PASS; `bun run test:integration` → 242/247 pass
- HAPP-003: reference hardware replay requires hardware access
- HAPP-004-HAPP-008: per-spec evidence captured in session package

### heliosCLI

- HCLI-001: git status confirmed (canonical is detached, needs return-to-main)
- HCLI-002: `bash commands/execute-phase-2-harness.sh` ran to completion
- HCLI-003-HCLI-005: evidence index refreshed, WARN/MISSING repos documented
- HCLI-006-HCLI-008: candidate shortlist and release packet pending
