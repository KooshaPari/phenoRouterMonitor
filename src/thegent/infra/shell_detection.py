"""Shell detection and preferred shell strategy for cross-platform execution."""

import shutil
import sys
from enum import Enum


class ShellType(str, Enum):
    ZSH = "zsh"
    BASH = "bash"
    PWSH = "pwsh"
    POWERSHELL = "powershell"
    CMD = "cmd"
    DASH = "dash"
    UNKNOWN = "unknown"


def get_preferred_shell(performance: bool = False) -> ShellType:
    """Determine the best shell for the current platform and user configuration.

    If performance=True, prioritizes shells with the lowest startup overhead (dash/cmd).
    """
    # 1. Check settings override
    from thegent.config import ThegentSettings

    settings = ThegentSettings()
    if settings.agent_shell:
        try:
            return ShellType(settings.agent_shell.lower())
        except ValueError:
            pass

    # 2. Platform-specific defaults
    if sys.platform == "win32":
        if performance:
            return ShellType.CMD
        if shutil.which("pwsh"):
            return ShellType.PWSH
        return ShellType.POWERSHELL

    # 3. macOS/Linux defaults
    # For performance, dash is the clear winner (<10ms vs 20-50ms for bash/zsh)
    shells = ["dash", "bash", "zsh"] if performance else ["zsh", "bash", "dash"]

    for shell in shells:
        if shutil.which(shell):
            return ShellType(shell)

    return ShellType.UNKNOWN


def get_shell_executable(shell_type: ShellType) -> str:
    """Get the full path to the shell executable."""
    executable = shutil.which(shell_type.value)
    if executable:
        return executable

    # Fallbacks for Windows
    if shell_type == ShellType.POWERSHELL:
        return "powershell.exe"
    if shell_type == ShellType.PWSH:
        return "pwsh.exe"
    if shell_type == ShellType.CMD:
        return "cmd.exe"

    return shell_type.value


def get_fast_command_prefix(shell_type: ShellType) -> list[str]:
    """Get the command prefix for the fastest execution on the given shell type."""
    exe = get_shell_executable(shell_type)
    if shell_type == ShellType.CMD:
        return [exe, "/c"]
    if shell_type in (ShellType.DASH, ShellType.BASH, ShellType.ZSH):
        return [exe, "-c"]
    return [exe, "-c"]  # Default fallback
