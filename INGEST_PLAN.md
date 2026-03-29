# Auto-Sync Docs Ingestion Plan

**Status:** Phase 1 Complete, Phase 2–6 Ready for Execution
**Start Date:** 2026-03-29
**Estimated Duration:** 5–7 days (aggressive agent-driven schedule)
**Work Location:** `repos/worktrees/auto-sync-docs/`

---

## Executive Summary

This plan establishes a bidirectional sync system between existing documentation in `/repos/docs/`, `/repos/worklogs/`, and root-level specification files (PRD.md, FUNCTIONAL_REQUIREMENTS.md, etc.) and the AgilePlus spec database.

**Key Deliverables:**
- Python ingestion script (scripts/ingest-docs-to-agileplus.py)
- Bidirectional sync script (scripts/sync-docs-bidirectional.py)
- CLI commands (agileplus sync-docs)
- Comprehensive tests (tests/test_doc_sync.py)
- Updated documentation

**Success Criteria:**
- All 80–100 specs ingested from filesystem without duplicates
- Idempotence verified (run twice = same result)
- No data loss
- FR traceability 100% linked
- Tests passing (>85% coverage)

---

## Phase 2: Ingest to AgilePlus DB (Days 1–2)

### Objective
Create Python script to parse all markdown files and ingest as AgilePlus specs.

### Deliverables

1. **scripts/ingest-docs-to-agileplus.py** (200 lines)
   - Parse all .md files in docs/, worklogs/, and root
   - Extract title, description, spec markers (FR-*, E*.*, ADR-*, etc.)
   - Create spec records in AgilePlus database
   - Set state to "researched" (already documented)
   - Link acceptance criteria from content

2. **Ingestion Algorithm**
   ```
   For each markdown file:
     1. Read file content
     2. Extract title (first H1 or filename)
     3. Extract description (first paragraph or section)
     4. Extract all spec markers via regex:
        - FR-{CAT}-{NNN}: \bFR-([A-Z]+)-(\d{3})\b
        - E{n}.{m}: \bE(\d+)\.(\d+)\b
        - ADR-{NNN}: \bADR-(\d{3})\b
        - P{n}.{m}: \bP(\d+)\.(\d+)\b
        - UJ-{N}: \bUJ-(\d+)\b
        - NFR-*: \bNFR-([A-Z_]+)\b
     5. Create Spec object:
        {
          title: extracted_title,
          description: truncated_description,
          state: "researched",
          source_file: relative_path,
          created_date: now(),
          acceptance_criteria: extract_criteria(content),
          markers: [list of found markers],
          file_size_bytes: len(content),
          line_count: len(content.split('\n'))
        }
     6. Insert into database (idempotent via content hash)
     7. For each marker found:
        - Create dependency link spec -> marker
        - Store line numbers where marker appears

   Return: count of specs created, list of new spec IDs
   ```

3. **Idempotence Strategy**
   - Hash file content (SHA-256)
   - Check if spec with same hash exists
   - Skip if exists (no re-ingestion)
   - Update metadata if file changed (detected by hash mismatch)

### Phase 2 Tasks

| Task | Owner | Duration | Deliverable |
|------|-------|----------|------------|
| P2.1 | agent | 1 hour | ingest-docs-to-agileplus.py (basic version) |
| P2.2 | agent | 30 min | database schema updates (if needed) |
| P2.3 | agent | 1 hour | test harness and dry-run on sample docs |
| P2.4 | agent | 30 min | full ingest run and validation |

**Acceptance Criteria:**
- Script runs without errors
- Creates 80–100 specs
- No duplicates (verified via DB count)
- All FR-*, E*.*, ADR-*, P*.* markers found and linked
- Dry-run output shows correct counts before commit

---

## Phase 3: Bidirectional Sync (Days 3–4)

### Objective
Establish bidirectional sync between filesystem and database.

### Deliverables

1. **scripts/sync-docs-bidirectional.py** (300 lines)
   - Mode 1: Filesystem → DB (one-way ingestion)
   - Mode 2: DB → Filesystem (export specs as design docs)
   - Mode 3: Conflict detection (report divergence)

2. **Filesystem → DB (Idempotent)**
   ```python
   def sync_from_filesystem(dry_run=False):
       for md_file in glob(docs/**/*.md, worklogs/**/*.md, root/*.md):
           if not is_node_module_or_ignored(md_file):
               spec = parse_and_ingest(md_file)
               if not spec_exists_with_same_hash(spec):
                   db.specs.insert(spec)
                   log_ingested(spec.id)
       return count_ingested
   ```

3. **DB → Filesystem (Export Design Docs)**
   ```python
   def sync_to_filesystem(dry_run=False):
       for spec in db.specs.find(state="active", source_file=None):
           # Newly created specs with no source file
           design_doc = generate_design_doc(spec)
           path = docs/changes/$(spec.category)/$(spec.id).md
           write_file(path, design_doc)
           log_exported(spec.id)
       return count_exported
   ```

4. **Conflict Detection**
   ```python
   def check_conflicts():
       conflicts = []
       for spec in db.specs:
           if spec.source_file:
               file_content = read_file(spec.source_file)
               file_hash = hash(file_content)
               db_hash = spec.content_hash
               if file_hash != db_hash:
                   conflicts.append({
                       spec_id: spec.id,
                       source_file: spec.source_file,
                       file_modified: get_mtime(spec.source_file),
                       db_modified: spec.updated_date,
                       action: "MANUAL_REVIEW_REQUIRED"
                   })
       return conflicts
   ```

### Design Doc Template (Jinja2)

```markdown
# {{ spec.title }}

**Spec ID:** {{ spec.id }}
**Type:** {{ spec.type }}
**State:** {{ spec.state }}
**Created:** {{ spec.created_date }}
**Updated:** {{ spec.updated_date }}

## Summary

{{ spec.description }}

## Acceptance Criteria

{% for criterion in spec.acceptance_criteria %}
- [ ] {{ criterion }}
{% endfor %}

## Related Specs

{% for dependency in spec.dependencies %}
- {{ dependency.id }}: {{ dependency.title }}
{% endfor %}

## Implementation Notes

Traces to: {{ spec.markers | join(', ') }}

Generated: {{ now() }}
```

### Phase 3 Tasks

| Task | Owner | Duration | Deliverable |
|------|-------|----------|------------|
| P3.1 | agent | 1.5 hours | sync-docs-bidirectional.py |
| P3.2 | agent | 1 hour | Jinja2 template and design doc generation |
| P3.3 | agent | 45 min | conflict detection logic |
| P3.4 | agent | 1 hour | integration testing (both directions) |

**Acceptance Criteria:**
- Filesystem → DB works idempotently
- DB → Filesystem generates valid markdown
- Conflict detection identifies all hash mismatches
- Round-trip test: ingest, export, verify no data loss

---

## Phase 4: CLI Integration (Day 5)

### Objective
Integrate sync commands into AgilePlus CLI.

### Deliverables

1. **New CLI Commands** (add to agileplus.py or governance_agileplus_cmds.py)

```bash
# Load docs → DB
agileplus sync-docs --from-filesystem [--dry-run]

# Export specs → design docs
agileplus sync-docs --to-filesystem [--dry-run]

# Check for conflicts
agileplus sync-docs --check-conflicts

# Status report
agileplus sync-docs --status
```

2. **Command Implementation** (50 lines Python)

```python
from typer import Typer, Option
from enum import Enum

app = Typer()

class SyncDirection(str, Enum):
    FROM_FILESYSTEM = "from-filesystem"
    TO_FILESYSTEM = "to-filesystem"
    CHECK_CONFLICTS = "check-conflicts"
    STATUS = "status"

@app.command()
def sync_docs(
    direction: SyncDirection = Option(..., help="Sync direction"),
    dry_run: bool = Option(False, help="Run without making changes"),
    verbose: bool = Option(False, help="Verbose output")
):
    """Sync documentation between filesystem and AgilePlus database"""

    if direction == SyncDirection.FROM_FILESYSTEM:
        count = sync_from_filesystem(dry_run=dry_run)
        print(f"Ingested {count} specs from filesystem")

    elif direction == SyncDirection.TO_FILESYSTEM:
        count = sync_to_filesystem(dry_run=dry_run)
        print(f"Exported {count} specs to filesystem")

    elif direction == SyncDirection.CHECK_CONFLICTS:
        conflicts = check_conflicts()
        if conflicts:
            print(f"Found {len(conflicts)} conflicts:")
            for conflict in conflicts:
                print(f"  {conflict['spec_id']}: {conflict['source_file']}")
        else:
            print("No conflicts detected")

    elif direction == SyncDirection.STATUS:
        stats = get_sync_status()
        print(stats)
```

### Phase 4 Tasks

| Task | Owner | Duration | Deliverable |
|------|-------|----------|------------|
| P4.1 | agent | 30 min | CLI command skeleton and typer integration |
| P4.2 | agent | 45 min | command implementations |
| P4.3 | agent | 30 min | help text and usage documentation |
| P4.4 | agent | 15 min | testing in local environment |

**Acceptance Criteria:**
- All four commands work
- Dry-run doesn't modify database/filesystem
- Help text is clear and complete
- Verbose output is informative

---

## Phase 5: Tests & Validation (Days 5–6)

### Objective
Comprehensive test suite with 85%+ coverage.

### Deliverables

1. **tests/test_doc_sync.py** (150 lines)

```python
import pytest
from scripts.ingest_docs_to_agileplus import (
    parse_markdown,
    extract_markers,
    ingest_doc,
)
from scripts.sync_docs_bidirectional import (
    sync_from_filesystem,
    sync_to_filesystem,
    check_conflicts,
)

# Test 1: Parse single doc
def test_parse_single_doc():
    """Test parsing a single markdown file"""
    path = Path("test_docs/sample.md")
    result = parse_markdown(path)
    assert result.title == "Sample Title"
    assert len(result.markers) > 0

# Test 2: Extract all marker types
def test_extract_markers():
    """Test all spec marker extraction"""
    content = """
    # Test
    This doc covers FR-EVT-001, E1.1, ADR-002, P1.3, UJ-2, NFR-SERDE
    """
    markers = extract_markers(content)
    assert "FR-EVT-001" in markers
    assert "E1.1" in markers
    assert "ADR-002" in markers
    assert "P1.3" in markers
    assert "UJ-2" in markers
    assert "NFR-SERDE" in markers

# Test 3: Idempotence
def test_ingest_idempotence():
    """Run twice, expect same result"""
    db.specs.clear()
    count1 = sync_from_filesystem()
    count2 = sync_from_filesystem()
    assert count1 == count2
    assert db.specs.count() == count1

# Test 4: Conflict detection
def test_conflict_detection():
    """Detect divergence between file and DB"""
    spec_id = ingest_doc("test_docs/sample.md")

    # Modify file
    with open("test_docs/sample.md", "a") as f:
        f.write("\n# Modified content")

    conflicts = check_conflicts()
    assert any(c["spec_id"] == spec_id for c in conflicts)

# Test 5: Round-trip no data loss
def test_roundtrip_no_data_loss():
    """Ingest → Export → Verify content preserved"""
    original_path = "test_docs/sample.md"
    original_content = read_file(original_path)

    # Ingest
    spec_id = ingest_doc(original_path)

    # Export
    export_path = "test_docs/export_sample.md"
    export_spec_to_filesystem(spec_id, export_path)
    exported_content = read_file(export_path)

    # Verify key content preserved
    assert "FR-" in exported_content  # Markers present
    assert spec_id in exported_content  # ID present
    assert len(exported_content) > 100  # Substantial content

# Test 6: All docs ingestible
def test_ingest_all_docs():
    """Ensure no docs fail during full ingest"""
    db.specs.clear()
    errors = []

    for md_file in glob_docs():
        try:
            ingest_doc(md_file)
        except Exception as e:
            errors.append((md_file, str(e)))

    assert len(errors) == 0, f"Failed to ingest: {errors}"

# Test 7: CLI commands functional
def test_cli_commands():
    """Test CLI command execution"""
    result = subprocess.run(
        ["agileplus", "sync-docs", "--from-filesystem", "--dry-run"],
        capture_output=True, text=True
    )
    assert result.returncode == 0
    assert "Ingested" in result.stdout

# Test 8: Export design docs valid markdown
def test_export_design_docs_valid():
    """Exported design docs are valid markdown"""
    spec_id = ingest_doc("test_docs/sample.md")
    export_path = "test_export/design.md"

    export_spec_to_filesystem(spec_id, export_path)

    # Validate markdown structure
    content = read_file(export_path)
    assert content.startswith("#")  # Has heading
    assert "Acceptance Criteria" in content
    assert "Related Specs" in content
```

### Phase 5 Tasks

| Task | Owner | Duration | Deliverable |
|------|-------|----------|------------|
| P5.1 | agent | 1 hour | Write all 8 test functions |
| P5.2 | agent | 45 min | Setup test fixtures and mocks |
| P5.3 | agent | 1 hour | Run tests and verify coverage |
| P5.4 | agent | 30 min | Fix failing tests (if any) |

**Acceptance Criteria:**
- All 8 tests passing
- Code coverage >= 85%
- No flaky tests
- Test execution < 30 seconds

---

## Phase 6: Documentation & Deployment (Days 6–7)

### Objective
Complete documentation and prepare for production deployment.

### Deliverables

1. **Updated README.md or docs/guides/DOCUMENTATION_SYNC_GUIDE.md**

```markdown
# Documentation Auto-Sync Guide

## Overview

The documentation auto-sync system keeps filesystem docs and the AgilePlus spec database in sync.

## Quick Start

### Initial Load

Load all existing documentation into AgilePlus:

```bash
agileplus sync-docs --from-filesystem
```

This creates ~100 specs in the database from:
- /repos/docs/
- /repos/worklogs/
- Root-level PRD.md, FUNCTIONAL_REQUIREMENTS.md, etc.

### Export New Specs

When you create new specs in AgilePlus, export them as design documents:

```bash
agileplus sync-docs --to-filesystem
```

Design docs are created in `docs/changes/{category}/{spec_id}.md`

### Check for Conflicts

When syncing becomes complicated:

```bash
agileplus sync-docs --check-conflicts
```

Reports any files that have diverged from database state.

## Bidirectional Sync

### Filesystem is Source-of-Truth For:
- Research documentation (docs/research/*)
- Governance docs (worklogs/*)
- Implemented features (already in code)

### Database is Source-of-Truth For:
- Active work (state: "planned", "in-progress", "review")
- New specifications (not yet implemented)

### Conflicts
If a spec exists in both places and hashes don't match:
1. Review both versions
2. Manual merge if needed
3. Delete the outdated version
4. Re-run sync

## Idempotence

All sync operations are idempotent:
- Running `--from-filesystem` twice = same result
- Running `--to-filesystem` twice = same result
- No duplicate specs created

## Automation

Add to CI/CD or pre-commit hooks:

```bash
# Before merge: check for conflicts
agileplus sync-docs --check-conflicts

# After merge: export new specs
agileplus sync-docs --to-filesystem
```
```

2. **AGENT_ONBOARDING.md** (update existing or create)

```markdown
# Agent Onboarding — Documentation Sync

## What Agents Need to Know

When you inherit a project with existing documentation, use:

```bash
# Step 1: Ingest existing docs
agileplus sync-docs --from-filesystem

# Step 2: Check for conflicts
agileplus sync-docs --check-conflicts

# Step 3: Create new specs in DB as you implement
# (Your changes auto-link to existing docs via markers)
```

## Markers Agents Use

When writing code or docs, include spec markers:

```markdown
# Feature X Implementation

Traces to: FR-EVT-001, E1.1, ADR-002

This implementation fulfills:
- [ ] FR-EVT-001: EventEnvelope initialization
- [ ] FR-EVT-002: Hash chain computation
```

The sync system automatically links these markers to the spec database.

## Before Committing

Run the quality gate:

```bash
agileplus sync-docs --check-conflicts
# Fix any conflicts
# Then commit
```
```

3. **Phase 6 Tasks**

| Task | Owner | Duration | Deliverable |
|------|-------|----------|------------|
| P6.1 | agent | 1 hour | Write comprehensive sync guide |
| P6.2 | agent | 45 min | Update AGENT_ONBOARDING.md |
| P6.3 | agent | 30 min | Final integration testing |
| P6.4 | agent | 30 min | Create summary and hand-off docs |

**Acceptance Criteria:**
- Documentation is clear and actionable
- All examples tested and working
- Ready for team use
- No manual steps required for ongoing use

---

## Execution Timeline

```
┌─────────────────────────────────────────────────────────────┐
│ PHASE 2: Ingestion Script                                   │
│ Day 1–2: Write ingest-docs-to-agileplus.py                  │
│ Deliverable: 80–100 specs in DB                              │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ PHASE 3: Bidirectional Sync                                 │
│ Day 3–4: Write sync-docs-bidirectional.py                   │
│ Deliverable: Filesystem ↔ DB sync working                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ PHASE 4: CLI Integration                                    │
│ Day 5: Add commands to agileplus CLI                         │
│ Deliverable: agileplus sync-docs commands                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ PHASE 5: Testing & Validation                               │
│ Day 5–6: Write tests, verify coverage >85%                  │
│ Deliverable: tests/test_doc_sync.py (all passing)           │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ PHASE 6: Documentation & Handoff                            │
│ Day 6–7: Write guides, update onboarding                    │
│ Deliverable: Ready for team use                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Success Metrics

| Metric | Target | Acceptance |
|--------|--------|-----------|
| Specs ingested | 80–100 | >= 80 |
| Duplicate prevention | 0 duplicates | 100% |
| Idempotence | 2x run = same | Pass idempotence test |
| Test coverage | >= 85% | Pass coverage check |
| CLI uptime | 100% | 0 crashes on valid input |
| Documentation clarity | All scenarios covered | Agent can use without help |
| Data preservation | 100% round-trip | Pass roundtrip test |

---

## Risk Mitigation

| Risk | Probability | Mitigation |
|------|-------------|-----------|
| DB schema mismatch | Medium | Use dry-run before commit |
| Large markdown files | Low | Implement file size limit |
| Circular spec dependencies | Low | Validate DAG during ingest |
| Regex marker extraction errors | Medium | Use comprehensive test corpus |
| Conflict resolution deadlock | Low | Document manual merge procedure |
| CLI integration delays | Medium | Test in isolation first |

---

## Assumptions

1. AgilePlus database schema is stable and writable
2. Markdown files are UTF-8 encoded
3. All spec markers follow exact format (FR-{CAT}-{NNN}, etc.)
4. No circular dependencies in spec graph
5. Ingest can run offline (no API calls required)

---

## Handoff Checklist

- [x] Phase 1: DOCUMENT_INVENTORY.md created
- [ ] Phase 2: Ingestion script tested and deployed
- [ ] Phase 3: Bidirectional sync tested end-to-end
- [ ] Phase 4: CLI commands integrated
- [ ] Phase 5: All tests passing, coverage >= 85%
- [ ] Phase 6: Documentation complete and verified
- [ ] Final: Team training and rollout

---

**Plan Created:** 2026-03-29
**Next Action:** Begin Phase 2 (ingest script development)
**Point of Contact:** [Agent Name/ID]
