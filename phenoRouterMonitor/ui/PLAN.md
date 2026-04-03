# HELIOS HARNESS EXECUTION PLAN

This plan replaces the overlapping research-style drafts and defines a concrete, low-risk
execution path with explicit dependencies.

## Current state (ground truth)

- Rust crates exist for core harness services and already replace most Python paths.
- Remaining migration opportunities are concentrated in:
  - `harness_cache`
  - `harness_runner`
  - `harness_scaling`
  - `harness_schema`
- Tooling and architecture guidance in use: Rust workspace + generated protocol artifacts.
- No confirmed workload has been published that justifies keeping Mojo/Zig/assembly as active core runtime
  components.

## Policy decisions

1. Prioritize measured performance work, not language novelty.
2. Keep behaviorally required generated TypeScript and JSON schema artifacts; do not drop TS support unless no
   downstream consumers remain.
3. Introduce inline assembly only when a verified hot path fails to meet targets after profiling and no stable
   compiler intrinsics exist.
4. Treat Zig and Mojo as optional satellites; use only where a measurable win is clear:
   - Zig: only for deterministic C ABI shims or ultra-low-level memory/compression components if benchmarks demand it.
   - Mojo: only for vectorized numeric/Ml-like workloads with clear compute-heavy justification.
5. Migrations must be gated by tests, benchmark baselines, and reversible fallback paths.

## Execution WBS (phased + dependencies)

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| 1 | P1A | Establish baseline benchmarks for cache/runner/scaling paths | - |
| 1 | P1B | Add benchmark harness + regression guard (latency p50/p99, throughput) | P1A |
| 1 | P1C | Audit current cache/runner/scaling ownership and prune dead code | P1B |
| 2 | P2A | Optimize `harness_cache` with profile-driven fixes (lock strategy, TTL/LRU validation) | P1B, P1C |
| 2 | P2B | Stabilize error handling and observability across all four crates | P2A |
| 3 | P3A | Define migration candidates by measured hotspot (LOC + profile) | P2A, P2B |
| 3 | P3B | Replace first candidate in Rust with clear fallback and tests | P3A |
| 3 | P3C | Migrate adjacent candidate(s) only if acceptance criteria are met | P3B |
| 4 | P4A | Add/remove temporary compatibility shims after first successful migration | P3C |
| 4 | P4B | Decommission deprecated Python modules once downstream callers are switched | P4A |
| 5 | P5A | Document final architecture and handover (no web-hallucinated claims) | P4B |

## Dependency DAG (explicit order)

- P1A -> P1B -> P1C
- P1C -> P2A
- P2A -> P2B -> P3A
- P3A -> P3B -> P3C -> P4A -> P4B -> P5A

## Scope for "no TS" / polyglot review

- **TypeScript (keep):** protocol and CLI-facing contracts still rely on generated TS for consumers.
- **Zig (gated):** only add if benchmark evidence shows a concrete win in C-level interoperability or data movement.
- **Mojo (gated):** only add if repeated numeric workloads dominate wall time and can be isolated behind a stable crate boundary.
- **Inline asm (gated):** avoid by default; prefer compiler intrinsics and clear Rust alternatives.
- Go/no-go escalation criteria for satellites: `P3B` remains Rust-first unless all three hold:
  - one workload exceeds 20% of end-to-end runtime in a hot path,
  - the hotspot is compatible with a stable ABI boundary,
  - and the change introduces measurable p99 gain after one week of dry-run traces.

## Acceptance criteria

- Baseline and post-change metrics recorded for each migrated path.
- No measurable UX regressions in TUI/CLI behavior.
- No untested behavioral edits to input handling, suspend/resume, or protocol boundaries.
- Each phase ships with docs/tests matching changed behavior.
- For each benchmarked path, record at least three workload runs and enforce hard numeric targets:
  - p50 latency improvement >= 15%
  - p99 latency improvement >= 10%
  - throughput improvement >= 10%
  - error/failure rate not worse than baseline by more than 2%
- For each phase, gate promotion with explicit artifact set:
  - `phase-metrics/<phase>-before.json`
  - `phase-metrics/<phase>-after.json`
  - `phase-artifacts/<phase>-evidence.md` containing canary results and rollback rationale.
- If any target is not met, stop and rework before moving to next phase.

## Migration gates (decommission path)

- P4A: keep temporary compatibility shims behind explicit feature flags and capture canary traffic for at least one full operational interval before removal.
- P4B sequence (one module at a time):
  1. `harness_cache`
  2. `harness_runner`
  3. `harness_scaling`
  4. `harness_schema`
- Per-module gate before next step:
  - all associated tests + canary for previous module pass,
  - post-change canary error/failure delta <= 2%,
  - removal evidence published in `phase-artifacts/<module>-decommission.md`.
- Rollback gate for each decommission step:
  - if canary error budget for the same interval fails, fully rollback the latest decommission module (re-enable prior shim, restore compatibility path), and pause progression.

## Open risks

- Toolchain complexity from extra language runtimes (build/test time inflation).
- Feature pressure to "remove" TS prematurely before client migration.
- False-positive perf improvements from non-representative benchmarks.
- Long-tail UI bindings behavior changes without golden snapshot validation.

## Owner and governance

Any new work in this plan should be implemented in a fresh reviewable branch per area and merged back only when
benchmarks plus tests are green.
