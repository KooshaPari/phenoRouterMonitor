# Project-Level Worklog: heliosCLI

**Project:** heliosCLI | **Updated:** 2026-03-29

---

## Summary by Priority

| Priority | Count | Items |
|----------|-------|-------|
| P0 | 1 | PTY Process Utilities (FORK-001) |
| P1 | 2 | Error Handling (FORK-002), CLI Modernization |
| P2 | 3 | Framework Audit, Architecture Patterns, Codecrafters-Style Learning |
| P3 | 1 | LLM Inference Optimization |

---

## All Entries (Chronological)

### 2026-03-29 - PTY Process Utilities (READY FOR FORK)

**Status:** pending | **Priority:** P0

**Categories:** dependencies, duplication

**Summary:** PTY utilities in `platforms/heliosCLI/codex-rs/utils/pty` are used by 3+ repos and should be forked to `phenotype-process`.

**Details:**
| Attribute | Value |
|-----------|-------|
| LOC | ~750 |
| Usage | vibe-kanban, agileplus-git, thegent |
| Cross-platform | Unix + Windows ConPTY |
| Priority | CRITICAL |

**Next Steps:**
- [ ] Fork to `libs/phenotype-process`
- [ ] Migrate users to new crate
- [ ] Test on CI (Unix + Windows)
- [ ] Publish migration guide

---

### 2026-03-29 - Error Handling Patterns (READY FOR FORK)

**Status:** pending | **Priority:** P1

**Categories:** dependencies, duplication

**Summary:** Error handling in `platforms/heliosCLI/codex-rs/core/src/error.rs` should be forked to `phenotype-error`.

**Details:**
| Attribute | Value |
|-----------|-------|
| LOC | ~400 (pattern), ~1148 (total) |
| Usage | 135 files across repos |
| Features | `is_retryable()`, `to_protocol_error()` |
| Priority | CRITICAL |

**Next Steps:**
- [ ] Fork to `libs/phenotype-error`
- [ ] Add AgilePlus-specific variants
- [ ] Create migration guide
- [ ] Migrate users gradually

---

### 2026-03-29 - CLI Modernization Audit (COMPLETED)

**Status:** completed | **Priority:** P1

**Categories:** architecture, dependencies

**Summary:** Audited heliosCLI for modernization opportunities.

**Findings:**
| Category | Current | Recommendation |
|----------|---------|----------------|
| CLI parsing | clap | Already optimal |
| Async runtime | tokio | Already optimal |
| Process management | raw fork | Migrate to `phenotype-process` |
| Config format | TOML | Consider figment |
| Progress feedback | None | Add indicatif |

**Dependencies to Add:**
```toml
# Missing modern tooling
indicatif = "0.17"
figment = "0.10"
```

**Next Steps:**
- [ ] Add indicatif for progress feedback
- [ ] Evaluate figment for config
- [ ] Fork PTY utilities

---

### 2026-03-29 - heliosCLI Architecture Patterns (COMPLETED)

**Status:** completed | **Priority:** P2

**Categories:** architecture

**Summary:** Reviewed heliosCLI architecture patterns for consistency.

**Pattern Comparison:**
| Pattern | heliosCLI | AgilePlus | Alignment |
|---------|-----------|-----------|-----------|
| Error handling | thiserror | thiserror | ✅ Match |
| CLI parsing | clap | clap | ✅ Match |
| Async runtime | tokio | tokio | ✅ Match |
| Config format | TOML | TOML | ✅ Match |

**Recommendations:**
1. Adopt `phenotype-error` when forked
2. Standardize on `command-group` for process management
3. Add `indicatif` for progress feedback
4. Document architecture decisions as ADRs

---

### 2026-03-29 - Framework Audit (COMPLETED)

**Status:** completed | **Priority:** P2

**Categories:** architecture

**Summary:** Audit of framework patterns across heliosCLI crates.

**Crates Reviewed:**
- `codex-rs/core`
- `codex-rs/utils`
- `codex-rs/cli`
- `codex-rs/agent`

**Findings:**
- Consistent use of `thiserror`
- Async patterns via `async-trait`
- Config via manual TOML parsing
- Missing progress feedback

---

### 2026-03-29 - Codecrafters-Style Learning Mode (CONCEPT)

**Status:** concept | **Priority:** P2

**Categories:** research

**Summary:** Consider adding codecrafters-style learning mode to heliosCLI.

**Concept:**
```
helios learn git
# → Interactive git internals tutorial
# → Build git from scratch
# → Compare with production implementation
```

**Similar To:**
- codecrafters-io/build-your-own-x (starred repo)
- GitHub's git tutorial

**Next Steps:**
- [ ] Evaluate feasibility
- [ ] Design learning curriculum
- [ ] Prototype first module

---

### 2026-03-29 - LLM Inference Optimization (CONCEPT)

**Status:** concept | **Priority:** P3

**Categories:** performance, research

**Summary:** Research from KushDocs on LLM inference optimization.

**Options:**
| Solution | Pros | Cons |
|----------|------|------|
| SGLang | Faster than vLLM | Newer, less tested |
| vLLM | Proven, PagedAttention | Slower than SGLang |
| Local (llama.cpp) | Privacy, no API cost | Slower, less capable |

**Next Steps:**
- [ ] Benchmark current setup
- [ ] Evaluate SGLang vs vLLM
- [ ] Consider hybrid approach

---

## Aggregation by Category

| Category | Entries | Items |
|----------|---------|-------|
| architecture | 3 | CLI Modernization, Architecture Patterns, Framework Audit |
| dependencies | 2 | PTY Process (FORK-001), Error Handling (FORK-002) |
| duplication | 1 | Dependencies Worklog |
| performance | 1 | LLM Inference |
| research | 1 | Codecrafters-Style Learning |

---

## Next Actions (This Week)

1. **P0**: Fork `utils/pty` to `phenotype-process`
2. **P1**: Fork `error.rs` to `phenotype-error`
3. **P1**: Add indicatif for progress feedback
4. **P2**: Evaluate figment for config
5. **P3**: Benchmark LLM inference

---

## Dependencies Status

| Dependency | Current | Target | Priority |
|------------|---------|--------|----------|
| tokio | 1.x | 1.x | ✅ Optimal |
| clap | 4.x | 4.x | ✅ Optimal |
| thiserror | 2.x | phenotype-error | P1 |
| git2 | 0.18 | gix | P2 |
| process-utils | internal | phenotype-process | P0 |
| indicatif | missing | 0.17 | P1 |
| figment | missing | 0.10 | P2 |

---

**Last Updated:** 2026-03-29
**Aggregated by:** worklogs/aggregate.sh
