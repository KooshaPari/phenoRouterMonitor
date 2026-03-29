"""Fast subprocess execution with async support and optimizations.

This module provides optimized subprocess execution with:
- Async subprocess support for concurrent execution
- Optimized process creation and management
- Better resource usage for concurrent operations
- Platform-specific optimizations

Performance improvements:
- Async execution for concurrent subprocesses (non-blocking)
- Optimized process creation flags
- Better resource management
"""

import asyncio
import os
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import sys
import time
from pathlib import Path


def _record_history(
    cmd: list[str],
    cwd: str | None,
    exit_code: int,
    duration_s: float,
    task_id: str | None = None,
    agent_id: str | None = None,
) -> None:
    """Helper to record command execution in the context-aware history."""
    try:
        from thegent.infra.history import ContextHistory, HistoryEntry

        history = ContextHistory()
        entry = HistoryEntry(
            command=" ".join(cmd),
            cwd=cwd or os.getcwd(),
            exit_code=exit_code,
            duration_s=duration_s,
            task_id=task_id or os.environ.get("THEGENT_TASK_ID"),
            agent_id=agent_id or os.environ.get("THEGENT_AGENT_ID"),
        )
        history.record(entry)
    except Exception:
        # Never let history recording fail the main execution
        pass


def _validate_command_safety(cmd: list[str]) -> None:
    """Validate that command is not trying to kill agent processes.

    Raises:
        ValueError: If command attempts to kill agent processes
    """
    if not cmd:
        return

    cmd_str = " ".join(cmd).lower()

    # Check for kill commands targeting agent processes
    kill_patterns = [
        "kill",
        "pkill",
        "killall",
        "xargs.*kill",  # xargs kill patterns
    ]

    agent_patterns = [
        "cursor-agent",
        "cursor agent",
        "thegent",
        "claude",
        "codex",
        "droid",
        "opencode",
        "copilot",
    ]

    # Check for xargs kill patterns (e.g., "ps ... | grep ... | xargs kill")
    import re

    if re.search(r"xargs.*kill|kill.*-9|kill.*-KILL", cmd_str):
        for agent_pattern in agent_patterns:
            if agent_pattern in cmd_str:
                raise ValueError(
                    f"SECURITY BLOCKED: Command attempts to kill agent processes via xargs/kill: {' '.join(cmd)}\n"
                    f"Agents cannot kill other agent processes. Use 'thegent mcp prune' for safe cleanup."
                )

    # Check if command contains kill + agent pattern
    has_kill = any(pattern in cmd_str for pattern in kill_patterns)
    if has_kill:
        # Check if it's targeting agent processes
        for agent_pattern in agent_patterns:
            if agent_pattern in cmd_str:
                # Allow if it's explicitly excluding the current process or is a safe operation
                # But block general kill commands targeting agents
                if "grep -v" not in cmd_str and "exclude" not in cmd_str:
                    raise ValueError(
                        f"SECURITY BLOCKED: Command attempts to kill agent processes: {' '.join(cmd)}\n"
                        f"Agents cannot kill other agent processes. Use 'thegent mcp prune' for safe cleanup."
                    )


class FastSubprocess:
    """High-performance subprocess execution with async support."""

    @staticmethod
    async def run_async(
        cmd: list[str],
        *,
        cwd: Path | str | None = None,
        env: dict[str, str] | None = None,
        timeout: float | None = None,
        check: bool = False,
        capture_output: bool = True,
        **kwargs,
    ) -> subprocess.CompletedProcess:
        """Run subprocess asynchronously (non-blocking).

        Args:
            cmd: Command and arguments
            cwd: Working directory
            env: Environment variables
            timeout: Timeout in seconds
            check: Raise exception on non-zero exit
            capture_output: Capture stdout/stderr
            **kwargs: Additional subprocess options

        Returns:
            CompletedProcess with stdout, stderr, returncode

        Performance:
            - Non-blocking execution
            - Better for concurrent subprocess execution
            - Lower resource usage than blocking shim_run()
        """
        # Prepare environment
        process_env = os.environ.copy()
        if env:
            process_env.update(env)

        # Optimize process creation flags
        creation_flags = kwargs.pop("creationflags", 0)
        if sys.platform == "win32":
            # Windows: CREATE_NO_WINDOW to avoid console windows
            creation_flags |= 0x08000000  # CREATE_NO_WINDOW
        elif sys.platform != "win32":
            # Unix: Start new session for daemon processes
            # Use start_new_session=True for long-running processes
            if kwargs.pop("start_new_session", False):
                kwargs["preexec_fn"] = os.setsid

        # Set up stdout/stderr
        if capture_output:
            kwargs.setdefault("stdout", subprocess.PIPE)
            kwargs.setdefault("stderr", subprocess.PIPE)

        # SECURITY: Validate command safety before execution
        _validate_command_safety(cmd)

        # Apply guardrails validation
        try:
            from thegent.security.guardrails import validate_command

            is_allowed, error = validate_command(cmd, operation_type="command_execution")
            if not is_allowed:
                raise ValueError(f"Guardrails blocked: {error}")
        except ImportError:
            # Fallback if guardrails not available
            pass

        # Create async subprocess
        process = await asyncio.create_subprocess_exec(
            *cmd,
            cwd=str(cwd) if cwd else None,
            env=process_env,
            creationflags=creation_flags if sys.platform == "win32" else 0,
            **kwargs,
        )

        # Wait for completion with timeout
        start_time = time.time()
        try:
            stdout, stderr = await asyncio.wait_for(process.communicate(), timeout=timeout)
        except TimeoutError:
            process.kill()
            await process.wait()
            raise subprocess.TimeoutExpired(cmd, timeout if timeout is not None else 0.0)

        duration = time.time() - start_time

        # Decode output
        if stdout:
            stdout = stdout.decode("utf-8", errors="replace")
        if stderr:
            stderr = stderr.decode("utf-8", errors="replace")

        returncode = process.returncode if process.returncode is not None else 0
        result = subprocess.CompletedProcess(cmd, returncode, stdout or "", stderr or "")

        # WP-22001: Record in context-aware history
        _record_history(
            cmd=cmd,
            cwd=str(cwd) if cwd else None,
            exit_code=result.returncode,
            duration_s=duration,
        )

        if check and result.returncode != 0:
            raise subprocess.CalledProcessError(result.returncode, cmd, result.stdout, result.stderr)

        return result

    @staticmethod
    def run_optimized(
        cmd: list[str],
        *,
        cwd: Path | str | None = None,
        env: dict[str, str] | None = None,
        timeout: float | None = None,
        check: bool = False,
        capture_output: bool = True,
        start_new_session: bool = False,
        close_fds: bool = True,
        input: str | bytes | None = None,
        **kwargs,
    ) -> subprocess.CompletedProcess:
        """Run subprocess with optimizations (synchronous).

        Args:
            cmd: Command and arguments
            cwd: Working directory
            env: Environment variables
            timeout: Timeout in seconds
            check: Raise exception on non-zero exit
            capture_output: Capture stdout/stderr
            start_new_session: Start new session (Unix)
            close_fds: Close file descriptors (Unix)
            **kwargs: Additional subprocess options

        Returns:
            CompletedProcess with stdout, stderr, returncode

        Optimizations:
            - start_new_session for daemon processes
            - close_fds to prevent FD leaks
            - Optimized process creation flags
        """
        # Prepare environment
        process_env = os.environ.copy()
        if env:
            process_env.update(env)

        # Optimize process creation
        if sys.platform == "win32":
            creation_flags = kwargs.pop("creationflags", 0)
            creation_flags |= 0x08000000  # CREATE_NO_WINDOW
            kwargs["creationflags"] = creation_flags
        else:
            # Unix optimizations
            if start_new_session:
                kwargs["start_new_session"] = True
            if close_fds:
                kwargs["close_fds"] = True

        # Set up stdout/stderr
        if capture_output:
            kwargs.setdefault("stdout", subprocess.PIPE)
            kwargs.setdefault("stderr", subprocess.PIPE)

        # Handle input parameter (for stdin) - shim_run handles this automatically
        # Just ensure text parameter is set correctly if input is provided
        if "input" in kwargs and "text" not in kwargs:
            # Default to text=True if input is string, text=False if bytes
            kwargs["text"] = isinstance(kwargs["input"], str)

        # SECURITY: Validate command safety before execution
        _validate_command_safety(cmd)

        # Apply guardrails validation
        try:
            from thegent.security.guardrails import validate_command

            is_allowed, error = validate_command(cmd, operation_type="command_execution")
            if not is_allowed:
                raise ValueError(f"Guardrails blocked: {error}")
        except ImportError:
            # Fallback if guardrails not available
            pass

        start_time = time.time()
        result = shim_run(cmd, cwd=str(cwd) if cwd else None, env=process_env, timeout=timeout, check=check, **kwargs)
        duration = time.time() - start_time

        # WP-22001: Record in context-aware history
        _record_history(
            cmd=cmd,
            cwd=str(cwd) if cwd else None,
            exit_code=result.returncode,
            duration_s=duration,
        )

        return result

    @staticmethod
    async def run_concurrent(
        commands: list[list[str]], *, max_concurrent: int = 10, **kwargs
    ) -> list[subprocess.CompletedProcess]:
        """Run multiple subprocesses concurrently.

        Args:
            commands: List of command lists to execute
            max_concurrent: Maximum concurrent processes
            **kwargs: Options passed to run_async

        Returns:
            List of CompletedProcess results

        Performance:
            - Executes multiple processes concurrently
            - Limits concurrency to avoid resource exhaustion
            - Much faster than sequential execution
        """
        semaphore = asyncio.Semaphore(max_concurrent)

        async def run_with_semaphore(cmd: list[str]) -> subprocess.CompletedProcess:
            async with semaphore:
                return await FastSubprocess.run_async(cmd, **kwargs)

        tasks = [run_with_semaphore(cmd) for cmd in commands]
        return await asyncio.gather(*tasks)


# Convenience functions
async def run_subprocess_async(
    cmd: list[str],
    *,
    cwd: Path | str | None = None,
    env: dict[str, str] | None = None,
    timeout: float | None = None,
    check: bool = False,
    capture_output: bool = True,
    **kwargs,
) -> subprocess.CompletedProcess:
    """Run subprocess asynchronously."""
    return await FastSubprocess.run_async(
        cmd, cwd=cwd, env=env, timeout=timeout, check=check, capture_output=capture_output, **kwargs
    )


def run_subprocess_optimized(
    cmd: list[str],
    *,
    cwd: Path | str | None = None,
    env: dict[str, str] | None = None,
    timeout: float | None = None,
    check: bool = False,
    capture_output: bool = True,
    start_new_session: bool = False,
    input: str | bytes | None = None,
    text: bool = True,
    **kwargs,
) -> subprocess.CompletedProcess:
    """Run subprocess with optimizations.

    Args:
        input: Input to send to stdin (str or bytes)
        text: If True, input/output are text (str), else bytes
        **kwargs: Additional subprocess options
    """
    if input is not None:
        kwargs["input"] = input
    kwargs["text"] = text
    return FastSubprocess.run_optimized(
        cmd,
        cwd=cwd,
        env=env,
        timeout=timeout,
        check=check,
        capture_output=capture_output,
        start_new_session=start_new_session,
        **kwargs,
    )


async def run_subprocesses_concurrent(
    commands: list[list[str]], *, max_concurrent: int = 10, **kwargs
) -> list[subprocess.CompletedProcess]:
    """Run multiple subprocesses concurrently."""
    return await FastSubprocess.run_concurrent(commands, max_concurrent=max_concurrent, **kwargs)
