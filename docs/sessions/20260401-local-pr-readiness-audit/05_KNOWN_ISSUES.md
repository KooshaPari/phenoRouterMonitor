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
- `policy-gate` is fixed on the refreshed PR head, but `#908` still has real failures or review
  pressure in `CodeQL Advanced`, `SonarCloud`, and repo-specific lint/license checks.
- `security/snyk (kooshapari)` is still failing for account-limit reasons, and `CodeRabbit` is
  intermittently failing with review rate-limit exhaustion rather than repository code defects.
- The PR still carries unresolved review threads that will continue to trip `pr-governance-gate`
  until comment resolution catches up with the code lane.
- `CODEOWNERS` is still modified in the worktree and was not part of the latest immutable-pin pass.

## heliosCLI

- The live PR lanes are `.worktrees/helioscli-pr179-policy-fix` for PR `#179` and
  `heliosCLI/.worktrees/chore-govern-pi` for PR `#182`; the repo root stays on `main`.
- The false-positive merge-history `policy-gate` failure is fixed on both active branches, and the
  generated-doc `codespell` typo was corrected on both branches.
- Local validation for PR `#179` passes:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- The PR `#179` worktree now also has a fresh local delta in `Cargo.toml`, `Cargo.lock`,
  `policy-gate.yml`, and `pr-governance-gate.yml`; those changes still need push plus fresh CI.
- Remaining GitHub blockers are now non-local or review-driven:
  - `security/snyk (kooshapari)` is failing because the private test quota is exhausted
  - some long-running platform and Bazel jobs are still pending or noisy
  - `pr-governance-gate` will still fail while requested changes and unresolved review threads stay open
  - refreshed runs may temporarily continue to show stale pre-patch Snyk or review-app failures
    until the new branch heads finish processing

## cliproxyapi-plusplus

- PR `#942` is no longer dirty, but it is still blocked pending fresh CI on the refreshed head.
- `security/snyk (kooshapari)` remains an external quota or billing blocker.
- `Go Quality` was failing on repo-local formatting drift; the exact files GitHub flagged have now
  been `gofmt`-normalized locally and still need push plus rerun.
- Local `go vet ./...` is still blocked by a mix of pre-existing import-cycle, duplicate-symbol,
  and upstream module-resolution failures that are not isolated to the current PR diff.
- `main` itself has recent failing `CI` runs, so the remaining compile failures on `#942` cannot be
  treated as solely introduced by this PR.
- `sast-quick.yml` now carries a changed-file Semgrep path filter locally, but the repo still needs
  a dedicated false-positive reduction pass before Semgrep can be trusted as a broad full-repo gate.

## heliosApp

- The local checkout is clean, but PR `#362` is still blocked by missing repo scripts
  (`bun run test`, `bun run test:coverage`, `bun run docs:index`) and an invalid `.oxfmtrc.json`.
- The branch also carries concrete lint/test cleanup items in the desktop/runtime/logger/ids
  surfaces surfaced during the expanded audit.
- External review and security apps are still noisy there:
  - Snyk private-test quota exhaustion
  - intermittent CodeRabbit/Kilo parse or rate-limit churn
  - Socket alerts on dependency updates

## AgilePlus

- The root checkout is dirty on `main`, ahead of `origin/main` by one local commit, and is not in
  a state where a new PR should be opened directly.
- The current lane mixes workflow/governance changes, runtime and CLI edits, process-compose drift,
  worklog changes, and untracked plan/spec files.
- There are no stashes or detached worktrees to peel the PR slice out automatically; the next step
  there is manual isolation into a clean branch or worktree.

## agentapi-plusplus

- The active root checkout is on `main` with broad unrelated drift, including tracked
  `docs/node_modules` deletions, so the current state is not safe for direct PR prep.
- The live PR lane remains `#398`, but its intended fixes are currently buried under unrelated repo
  noise and should be moved into a clean worktree before further push activity.
