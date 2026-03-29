"""
Document Processor

Processes markdown files from the queue, applying transformations,
analysis, and categorization.
"""

import logging
from collections.abc import Callable
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Any


logger = logging.getLogger(__name__)


class ProcessingStatus(Enum):
    """Status of document processing."""

    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class ProcessingResult:
    """Result of processing a document."""

    filepath: str
    status: ProcessingStatus
    metadata: dict[str, Any] | None = None
    error: str | None = None
    processing_time: float | None = None

    def __post_init__(self) -> None:
        if self.metadata is None:
            self.metadata = {}


class ProcessingPipeline:
    """Pipeline for processing documents through multiple stages."""

    def __init__(self) -> None:
        self.stages: list[Callable[[Path], dict[str, Any]]] = []

    def add_stage(self, stage: Callable[[Path], dict[str, Any]]) -> "ProcessingPipeline":
        """Add a processing stage to the pipeline."""
        self.stages.append(stage)
        return self

    def process(self, filepath: Path) -> ProcessingResult:
        """Process a document through all stages."""
        import time

        start_time = time.time()

        try:
            metadata = {}
            for stage in self.stages:
                stage_result = stage(filepath)
                if isinstance(stage_result, dict):
                    metadata.update(stage_result)

            return ProcessingResult(
                filepath=str(filepath),
                status=ProcessingStatus.COMPLETED,
                metadata=metadata,
                processing_time=time.time() - start_time,
            )
        except Exception as e:
            return ProcessingResult(
                filepath=str(filepath),
                status=ProcessingStatus.FAILED,
                error=str(e),
                processing_time=time.time() - start_time,
            )


class DocumentProcessor:
    """Processes documents from the queue."""

    def __init__(self, pipeline: ProcessingPipeline | None = None) -> None:
        self.pipeline = pipeline or ProcessingPipeline()
        self.results: list[ProcessingResult] = []

    def process_file(self, filepath: str) -> ProcessingResult:
        """Process a single file."""
        path = Path(filepath)
        if not path.exists():
            return ProcessingResult(filepath=filepath, status=ProcessingStatus.FAILED, error="File not found")

        result = self.pipeline.process(path)
        self.results.append(result)
        return result

    def process_batch(self, filepaths: list[str]) -> list[ProcessingResult]:
        """Process multiple files."""
        results = []
        for filepath in filepaths:
            result = self.process_file(filepath)
            results.append(result)
        return results

    def get_statistics(self) -> dict[str, Any]:
        """Get processing statistics."""
        total = len(self.results)
        if total == 0:
            return {"total": 0, "completed": 0, "failed": 0, "skipped": 0, "avg_processing_time": 0.0}

        completed = sum(1 for r in self.results if r.status == ProcessingStatus.COMPLETED)
        failed = sum(1 for r in self.results if r.status == ProcessingStatus.FAILED)
        skipped = sum(1 for r in self.results if r.status == ProcessingStatus.SKIPPED)

        processing_times = [r.processing_time for r in self.results if r.processing_time is not None]
        avg_time = sum(processing_times) / len(processing_times) if processing_times else 0.0

        return {
            "total": total,
            "completed": completed,
            "failed": failed,
            "skipped": skipped,
            "avg_processing_time": avg_time,
        }


# Built-in processing stages


def compute_file_hash(filepath: Path) -> dict[str, Any]:
    """Compute file hash."""
    import hashlib

    with open(filepath, "rb") as f:
        content = f.read()
        sha256 = hashlib.sha256(content).hexdigest()
    return {"hash": sha256}


def extract_metadata(filepath: Path) -> dict[str, Any]:
    """Extract basic file metadata."""
    stat = filepath.stat()
    return {
        "size": stat.st_size,
        "modified": stat.st_mtime,
        "created": stat.st_ctime,
    }


def count_lines(filepath: Path) -> dict[str, Any]:
    """Count lines in file."""
    try:
        with open(filepath, encoding="utf-8") as f:
            line_count = sum(1 for _ in f)
        return {"line_count": line_count}
    except Exception:
        return {"line_count": 0}


def extract_frontmatter(filepath: Path) -> dict[str, Any]:
    """Extract YAML frontmatter from markdown file."""
    try:
        content = filepath.read_text(encoding="utf-8")
        if not content.startswith("---"):
            return {}

        parts = content.split("---", 2)
        if len(parts) < 3:
            return {}

        import yaml

        try:
            frontmatter = yaml.safe_load(parts[1])
        except yaml.YAMLError as exc:
            logger.debug("Failed to parse YAML frontmatter for %s: %s", filepath, exc)
            return {}

        if not isinstance(frontmatter, dict):
            logger.debug("Ignoring non-dictionary frontmatter in %s", filepath)
            return {}

        return {"frontmatter": frontmatter} if frontmatter else {}
    except OSError as exc:
        logger.warning("Failed to read markdown for frontmatter extraction: %s", filepath, exc_info=exc)
    except Exception as exc:
        logger.debug("Frontmatter extraction failed for %s: %s", filepath, exc)
    return {}


def extract_headings(filepath: Path) -> dict[str, Any]:
    """Extract headings from markdown file."""
    import re

    try:
        content = filepath.read_text(encoding="utf-8")
        headings = re.findall(r"^(#{1,6})\s+(.+)$", content, re.MULTILINE)
        heading_levels = [len(h[0]) for h in headings]
        return {
            "headings": [h[1].strip() for h in headings],
            "heading_count": len(headings),
            "max_heading_level": max(heading_levels) if heading_levels else 0,
        }
    except Exception:
        return {"headings": [], "heading_count": 0, "max_heading_level": 0}


def extract_links(filepath: Path) -> dict[str, Any]:
    """Extract links from markdown file."""
    import re

    try:
        content = filepath.read_text(encoding="utf-8")
        # Extract markdown links [text](url)
        md_links = re.findall(r"\[([^\]]+)\]\(([^\)]+)\)", content)
        # Extract plain URLs
        urls = re.findall(r"https?://[^\s\)]+", content)
        return {
            "markdown_links": len(md_links),
            "urls": len(urls),
            "total_links": len(md_links) + len(urls),
        }
    except Exception:
        return {"markdown_links": 0, "urls": 0, "total_links": 0}


def extract_code_blocks(filepath: Path) -> dict[str, Any]:
    """Extract code block information."""
    import re

    try:
        content = filepath.read_text(encoding="utf-8")
        # Find code blocks with language tags
        code_blocks = re.findall(r"```(\w+)?\n(.*?)```", content, re.DOTALL)
        languages = [lang for lang, _ in code_blocks if lang]
        return {
            "code_block_count": len(code_blocks),
            "languages": list(set(languages)),
            "language_count": len(set(languages)),
        }
    except Exception:
        return {"code_block_count": 0, "languages": [], "language_count": 0}


def calculate_readability(filepath: Path) -> dict[str, Any]:
    """Calculate basic readability metrics."""
    try:
        content = filepath.read_text(encoding="utf-8")
        words = content.split()
        sentences = content.split(".")
        paragraphs = [p.strip() for p in content.split("\n\n") if p.strip()]

        avg_words_per_sentence = len(words) / len(sentences) if sentences else 0
        avg_sentences_per_paragraph = len(sentences) / len(paragraphs) if paragraphs else 0

        return {
            "word_count": len(words),
            "sentence_count": len(sentences),
            "paragraph_count": len(paragraphs),
            "avg_words_per_sentence": round(avg_words_per_sentence, 2),
            "avg_sentences_per_paragraph": round(avg_sentences_per_paragraph, 2),
        }
    except Exception:
        return {
            "word_count": 0,
            "sentence_count": 0,
            "paragraph_count": 0,
            "avg_words_per_sentence": 0.0,
            "avg_sentences_per_paragraph": 0.0,
        }
