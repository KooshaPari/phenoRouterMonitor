# soul.md — repos shelf agent identity & configuration manifest

## identity

**name**: repos shelf  
**nature**: polyrepo containing ~200+ independent git repositories  
**home**: `/Users/kooshapari/CodeProjects/Phenotype/repos`  
**guardian**: kooshapari  
**purpose**: Agent orchestration, DevOps tooling, development frameworks, and the Phenotype Federated Hybrid Architecture

---

## what we are

A "shelf" — an organizational layer above individual projects. Think of it like `~/code/` or `/opt/` but versioned as a polyrepo containing related but independent repositories. Each project is a standalone git repo; the shelf is their shared home.

**Four agent roles operate here:**
| Agent | Role | Specialty |
|-------|------|-----------|
| **Forge** | Main coding/impl agent | Feature implementation, architecture |
| **Muse** | Code review/quality | Review, polish, critique |
| **Sage** | Research/investigation | Analysis, exploration, debugging |
| **Helios** | Runtime/testing specialist | Testing, CI/CD, deployment |

---

## koosha's patterns

### work style
- **AgilePlus-first**: All work tracked in AgilePlus (`cd AgilePlus && agileplus <command>`)
- **Spec-driven**: No code without corresponding AgilePlus spec
- **Quality-obsessed**: Uses `task quality` for tach/vale/ruff checks
- **Multi-language**: Rust (primary), TypeScript, Go, Python, Zig, Swift, Kotlin, Elixir
- **Hexagonal architecture**: Heavy use of ports/adapters patterns across languages
- **Template-driven**: 15+ language/domain templates for bootstrapping
- **Naming convention**: Uses `*kit` suffix (authkit, cachekit, logkit, etc.)

### conventions observed
- Git worktrees for feature work (`.worktrees/<project>/<category>/<branch>`)
- Session documentation in `docs/sessions/<session-id>/`
- Worklogs aggregated at shelf level (`worklogs/`)
- UTF-8 mandatory for all markdown
- Conventional commits: `<type>(<scope>): <description>`
- File size limits: ≤350 lines target, ≤500 hard limit
- All public types implement `Debug` and `Clone`
- `thiserror` for Rust error types

---

## existing configuration audit

### .claude (shelf-level) — 13 locations found
**Root**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.claude/`
```
.claude/
├── settings.json              # MCP server hooks (DINOForge game automation)
├── mcp-servers.json           # DINOForge server config
├── AGENT_ASSIGNMENTS.md       # Star Wars mod agent matrix (413 lines)
├── EXECUTION_SUMMARY.md        # Phase tracking
├── PHASE_DEPENDENCY_DAG.md     # Dependency visualization
├── README_STARWARS_PLAN.md     # Campaign documentation
├── STARWARS_COMPLETION_PLAN.md # Detailed roadmap
├── nuget_publish_guide.md      # Publishing workflow
├── commands/                   # 30+ command files
│   ├── add-unit.md
│   ├── asset-create.md
│   ├── build-all.sh
│   ├── check-ci.md
│   ├── check-game.md
│   ├── deploy.sh
│   ├── dev-harness.md
│   ├── dump-analyze.sh
│   ├── entity-dump.md
│   ├── eval-*.md (multiple)
│   ├── game-*.md (multiple)
│   ├── launch-game.md
│   ├── new-pack.md
│   ├── pack-deploy.md
│   ├── prove-*.md (multiple)
│   ├── release.md
│   ├── spawn-unit.md
│   ├── status.md
│   ├── test*.sh
│   └── validate.sh
├── contracts/                  # Project contracts
└── sw_search/                  # Star Wars search utilities
```

**Project-level .claude directories:**
- `thegent/.claude/` — 15 spec-kitty command files
- `AgilePlus/.claude/`
- `Profila/.claude/`
- `phenoSDK/.claude/`
- `Dino/.claude/`
- `AgentMCP/.claude/`
- `Tracera/.claude/`
- Plus worktree directories

### .codex (9 locations)
- `helios-cli/.codex/`
- `thegent/.codex/`
- `AgilePlus/.codex/`
- `Tracera/.codex/`
- Plus others

### .cursor (8 locations)
- `thegent/.cursor/`
- `AgilePlus/.cursor/`
- `cloud/.cursor/`
- `Tracera/.cursor/`
- Plus others

### .serena (15+ locations)
**Root**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.serena/`
```yaml
project_name: "repos"
languages: [vue]
encoding: utf-8
ignore_all_files_in_gitignore: true
read_only: false
excluded_tools: []
base_modes: []
default_modes: []
initial_prompt: ""
```

**Project-level .serena/:**
- `thegent/.serena/`
- `Tracera/.serena/`
- `portage/.serena/`
- Plus many others

### .forge (3 locations)
**Root**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.forge/`
```
.forge/
├── LOC_REDUCTION_OPPORTUNITIES.md
├── PR_DESC_1.md
└── PR_DESC_2.md
```

### agents.toml (5 files)
All identical, referencing `getsentry/dotagents`:
- `/Users/kooshapari/CodeProjects/Phenotype/repos/agents.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/portage/agents.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/phenodocs/agents.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/helios-cli/agents.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/agents.toml`

### CLAUDE.md files (20+)
**Root**: Has MERGE CONFLICTS (needs cleanup)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/agent-devops-setups/CLAUDE.md` — 841 lines, global instructions
- `/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/CLAUDE.md` — Clean, AgilePlus-integrated
- `/Users/kooshapari/CodeProjects/Phenotype/repos/Profila/CLAUDE.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/CLAUDE.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/phenoSDK/CLAUDE.md`
- Plus many more

### .claudeignore (6 files)
- `helios-cli/.claudeignore`
- `AgilePlus/.claudeignore`
- `Profila/.claudeignore`
- `phenotype-dep-guard/.claudeignore`
- Plus templates

### harnesses/
Agent harness configurations at `/Users/kooshapari/CodeProjects/Phenotype/repos/harnesses/`:
```
harnesses/
├── CLAUDE-CODE.md     # Claude Code integration + AgilePlus workflow
├── CODEX.md           # Codex CLI integration
└── CURSOR.md          # Cursor IDE integration
```

### cloud profile infrastructure
**Rich profile system** in `cloud/src/lib/agent/`:
```
cloud/src/lib/agent/
├── profile-service.ts         # CRUD for agent environment profiles
├── profile-vars-service.ts     # Environment variable management
├── profile-commands-service.ts # Setup commands management
├── profile-session-config.ts   # Session configuration merging
├── profile-utils.ts            # Utility functions
└── types.ts                    # TypeScript type definitions
```

Router: `cloud/src/routers/agent-profiles-router.ts`  
Mock data: `cloud/storybook/src/mockData/profiles.ts`

---

## gaps & opportunities

### critical issues
1. **CLAUDE.md and AGENTS.md at root have git merge conflicts** — blocks reliable canonical docs
2. **No unified profile/soul system** — Agent roles defined but no central config management
3. **No hermes-specific configuration** — References to "hermes" only for NousResearch AI model

### missing (immediate needs)
| Item | Priority | Location |
|------|----------|----------|
| soul.md (this file) | DONE | `/Users/kooshapari/CodeProjects/Phenotype/repos/soul.md` |
| Merge conflict resolution | HIGH | CLAUDE.md, AGENTS.md |
| Unified profile sync | HIGH | Cross-project profile sharing |
| Hermes MCP bridge | MEDIUM | Connect to cloud profile system |
| Skill registry consolidation | MEDIUM | Central skill catalog |
| Plugin discovery system | MEDIUM | Auto-source plugins from repos |

---

## recommended architecture

### phase 1: foundation (immediate)
1. **Resolve merge conflicts** in CLAUDE.md and AGENTS.md
2. **Establish soul.md as canonical** identity document
3. **Create `.soul/` directory** for profile configurations:
   ```
   .soul/
   ├── profiles/           # Agent role profiles
   │   ├── forge.yaml
   │   ├── muse.yaml
   │   ├── sage.yaml
   │   └── helios.yaml
   ├── plugins/            # Plugin registry
   │   ├── index.yaml
   │   └── sources/
   ├── contexts/           # Project context templates
   │   ├── rust-kit.yaml
   │   ├── ts-app.yaml
   │   └── go-cli.yaml
   └── config.yaml         # Master config
   ```

### phase 2: profile system (next)
1. **Bridge cloud profile system** to local agent configs
2. **Create profile selection mechanism** based on task type
3. **Sync environment variables** from cloud to local contexts
4. **Implement profile-aware routing** for agent selection

### phase 3: hermes integration (future)
1. **Hermes MCP server** connecting to existing profile infrastructure
2. **Dynamic skill loading** based on project context
3. **Plugin marketplace integration** with cloud registry
4. **Auto-discovery** of new capabilities across repos

---

## project catalog summary

**Total**: ~200+ projects across categories:

| Category | Count | Examples |
|----------|-------|----------|
| Rust crates | 45+ | phenotype-event-sourcing, cache-adapter, policy-engine, state-machine |
| TypeScript/JS | 35+ | cloud, phenoSDK, AgilePlus UI |
| Go | 15+ | helios-cli, pheno-cli, Kogito |
| Python | 10+ | phenotype-middleware-py, phenotype-research-engine |
| Templates | 15+ | template-lang-*, template-domain-* |
| CLI tools | 20+ | sharecli, Cmdra, Logify, Quillr |
| Infra/Ops | 15+ | AgentMCP, agentapi-plusplus, portage |
| Docs/Config | 20+ | phenotype-docs-engine, governance |

**Key projects:**
- **AgilePlus** — Spec-driven delivery framework (Rust monorepo)
- **thegent** — Dotfiles manager with spec-kitty command framework
- **heliosCLI/helios-cli** — CLI frameworks
- **cloud** — Cloudflare workers, profile infrastructure
- **phenoSDK** — SDK for the ecosystem
- **Dino** — Game/modding platform (DINOForge)

---

## next steps

### immediate (this session)
1. ✅ Created soul.md (this document)
2. 🔲 Resolve CLAUDE.md merge conflicts
3. 🔲 Resolve AGENTS.md merge conflicts
4. 🔲 Audit cloud profile system for integration points

### short term (next sessions)
1. Create `.soul/` directory structure
2. Define Forge/Muse/Sage/Helios profiles
3. Bridge cloud profile API to local config
4. Implement profile-based agent routing

### medium term (future work)
1. Hermes MCP server implementation
2. Plugin discovery and auto-loading
3. Skill registry consolidation
4. Cross-project context sharing

---

## how to use this soul

**For agents:**
- Read this file at session start to understand context
- Reference identity section for work style alignment
- Check gaps section for current priorities
- Follow phase recommendations for roadmap

**For koosha:**
- This document captures your infrastructure and preferences
- Edit the identity/patterns sections to refine
- Use gaps section to track technical debt
- Reference architecture section for planning

---

*Created: 2026-04-02 by Hermes Agent (Forge mode)*  
*Location: `/Users/kooshapari/CodeProjects/Phenotype/repos/soul.md`*  
*Related: `CLAUDE.md`, `AGENTS.md`, `README.md`, `harnesses/`, `.claude/`, `.serena/`, `cloud/src/lib/agent/`*
