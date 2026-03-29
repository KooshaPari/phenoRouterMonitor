"""Mojo Bridge - Python to Mojo Interoperability.

This module provides a bridge for calling compiled Mojo modules from Python.
It supports subprocess-based execution of compiled Mojo binaries with JSON I/O.

Note: Mojo ecosystem is still maturing. This bridge uses subprocess to call
compiled Mojo binaries, with future support for C-ABI integration when stable.
"""

import asyncio
import contextlib
import importlib
import inspect
import json
import logging
import os
import platform
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from thegent.infra.cache_v2 import CacheV2

logger = logging.getLogger(__name__)


class MojoNotAvailableError(Exception):
    """Raised when Mojo is not installed or not accessible."""


@dataclass
class MojoModule:
    """Represents a compiled Mojo module."""

    name: str
    path: Path
    compiled: bool = False


@dataclass
class MojoTask:
    """Task to be executed in Mojo."""

    task_id: str
    module: str
    function: str
    args: dict[str, Any]
    timeout: float = 30.0


@dataclass(frozen=True)
class MojoKernelContract:
    """Contract for deterministic Mojo kernel invocations (WL-133 slice)."""

    module: str
    function: str
    required_args: tuple[str, ...]


MOJO_KERNEL_CONTRACTS: dict[tuple[str, str], MojoKernelContract] = {
    ("math", "calculate_provider_score"): MojoKernelContract(
        module="math",
        function="calculate_provider_score",
        required_args=("cost_score", "quality_score", "latency_score"),
    )
}


def validate_kernel_contract(module: str, function: str, args: dict[str, Any]) -> None:
    """Validate required arguments for a known kernel contract."""
    contract = MOJO_KERNEL_CONTRACTS.get((module, function))
    if contract is None:
        return
    missing = [key for key in contract.required_args if key not in args]
    if missing:
        raise ValueError(f"Missing required args for {module}.{function}: {', '.join(missing)}")


def build_provider_score_kernel_script() -> str:
    """Build deterministic provider score kernel script text."""
    return """
from python import Python
from math import floor

fn main():
    let env = Python.import_module("os").environ
    let raw_args = env.get("THEGENT_MOJO_ARGS", "{}")
    let json_mod = Python.import_module("json")
    let data = json_mod.loads(raw_args)

    let cost = Float64(data.get("cost_score", 0.0))
    let quality = Float64(data.get("quality_score", 0.0))
    let latency = Float64(data.get("latency_score", 0.0))

    # Deterministic weighted sum with bounded output.
    let score = (0.4 * quality) + (0.35 * cost) + (0.25 * latency)
    let bounded = max(0.0, min(1.0, score))
    let result = {"score": floor(bounded * 1000.0) / 1000.0, "success": True}
    print(json_mod.dumps(result))
"""


def build_python_dispatch_kernel_script(module: str, function: str) -> str:
    """Build a Mojo script that dispatches to a Python module/function target."""
    module_json = json.dumps(module)
    function_json = json.dumps(function)
    return f"""
from python import Python

fn main():
    let env = Python.import_module("os").environ
    let json_mod = Python.import_module("json")
    let inspect_mod = Python.import_module("inspect")
    let raw_args = env.get("THEGENT_MOJO_ARGS", "{{}}")
    let args = json_mod.loads(raw_args)
    let target_module = Python.import_module({module_json})
    let target_fn = getattr(target_module, {function_json})
    let result = target_fn(**args)
    if inspect_mod.iscoroutine(result):
        result = Python.import_module("asyncio").run(result)
    print(json_mod.dumps({{"success": True, "result": result}}))
"""


def build_dispatch_script(task: MojoTask) -> str:
    """Build a dispatch script for the requested task target."""
    if not isinstance(task.args, dict):
        raise ValueError(f"Malformed args payload for {task.module}.{task.function}: expected object")

    contract_key = (task.module, task.function)
    if contract_key in MOJO_KERNEL_CONTRACTS:
        validate_kernel_contract(task.module, task.function, task.args)
        return build_provider_score_kernel_script()

    try:
        module_obj = importlib.import_module(task.module)
    except ModuleNotFoundError as exc:
        raise ValueError(f"Unknown module '{task.module}' for Mojo dispatch.") from exc

    target_fn = getattr(module_obj, task.function, None)
    if target_fn is None or not callable(target_fn):
        raise ValueError(f"Unknown function '{task.function}' in module '{task.module}'.")

    try:
        inspect.signature(target_fn).bind(**task.args)
    except TypeError as exc:
        raise ValueError(f"Signature mismatch for {task.module}.{task.function}: {exc}") from exc

    return build_python_dispatch_kernel_script(task.module, task.function)


class MojoBridge:
    """Bridge for calling Mojo modules from Python.

    Supports:
    - Subprocess-based execution of compiled Mojo binaries
    - Graceful fallback when Mojo is not available
    - JSON-based I/O for data exchange
    - Async execution for better integration

    Future:
    - C-ABI integration when Mojo's foreign function interface stabilizes
    - Direct memory sharing for high-performance scenarios
    """

    def __init__(
        self,
        mojo_root: Path | None = None,
        cache_root: Path | None = None,
    ) -> None:
        """Initialize the Mojo bridge.

        Args:
            mojo_root: Root directory for Mojo modules (default: ~/.thegent/mojo)
            cache_root: Root directory for cache (default: /tmp/thegent-mojo-cache)
        """
        self.mojo_root = mojo_root or Path.home() / ".thegent" / "mojo"
        self.cache_root = cache_root or (Path(tempfile.gettempdir()) / "thegent-mojo-cache")

        # Ensure directories exist
        self.mojo_root.mkdir(parents=True, exist_ok=True)
        self.cache_root.mkdir(parents=True, exist_ok=True)

        # Initialize cache
        self._cache = CacheV2(self.cache_root, namespace="mojo_bridge")

        # Track Mojo availability
        self._mojo_path: Path | None = None
        self._is_available: bool | None = None

        # Platform-specific settings
        is_mac = platform.system() == "Darwin"
        self.default_timeout = 60.0 if is_mac else 30.0

    @property
    def is_available(self) -> bool:
        """Check if Mojo is installed and available."""
        if self._is_available is not None:
            return self._is_available

        self._is_available = self._check_mojo()
        return self._is_available

    def _check_mojo(self) -> bool:
        """Check if Mojo is installed on the system.

        Checks:
        1. Direct `mojo` command in PATH
        2. Modular CLI in common locations
        3. Mojo SDK in standard installation paths
        """
        # Check for mojo command
        mojo_cmd = shutil.which("mojo")
        if mojo_cmd:
            self._mojo_path = Path(mojo_cmd)
            return True

        # Check for modular CLI (Mojo's package manager)
        modular_cmd = shutil.which("modular")
        if modular_cmd:
            # Try to get mojo version through modular
            try:
                _result = shim_run(
                    ["modular", "auth"],
                    capture_output=True,
                    timeout=5,
                    check=False,
                )
                # If modular is set up, mojo should be available
                return True
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass

        # Check common installation paths on macOS
        if platform.system() == "Darwin":
            common_paths = [
                Path.home() / ".modular" / "pkg" / "mojo-nightly" / "bin" / "mojo",
                Path.home() / ".modular" / "pkg" / "mojo" / "bin" / "mojo",
            ]
            for path in common_paths:
                if path.exists():
                    self._mojo_path = path
                    return True

        # Check common installation paths on Linux
        if platform.system() == "Linux":
            common_paths = [
                Path.home() / ".modular" / "pkg" / "mojo-nightly" / "bin" / "mojo",
                Path.home() / ".modular" / "pkg" / "mojo" / "bin" / "mojo",
                Path("/opt/mojo/bin/mojo"),
            ]
            for path in common_paths:
                if path.exists():
                    self._mojo_path = path
                    return True

        return False

    async def get_version(self) -> str | None:
        """Get the Mojo version if available.

        Returns:
            Version string or None if not available
        """
        if not self.is_available:
            return None

        try:
            cmd = "mojo" if self._mojo_path is None else str(self._mojo_path)
            process = await asyncio.create_subprocess_exec(
                cmd,
                "--version",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            stdout, _stderr = await asyncio.wait_for(process.communicate(), timeout=10.0)
            if process.returncode == 0:
                return stdout.decode().strip()
        except (TimeoutError, FileNotFoundError, subprocess.SubprocessError):
            pass

        return None

    async def run_mojo_script(
        self,
        script: str,
        args: dict[str, Any] | None = None,
        timeout: float | None = None,
    ) -> dict[str, Any]:
        """Run a Mojo script with JSON input/output.

        Args:
            script: Mojo source code or path to .mojo file
            args: Arguments to pass to the Mojo script
            timeout: Timeout in seconds

        Returns:
            JSON response from the Mojo script
        """
        if not self.is_available:
            raise MojoNotAvailableError(
                "Mojo is not installed. Install via: brew install mojo (macOS) or see https://docs.modular.com/mojo"
            )

        timeout = timeout or self.default_timeout
        args = args or {}

        # Create temporary file for the script if it's inline code
        with tempfile.NamedTemporaryFile(mode="w", suffix=".mojo", delete=False) as f:
            f.write(script)
            script_path = f.name

        try:
            # Run mojo with JSON args
            cmd = "mojo" if self._mojo_path is None else str(self._mojo_path)

            # Pass args as JSON environment variable
            env = os.environ.copy()
            env["THEGENT_MOJO_ARGS"] = json.dumps(args)

            process = await asyncio.create_subprocess_exec(
                cmd,
                script_path,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
            )

            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=timeout,
            )

            if process.returncode != 0:
                error_msg = stderr.decode() if stderr else "Unknown error"
                raise RuntimeError(f"Mojo script failed: {error_msg}")

            output = stdout.decode().strip()
            if output:
                try:
                    return json.loads(output)
                except json.JSONDecodeError:
                    return {"output": output}
            return {}

        finally:
            # Clean up temp file
            with contextlib.suppress(OSError):
                Path(script_path).unlink()

    async def run_compiled(
        self,
        binary_path: Path,
        input_data: dict[str, Any] | None = None,
        timeout: float | None = None,
    ) -> dict[str, Any]:
        """Run a compiled Mojo binary.

        Args:
            binary_path: Path to the compiled Mojo binary
            input_data: JSON input to pass to the binary
            timeout: Timeout in seconds

        Returns:
            JSON response from the binary
        """
        if not self.is_available:
            raise MojoNotAvailableError("Mojo is not installed")

        timeout = timeout or self.default_timeout
        input_data = input_data or {}

        # Write input to temp file
        with tempfile.NamedTemporaryFile(
            mode="w",
            suffix=".json",
            delete=False,
        ) as f:
            json.dump(input_data, f)
            input_path = f.name

        try:
            process = await asyncio.create_subprocess_exec(
                str(binary_path),
                input_path,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=timeout,
            )

            if process.returncode != 0:
                error_msg = stderr.decode() if stderr else "Unknown error"
                raise RuntimeError(f"Mojo binary failed: {error_msg}")

            output = stdout.decode().strip()
            if output:
                try:
                    return json.loads(output)
                except json.JSONDecodeError:
                    return {"output": output}
            return {}

        finally:
            with contextlib.suppress(OSError):
                Path(input_path).unlink()

    def install_instructions(self) -> str:
        """Get installation instructions for Mojo.

        Returns:
            Installation instructions as a string
        """
        if platform.system() == "Darwin":
            return """# Install Mojo on macOS

Using Homebrew:
    brew install mojo

Or manually:
    1. Visit https://docs.modular.com/mojo/manual/install/
    2. Download the Mojo SDK for your platform
    3. Run the installer and follow the instructions
"""
        if platform.system() == "Linux":
            return """# Install Mojo on Linux

    1. Visit https://docs.modular.com/mojo/manual/install/
    2. Download the Mojo SDK for Linux
    3. Run the installer and follow the instructions
"""
        return """# Install Mojo

Visit https://docs.modular.com/mojo/manual/install/ for platform-specific instructions.
"""

    async def dispatch(self, task: MojoTask) -> dict[str, Any]:
        """Dispatch a task to Mojo for execution.

        Args:
            task: The Mojo task to execute

        Returns:
            Result from the Mojo execution
        """
        # For now, we execute via subprocess
        # Future: use C-ABI when stable

        # Check cache first
        cache_key = f"mojo_task:{task.module}:{task.function}:{json.dumps(task.args, sort_keys=True)}"
        cached = await self._cache.get(cache_key)
        if cached:
            return cached

        if not self.is_available:
            return {
                "error": "mojo_not_available",
                "message": "Mojo is not installed. " + self.install_instructions(),
                "task_id": task.task_id,
            }

        mojo_code = build_dispatch_script(task)

        try:
            result = await self.run_mojo_script(mojo_code, task.args, task.timeout)

            # Cache the result
            await self._cache.set(cache_key, result, ttl=3600)

            return result

        except MojoNotAvailableError:
            # Return gracefully when mojo is not available
            return {
                "error": "mojo_not_available",
                "message": "Mojo is not installed. " + self.install_instructions(),
                "task_id": task.task_id,
            }

    async def compile_module(
        self,
        source_path: Path,
        output_path: Path | None = None,
    ) -> Path | None:
        """Compile a Mojo source file to a binary.

        Note: This requires the full Mojo SDK to be installed.

        Args:
            source_path: Path to .mojo source file
            output_path: Path for the compiled binary (default: same name without .mojo)

        Returns:
            Path to the compiled binary, or None if compilation failed
        """
        if not self.is_available:
            return None

        output_path = output_path or source_path.with_name(source_path.stem)

        try:
            cmd = "mojo" if self._mojo_path is None else str(self._mojo_path)
            process = await asyncio.create_subprocess_exec(
                cmd,
                "compile",
                str(source_path),
                str(output_path),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            _stdout, _stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=120.0,
            )

            if process.returncode == 0:
                return output_path
            return None

        except TimeoutError:
            return None
        except Exception:
            return None

    async def shutdown(self) -> None:
        """Clean up resources."""
        logger.debug("Shutting down MojoBridge cache at %s", self.cache_root)
        await self._cache.clear_expired()
        await self._cache.clear()


# Global bridge instance
_bridge: MojoBridge | None = None


def get_bridge() -> MojoBridge:
    """Get the global Mojo bridge instance."""
    global _bridge
    if _bridge is None:
        _bridge = MojoBridge()
    return _bridge


async def check_mojo_status() -> dict[str, Any]:
    """Check Mojo installation status.

    Returns:
        Dictionary with availability and version info
    """
    bridge = get_bridge()

    result = {
        "available": bridge.is_available,
        "version": None,
        "install_instructions": None,
    }

    if bridge.is_available:
        result["version"] = await bridge.get_version()
    else:
        result["install_instructions"] = bridge.install_instructions()

    return result


# Example usage when run directly
if __name__ == "__main__":

    async def main():
        bridge = get_bridge()
        status = await check_mojo_status()

        if not status["available"]:
            pass

        # Try a simple test if available
        if status["available"]:
            test_task = MojoTask(
                task_id="test_001",
                module="test",
                function="hello",
                args={"name": "thegent"},
            )
            _result = await bridge.dispatch(test_task)

    asyncio.run(main())
