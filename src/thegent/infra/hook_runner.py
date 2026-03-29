"""Hook runner with shell detection and cross-platform support."""

import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import sys
from pathlib import Path

from thegent.config import get_settings
from thegent.infra.shell_detection import ShellType, get_preferred_shell, get_shell_executable


def _resolve_shell_type() -> ShellType:
    """Resolve shell type from settings or platform-preferred defaults."""
    settings = get_settings()
    if settings.hook_shell:
        return ShellType(settings.hook_shell.lower())
    return get_preferred_shell(performance=True)


def _build_hook_command(shell_exe: str, shell_type: ShellType, hook_path: Path) -> list[str]:
    """Build hook execution command for the selected shell."""
    if shell_type in [ShellType.PWSH, ShellType.POWERSHELL]:
        return [shell_exe, "-NoProfile", "-NonInteractive", "-File", str(hook_path)]
    return [shell_exe, str(hook_path)]


def _normalize_stream_text(stream: str | bytes | None) -> str:
    """Normalize subprocess output streams to text."""
    if stream is None:
        return ""
    if isinstance(stream, bytes):
        return stream.decode()
    return stream


def run_hook(hook_path: Path, input_data: str | None = None, timeout: int = 60) -> subprocess.CompletedProcess:
    """Run a hook script using the preferred shell."""
    # 1. Determine shell to use
    shell_type = _resolve_shell_type()
    shell_exe = get_shell_executable(shell_type)

    # 2. Build command
    cmd = _build_hook_command(shell_exe, shell_type, hook_path)

    # 4. Execute
    try:
        result = shim_run(cmd, input=input_data, capture_output=True, text=True, timeout=timeout, check=False)
        return result
    except subprocess.TimeoutExpired as e:
        return subprocess.CompletedProcess(
            args=cmd,
            returncode=124,
            stdout=_normalize_stream_text(e.stdout),
            stderr=f"Hook timed out after {timeout}s\n{_normalize_stream_text(e.stderr)}",
        )
    except Exception as e:
        return subprocess.CompletedProcess(args=cmd, returncode=1, stdout="", stderr=f"Failed to run hook: {e}")


def main():
    """CLI entry point for running a hook."""
    if len(sys.argv) < 2:
        sys.exit(1)

    hook_path = Path(sys.argv[1])
    input_data = sys.stdin.read() if not sys.stdin.isatty() else None

    result = run_hook(hook_path, input_data)

    sys.stdout.write(result.stdout)
    sys.stderr.write(result.stderr)
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
