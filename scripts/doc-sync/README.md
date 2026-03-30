# Documentation Auto-Sync Ingestion System

A comprehensive system for indexing, tracking, and synchronizing documentation across the Phenotype monorepo.

## Features

- **Spec Marker Extraction**: Automatically identifies FR-*, ADR-*, P*.*, UJ-*, NFR-*, and E*.*.* markers
- **Cross-Reference Mapping**: Builds bidirectional reference graph of document links
- **Document Indexing**: Generates structured JSON indexes of all documentation
- **Change Detection**: Identifies new, modified, and deleted documents
- **JSON Output**: Machine-readable indexes and sync manifests for tooling integration

## Quick Start

### Generate Index

```bash
python scripts/doc-sync/ingester.py
```

This generates:
- `.agileplus/doc-index.json` — Complete document index
- `.agileplus/doc-sync-manifest.json` — Initial sync manifest

### Compare Against Previous Index

```bash
python scripts/doc-sync/ingester.py --compare-index .agileplus/doc-index.json
```

This detects:
- New documents (added)
- Modified documents (file hash changed)
- Unchanged documents
- Deleted documents (in previous but not current)

### Custom Output Paths

```bash
python scripts/doc-sync/ingester.py \
  --output-index /tmp/index.json \
  --output-manifest /tmp/manifest.json
```

## Output Formats

### Document Index (`.agileplus/doc-index.json`)

```json
{
  "generated_at": "2026-03-29T15:30:00.000000",
  "total_documents": 65,
  "categories": {
    "reference": 8,
    "reports": 9,
    "worklog": 39,
    "governance": 1
  },
  "marker_summary": {
    "FR": 42,
    "P": 85,
    "ADR": 3,
    "NFR": 12
  },
  "documents": [
    {
      "path": "reference/FR_TRACKER.md",
      "absolute_path": "/path/to/docs/reference/FR_TRACKER.md",
      "title": "Functional Requirements Tracker",
      "category": "reference",
      "status": "active",
      "spec_markers": [
        {
          "type": "FR",
          "value": "FR-AUTH-001",
          "line_number": 5,
          "context": "| FR-AUTH-001 | Implemented |"
        }
      ],
      "references": [
        {
          "source_path": "reference/FR_TRACKER.md",
          "target_path": "../reference/CODE_ENTITY_MAP.md",
          "link_text": "code map",
          "line_number": 10
        }
      ],
      "file_hash": "abc123def456...",
      "last_modified": "2026-03-29T14:00:00",
      "size_bytes": 1024,
      "metadata": {
        "title": "Functional Requirements Tracker",
        "category": "reference",
        "status": "active"
      }
    }
  ]
}
```

### Sync Manifest (`.agileplus/doc-sync-manifest.json`)

```json
{
  "generated_at": "2026-03-29T15:30:00.000000",
  "comparison_index": null,
  "total_documents": 65,
  "actions": [
    {
      "path": "reference/FR_TRACKER.md",
      "action": "add",
      "reason": "new file",
      "new_hash": "abc123...",
      "old_hash": null
    },
    {
      "path": "reference/CODE_ENTITY_MAP.md",
      "action": "update",
      "reason": "file hash changed",
      "old_hash": "old123...",
      "new_hash": "new456..."
    }
  ],
  "summary": {
    "added": 2,
    "updated": 5,
    "unchanged": 58,
    "deleted": 0
  }
}
```

## Spec Marker Patterns

The ingester recognizes the following specification marker patterns:

| Pattern | Meaning | Example |
|---------|---------|---------|
| `FR-[A-Z]+-\d+` | Functional Requirement | `FR-AUTH-001`, `FR-STORAGE-042` |
| `E\d+\.\d+\.\d+` | Epic (semver-like) | `E1.2.3`, `E2.0.1` |
| `ADR-\d+` | Architecture Decision Record | `ADR-001`, `ADR-042` |
| `P\d+\.\d+` | Plan task ID | `P1.1`, `P2.3.5` |
| `UJ-\d+` | User Journey ID | `UJ-1`, `UJ-42` |
| `NFR-[A-Z]+-\d+` | Non-Functional Requirement | `NFR-PERF-001`, `NFR-SEC-010` |

## Document Category Inference

Documents are automatically categorized based on their directory path:

| Path | Category |
|------|----------|
| `docs/reference/` | reference |
| `docs/reports/` | reports |
| `docs/research/` | research |
| `docs/governance/` | governance |
| `docs/worklogs/` | worklog |
| `docs/guide/` | guide |
| `docs/sessions/` | session |
| Root `.md` files | root_spec |

## Module Structure

- **`schema.py`**: Pydantic models, marker patterns, extraction utilities
- **`ingester.py`**: Main CLI tool, ingestion engine, sync detection
- **`test_ingester.py`**: Comprehensive unit and integration tests

## Testing

Run tests:

```bash
pytest tests/doc-sync/ -v
```

Test coverage:

```bash
pytest tests/doc-sync/ --cov=scripts/doc-sync
```

## Architecture

```
DocumentIngester
  ├─ walk_docs() → List[Path]
  ├─ ingest_document(Path) → DocumentIndex
  ├─ ingest_all() → List[DocumentIndex]
  ├─ build_index() → DocumentIndexFile
  ├─ detect_changes(DocumentIndexFile) → (List[SyncAction], Dict)
  └─ build_manifest(Optional[DocumentIndexFile]) → SyncManifest
```

## Integration with AgilePlus (Future)

The ingestion system is designed to integrate with AgilePlus for bidirectional sync:

1. **Doc → AgilePlus**: New/modified documents → create/update specs
2. **AgilePlus → Doc**: Spec changes → update doc metadata/status
3. **Cross-linking**: Map doc changes to implementation progress
4. **Traceability**: Connect docs to code via spec markers

## Development Notes

- All models use Pydantic v2 for validation
- File hashing uses SHA256 for reliable change detection
- Cross-references are extracted from markdown link syntax only
- Frontmatter parsing supports simple YAML key:value format
- All timestamps are ISO 8601 UTC

## Performance

- Ingests 65+ documents in < 100ms
- File hashing optimized with 4KB block reads
- No external service dependencies

## License

Part of the Phenotype project ecosystem.
