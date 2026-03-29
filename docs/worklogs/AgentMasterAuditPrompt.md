# Master Audit Prompt for Agents

## System Organization Context

```
CodeProjects/Phenotype/repos/
├── repos/                          # USE CASES (not a repo itself - use for stashes/backups only)
│   ├── agileplus/                  # Project: Agentic project management
│   ├── phenotype-infrakit/         # Project: Infrastructure tooling (THIS REPO)
│   ├── phenotype-shared-wtrees/    # Worktrees for phenotype-shared
│   ├── vibe-kanban/                # Project: Kanban board
│   └── [other project repos]/
├── .worktrees/                     # Local worktrees (non-canonical)
├── .archive/                       # Trash/old items
└── docs/worklogs/                  # CANONICAL: Audit worklogs (READ FIRST)
```

---

## Core Audit Directive

```
use subagents liberally identify duplication among and inside indiv projects\cross project.
opportunity for libification \ pattern generation \ productization \ consolidation \ general
(LOC-- Quality>=Same)
use web search to identify repos or packages that can be forked\used\wrapped et cetera as well
to help in this regard or generally just to improv sys if loc+ is small enough to justify;
focus on repo duplication too. look at both local states and remote.
some items i've noticed may be dupl
```

---

## Folder Handling Rules

### 1. Inactive Folders (NOT Canonical)

**Skip unless:**
- Ensure at latest: local AND `origin/main` with merged stashes
- Review for potential archival/migration

**Action:**
- Update to latest if behind
- Merge stashes
- Move to `.archive/` if truly inactive

### 2. Worktrees

**Action:**
- Finished + pushed to cloud → Open PR → Delete after merge review
- Keep clean `.worktrees/` directory

### 3. Canonical Folders

**Organization:**
- Top-level `repos/` contains PROJECTS (canonical folders)
- Each project should have:
  - Hyper-organized root level
  - `README.md` at top
  - `docs/` for documentation
  - `src/` for code
  - `tests/` for tests
  - `INDEX.md` for large directories

**For Agents:**
- Use canonical folders to run tests/validate
- Use worktrees for real work

---

## Where to Write

### AUDIT WORKLOGS (Primary)

| File | Purpose |
|------|---------|
| `docs/worklogs/DUPLICATION.md` | Cross-repo code duplication |
| `docs/worklogs/ARCHITECTURE.md` | Port/trait analysis, hexagonal patterns |
| `docs/worklogs/DEPENDENCIES.md` | External dependency audits |
| `docs/worklogs/RESEARCH.md` | Technology research |
| `docs/worklogs/PERFORMANCE.md` | Performance analysis |
| `docs/worklogs/GOVERNANCE.md` | Policy & compliance |
| `docs/worklogs/WorkLog.md` | Work item tracking |

### MASTER INDEX

| File | Purpose |
|------|---------|
| `docs/worklogs/README.md` | Index & aggregation guide |

### PLANS (Secondary - for detailed execution plans)

| Path | Purpose |
|------|---------|
| `plans/YYYY-MM-DD-*-PLAN-*.md` | Execution plans |
| `plans/YYYY-MM-DD-*-AUDIT-*.md` | Audit-specific plans |

---

## External 3rd Party Analysis (CRITICAL)

### For Each Dependency/Package, Analyze:

#### 1. Integration Level

| Level | Description | Example |
|-------|-------------|---------|
| **BLACKBOX** | Direct dependency, no modification | `use anyhow::Error` |
| **WHITEBOX** | Fork + modify | Fork `eventually` for custom traits |
| **WRAPPER** | Custom impl wrapping external | `phenotype-event-sourcing` wrapping `eventually` |
| **INSPIRATION** | Study patterns, implement differently | Study `casbin`, implement `phenotype-policy-engine` |
| **REPLACE** | Drop external for internal | Replace `serde_json` with `rmp` |

#### 2. Developer Quality Assessment

| Factor | Questions |
|--------|-----------|
| **Active Maintenance** | Last commit < 6 months? |
| **Community Size** | Stars, contributors, issues? |
| **Documentation** | Docs.rs, examples, guides? |
| **Breaking Changes** | Version stability? |
| **License** | Permissive for commercial use? |

#### 3. Fork/Modify Decision Matrix

```
                    High Quality Dev                    Low Quality Dev
                   /                    \              /                \
              Large Gap                                              Small Gap
             /        \                                          /            \
        FORK+WRAP   WRAP+CONTRIB                           WRAP          BLACKBOX
        (long-term) (medium-term)
```

#### 4. Document in RESEARCH.md

```markdown
### [Package Name]

**Integration Level:** BLACKBOX | WHITEBOX | WRAPPER | INSPIRATION | REPLACE

**Developer Quality:**
- Stars: X,XXX
- Contributors: XX
- Last Commit: YYYY-MM-DD
- License: MIT/Apache/Proprietary
- Breaking Changes: Frequent/Moderate/Rare

**Gap Analysis:**
- What we need: [feature list]
- What they have: [feature list]
- Delta: [gaps]

**Recommendation:** FORK | WRAP | BLACKBOX | REPLACE | MONITOR

**Effort:** X days | X weeks | X months

**Rationale:** [why this level of integration]
```

---

## Audit Categories

### 1. DUPLICATION AUDIT

**Focus:**
- [ ] Identify duplicate code within projects
- [ ] Identify duplicate code across projects
- [ ] Identify duplicate patterns (not just code)
- [ ] Identify duplicate dependencies
- [ ] Identify duplicate configurations

**Output:** `docs/worklogs/DUPLICATION.md`

### 2. LIBIFICATION AUDIT

**Focus:**
- [ ] Extractable patterns > 100 LOC
- [ ] Common utilities that could be shared
- [ ] Duplicated business logic
- [ ] Productization opportunities (internal tools → product)

**Output:** Section in `docs/worklogs/ARCHITECTURE.md`

### 3. EXTERNAL_PKG AUDIT

**Focus:**
- [ ] Identify custom implementations that match external packages
- [ ] Evaluate fork/modify/wrap opportunities
- [ ] Assess developer quality of external packages
- [ ] Cross-ecosystem analysis (crates.io, npm, PyPI, GitHub)

**Output:** `docs/worklogs/RESEARCH.md`

### 4. PATTERN_INVENTORY

**Focus:**
- [ ] Hexagonal architecture patterns
- [ ] Event sourcing patterns
- [ ] State machine patterns
- [ ] Policy engine patterns
- [ ] Repository patterns

**Output:** Section in `docs/worklogs/ARCHITECTURE.md`

### 5. SYS_IMPROVE_AUDIT

**Focus:**
- [ ] Quick wins < 50 LOC
- [ ] Code quality improvements
- [ ] Performance optimizations
- [ ] Security hardening

**Output:** `docs/worklogs/PERFORMANCE.md`

### 6. DECOMPOSITION_AUDIT

**Focus:**
- [ ] Microservice boundaries
- [ ] Module boundaries
- [ ] Shared kernel identification
- [ ] Coupling analysis

**Output:** Section in `docs/worklogs/ARCHITECTURE.md`

---

## Agent Coordination Protocol

### For Each Audit Task:

1. **Read canonical worklogs first**
   - `docs/worklogs/README.md` - Index
   - Relevant category file (e.g., `DUPLICATION.md`)

2. **Run subagents for parallel work**
   - Each subagent focuses on one aspect
   - Aggregate findings into canonical file

3. **Cite file locations**
   - Use `filepath:line` format
   - Example: `crates/agileplus-domain/src/error.rs:18-20`

4. **Use checkbox format**
   - `- [ ]` for pending
   - `- [x]` for completed

5. **Priority indicators**
   - 🔴 CRITICAL - Immediate action required
   - 🟡 HIGH - This week
   - 🟠 MEDIUM - This month
   - 🟢 LOW - Quarterly

---

## Output Requirements

### Checkbox Format

```markdown
## [Category] Duplication in [Repo/Package]

| Pattern | Location | LOC | Canonical Location |
|---------|----------|-----|-------------------|
| [desc]  | [path]   | N   | [where it should go] |

- [ ] Item 1
- [x] Item 2 (completed)
```

### File Location Citations

- Use `filepath:line` for single line
- Use `filepath:startLine-endLine` for ranges
- Example: `crates/agileplus-domain/src/error.rs:18-20`

### Libification Recommendations

When LOC > 100:
- Recommend extraction
- Estimate effort
- Identify dependencies

### External Fork/Wrap Candidates

When applicable:
- Link to original repo/package
- Assess integration level
- Estimate fork/modify effort

---

## Codebase Atlas Integration

For every pattern identified:
- **File path:** Exact location
- **Feature domain:** What it does
- **LOC:** Lines of code
- **Language:** Rust, TypeScript, Python, etc.

Flag opportunities for:
- **libification:** Extract to shared library
- **productization:** Internal tool → standalone product
- **fork candidates:** External repo worth forking
- **wrap candidates:** External pkg worth wrapping

---

## Cross-Ecosystem Search Examples

### Rust (crates.io)
```bash
# Event sourcing
cargo search eventually
cargo search event-sourcing
cargo search cqrs

# Policy engine
cargo search casbin
cargo search policy-engine

# State machines
cargo search state-machine
cargo search saga
```

### npm
```bash
# Event sourcing
npm search @eventually/core
npm search event-sourcing

# State machines
npm search xstate
npm search workflow-engine
```

### PyPI
```bash
# Event sourcing
pip search eventsourcing

# Policy engine
pip search casbin
```

### GitHub
```bash
# General search
gh search repos "event sourcing rust" --stars >10
gh search repos "policy engine" --language rust
```

---

## Example Agent Prompt (copy/paste for subagents)

```
## Your Task: [DUPLICATION|LIBIFICATION|EXTERNAL_PKG|PATTERN|SYS_IMPROVE] AUDIT

### Scope
- Project: [phenotype-infrakit|agileplus|vibe-kanban|CROSS-ALL]
- Focus: [specific crate|pattern|dependency]
- LOC Threshold: [>50|>100|>200]

### Instructions
1. Read canonical worklog: `docs/worklogs/[RELEVANT].md`
2. Search for duplication/patterns
3. Evaluate external alternatives if applicable
4. Update canonical worklog with findings
5. Use checkbox format for action items
6. Cite file locations with `filepath:line`

### Output
- Append findings to canonical worklog
- Use checkbox format
- Priority indicators: 🔴🟡🟠🟢

### External Analysis (if applicable)
For each external package:
- Integration level: BLACKBOX|WHITEBOX|WRAPPER|INSPIRATION
- Developer quality: Stars, last commit, license
- Recommendation: FORK|WRAP|BLACKBOX|REPLACE
```

---

## Metadata

| Field | Value |
|-------|-------|
| Version | 1.0 |
| Created | 2026-03-29 |
| Author | System |
| Purpose | Master audit directive for agents |
| Update | Add new audit categories as needed |

---

## Related

- Master Index: `docs/worklogs/README.md`
- Duplication Audit: `docs/worklogs/DUPLICATION.md`
- Architecture Audit: `docs/worklogs/ARCHITECTURE.md`
- Dependencies Audit: `docs/worklogs/DEPENDENCIES.md`
- Research: `docs/worklogs/RESEARCH.md`
- Performance: `docs/worklogs/PERFORMANCE.md`
- Governance: `docs/worklogs/GOVERNANCE.md`
- Work Log: `docs/worklogs/WorkLog.md`
