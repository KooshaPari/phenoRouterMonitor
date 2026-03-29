"""Enhanced error handling with actionable context and recovery suggestions.

This module provides utilities for creating rich, actionable error messages
that help users understand what went wrong, why it happened, and how to fix it.
"""

import sys
import traceback
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rich.console import Console
from rich.panel import Panel

console = Console()


@dataclass
class ErrorContext:
    """Rich context for error reporting."""

    error_type: str
    error_message: str
    what_happened: str
    why_it_happened: str
    how_to_fix: list[str]
    related_files: list[Path] | None = None
    related_config: dict[str, Any] | None = None
    documentation_link: str | None = None
    command_suggestion: str | None = None

    def __post_init__(self) -> None:
        if self.related_files is None:
            self.related_files = []
        if self.related_config is None:
            self.related_config = {}


class EnhancedError(Exception):
    """Base exception class with enhanced error reporting."""

    def __init__(
        self,
        message: str,
        context: ErrorContext | None = None,
        cause: Exception | None = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.context = context or ErrorContext(
            error_type=self.__class__.__name__,
            error_message=message,
            what_happened="An error occurred",
            why_it_happened="Unknown cause",
            how_to_fix=["Check the error message and try again"],
        )
        self.cause = cause

    def display(self) -> None:
        """Display the error with rich formatting."""
        # Error header
        console.print(f"\n[bold red]✗ {self.context.error_type}[/bold red]")
        console.print(f"[red]{self.context.error_message}[/red]\n")

        # What happened
        console.print(Panel(f"[bold]What happened:[/bold]\n{self.context.what_happened}", border_style="red"))

        # Why it happened
        console.print(Panel(f"[bold]Why it happened:[/bold]\n{self.context.why_it_happened}", border_style="yellow"))

        # How to fix
        fix_text = "\n".join(f"  • {fix}" for fix in self.context.how_to_fix)
        console.print(Panel(f"[bold]How to fix:[/bold]\n{fix_text}", border_style="green"))

        # Related files
        if self.context.related_files:
            files_text = "\n".join(f"  • {f}" for f in self.context.related_files)
            console.print(Panel(f"[bold]Related files:[/bold]\n{files_text}", border_style="blue"))

        # Command suggestion
        if self.context.command_suggestion:
            console.print(f"\n[dim]Try running:[/dim] [bold cyan]{self.context.command_suggestion}[/bold cyan]")

        # Documentation link
        if self.context.documentation_link:
            console.print(f"\n[dim]Learn more:[/dim] {self.context.documentation_link}")

        # Cause (if available)
        if self.cause:
            console.print(f"\n[dim]Original error:[/dim] {self.cause}")


class ConfigurationError(EnhancedError):
    """Error related to configuration issues."""


class InfraRuntimeError(EnhancedError):
    """Error related to runtime selection or execution."""


class DependencyError(EnhancedError):
    """Error related to missing or incompatible dependencies."""


class NetworkError(EnhancedError):
    """Error related to network connectivity."""


def format_error_with_context(error: Exception, context: ErrorContext | None = None) -> None:
    """Format and display an error with rich context."""
    if isinstance(error, EnhancedError):
        error.display()
    else:
        # Convert standard exception to enhanced error
        enhanced = EnhancedError(
            str(error),
            context=context
            or ErrorContext(
                error_type=type(error).__name__,
                error_message=str(error),
                what_happened="An unexpected error occurred",
                why_it_happened="Unknown cause",
                how_to_fix=[
                    "Check the error message above",
                    "Review the documentation at docs/guides/TROUBLESHOOTING.md",
                    "Run 'thegent doctor' to check environment health",
                    "Report the issue with 'thegent error report'",
                ],
            ),
            cause=error,
        )
        enhanced.display()


def create_config_error(message: str, config_file: Path, suggestion: str | None = None) -> ConfigurationError:
    """Create a configuration error with context."""
    context = ErrorContext(
        error_type="ConfigurationError",
        error_message=message,
        what_happened=f"Failed to load or validate configuration from {config_file}",
        why_it_happened="The configuration file may be missing, malformed, or contain invalid values",
        how_to_fix=[
            f"Check the configuration file at {config_file}",
            "Run 'thegent config validate' to check for issues",
            "See docs/guides/CONFIGURATION.md for configuration reference",
            suggestion or "Review the error message above for specific issues",
        ],
        related_files=[config_file],
        command_suggestion="thegent config validate",
        documentation_link="docs/guides/CONFIGURATION.md",
    )
    return ConfigurationError(message, context=context)


def create_runtime_error(
    message: str, runtime: str, available_runtimes: list[str], suggestion: str | None = None
) -> InfraRuntimeError:
    """Create a runtime error with context."""
    context = ErrorContext(
        error_type="RuntimeError",
        error_message=message,
        what_happened=f"Failed to use runtime '{runtime}'",
        why_it_happened=f"The requested runtime '{runtime}' is not available or not properly configured",
        how_to_fix=[
            f"Available runtimes: {', '.join(available_runtimes)}",
            "Run 'thegent doctor --runtime' to check runtime health",
            "See docs/architecture/RUNTIME_SELECTION_GUIDE.md for runtime selection",
            suggestion or "Try using a different runtime",
        ],
        command_suggestion="thegent doctor --runtime",
        documentation_link="docs/architecture/RUNTIME_SELECTION_GUIDE.md",
    )
    return InfraRuntimeError(message, context=context)


def create_dependency_error(message: str, dependency: str, install_command: str | None = None) -> DependencyError:
    """Create a dependency error with context."""
    fixes = [
        f"Install {dependency}",
        "Run 'thegent doctor' to check all dependencies",
    ]
    if install_command:
        fixes.insert(1, f"Run: {install_command}")

    context = ErrorContext(
        error_type="DependencyError",
        error_message=message,
        what_happened=f"Missing or incompatible dependency: {dependency}",
        why_it_happened=f"The required dependency '{dependency}' is not installed or not in PATH",
        how_to_fix=fixes,
        command_suggestion="thegent doctor",
        documentation_link="docs/guides/INSTALLATION.md",
    )
    return DependencyError(message, context=context)


def create_network_error(message: str, endpoint: str | None = None, suggestion: str | None = None) -> NetworkError:
    """Create a network error with context."""
    fixes = [
        "Check your internet connection",
        "Verify the endpoint is accessible",
        "Run 'thegent doctor --network' to diagnose network issues",
    ]
    if endpoint:
        fixes.insert(1, f"Test connectivity to: {endpoint}")
    if suggestion:
        fixes.append(suggestion)

    context = ErrorContext(
        error_type="NetworkError",
        error_message=message,
        what_happened="Network request failed",
        why_it_happened="Unable to connect to the remote endpoint. This could be due to network issues, firewall rules, or the endpoint being unavailable",
        how_to_fix=fixes,
        command_suggestion="thegent doctor --network",
        documentation_link="docs/guides/TROUBLESHOOTING.md#network-issues",
    )
    return NetworkError(message, context=context)


def error_report(error: Exception, include_traceback: bool = True) -> dict[str, Any]:
    """Generate a detailed error report for bug reporting."""
    report: dict[str, Any] = {
        "error_type": type(error).__name__,
        "error_message": str(error),
        "python_version": sys.version,
        "platform": sys.platform,
    }

    if isinstance(error, EnhancedError) and error.context is not None:
        context = error.context
        report["context"] = {
            "what_happened": context.what_happened,
            "why_it_happened": context.why_it_happened,
            "how_to_fix": context.how_to_fix,
            "related_files": [str(f) for f in context.related_files] if context.related_files else [],
        }

    if include_traceback:
        report["traceback"] = traceback.format_exc()

    return report


def format_error(exc: Exception) -> str:
    """Convert common exceptions to actionable, user-friendly error messages.

    # @trace WL-040 WP-4001

    Maps well-known exception types to concise messages with suggestions.
    For unrecognised exceptions the raw ``str(exc)`` is returned.

    Args:
        exc: The exception to convert.

    Returns:
        A human-readable, actionable error string.
    """
    if isinstance(exc, FileNotFoundError):
        path = exc.filename or str(exc)
        return f"File not found: {path}. Run `thegent doctor --fix` to repair."
    if isinstance(exc, PermissionError):
        path = exc.filename or str(exc)
        return f"Permission denied: {path}. Check file permissions."
    if isinstance(exc, ConnectionError):
        # Best-effort extraction of host from error message
        msg = str(exc)
        # ConnectionError subclasses often include host/address in the message
        host = msg.split("[Errno", maxsplit=1)[0].strip().strip("'\"") or "remote host"
        return f"Cannot connect to {host}. Check network and `thegent status`."
    if isinstance(exc, KeyError):
        key = exc.args[0] if exc.args else str(exc)
        return f"Missing config key: {key}. Run `thegent config wizard` to set up."
    # Fallback: return the raw message
    return str(exc)
