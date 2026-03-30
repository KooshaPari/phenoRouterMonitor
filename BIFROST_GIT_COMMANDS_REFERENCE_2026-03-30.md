# Bifrost & LLM Routing — Git Commands Reference

**Quick commands to access all bifrost and routing work**

---

## View Research & Analysis Commits

### LLM Proxy Landscape Research (Feb 22, 2026)
```bash
git show 009f1dd62

# View commit details
git show --stat 009f1dd62
```

**What it contains:**
- Competitive analysis of 15+ LLM proxy/routing projects
- Feature matrix and embeddability analysis
- Performance benchmarks
- Strategic recommendations

---

## View LiteLLM Integration Commits

### Full LiteLLM Router Integration
```bash
git show eafd29980

# View what files changed
git show --name-only eafd29980
```

### Provider Type Classification
```bash
git show 58ab24c26

# See implementation
git show 58ab24c26:src/thegent/routing/provider_types.py
```

### LiteLLM Router Wrapper
```bash
git show 9e4249563

# View implementation details
git show 9e4249563:src/thegent/routing/litellm_router.py
```

### CodexProxyRunner Routing
```bash
git show d0ca83465
```

### LiteLLM Configuration Settings
```bash
git show d97e66023
```

### Integration Tests
```bash
git show 285e958e6
```

### Add LiteLLM Dependency
```bash
git show 0ff804b75

# View pyproject.toml changes
git show 0ff804b75:pyproject.toml
```

---

## View All Routing-Related Commits

### List all commits with routing/bifrost/litellm
```bash
git log --all --grep="bifrost\|routing\|litellm\|provider" --oneline

# More detailed view with dates
git log --all --grep="bifrost\|routing\|litellm\|provider" --oneline --date=short

# Show full commit messages
git log --all --grep="bifrost\|routing\|litellm\|provider" -i
```

### See statistics
```bash
git log --all --grep="routing" --oneline | wc -l
```

---

## Work with Bifrost-Related Stashes

### List all stashes
```bash
git stash list
```

### View stash content (without popping)
```bash
git stash show stash@{5}
git stash show stash@{6}
```

### View detailed stash diff
```bash
git stash show -p stash@{5}
```

### Recover bifrost-related stash #1
```bash
git stash pop stash@{5}

# If conflicts occur, resolve and continue
git add .
git commit -m "Recovered: phenotype-mcp routing work from stash"
```

### Recover bifrost-related stash #2
```bash
git stash pop stash@{6}
```

### If you need to undo a stash pop
```bash
git reset --hard HEAD~1
```

---

## Create Bifrost-Routing Branch/Worktree

### Option A: Create a feature branch
```bash
git checkout -b feat/bifrost-routing-consolidation

# After staging changes
git add .
git commit -m "feat: consolidate bifrost and routing infrastructure

- Consolidate specs from AgilePlus
- Include LiteLLM integration plans
- Migrate routing module code
- Copy test suite
- Add pareto routing task breakdown
"
```

### Option B: Create a git worktree (recommended)
```bash
# Create worktree directory
mkdir -p .worktrees/bifrost-routing

# Create worktree from main
git worktree add .worktrees/bifrost-routing/impl main

# Navigate to worktree
cd .worktrees/bifrost-routing/impl

# Create feature branch
git checkout -b feat/bifrost-routing-consolidation

# Do work, then commit
git add .
git commit -m "feat: create bifrost-routing fork with consolidated specs, plans, and code"
```

---

## Review Bifrost-Related Files

### View bifrost.py integration module
```bash
git show HEAD:platforms/thegent/src/thegent/integrations/bifrost.py | less
```

### View provider_types.py
```bash
git show HEAD:platforms/thegent/src/thegent/routing/provider_types.py
```

### View litellm_router.py
```bash
git show HEAD:platforms/thegent/src/thegent/routing/litellm_router.py | head -50
```

---

## Search Git History

### Find commits mentioning "bifrost"
```bash
git log -i --grep="bifrost" --oneline
```

### Find commits mentioning "routing"
```bash
git log -i --grep="routing" --oneline
```

### Find commits mentioning "litellm"
```bash
git log -i --grep="litellm" --oneline
```

### Find commits mentioning "provider"
```bash
git log -i --grep="provider" --oneline
```

### Combined search
```bash
git log --all -i --grep="bifrost\|routing\|litellm\|provider" --oneline | head -30
```

### Search commit content
```bash
git log -p -S "bifrost" --oneline
```

---

## Compare Commits

### See what changed between two commits
```bash
git diff 009f1dd62 eafd29980

# Specific file
git diff 009f1dd62 eafd29980 -- platforms/thegent/src/thegent/routing/litellm_router.py
```

### See files changed in a commit
```bash
git show --name-only 009f1dd62
```

---

## Branch Information

### Check current branch
```bash
git status --short --branch
```

### List all branches (local)
```bash
git branch
```

### List all branches (including remote)
```bash
git branch -a
```

### List branches with most recent commits
```bash
git for-each-ref --sort=-committerdate refs/heads/ --format='%(refname:short) %(committerdate:short)'
```

---

## Worktree Commands

### List all worktrees
```bash
git worktree list
```

### Create new worktree for bifrost work
```bash
git worktree add .worktrees/bifrost-routing/bifrost-consolidation main
```

### Remove worktree (when done)
```bash
git worktree remove .worktrees/bifrost-routing/bifrost-consolidation
```

---

## Stash Management

### List all stashes
```bash
git stash list
```

### View stash differences
```bash
git stash show -p stash@{5}
git stash show -p stash@{6}
```

### Apply stash without removing
```bash
git stash apply stash@{5}
```

### Pop stash (remove after applying)
```bash
git stash pop stash@{5}
```

### Drop stash (delete without applying)
```bash
git stash drop stash@{5}
```

### Create stash with message
```bash
git stash push -m "bifrost routing wip" -- src/thegent/routing/
```

---

## View File History

### View history of litellm_router.py
```bash
git log --oneline platforms/thegent/src/thegent/routing/litellm_router.py
```

### See blame (who changed what)
```bash
git blame platforms/thegent/src/thegent/routing/litellm_router.py
```

### View specific version
```bash
git show 9e4249563:src/thegent/routing/litellm_router.py
```

---

## Tag & Release (if needed)

### View tags related to routing
```bash
git tag | grep -i routing
git tag | grep -i bifrost
```

### Create tag for routing release
```bash
git tag -a v1.0.0-routing -m "LiteLLM routing integration v1.0.0"
```

### Push tag
```bash
git push origin v1.0.0-routing
```

---

## Advanced: Cherry-Pick Commits

### Cherry-pick specific routing commits to new branch
```bash
git checkout -b feat/routing-from-main main

# Cherry-pick commits in order
git cherry-pick 0ff804b75  # Add litellm dependency
git cherry-pick 58ab24c26  # Provider types
git cherry-pick 9e4249563  # LiteLLM router
# ... etc
```

---

## Helpful Shortcuts

### Alias for routing commits
```bash
# Add to .git/config or ~/.gitconfig
git config --global alias.routing 'log --all --grep="bifrost\|routing\|litellm\|provider" --oneline'

# Then use
git routing
```

### Alias for bifrost commits
```bash
git config --global alias.bifrost 'log --all --grep="bifrost" --oneline'
git bifrost
```

---

## Quick Command Cheat Sheet

```bash
# View all routing commits
git log --all --grep="routing" --oneline

# View specific bifrost commit
git show 009f1dd62

# List stashes
git stash list

# Recover bifrost work
git stash pop stash@{5}

# Create worktree
git worktree add .worktrees/bifrost-routing/impl main

# Search commits by content
git log -p -S "litellm" --oneline

# View file at specific commit
git show 9e4249563:src/thegent/routing/litellm_router.py

# Compare commits
git diff 009f1dd62 eafd29980

# Create feature branch
git checkout -b feat/bifrost-routing-consolidation
git add .
git commit -m "feat: consolidate bifrost work"
```

---

## Related Documents

**For detailed information, see:**
- `/Users/kooshapari/CodeProjects/Phenotype/repos/BIFROST_ROUTING_WORK_INVENTORY_2026-03-30.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/BIFROST_AND_ROUTING_RECOVERY_COMPREHENSIVE_REPORT_2026-03-30.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/BIFROST_QUICK_REFERENCE_RECOVERY_GUIDE_2026-03-30.md`

---

**Last Updated:** 2026-03-30
