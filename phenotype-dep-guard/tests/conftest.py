"""Pytest configuration for BDD tests."""

from __future__ import annotations

import sys
from pathlib import Path

# Add src to path for imports
src_path = Path(__file__).parent.parent / "src"
if str(src_path) not in sys.path:
    sys.path.insert(0, str(src_path))

# pytest-bdd configuration


def pytest_bdd_step_error(step, exception):
    """Handle step errors with detailed output."""
    print(f"\nStep failed: {step}")
    print(f"Exception: {exception}")


def pytest_bdd_apply_tag(tag, function):
    """Apply tags as pytest markers."""
    import pytest

    if tag in (
        "smoke",
        "security",
        "critical",
        "integration",
        "compliance",
        "audit",
        "http",
        "report",
    ):
        marker = getattr(pytest.mark, tag)
        marker(function)
    return True
