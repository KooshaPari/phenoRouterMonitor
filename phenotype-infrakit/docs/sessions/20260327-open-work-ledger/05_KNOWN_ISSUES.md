# Open Work — Known Issues

## Cross-Cutting Blockers

- **Linux/WSL host required** — portage quickcheck cannot run on macOS. Needs a Linux/WSL host with Node.js/Bun available.
- **Org approval required** — G037 Plane fork decision needs explicit approval before implementation begins.
- **GitHub admin access required** — G037-WP1 (fork Plane) requires GitHub org admin to create the fork.

## heliosApp

- Integration suite has 5 remaining failures (performance/benchmark timeouts, not correctness bugs): storage persistence size, audit search performance (3x), Ghostty GPU status. These are sandbox timing issues, not code bugs.
- Desktop MVP spec families (03, 05, 07) are not yet fully classified as canonical vs. stubs.

## heliosCLI

- Many repos in `artifacts/phase-2/evidence-index.ndjson` still show WARN (commands:0) and MISSING status. Moving these to PASS requires adding per-repo command definitions and ensuring clones are available.
- Canonical checkout is detached HEAD (no branch) — needs a return-to-main integration before the repo can be merged.
