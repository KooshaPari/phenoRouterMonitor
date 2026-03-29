"""
thegent Document Agent Module

Provides document scanning, queue management, and processing capabilities
for markdown files across multiple locations and time periods.
"""

from .analyzer import DocumentAnalyzer, DocumentCategory
from .processor import DocumentProcessor, ProcessingPipeline
from .queue_manager import QueueManager, QueueState
from .scanner import MarkdownScanner, ScanConfig

__all__ = [
    "DocumentAnalyzer",
    "DocumentCategory",
    "DocumentProcessor",
    "MarkdownScanner",
    "ProcessingPipeline",
    "QueueManager",
    "QueueState",
    "ScanConfig",
]

__version__ = "0.1.0"
