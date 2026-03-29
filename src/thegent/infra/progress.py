"""Progress indicators and status updates for long-running operations.

This module provides utilities for displaying progress bars, spinners, and
status updates in a consistent, beautiful way.
"""

import time
from contextlib import contextmanager

from rich.console import Console
from rich.progress import (
    BarColumn,
    Progress,
    SpinnerColumn,
    TextColumn,
    TimeElapsedColumn,
    TimeRemainingColumn,
)

console = Console()


@contextmanager
def progress_context(
    description: str,
    total: int | None = None,
    show_eta: bool = True,
    show_speed: bool = False,
):
    """Context manager for progress tracking.

    Args:
        description: Description of the operation
        total: Total number of steps (None for indeterminate)
        show_eta: Show estimated time remaining
        show_speed: Show processing speed

    Example:
        >>> with progress_context("Processing files", total=100) as progress:
        ...     for i in range(100):
        ...         progress.update(1)
    """
    columns = [
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
    ]

    if show_eta:
        columns.append(TimeRemainingColumn())

    if show_speed:
        columns.append(TextColumn("[progress.speed]{task.speed:.2f}/s"))

    columns.append(TimeElapsedColumn())

    with Progress(*columns, console=console) as progress:
        _task = progress.add_task(description, total=total)
        yield progress


@contextmanager
def spinner_context(message: str):
    """Context manager for spinner display.

    Args:
        message: Message to display while spinning

    Example:
        >>> with spinner_context("Loading data..."):
        ...     time.sleep(2)
    """
    with console.status(f"[bold cyan]{message}[/bold cyan]", spinner="dots"):
        yield


def print_status(message: str, status: str = "info") -> None:
    """Print a status message with appropriate styling.

    Args:
        message: Status message
        status: Status type (info, success, warning, error)

    Example:
        >>> print_status("Operation completed", "success")
    """
    icons = {
        "info": "[bold blue]ℹ[/bold blue]",
        "success": "[bold green]✓[/bold green]",
        "warning": "[bold yellow]⚠[/bold yellow]",
        "error": "[bold red]✗[/bold red]",
    }
    icon = icons.get(status, icons["info"])
    console.print(f"{icon} {message}")


def print_step(step: int, total: int, message: str) -> None:
    """Print a step indicator.

    Args:
        step: Current step number
        total: Total number of steps
        message: Step message

    Example:
        >>> print_step(1, 3, "Installing dependencies")
    """
    console.print(f"[bold cyan][{step}/{total}][/bold cyan] {message}")


def print_section(title: str) -> None:
    """Print a section header.

    Args:
        title: Section title

    Example:
        >>> print_section("Configuration")
    """
    console.print(f"\n[bold cyan]{title}[/bold cyan]")
    console.print("[dim]" + "─" * len(title) + "[/dim]\n")


def measure_time(description: str):
    """Decorator to measure and display execution time.

    Args:
        description: Description of the operation

    Example:
        >>> @measure_time("Processing data")
        ... def process_data():
        ...     time.sleep(1)
    """

    def decorator(func):
        def wrapper(*args, **kwargs):
            start = time.time()
            with spinner_context(description):
                result = func(*args, **kwargs)
            elapsed = time.time() - start
            print_status(f"{description} completed in {elapsed:.2f}s", "success")
            return result

        return wrapper

    return decorator
