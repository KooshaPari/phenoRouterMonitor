"""Validation helpers for thegent.

Common validation utilities.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any


def validate_email(email: str) -> bool:
    """Validate email format."""
    pattern = r"^[\w\.-]+@[\w\.-]+\.\w+$"
    return bool(re.match(pattern, email))


def validate_url(url: str) -> bool:
    """Validate URL format."""
    return url.startswith(("http://", "https://"))


def validate_port(port: int) -> bool:
    """Validate port number."""
    return 1 <= port <= 65535


def validate_path(path: str) -> bool:
    """Validate path exists."""
    return Path(path).exists()


def sanitize_filename(name: str) -> str:
    """Sanitize filename."""
    return re.sub(r'[<>:"/\\|?*]', "_", name)
