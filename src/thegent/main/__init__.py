"""Thegent CLI entry point.

NOTE: This module is a stub. The actual CLI is located at:
- phenotype-thegent-cli package (apps structure)
- src/thegent/cli/commands/ (command implementations)

This stub exists for backwards compatibility with existing tests.
"""

from typing import Any

# Attempt to import from the CLI commands structure
try:
    from thegent.cli.apps.main import app
except ImportError:
    # Fallback: create a minimal stub app
    from typer import Typer
    app: Any = Typer()
    app.__doc__ = "Thegent CLI (stub - actual implementation in phenotype-thegent-cli)"

__all__ = ["app"]
