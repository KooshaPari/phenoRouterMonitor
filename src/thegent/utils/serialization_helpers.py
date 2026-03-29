"""Serialization utilities for thegent.

Common serialization helpers.
"""

from __future__ import annotations

import json
from typing import Any


def to_json(obj: Any, indent: int = 2) -> str:
    """Convert object to JSON string."""
    return json.dumps(obj, indent=indent, default=str)


def from_json(text: str) -> Any:
    """Parse JSON string to object."""
    return json.loads(text)


def to_json_file(obj: Any, path: str, **kwargs: Any) -> None:
    """Write object to JSON file."""
    with open(path, "w") as f:
        json.dump(obj, f, default=str, **kwargs)


def from_json_file(path: str) -> Any:
    """Read object from JSON file."""
    with open(path) as f:
        return json.load(f)
