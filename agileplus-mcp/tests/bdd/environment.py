"""Behave environment setup for AgilePlus MCP BDD tests."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from behave.runner import Context


def before_all(context: Context) -> None:
    """Set up shared state for all BDD scenarios."""
    # Future: start a test MCP server instance
    context.mcp_host = "localhost"
    context.mcp_port = 8765


def after_all(context: Context) -> None:
    """Tear down shared state after all BDD scenarios."""
    # Future: stop the test MCP server instance
    pass


def before_scenario(context: Context, scenario: object) -> None:
    """Set up state before each BDD scenario."""
    pass


def after_scenario(context: Context, scenario: object) -> None:
    """Tear down state after each BDD scenario."""
    pass
