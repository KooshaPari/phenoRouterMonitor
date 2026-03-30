"""Pydantic models and spec marker patterns for document ingestion."""

import re
from dataclasses import dataclass
from datetime import datetime
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


# Spec marker patterns (regex) — ordered so longer patterns match first
SPEC_MARKERS = {
    "NFR": re.compile(r"NFR-[A-Z]+-\d+"),  # Non-Functional Requirements (must come before FR)
    "FR": re.compile(r"(?<!N)FR-[A-Z]+-\d+"),  # Functional Requirements (negative lookbehind for N)
    "E": re.compile(r"E\d+\.\d+\.\d+"),  # Epic IDs
    "ADR": re.compile(r"ADR-\d+"),  # Architecture Decision Records
    "P": re.compile(r"P\d+\.\d+"),  # Plan task IDs
    "UJ": re.compile(r"UJ-\d+"),  # User Journey IDs
}

# Document category mapping from path
CATEGORY_MAPPING = {
    "reference": "reference",
    "reports": "reports",
    "research": "research",
    "governance": "governance",
    "worklogs": "worklog",
    "guide": "guide",
    "sessions": "session",
}


class DocumentReference(BaseModel):
    """A reference from one document to another."""

    source_path: str = Field(description="Path to document containing the reference")
    target_path: str = Field(description="Path to referenced document (relative or absolute)")
    link_text: Optional[str] = Field(None, description="Markdown link text")
    line_number: Optional[int] = Field(None, description="Line number of reference")


class SpecMarker(BaseModel):
    """A spec marker found in a document."""

    type: str = Field(description="Marker type: FR, E, ADR, P, UJ, NFR")
    value: str = Field(description="Marker value (e.g., FR-AUTH-001)")
    line_number: Optional[int] = Field(None, description="Line number where found")
    context: Optional[str] = Field(None, description="Surrounding text for context")


class DocumentMetadata(BaseModel):
    """Metadata extracted from a document."""

    title: Optional[str] = Field(None, description="Document title from header or frontmatter")
    category: Optional[str] = Field(None, description="Category from path or frontmatter")
    status: Optional[str] = Field(None, description="Status from frontmatter")
    tags: List[str] = Field(default_factory=list, description="Tags from frontmatter")
    author: Optional[str] = Field(None, description="Author from frontmatter")
    created_at: Optional[str] = Field(None, description="Creation date from frontmatter")
    updated_at: Optional[str] = Field(None, description="Update date from frontmatter")


class DocumentIndex(BaseModel):
    """A document entry in the index."""

    path: str = Field(description="Relative path from docs/ root")
    absolute_path: str = Field(description="Absolute filesystem path")
    title: Optional[str] = Field(None, description="Document title")
    category: str = Field(description="Document category")
    status: Optional[str] = Field(None, description="Document status")
    spec_markers: List[SpecMarker] = Field(
        default_factory=list, description="Spec markers found in document"
    )
    references: List[DocumentReference] = Field(
        default_factory=list, description="References to other documents"
    )
    file_hash: str = Field(description="SHA256 hash of file contents")
    last_modified: str = Field(description="Last modified timestamp (ISO 8601)")
    size_bytes: int = Field(description="File size in bytes")
    metadata: DocumentMetadata = Field(default_factory=DocumentMetadata)


class SyncAction(BaseModel):
    """An action to take for a document during sync."""

    path: str = Field(description="Document path")
    action: str = Field(description="Action: add, update, unchanged, or delete")
    reason: Optional[str] = Field(None, description="Why this action was chosen")
    old_hash: Optional[str] = Field(None, description="Previous file hash (for updates)")
    new_hash: Optional[str] = Field(None, description="New file hash")


class SyncManifest(BaseModel):
    """Summary of sync operations."""

    generated_at: str = Field(description="Generation timestamp (ISO 8601)")
    comparison_index: Optional[str] = Field(None, description="Path to index being compared against")
    total_documents: int = Field(description="Total documents processed")
    actions: List[SyncAction] = Field(description="Sync actions for each document")
    summary: Dict[str, int] = Field(
        description="Count by action type: added, updated, unchanged, deleted"
    )


class DocumentIndexFile(BaseModel):
    """Full document index file structure."""

    generated_at: str = Field(description="Generation timestamp (ISO 8601)")
    total_documents: int = Field(description="Total documents indexed")
    categories: Dict[str, int] = Field(description="Count of documents per category")
    marker_summary: Dict[str, int] = Field(description="Count of each marker type found")
    documents: List[DocumentIndex] = Field(description="All indexed documents")
    metadata: Dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata about the index"
    )


@dataclass
class ExtractionResult:
    """Result of extracting markers from a document."""

    markers: List[SpecMarker]
    references: List[DocumentReference]


def extract_spec_markers(text: str, path: str) -> ExtractionResult:
    """
    Extract spec markers from document text.

    Args:
        text: Document text content
        path: Document path (for reference building)

    Returns:
        ExtractionResult with markers and references
    """
    markers = []
    references = []

    for line_num, line in enumerate(text.split("\n"), 1):
        # Extract markers — use a dedicated set to track already-matched ranges to avoid overlaps
        matched_ranges = set()

        # Process NFR first (most specific)
        for match in SPEC_MARKERS["NFR"].finditer(line):
            markers.append(
                SpecMarker(
                    type="NFR",
                    value=match.group(),
                    line_number=line_num,
                    context=line.strip()[:100],
                )
            )
            matched_ranges.add((match.start(), match.end()))

        # Then process FR (with negative lookbehind already in pattern)
        for match in SPEC_MARKERS["FR"].finditer(line):
            if (match.start(), match.end()) not in matched_ranges:
                markers.append(
                    SpecMarker(
                        type="FR",
                        value=match.group(),
                        line_number=line_num,
                        context=line.strip()[:100],
                    )
                )
                matched_ranges.add((match.start(), match.end()))

        # Process other marker types
        for marker_type in ["E", "ADR", "P", "UJ"]:
            for match in SPEC_MARKERS[marker_type].finditer(line):
                if (match.start(), match.end()) not in matched_ranges:
                    markers.append(
                        SpecMarker(
                            type=marker_type,
                            value=match.group(),
                            line_number=line_num,
                            context=line.strip()[:100],
                        )
                    )
                    matched_ranges.add((match.start(), match.end()))

        # Extract markdown links [text](path)
        link_pattern = re.compile(r"\[([^\]]+)\]\(([^\)]+)\)")
        for match in link_pattern.finditer(line):
            link_text = match.group(1)
            target_path = match.group(2)
            # Only include markdown file links
            if target_path.endswith(".md") or not target_path.startswith("http"):
                references.append(
                    DocumentReference(
                        source_path=path,
                        target_path=target_path,
                        link_text=link_text,
                        line_number=line_num,
                    )
                )

    return ExtractionResult(markers=markers, references=references)


def infer_title_from_markdown(text: str) -> Optional[str]:
    """
    Extract title from markdown content.

    Looks for:
    1. frontmatter 'title' field
    2. First H1 header (#)
    3. First H2 header (##) if no H1

    Args:
        text: Document text

    Returns:
        Extracted title or None
    """
    lines = text.split("\n")

    # Check for YAML frontmatter
    if lines[0].strip() == "---":
        for line in lines[1:]:
            if line.strip() == "---":
                break
            if line.startswith("title:"):
                # Extract quoted or unquoted title
                title = line.split(":", 1)[1].strip()
                return title.strip('"\'')

    # Look for headers
    for line in lines:
        line = line.strip()
        if line.startswith("# ") and not line.startswith("## "):
            return line[2:].strip()
        if line.startswith("## "):
            return line[3:].strip()

    return None


def infer_category_from_path(path: str) -> str:
    """
    Infer document category from file path.

    Args:
        path: Relative path from docs/ root

    Returns:
        Category string
    """
    parts = path.split("/")

    # Check first path component
    if parts and parts[0] in CATEGORY_MAPPING:
        return CATEGORY_MAPPING[parts[0]]

    # Root-level docs
    if len(parts) == 1:
        return "root_spec"

    # Default to directory name
    return parts[0] if parts else "uncategorized"


def extract_frontmatter(text: str) -> Dict[str, Any]:
    """
    Extract YAML frontmatter from document.

    Args:
        text: Document text

    Returns:
        Dict of frontmatter fields or empty dict
    """
    if not text.startswith("---"):
        return {}

    lines = text.split("\n")
    end_idx = -1

    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            end_idx = i
            break

    if end_idx == -1:
        return {}

    # Simple YAML parsing (key: value format only)
    fm = {}
    for line in lines[1:end_idx]:
        if ":" in line:
            key, value = line.split(":", 1)
            fm[key.strip()] = value.strip().strip('"\'')

    return fm
