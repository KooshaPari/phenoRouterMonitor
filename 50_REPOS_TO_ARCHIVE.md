# 50 Repositories for Archival/Deletion

## Summary
**Total GitHub Repos:** 169  
**Target for Deletion:** 50 (30% reduction)  
**Expected Remaining:** ~119

---

## 🔴 ARCHIVED REPOS (Safe to Delete - 25 repos)

These are already archived on GitHub and can be safely deleted:

| # | Repository | Status | Reason |
|---|------------|--------|--------|
| 1 | `cloud` | archived, fork | Old fork, not maintained |
| 2 | `FixitGo` | archived | Superseded by other tools |
| 3 | `Synthia` | archived, fork | Fork, not used |
| 4 | `CLIProxyAPI` | archived | Migrated to cliproxyapi-plusplus |
| 5 | `agentapi-deprec` | archived | Migrated to agentapi-plusplus |
| 6 | `FixitRs` | archived | Superseded |
| 7 | `KaskMan` | archived | Old experiment |
| 8 | `ccusage` | archived, fork | Fork, not used |
| 9 | `vibe-kanban` | archived, fork | Fork, not used |
| 10 | `template-program-ops` | archived | Duplicated in HexaKit |
| 11 | `KWatch` | archived | Old project |
| 12 | `slickport` | archived | Old project |
| 13 | `chatta` | archived | Old project |
| 14 | `localbase3` | archived | Old experiment |
| 15 | `TripleM` | archived | Old experiment |
| 16 | `AtomsBot` | archived | Old project |
| 17 | `atoms.tech` | archived | Old website |
| 18 | `agentapi` | archived | Migrated to agentapi-plusplus |
| 19 | `argisexec` | archived | Old experiment |
| 20 | `router-docs` | archived | Old docs |
| 21 | `helix-logging` | archived | Merged into Tracely |
| 22 | `helix-tracing` | archived | Merged into Tracely |
| 23 | `phenotype-docs-engine` | archived | Migrated to phenodocs |
| 24 | `phenotype-task-engine` | archived | Migrated to packages/ |
| 25 | `Traceon` | archived | Merged into Tracely |

**Delete Commands:**
```bash
for repo in cloud FixitGo Synthia CLIProxyAPI agentapi-deprec FixitRs KaskMan ccusage vibe-kanban template-program-ops KWatch slickport chatta localbase3 TripleM AtomsBot atoms.tech agentapi argisexec router-docs helix-logging helix-tracing phenotype-docs-engine phenotype-task-engine Traceon; do
  gh repo delete KooshaPari/$repo --yes 2>/dev/null && echo "Deleted: $repo" || echo "Already gone: $repo"
done
```

---

## 🟡 FORKS (Evaluate - 10 repos)

These are forks that may have local modifications. Check if changes exist before deleting:

| # | Repository | Status | Evaluation |
|---|------------|--------|------------|
| 26 | `portage` | fork | Harbor RL framework - check for modifications |
| 27 | `vibeproxy` | fork | macOS menu bar - check for modifications |
| 28 | `HeliosLab` | fork | Colab - check for modifications |
| 29 | `Planify` | fork | Project management - check for modifications |
| 30 | `Tossy` | fork | Trash CLI - check for modifications |
| 31 | `MCPForge` | fork | MCP language server - check for modifications |
| 32 | `DINOForge-UnityDoorstop` | fork | Unity modding - check for modifications |
| 33 | `ccusage` | fork, archived | Already in archived list above |
| 34 | `vibe-kanban` | fork, archived | Already in archived list above |
| 35 | `cliproxyapi-plusplus` | fork | Check for local changes vs CLIProxyAPI |

**Evaluation Command:**
```bash
for repo in portage vibeproxy HeliosLab Planify Tossy MCPForge DINOForge-UnityDoorstop; do
  echo "=== $repo ==="
  gh repo view KooshaPari/$repo --json name,source,pushedAt,isFork 2>/dev/null || echo "Not found"
done
```

---

## 🟠 SINGLE-FILE/OBSOLETE (15 repos)

These appear to be minimal or superseded by other projects:

| # | Repository | Size | Reason |
|---|------------|------|--------|
| 36 | `phenoPatch` | minimal | Single module, merge into phenotype-patch |
| 37 | `phenoResearchEngine` | minimal | Research functionality merged elsewhere |
| 38 | `phenoRouterMonitor` | minimal | Merge into phenotype-router-monitor |
| 39 | `phenoSentinel` | minimal | Merge into phenotype-sentinel |
| 40 | `phenoSkills` | minimal | Merge into phenotype-skills |
| 41 | `phenoStandards` | minimal | Merge into phenotype-standards |
| 42 | `phenoTypes` | minimal | Merge into phenotype-types |
| 43 | `phenoVessel` | minimal | Merge into phenotype-vessel |
| 44 | `phenoXdd` | minimal | Merge into phenotype-xdd |
| 45 | `phenoXddLib` | minimal | Merge into phenotype-xdd-lib |
| 46 | `phenoConfigTs` | minimal | Merge into phenotype-config-ts |
| 47 | `phenoMiddlewarePy` | minimal | Merge into phenotype-middleware-py |
| 48 | `phenoCipher` | minimal | Merge into phenotype-cipher |
| 49 | `phenoLoggingZig` | minimal | Merge into phenotype-logging-zig |
| 50 | `phenoHub` | minimal | Merge into phenotype-hub |

**Note:** These `pheno*` minimal repos should be checked for unique content before merging. They may be empty shells.

---

## DELETE EXECUTION SUMMARY

### Tier 1: Immediate Deletion (25 repos)
All **archived** repos can be deleted immediately without risk.

### Tier 2: Fork Evaluation (10 repos)
Check for local modifications before deleting forks.

### Tier 3: Merge & Delete (15 repos)
Merge any unique content from `pheno*` minimal repos into their full equivalents, then delete.

**Total: 50 repositories targeted for deletion**
**Expected final count: ~119 repositories**

---

## VERIFICATION

```bash
# Before deletion
gh repo list KooshaPari --limit 200 | wc -l
# Expected: 169

# After deletion of 50 repos
# Expected: ~119
```
