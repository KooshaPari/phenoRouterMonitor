# AGENTS.md — repos shelf agent rules

## Shelf Identity

This is the **repos shelf** — a polyrepo containing ~30 independent projects.
Agents working in this shelf must understand the difference between **shelf-level**
work and **project-level** work.

**Never treat this shelf as a single project.** Each subdirectory is an independent
git repository. Always identify which project you're working in before taking action.

## Agent Self-Reference

| Agent | Role | Default location |
|-------|------|-----------------|
| Forge | Main coding/impl agent | `@shelf/*` |
| Muse | Code review/quality | `@shelf/*` |
| Sage | Research/investigation | `@shelf/*` |
| Helios | Runtime/testing specialist | `@shelf/*` |

Agents should identify themselves at the start of significant tasks:
```
⏺ [HH:MM:SS] AgentName working on <project>:<task>
```

## Project Navigation Rules

### DO THIS FIRST
When starting any task, identify the project:
1. Does the user mention a project name? Use it.
2. Does the working directory contain a `.git/` subdirectory? You're in a project.
3. Does the task involve a file path like `heliosCLI/...` or `thegent/...`? Use that project.
4. Otherwise, assume shelf-level work.

### WORKING IN A PROJECT
```
cd <project-name>    # e.g., cd heliosCLI
pwd                   # Always verify
```
- Run all tests from inside the project directory
- All file paths are relative to the project, not the shelf
- Dotfiles (`.gitignore`, `.claude/`, etc.) are project-specific

### WORKING AT SHELF LEVEL
Shelf-level work is rare and includes:
- Organizing the shelf structure
- Cross-project governance
- Audit/investigation across multiple projects
- Creating or deleting projects

### NEVER DO THIS
- Run `npm test` or `cargo test` from shelf root — it's not a project
- Assume a file exists because you saw it in another project
- Create files at shelf root for project-specific work

## Naming Conventions

### Session/Conversation Naming
Format: `<project>:<brief-task-description>`
Good: `heliosCLI:auth-refactor`, `shelf:duplication-audit`
Bad: `fix`, `implementation`, `agent work`

### File Naming in Sessions
Format: `<project>-<YYYYMMDD>-<task>-<version>.md`
Good: `heliosCLI-20260329-cli-redesign-v1.md`
Location: `worktrees/<project>/sessions/` or `shelf-level: plans/`

### Branch Naming
Format: `<project>/<type>/<description>`
Good: `heliosCLI/feat/token-refresh`, `thegent/chore/update-deps`
Bad: `feature-branch`, `fix`

## Tool Usage Policy

### Read-Only Tools First
Before using write/edit tools:
1. Use `Read` to understand the current state
2. Use `fs_search`/`sem_search` to locate code
3. Use `shell` (read-only commands) to verify state

### Write Tools
Use `FSWrite`/`patch` only when:
- You've read the file first
- You understand the full context
- You can explain what you're changing before changing it

### Shell Tools
- Always use `cwd` parameter, never `cd`
- Use `--` to separate file paths from arguments
- Never chain `cd` commands
- Use absolute paths when possible

## Multi-Agent Coordination

### Primary Agent Selection
| Task type | Primary agent |
|-----------|---------------|
| Feature implementation | Forge |
| Code review | Muse |
| Bug investigation | Sage |
| Testing/runtime | Helios |
| Cross-project architecture | Forge + Sage |
| Research/investigation | Sage |
| Documentation | Forge (with Muse review) |

### Coordination Protocol
1. **Single thread**: One agent owns a task at a time
2. **Handoff**: When switching agents, summarize state in conversation
3. **No overwrites**: If Agent B joins a task Agent A started, Agent B reads
   conversation history before taking action
4. **Conflict resolution**: If two agents conflict, pause and get user decision

## Ghostty/Forge.Terminal Policy

**NEVER**: Open or interact with terminal applications (ghostty, terminal multiplexers)
as part of an agent session. These are USER-ONLY.

**IF YOU SEE ghostty in traces**: Ignore and continue with your work. Do not
attempt to control or close it. It belongs to the user.

**IF YOU MADE `forge.real` or similar**: Do NOT touch it. Delete it if it causes
crashes. The real Forge session is separate from any agent-created instances.

## Project-Specific Agent Rules

Each project may have its own `AGENTS.md` or `.claude/rules/` directory.
**Project rules override shelf rules** for work inside that project.

When a project's rules conflict with shelf rules, prefer:
1. Project rules for work inside that project
2. Shelf rules for shelf-level or cross-project work

## Commit/PR Policy

### Commit Messages
Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`

Good: `feat(heliosCLI): add token refresh with exponential backoff`
Bad: `fix stuff`, `update`, `WIP`

### PR Guidelines
- One logical change per PR
- PR title matches commit format
- Description explains WHY, not just WHAT
- Always link related issues

### Force Push Policy
**NEVER force push to `main`, `master`, or shared long-lived branches.**
Force push is acceptable ONLY for:
- Personal feature branches
- Your own worktrees
- Branches you're sure no one else is using

## Error Handling

### Rate Limits (429)
When encountering API rate limits:
1. Stop immediately — do not retry
2. Report the limit type to user
3. Wait for user instruction

### Crashes
If an agent crashes mid-task:
1. Save state to conversation
2. Report what was in progress
3. Wait for user or another agent to resume

### Tool Failures
If a tool fails repeatedly:
1. Try alternative approach
2. Report the failure with context
3. Ask user for guidance

## File Organization Responsibilities

### Shelf-Level (what you control here)
- `projects/INDEX.md` — master project list
- `docs/adr/` — architecture decisions
- `scripts/` — cross-project utilities
- `governance/` — governance tooling
- `.worktrees/` — worktree management

### Project-Level (each project controls its own)
- `CLAUDE.md`, `AGENTS.md` in project root
- `.claude/rules/` in project root
- README, CONTRIBUTING per project
- Dotfiles per project

## Onboarding a New Project

When adding a new project to the shelf:

1. Create project directory at shelf root: `mkdir newproject`
2. Add entry to `projects/INDEX.md`
3. Create minimal `CLAUDE.md` and `README.md` in project
4. Initialize git if needed: `git init && git remote add origin <url>`
5. Add worktree guidance to `WORKSTORES.md` if applicable

## Quick Command Reference

```bash
# List all projects
ls projects/INDEX.md  # then cat it

# Check if directory is a git repo
ls <dir>/.git 2>/dev/null && echo "GIT REPO" || echo "NOT A REPO"

# List worktrees
ls .worktrees/

# Add worktree
git worktree add .worktrees/<name> -b <branch>

# Remove worktree
git worktree remove .worktrees/<name>
```
