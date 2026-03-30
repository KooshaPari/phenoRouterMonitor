# Archive Management Guide

**Last updated:** 2026-03-30
**Audit status:** Complete and verified
**Archive location:** `/repos/.archive/`

---

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Archive Location** | `/repos/.archive/` |
| **Total Size** | 13MB (1,482 files) |
| **Git Status** | Properly ignored in .gitignore (line 9) |
| **Active Dependencies** | Zero (verified) |
| **Retention Policy** | Indefinite (Phenotype Long-Term Stability protocol) |
| **Detailed Manifest** | `.archive/ARCHIVE_MANIFEST.md` |
| **Audit Report** | `docs/reports/ARCHIVE_AUDIT_2026-03-30.md` |

---

## Archive Contents (Summary)

### 1. Specifications (kitty-specs/) — 16KB

**Status:** Completed and archived
**Items:** phenotype-infrakit-lockfile-repair (1 spec)
**Reason:** Migrated to AgilePlus format; legacy BMAD refs archived

**Retention:** Indefinite
**Last Updated:** 2026-03-28

### 2. Planning Documents (plans/) — 24KB

**Status:** Reference material
**Items:** 2026-03-29-DUPLICATION_MERGED-v1.md (557 lines)
**Reason:** Duplication analysis completed; work items extracted to AgilePlus

**Key Finding:** 35,000 LOC duplication across 9.9M LOC workspace
**Retention:** Indefinite
**Last Updated:** 2026-03-29

### 3. Experimental Artifacts (temp-directories/) — 13MB

**Status:** Stable, non-functional
**Items:** 4 experimental projects (1,400+ files)
**Reason:** Experimental scaffolding, superseded approaches, template generators

| Project | Size | Status | Use Case |
|---------|------|--------|----------|
| agent-wave-monorepo-temp | ~6MB | Superseded | Architectural pattern reference |
| phenotype-go-kit-temp | ~4MB | Superseded | Go library scaffold examples |
| template-commons-temp | ~2.5MB | **Stable** | **Architecture templates + design docs** |
| tokenledger-temp | ~0.5MB | Experimental | Token accounting system |

**Retention:** Stable — Preserved for reference, low maintenance burden
**Last Updated:** 2026-03-29

---

## Codebase Integration

### .gitignore Status

```gitignore
.archive/                 # Line 9 in .gitignore
```

**Effect:**
- Archive is excluded from git tracking
- Large experimental artifacts (13MB) do not bloat repository
- Clear separation between active and archived code

### Dependency Verification

✅ **No active code references found**

```bash
# Verified against:
grep -r "\.archive\|temp-directories\|template-commons" --exclude-dir=.archive
# Result: No active code dependencies (only test function names like "archived")
```

---

## Restoration Procedures

### For Each Item Type

#### Specifications (kitty-specs/)

```bash
# 1. Copy spec to active location
cp -r .archive/kitty-specs/phenotype-infrakit-lockfile-repair/ docs/specs/

# 2. Verify the spec
ls -la docs/specs/phenotype-infrakit-lockfile-repair/

# 3. Commit with provenance
git add docs/specs/phenotype-infrakit-lockfile-repair/
git commit -m "restore: restore phenotype-infrakit-lockfile-repair spec from archive

Original archive: .archive/kitty-specs/phenotype-infrakit-lockfile-repair/
Archive date: 2026-03-28
Reason: [explain why needed again]
"
```

#### Planning Documents (plans/)

```bash
# 1. View for reference
cat .archive/plans/2026-03-29-DUPLICATION_MERGED-v1.md

# 2. If implementing referenced work items:
# - Refer to AgilePlus for current work package status
# - Use this for historical context and planning approach

# 3. Copy if needing as reference document
cp .archive/plans/*.md docs/reference/
```

#### Experimental Projects (temp-directories/)

```bash
# 1. For template-commons-temp (highest reference value):
# - Use as architecture pattern reference (read-only)
# - Copy specific templates as guides
cp -r .archive/temp-directories/template-commons-temp/hexagonal-rs/ docs/reference/templates/

# 2. For other experimental items:
# - Review design documentation before restoration
# - Check compatibility and dependencies
# - Copy to active crates if needed

cp -r .archive/temp-directories/tokenledger-temp/ crates/phenotype-tokenledger
cd crates/phenotype-tokenledger && cargo check
```

---

## Adding New Items to Archive

### Step-by-step Process

```bash
# 1. Create logical subdirectory
mkdir -p .archive/category/item-name/

# 2. Copy/move items (use git add <specific-paths>, NEVER git add -A)
cp -r /path/to/item .archive/category/item-name/

# 3. Create provenance README
cat > .archive/category/item-name/README.md << 'EOF'
# [Item Name] — Archive Entry

**Archive Date:** YYYY-MM-DD
**Reason:** [Why archived]
**Original Location:** [Where it came from]
**Active Dependencies:** None

## Retention Policy

Indefinite [or Conditional/Temporary]

## Restoration Instructions

[How to restore if needed]

## Related Work

[Links to related PRs, issues]
EOF

# 4. Stage specific paths only
git add .archive/category/item-name/

# 5. Commit with clear message
git commit -m "chore(archive): archive [item-name]

Reason: [detailed explanation]
Location: .archive/category/item-name/
Related PR: #XXX
"
```

### Important Notes

- **Never use `git add -A` with archive** — Can hang due to embedded repos
- **Use `git add <specific-paths>`** when adding archive items
- **Verify with `git status`** before committing
- **Include provenance** in all commit messages
- **Check for active references** before archiving

---

## Verification Before Archival

```bash
# Search for references (adjust patterns as needed)
grep -r "archived-item-name" --exclude-dir=.archive /repos

# Check git log for recent references
git log --all --grep="item-name" | head -10

# If found references, determine if they're active or historical
# If active, don't archive yet
```

---

## Retention Policy Details

### Default: Indefinite Retention

Items are kept because:
1. **Historical Value** — Documents completed work and decisions
2. **Zero Dependencies** — No active code depends on archived materials
3. **Recovery Path** — Items can be restored without data loss
4. **Learning Resource** — Demonstrates patterns, approaches, evolution

### Conditions for Purge (Requires Explicit Approval)

Archive items may be purged only with explicit user approval when:
1. Materials >18 months old with zero historical value
2. Space constraints exceed policy (>100MB)
3. Legal/compliance requirement
4. Explicit user request

### Purge Process

```bash
# 1. Verify no active references
grep -r "item-name" --exclude-dir=.archive /repos

# 2. Create git bundle if needed (for recovery)
git bundle create item-name.bundle <ref>

# 3. Remove from archive
rm -rf .archive/category/item-name/

# 4. Commit with documented justification
git commit -m "purge(archive): remove obsolete item

Item: [name]
Reason: [specific justification]
Verified dependencies: [confirmation]
Backup: item-name.bundle (if needed)
Date: YYYY-MM-DD
"
```

---

## Maintenance Schedule

### Quarterly (Jan, Apr, Jul, Oct)

- Verify no new active references have appeared
- Check for new archival opportunities
- Update MANIFEST.md if items were archived

### Annually (January)

- Comprehensive audit of all archived items
- Assess retention value of experimental items
- Update MANIFEST.md with audit summary
- Document scheduled next review

---

## Documentation Files

| File | Location | Purpose |
|------|----------|---------|
| **ARCHIVE_MANIFEST.md** | `.archive/` | Detailed manifest with restoration instructions |
| **ARCHIVE_INDEX.md** | `.archive/` | Archive contents index (legacy) |
| **ARCHIVE_AUDIT_2026-03-30.md** | `docs/reports/` | Comprehensive audit report |
| **ARCHIVE_MANAGEMENT.md** | `docs/reference/` | This guide |

---

## Key Principles

From **Phenotype Long-Term Stability and Non-Destructive Change Protocol**:

1. **No destructive deletions** — Use `.archive/` instead of `rm`
2. **Preserve git history** — Original commit history is intact
3. **Enable restoration** — Any item can be restored without data loss
4. **Maintain auditability** — Central index and manifest required
5. **Document everything** — Provenance, reason, date, related work

---

## Questions & Support

For questions about the archive:

1. Check `.archive/ARCHIVE_MANIFEST.md` for your item
2. Review `.archive/ARCHIVE_INDEX.md` for category overview
3. Look up original PR/issue in GitHub
4. Check git log: `git log --all -- .archive/`
5. Consult AgilePlus for related work packages

---

**Last Audit:** 2026-03-30
**Next Review:** 2026-07-30 (6 months)
**Archive Status:** ✅ Verified safe for long-term retention
