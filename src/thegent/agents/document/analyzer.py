"""
Document Analyzer

Analyzes markdown files to categorize, extract metadata, and identify
patterns or characteristics.
"""

import re
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import ClassVar


class DocumentCategory(Enum):
    """Categories for documents."""

    DOCUMENTATION = "documentation"
    RESEARCH = "research"
    PLAN = "plan"
    REPORT = "report"
    GUIDE = "guide"
    SPECIFICATION = "specification"
    README = "readme"
    CHANGELOG = "changelog"
    UNKNOWN = "unknown"


@dataclass
class DocumentAnalysis:
    """Analysis results for a document."""

    filepath: str
    category: DocumentCategory
    keywords: set[str]
    estimated_reading_time: float  # minutes
    has_code_blocks: bool
    has_images: bool
    has_links: bool
    section_count: int
    word_count: int

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "filepath": self.filepath,
            "category": self.category.value,
            "keywords": list(self.keywords),
            "estimated_reading_time": self.estimated_reading_time,
            "has_code_blocks": self.has_code_blocks,
            "has_images": self.has_images,
            "has_links": self.has_links,
            "section_count": self.section_count,
            "word_count": self.word_count,
        }


class DocumentAnalyzer:
    """Analyzes markdown documents."""

    # Category patterns
    CATEGORY_PATTERNS: ClassVar[dict[DocumentCategory, list[str]]] = {
        DocumentCategory.RESEARCH: [r"research", r"study", r"analysis", r"investigation"],
        DocumentCategory.PLAN: [r"plan", r"roadmap", r"strategy", r"proposal"],
        DocumentCategory.REPORT: [r"report", r"summary", r"audit", r"review"],
        DocumentCategory.GUIDE: [r"guide", r"tutorial", r"how.?to", r"walkthrough"],
        DocumentCategory.SPECIFICATION: [r"spec", r"specification", r"requirements", r"design"],
        DocumentCategory.README: [r"readme", r"read.?me"],
        DocumentCategory.CHANGELOG: [r"changelog", r"changes", r"history"],
    }

    def __init__(self) -> None:
        self.compiled_patterns = {
            cat: [re.compile(pattern, re.IGNORECASE) for pattern in patterns]
            for cat, patterns in self.CATEGORY_PATTERNS.items()
        }

    def analyze(self, filepath: Path) -> DocumentAnalysis:
        """Analyze a markdown file."""
        try:
            content = filepath.read_text(encoding="utf-8")
        except Exception:
            # Return minimal analysis for unreadable files
            return DocumentAnalysis(
                filepath=str(filepath),
                category=DocumentCategory.UNKNOWN,
                keywords=set(),
                estimated_reading_time=0.0,
                has_code_blocks=False,
                has_images=False,
                has_links=False,
                section_count=0,
                word_count=0,
            )

        # Extract features
        category = self._categorize(filepath, content)
        keywords = self._extract_keywords(content)
        word_count = len(content.split())
        estimated_reading_time = word_count / 200.0  # Average reading speed
        has_code_blocks = bool(re.search(r"```", content))
        has_images = bool(re.search(r"!\[.*?\]\(.*?\)", content))
        has_links = bool(re.search(r"\[.*?\]\(.*?\)", content))
        section_count = len(re.findall(r"^#+\s+", content, re.MULTILINE))

        return DocumentAnalysis(
            filepath=str(filepath),
            category=category,
            keywords=keywords,
            estimated_reading_time=estimated_reading_time,
            has_code_blocks=has_code_blocks,
            has_images=has_images,
            has_links=has_links,
            section_count=section_count,
            word_count=word_count,
        )

    def _categorize(self, filepath: Path, content: str) -> DocumentCategory:
        """Categorize document based on path and content."""
        path_lower = str(filepath).lower()
        content_lower = content.lower()

        # Check path patterns first
        if "readme" in path_lower:
            return DocumentCategory.README
        if "changelog" in path_lower or "changes" in path_lower:
            return DocumentCategory.CHANGELOG

        # Check content patterns
        for category, patterns in self.compiled_patterns.items():
            for pattern in patterns:
                if pattern.search(path_lower) or pattern.search(content_lower[:500]):
                    return category

        # Check directory structure
        path_parts = path_lower.split("/")
        if "research" in path_parts:
            return DocumentCategory.RESEARCH
        if "plan" in path_parts or "plans" in path_parts:
            return DocumentCategory.PLAN
        if "docs" in path_parts or "documentation" in path_parts:
            return DocumentCategory.DOCUMENTATION
        if "guide" in path_parts or "guides" in path_parts:
            return DocumentCategory.GUIDE

        return DocumentCategory.UNKNOWN

    def _extract_keywords(self, content: str, max_keywords: int = 10) -> set[str]:
        """Extract keywords from content."""
        # Simple keyword extraction (can be enhanced with NLP)
        words = re.findall(r"\b[a-z]{4,}\b", content.lower())

        # Common stop words to exclude
        stop_words = {
            "that",
            "this",
            "with",
            "from",
            "have",
            "will",
            "would",
            "could",
            "should",
            "there",
            "their",
            "these",
            "those",
            "which",
            "where",
            "when",
            "what",
            "about",
            "after",
            "before",
            "during",
            "while",
            "under",
            "over",
            "above",
        }

        # Count word frequencies
        word_freq = {}
        for word in words:
            if word not in stop_words and len(word) > 3:
                word_freq[word] = word_freq.get(word, 0) + 1

        # Return top keywords
        sorted_words = sorted(word_freq.items(), key=lambda x: x[1], reverse=True)
        return {word for word, _ in sorted_words[:max_keywords]}
