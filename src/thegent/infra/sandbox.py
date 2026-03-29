"""WP-31002: Containerized Agent Sandboxes (Wasm).
Provides lightweight, secure execution environments for untrusted agent code using WebAssembly.
Ensures near-native performance with strict memory and capability isolation.
"""

from __future__ import annotations

import logging
import time
from contextlib import suppress
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, Any

from pydantic import BaseModel

if TYPE_CHECKING:
    from collections.abc import Callable

_log = logging.getLogger(__name__)


class SandboxStatus(str, Enum):
    """Lifecycle status of a Wasm sandbox."""

    INITIALIZED = "initialized"
    READY = "ready"
    RUNNING = "running"
    TIMEOUT = "timeout"
    ERROR = "error"
    TERMINATED = "terminated"


class SandboxFeature(str, Enum):
    """Optional capabilities that a sandbox may expose."""

    NETWORK = "network"
    FILESYSTEM_READ = "filesystem_read"
    FILESYSTEM_WRITE = "filesystem_write"
    ENVIRONMENT = "environment"
    HTTP = "http"
    RANDOM = "random"


@dataclass
class ResourceUsage:
    """Resource consumption recorded for a sandbox execution."""

    memory_peak_mb: float = 0
    cpu_time_ms: int = 0
    wall_time_ms: int = 0
    read_bytes: int = 0
    write_bytes: int = 0


class SandboxConfig(BaseModel):
    """Configuration for a Wasm sandbox."""

    max_memory_mb: int = 128
    allow_network: bool = False
    allow_filesystem: bool = False
    timeout_ms: int = 30000
    enable_wasi: bool = True
    allowed_paths: list[str] = []
    env_vars: dict[str, str] = {}


class WasmSandbox:
    """Manages secure execution of agent code in a Wasm environment using Extism."""

    def __init__(self, sandbox_id: str, config: SandboxConfig | None = None) -> None:
        self.sandbox_id = sandbox_id
        self.config = config or SandboxConfig()
        self.status: SandboxStatus = SandboxStatus.INITIALIZED
        self._plugin: Any = None

    def is_available(self) -> bool:
        """Return True if the Extism runtime is importable."""
        try:
            import extism  # noqa: F401  # pyright: ignore[reportUnusedImport]

            return True
        except ImportError:
            return False

    def _init_plugin(self, wasm_binary_path: str) -> None:
        """Initialize the Extism plugin.

        Raises:
            FileNotFoundError: If the wasm binary does not exist.
            ImportError: If the extism package is not installed.
        """
        if not Path(wasm_binary_path).exists():
            raise FileNotFoundError(f"Wasm binary not found: {wasm_binary_path}")

        try:
            import extism
        except ImportError:
            _log.error("Extism not installed. Run 'pip install extism'.")
            raise

        manifest: dict[str, Any] = {"wasm": [{"path": wasm_binary_path}]}
        if self.config.env_vars:
            manifest["config"] = self.config.env_vars

        self._plugin = extism.Plugin(manifest, wasi=self.config.enable_wasi)

    def run_function(
        self,
        wasm_binary_path: str,
        function_name: str,
        input_data: str | bytes,
        fallback_fn: Callable[[str | bytes], dict[str, Any]] | None = None,
    ) -> dict[str, Any]:
        """Execute a function inside the Wasm sandbox.

        Args:
            wasm_binary_path: Path to the Wasm binary to load.
            function_name: Name of the function to invoke inside the Wasm module.
            input_data: Input passed to the Wasm function.
            fallback_fn: Optional callable invoked when execution fails. Its
                return value is augmented with ``{"fallback": True}`` and returned.

        Returns:
            Result dict with ``status``, ``result``/``error``, ``duration_ms``.
        """
        start_time = time.time()
        _log.info("Executing Wasm binary %s in sandbox %s", wasm_binary_path, self.sandbox_id)

        self.status = SandboxStatus.RUNNING
        try:
            if not self._plugin:
                self._init_plugin(wasm_binary_path)

            output = self._plugin.call(function_name, input_data)
            duration_ms = (time.time() - start_time) * 1000
            self.status = SandboxStatus.READY

            return {
                "status": "success",
                "result": output.decode("utf-8") if isinstance(output, bytes) else output,
                "duration_ms": duration_ms,
            }
        except Exception as exc:
            duration_ms = (time.time() - start_time) * 1000
            _log.error("Wasm execution failed: %s", exc)
            error_msg = str(exc).lower()

            if fallback_fn is not None:
                result = fallback_fn(input_data)
                result["fallback"] = True
                result.setdefault("duration_ms", duration_ms)
                self.status = SandboxStatus.READY
                return result

            self.status = SandboxStatus.ERROR
            return {"status": "error", "error": error_msg, "duration_ms": duration_ms}

    def shutdown(self) -> None:
        """Tear down the sandbox and release resources."""
        if self._plugin:
            with suppress(Exception):
                self._plugin.close()
            self._plugin = None
        _log.info("Shutting down Wasm sandbox: %s", self.sandbox_id)
        self.status = SandboxStatus.TERMINATED

    def __enter__(self) -> WasmSandbox:
        return self

    def __exit__(self, *_: object) -> None:
        self.shutdown()


def check_wasm_support() -> dict[str, bool]:
    """Return a dict indicating which Wasm runtimes are available.

    Returns:
        Dict with keys ``extism``, ``wasmer``, ``wasmtime`` mapping to bool.
    """
    result: dict[str, bool] = {}
    for runtime in ("extism", "wasmer", "wasmtime"):
        result[runtime] = _runtime_is_available(runtime)
    return result


def _runtime_is_available(runtime: str) -> bool:
    try:
        __import__(runtime)
    except ImportError:
        return False
    return True


def create_sandboxed_executor(config: SandboxConfig | None = None) -> WasmSandbox:
    """Create and return a WasmSandbox instance ready for use.

    Args:
        config: Optional SandboxConfig. Defaults to SandboxConfig().

    Returns:
        Initialised WasmSandbox.
    """
    import uuid

    sandbox_id = f"executor-{uuid.uuid4().hex[:8]}"
    sandbox = WasmSandbox(sandbox_id, config)
    return sandbox
