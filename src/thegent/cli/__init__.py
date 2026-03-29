"""thegent CLI module.

This module provides the command-line interface for thegent.
"""

try:
    from thegent.cli.commands import bg_impl, run_impl
    __all__ = ["bg_impl", "run_impl"]
except ImportError:
    __all__ = []
