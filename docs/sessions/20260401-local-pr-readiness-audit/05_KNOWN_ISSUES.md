# Known Issues

## agentapi-plusplus

- `go build ./...` and `go test ./...` fail immediately because the repo vendor snapshot is out of
  sync with `go.mod`.
- `go build -mod=mod ./...` still fails in pre-existing code at
  `lib/screentracker/pty_conversation.go` due to an undefined `strings` reference.
- `bash ./scripts/quality-gate.sh verify` fails on broad existing `gofmt` drift, including tracked
  vendored files.
- The repo root is still noisy because `docs/node_modules` is tracked and currently appears as a
  large unrelated deletion set. PR prep must avoid staging or reverting that unrelated drift.

## thegent

- The live PR lane is in `.worktrees/thegent-pr908-policy-fix`; the root checkout on `main` is not
  the branch that should be updated for PR `#908`.
- `policy-gate` is fixed and passing on the refreshed PR head, but `#908` still has real failures
  in `CodeQL Advanced`, `SAST Quick Check`, `SonarCloud`, and repo-specific lint/license checks.
- `security/snyk (kooshapari)` is still failing for account-limit reasons, and `CodeRabbit` is
  intermittently failing with review rate-limit exhaustion rather than repository code defects.
- The PR still carries unresolved review threads that will continue to trip `pr-governance-gate`
  until comment resolution catches up with the code lane.

## heliosCLI

- The live PR lanes are `.worktrees/helioscli-pr179-policy-fix` for PR `#179` and
  `heliosCLI/.worktrees/chore-govern-pi` for PR `#182`; the repo root stays on `main`.
- The false-positive merge-history `policy-gate` failure is fixed on both active branches, and the
  generated-doc `codespell` typo was corrected on both branches.
- Local validation for PR `#179` passes:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- Remaining GitHub blockers are now non-local or review-driven:
  - `security/snyk (kooshapari)` is failing because the private test quota is exhausted
  - some long-running platform and Bazel jobs are still pending or noisy
  - `pr-governance-gate` will still fail while requested changes and unresolved review threads stay open
  - refreshed runs may temporarily continue to show stale pre-patch Snyk or review-app failures
    until the new branch heads finish processing

## cliproxyapi-plusplus

- PR `#942` is no longer dirty, but it is still blocked pending fresh CI on head `52b60351`.
- `security/snyk (kooshapari)` remains an external quota or billing blocker.
- Local `go vet ./...` is still blocked by pre-existing import-cycle and Go proxy fetch failures,
  so Go-quality green status depends on the GitHub runner environment and further repo cleanup.
- The repo-local custom Semgrep rules are now valid YAML and Go-oriented, but they are not yet safe
  to hard-gate this repository without a dedicated false-positive reduction pass.
