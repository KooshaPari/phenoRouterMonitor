"""CLI UX improvements: command suggestions, interactive prompts, and better help.

This module provides utilities for improving the command-line user experience
with suggestions, better formatting, and interactive elements.
"""

import difflib

from rich.console import Console
from rich.panel import Panel
from rich.table import Table

console = Console()

# All thegent commands for suggestion
THGENT_COMMANDS = [
    "run",
    "bg",
    "ps",
    "plan",
    "doctor",
    "setup",
    "config",
    "status",
    "logs",
    "stop",
    "resume",
    "retry",
    "purge",
    "sweep",
    "takeover",
    "history",
    "inspect",
    "feedback",
    "explorer",
    "modes",
    "operations",
    "drift",
    "audit-verify",
    "compliance-report",
    "cost-status",
    "data-protection",
    "deep-research",
    "discovery-parse",
    "discovery-register",
    "discovery-scan",
    "forensics-snapshot",
    "govern-configure",
    "govern-go-cycle",
    "govern-go-health",
    "govern-go-status",
    "govern-go-watch",
    "handoff-list",
    "handoff-show",
    "interruption-list",
    "interruption-snooze",
    "list-agents",
    "list-droids",
    "list-models",
    "install",
    "scaffold",
    "load-status",
    "migration",
    "policy-show",
    "project",
    "project init",
    "project list",
    "resolve-model-route",
    "rules-sync",
    "session-contract-health-gate",
    "session-contract-health-report",
    "session-contract-health-trend",
    "sitback-dashboard",
    "summary",
    "terminal-route",
    "usage",
    "workstream-dashboard",
    "workstream-query",
    "workstream-stats",
]


def suggest_command(command: str, commands: list[str] | None = None) -> list[str]:
    """Suggest similar commands for a typo.

    Args:
        command: The command that was not found
        commands: List of available commands (default: THGENT_COMMANDS)

    Returns:
        List of suggested commands, sorted by similarity
    """
    if commands is None:
        commands = THGENT_COMMANDS

    # Use difflib to find similar commands
    suggestions = difflib.get_close_matches(command, commands, n=3, cutoff=0.6)
    return suggestions


def display_command_suggestion(command: str, suggestions: list[str]) -> None:
    """Display command suggestions in a helpful format.

    Args:
        command: The command that was not found
        suggestions: List of suggested commands
    """
    if not suggestions:
        return

    console.print(f"\n[yellow]Command '{command}' not found.[/yellow]")

    if len(suggestions) == 1:
        console.print(f"[bold]Did you mean:[/bold] [cyan]{suggestions[0]}[/cyan]")
    else:
        console.print("[bold]Did you mean one of:[/bold]")
        for suggestion in suggestions:
            console.print(f"  • [cyan]{suggestion}[/cyan]")


def format_command_help(command: str, description: str, examples: list[str] | None = None) -> str:
    """Format command help with examples.

    Args:
        command: Command name
        description: Command description
        examples: List of example usage strings

    Returns:
        Formatted help string
    """
    help_text = f"[bold cyan]{command}[/bold cyan]\n\n{description}"

    if examples:
        help_text += "\n\n[bold]Examples:[/bold]"
        for example in examples:
            help_text += f"\n  [dim]$[/dim] {example}"

    return help_text


def interactive_confirm(message: str, default: bool = True) -> bool:
    """Interactive confirmation prompt with better formatting.

    Args:
        message: Confirmation message
        default: Default value

    Returns:
        True if confirmed, False otherwise
    """
    from rich.prompt import Confirm

    return Confirm.ask(f"[bold]{message}[/bold]", default=default)


def display_command_examples(command: str, examples: list[dict[str, str]]) -> None:
    """Display command examples in a table.

    Args:
        command: Command name
        examples: List of example dicts with 'description' and 'command' keys
    """
    table = Table(title=f"{command} Examples", show_header=True, header_style="bold cyan")
    table.add_column("Description", style="yellow")
    table.add_column("Command", style="green")

    for example in examples:
        table.add_row(example.get("description", ""), example.get("command", ""))

    console.print(table)


def format_error_with_suggestion(error: Exception, command: str | None = None) -> None:
    """Format an error with command suggestions if applicable.

    Args:
        error: The error that occurred
        command: The command that caused the error (if applicable)
    """
    from thegent.infra.enhanced_errors import format_error_with_context

    # If it's a command not found error, add suggestions
    if command and "not found" in str(error).lower():
        suggestions = suggest_command(command)
        if suggestions:
            display_command_suggestion(command, suggestions)

    format_error_with_context(error)


def print_command_header(command: str, description: str) -> None:
    """Print a formatted command header.

    Args:
        command: Command name
        description: Command description
    """
    console.print(Panel(f"[bold cyan]{command}[/bold cyan]\n[dim]{description}[/dim]", border_style="cyan"))


def print_section_header(title: str) -> None:
    """Print a formatted section header.

    Args:
        title: Section title
    """
    console.print(f"\n[bold cyan]{title}[/bold cyan]")
    console.print("[dim]" + "─" * len(title) + "[/dim]\n")
