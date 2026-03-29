"""Install helpers for thegent installation.

Helper functions for installation process.
"""

from pathlib import Path


def _get_thegent_root() -> Path:
    """Get thegent root directory."""
    return Path(__file__).parent.parent.resolve()


def should_exclude(path: str) -> bool:
    """Check if path should be excluded from installation."""
    excludes = {".git", "__pycache__", ".pytest_cache", "node_modules"}
    return any(ex in path for ex in excludes)
