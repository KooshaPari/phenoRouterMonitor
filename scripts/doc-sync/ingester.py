#!/usr/bin/env python3
"""Documentation auto-sync ingestion CLI tool.

Walks docs/ directory, extracts spec markers and cross-references,
generates JSON index, and detects sync changes.

Usage:
    python ingester.py [--compare-index PATH] [--output PATH]

Examples:
    # Generate fresh index
    python ingester.py

    # Generate index and compare against previous
    python ingester.py --compare-index .agileplus/doc-index.json

    # Output to custom location
    python ingester.py --output /tmp/index.json
"""

import argparse
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional

from schema import (
    DocumentIndex,
    DocumentIndexFile,
    DocumentMetadata,
    DocumentReference,
    SyncAction,
    SyncManifest,
    extract_frontmatter,
    extract_spec_markers,
    infer_category_from_path,
    infer_title_from_markdown,
)


class DocumentIngester:
    """Main ingestion engine."""

    def __init__(self, docs_root: Path):
        """
        Initialize ingester.

        Args:
            docs_root: Root path to docs directory
        """
        self.docs_root = docs_root
        self.documents: List[DocumentIndex] = []
        self.category_counts: Dict[str, int] = {}
        self.marker_counts: Dict[str, int] = {}

    def walk_docs(self) -> List[Path]:
        """
        Walk docs directory and find all markdown files.

        Returns:
            List of markdown file paths
        """
        if not self.docs_root.exists():
            raise FileNotFoundError(f"Docs root not found: {self.docs_root}")

        markdown_files = list(self.docs_root.rglob("*.md"))
        return sorted(markdown_files)

    def compute_file_hash(self, path: Path) -> str:
        """Compute SHA256 hash of file."""
        sha256_hash = hashlib.sha256()
        with open(path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()

    def ingest_document(self, file_path: Path) -> Optional[DocumentIndex]:
        """
        Parse a single markdown document.

        Args:
            file_path: Path to markdown file

        Returns:
            DocumentIndex entry or None if parsing fails
        """
        try:
            content = file_path.read_text(encoding="utf-8")
        except Exception as e:
            print(f"Error reading {file_path}: {e}", file=sys.stderr)
            return None

        # Relative path from docs root
        rel_path = file_path.relative_to(self.docs_root).as_posix()

        # Extract metadata
        title = infer_title_from_markdown(content)
        category = infer_category_from_path(rel_path)
        frontmatter = extract_frontmatter(content)

        status = frontmatter.get("status")
        author = frontmatter.get("author")
        created_at = frontmatter.get("created_at")
        updated_at = frontmatter.get("updated_at")

        # Extract spec markers and references
        extraction = extract_spec_markers(content, rel_path)
        markers = extraction.markers
        references = extraction.references

        # Update counts
        self.category_counts[category] = self.category_counts.get(category, 0) + 1
        for marker in markers:
            self.marker_counts[marker.type] = self.marker_counts.get(marker.type, 0) + 1

        # Build document index entry
        doc = DocumentIndex(
            path=rel_path,
            absolute_path=str(file_path.absolute()),
            title=title or rel_path,
            category=category,
            status=status,
            spec_markers=markers,
            references=references,
            file_hash=self.compute_file_hash(file_path),
            last_modified=datetime.fromtimestamp(file_path.stat().st_mtime).isoformat(),
            size_bytes=file_path.stat().st_size,
            metadata=DocumentMetadata(
                title=title,
                category=category,
                status=status,
                author=author,
                created_at=created_at,
                updated_at=updated_at,
            ),
        )

        return doc

    def ingest_all(self) -> List[DocumentIndex]:
        """
        Ingest all documents in docs directory.

        Returns:
            List of DocumentIndex entries
        """
        files = self.walk_docs()
        print(f"Found {len(files)} markdown files", file=sys.stderr)

        documents = []
        for file_path in files:
            doc = self.ingest_document(file_path)
            if doc:
                documents.append(doc)

        self.documents = sorted(documents, key=lambda d: d.path)
        return self.documents

    def build_index(self) -> DocumentIndexFile:
        """
        Build complete index file.

        Returns:
            DocumentIndexFile with all metadata
        """
        now = datetime.now(timezone.utc).isoformat()
        return DocumentIndexFile(
            generated_at=now,
            total_documents=len(self.documents),
            categories=self.category_counts,
            marker_summary=self.marker_counts,
            documents=self.documents,
            metadata={
                "docs_root": str(self.docs_root),
                "generation_timestamp": now,
            },
        )

    def detect_changes(
        self, previous_index: DocumentIndexFile
    ) -> tuple[List[SyncAction], Dict[str, int]]:
        """
        Compare current state with previous index.

        Args:
            previous_index: Previous DocumentIndexFile

        Returns:
            Tuple of (sync_actions, summary)
        """
        current_paths = {doc.path: doc for doc in self.documents}
        previous_paths = {doc.path: doc for doc in previous_index.documents}

        actions = []
        summary = {"added": 0, "updated": 0, "unchanged": 0, "deleted": 0}

        # Check for modified and unchanged
        for path, current_doc in current_paths.items():
            if path in previous_paths:
                prev_doc = previous_paths[path]
                if current_doc.file_hash == prev_doc.file_hash:
                    actions.append(
                        SyncAction(
                            path=path,
                            action="unchanged",
                            new_hash=current_doc.file_hash,
                        )
                    )
                    summary["unchanged"] += 1
                else:
                    actions.append(
                        SyncAction(
                            path=path,
                            action="update",
                            reason="file hash changed",
                            old_hash=prev_doc.file_hash,
                            new_hash=current_doc.file_hash,
                        )
                    )
                    summary["updated"] += 1
            else:
                actions.append(
                    SyncAction(
                        path=path,
                        action="add",
                        reason="new file",
                        new_hash=current_doc.file_hash,
                    )
                )
                summary["added"] += 1

        # Check for deleted
        for path in previous_paths:
            if path not in current_paths:
                actions.append(
                    SyncAction(
                        path=path,
                        action="delete",
                        reason="file deleted",
                        old_hash=previous_paths[path].file_hash,
                    )
                )
                summary["deleted"] += 1

        return sorted(actions, key=lambda a: a.path), summary

    def build_manifest(
        self, previous_index: Optional[DocumentIndexFile]
    ) -> SyncManifest:
        """
        Build sync manifest.

        Args:
            previous_index: Previous index for comparison (optional)

        Returns:
            SyncManifest
        """
        if previous_index:
            actions, summary = self.detect_changes(previous_index)
        else:
            # No previous index: all current docs are "added"
            actions = [
                SyncAction(
                    path=doc.path,
                    action="add",
                    reason="baseline ingestion",
                    new_hash=doc.file_hash,
                )
                for doc in self.documents
            ]
            summary = {
                "added": len(self.documents),
                "updated": 0,
                "unchanged": 0,
                "deleted": 0,
            }

        return SyncManifest(
            generated_at=datetime.now(timezone.utc).isoformat(),
            comparison_index=None,
            total_documents=len(self.documents),
            actions=actions,
            summary=summary,
        )


def load_previous_index(index_path: Path) -> Optional[DocumentIndexFile]:
    """Load previous index from file."""
    if not index_path.exists():
        return None

    try:
        data = json.loads(index_path.read_text(encoding="utf-8"))
        return DocumentIndexFile(**data)
    except Exception as e:
        print(f"Warning: could not load previous index: {e}", file=sys.stderr)
        return None


def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Documentation auto-sync ingestion tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--docs-root",
        type=Path,
        default=None,
        help="Path to docs root (default: ./docs relative to script parent)",
    )
    parser.add_argument(
        "--compare-index",
        type=Path,
        default=None,
        help="Path to previous index for change detection",
    )
    parser.add_argument(
        "--output-index",
        type=Path,
        default=None,
        help="Path to write index JSON (default: .agileplus/doc-index.json)",
    )
    parser.add_argument(
        "--output-manifest",
        type=Path,
        default=None,
        help="Path to write sync manifest (default: .agileplus/doc-sync-manifest.json)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Verbose output",
    )

    args = parser.parse_args()

    # Determine docs root
    if args.docs_root is None:
        script_dir = Path(__file__).parent
        docs_root = script_dir.parent.parent / "docs"
    else:
        docs_root = args.docs_root

    # Determine output paths
    repo_root = Path(__file__).parent.parent.parent
    if args.output_index is None:
        output_index = repo_root / ".agileplus" / "doc-index.json"
    else:
        output_index = args.output_index

    if args.output_manifest is None:
        output_manifest = repo_root / ".agileplus" / "doc-sync-manifest.json"
    else:
        output_manifest = args.output_manifest

    # Create output directory if needed
    output_index.parent.mkdir(parents=True, exist_ok=True)
    output_manifest.parent.mkdir(parents=True, exist_ok=True)

    # Run ingestion
    print(f"Ingesting docs from: {docs_root}", file=sys.stderr)
    ingester = DocumentIngester(docs_root)

    try:
        ingester.ingest_all()
    except Exception as e:
        print(f"Error during ingestion: {e}", file=sys.stderr)
        return 1

    # Build index
    index = ingester.build_index()
    print(f"Indexed {index.total_documents} documents", file=sys.stderr)
    print(f"Categories: {dict(index.categories)}", file=sys.stderr)
    print(f"Markers: {dict(index.marker_summary)}", file=sys.stderr)

    # Write index
    output_index.write_text(index.model_dump_json(indent=2), encoding="utf-8")
    print(f"Wrote index to: {output_index}", file=sys.stderr)

    # Build and write manifest
    previous_index = None
    if args.compare_index:
        previous_index = load_previous_index(args.compare_index)

    manifest = ingester.build_manifest(previous_index)
    output_manifest.write_text(manifest.model_dump_json(indent=2), encoding="utf-8")
    print(f"Wrote manifest to: {output_manifest}", file=sys.stderr)
    print(
        f"Sync summary: added={manifest.summary['added']}, "
        f"updated={manifest.summary['updated']}, "
        f"unchanged={manifest.summary['unchanged']}, "
        f"deleted={manifest.summary['deleted']}",
        file=sys.stderr,
    )

    return 0


if __name__ == "__main__":
    sys.exit(main())
