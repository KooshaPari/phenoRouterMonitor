"""WP-31002: Wasm Plugin System using Extism.

Provides a comprehensive Wasm-based plugin system for sandboxed tool execution.
This module integrates with Extism to enable secure, isolated execution of
Zig-compiled Wasm tools.

Key features:
- Extism Python bindings integration
- Plugin loading and execution
- Resource limits (memory, CPU time)
- Graceful fallback if Wasm not available
"""

import orjson as json
import logging
import threading
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)


class WasmRuntimeStatus(Enum):
    """Status of the Wasm runtime."""

    UNINITIALIZED = "uninitialized"
    AVAILABLE = "available"
    UNAVAILABLE = "unavailable"
    ERROR = "error"


class PluginStatus(Enum):
    """Status of a loaded plugin."""

    UNLOADED = "unloaded"
    LOADING = "loading"
    READY = "ready"
    RUNNING = "running"
    ERROR = "error"
    TERMINATED = "terminated"


class WasmCapability(Enum):
    """Capabilities that can be granted to a Wasm plugin."""

    HTTP_CLIENT = "http_client"
    FILE_READ = "file_read"
    FILE_WRITE = "file_write"
    ENVIRONMENT = "environment"
    CLOCK = "clock"
    RANDOM = "random"


@dataclass
class ResourceLimits:
    """Resource limits for Wasm execution.

    These limits help ensure that Wasm plugins cannot consume
    excessive resources or run indefinitely.
    """

    max_memory_mb: int = 128
    max_cpu_time_ms: int = 5000
    max_execution_time_ms: int = 30000
    max_call_stack_depth: int = 64
    max_output_size_bytes: int = 1024 * 1024  # 1MB


@dataclass
class WasmPluginMetadata:
    """Metadata for a Wasm plugin."""

    name: str
    version: str = "0.1.0"
    description: str = ""
    author: str = ""
    capabilities: list[WasmCapability] = field(default_factory=list)
    entry_point: str = "run"


@dataclass
class WasmExecutionResult:
    """Result of Wasm plugin execution."""

    status: str
    output: str | None = None
    error: str | None = None
    duration_ms: float = 0
    memory_used_mb: float = 0
    cpu_time_ms: float = 0


class ExtismRuntime:
    """Extism runtime wrapper with resource management."""

    _instance: "ExtismRuntime | None" = None
    _lock = threading.Lock()

    def __new__(cls) -> "ExtismRuntime":
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    instance = super().__new__(cls)
                    # Initialize attributes before __init__ is called
                    instance.extism = None
                    instance.status_value = WasmRuntimeStatus.UNINITIALIZED
                    instance.error_message_value = None
                    instance.initialized = True
                    cls._instance = instance
        return cls._instance

    def __init__(self) -> None:
        # Skip re-initialization for singleton
        if hasattr(self, "initialized") and self.initialized:
            return
        self.extism = None
        self.status_value = WasmRuntimeStatus.UNINITIALIZED
        self.error_message_value = None
        self.initialized = True

    @property
    def status(self) -> WasmRuntimeStatus:
        """Get the current runtime status."""
        return self.status_value

    @property
    def is_available(self) -> bool:
        """Check if Extism is available."""
        return self.status_value == WasmRuntimeStatus.AVAILABLE

    @property
    def error_message(self) -> str | None:
        """Get the error message if status is ERROR."""
        return self.error_message_value

    def initialize(self) -> bool:
        """Initialize the Extism runtime.

        Returns:
            True if initialization was successful, False otherwise.
        """
        if self.status_value == WasmRuntimeStatus.AVAILABLE:
            return True

        try:
            import extism  # type: ignore[import-untyped]

            self.extism = extism
            self.status_value = WasmRuntimeStatus.AVAILABLE
            _log.info("Extism runtime initialized successfully")
            return True
        except ImportError as e:
            self.status_value = WasmRuntimeStatus.UNAVAILABLE
            self.error_message_value = f"Extism not installed: {e}"
            _log.warning("Extism not available: %s", e)
            return False
        except Exception as e:
            self.status_value = WasmRuntimeStatus.ERROR
            self.error_message_value = f"Failed to initialize Extism: {e}"
            _log.error("Extism initialization failed: %s", e)
            return False

    def get_extism(self) -> Any | None:
        """Get the Extism module.

        Returns:
            The Extism module if available, None otherwise.
        """
        if self.status_value != WasmRuntimeStatus.AVAILABLE:
            self.initialize()
        return self.extism


class WasmPlugin(ABC):
    """Abstract base class for Wasm plugins."""

    def __init__(
        self,
        plugin_path: Path,
        metadata: WasmPluginMetadata,
        limits: ResourceLimits | None = None,
        config: dict[str, str] | None = None,
    ) -> None:
        self.plugin_path = plugin_path
        self.metadata = metadata
        self.limits = limits or ResourceLimits()
        self.config = config or {}
        self._plugin: Any = None
        self._status = PluginStatus.UNLOADED
        self._lock = threading.Lock()

    @property
    def status(self) -> PluginStatus:
        """Get the current plugin status."""
        return self._status

    @abstractmethod
    def load(self) -> bool:
        """Load the plugin into the runtime.

        Returns:
            True if loading was successful, False otherwise.
        """

    @abstractmethod
    def execute(self, input_data: str | bytes) -> WasmExecutionResult:
        """Execute the plugin with the given input.

        Args:
            input_data: Input data to pass to the plugin.

        Returns:
            Execution result containing output or error.
        """

    @abstractmethod
    def unload(self) -> bool:
        """Unload the plugin and release resources.

        Returns:
            True if unloading was successful, False otherwise.
        """


class ExtismPlugin(WasmPlugin):
    """Extism-based Wasm plugin implementation."""

    def __init__(
        self,
        plugin_path: Path,
        metadata: WasmPluginMetadata,
        limits: ResourceLimits | None = None,
        config: dict[str, str] | None = None,
        allow_wasi: bool = True,
    ) -> None:
        super().__init__(plugin_path, metadata, limits, config)
        self._runtime = ExtismRuntime()
        self._allow_wasi = allow_wasi

    def load(self) -> bool:
        """Load the plugin into the Extism runtime."""
        with self._lock:
            if self._status == PluginStatus.READY:
                return True

            if not self._runtime.is_available:
                _log.error("Cannot load plugin: Extism runtime not available")
                self._status = PluginStatus.ERROR
                return False

            self._status = PluginStatus.LOADING
            try:
                extism = self._runtime.get_extism()
                if extism is None:
                    raise RuntimeError("Extism not available")

                manifest: dict[str, Any] = {"wasm": [{"path": str(self.plugin_path)}]}

                # Add config if provided
                if self.config:
                    manifest["config"] = self.config

                # Configure WASI
                if self._allow_wasi:
                    # Extism enables WASI by default when wasi=True
                    # Additional WASI config can be added here if needed
                    pass

                self._plugin = extism.Plugin(manifest, wasi=self._allow_wasi)
                self._status = PluginStatus.READY
                _log.info("Loaded Wasm plugin: %s from %s", self.metadata.name, self.plugin_path)
                return True

            except FileNotFoundError:
                _log.error("Plugin file not found: %s", self.plugin_path)
                self._status = PluginStatus.ERROR
                return False
            except Exception as e:
                _log.error("Failed to load plugin %s: %s", self.metadata.name, e)
                self._status = PluginStatus.ERROR
                return False

    def execute(self, input_data: str | bytes) -> WasmExecutionResult:
        """Execute the plugin with the given input."""
        with self._lock:
            if self._status not in (PluginStatus.READY, PluginStatus.RUNNING):
                if not self.load():
                    return WasmExecutionResult(
                        status="error",
                        error=f"Plugin not loaded: {self._status.value}",
                    )

            self._status = PluginStatus.RUNNING

        start_time = time.time()
        start_cpu = time.process_time()

        try:
            # Convert input to bytes if needed
            input_bytes = input_data.encode("utf-8") if isinstance(input_data, str) else input_data

            # Execute the plugin
            output = self._plugin.call(self.metadata.entry_point, input_bytes)

            # Check output size
            output_size = len(output) if output else 0
            if output_size > self.limits.max_output_size_bytes:
                return WasmExecutionResult(
                    status="error",
                    error=f"Output size {output_size} exceeds limit {self.limits.max_output_size_bytes}",
                )

            # Calculate execution time
            duration_ms = (time.time() - start_time) * 1000
            cpu_time_ms = (time.process_time() - start_cpu) * 1000

            # Check time limits
            if duration_ms > self.limits.max_execution_time_ms:
                _log.warning(
                    "Plugin %s execution time %sms exceeds limit %sms",
                    self.metadata.name,
                    duration_ms,
                    self.limits.max_execution_time_ms,
                )

            with self._lock:
                self._status = PluginStatus.READY

            return WasmExecutionResult(
                status="success",
                output=output.decode("utf-8") if isinstance(output, bytes) else output,
                duration_ms=duration_ms,
                cpu_time_ms=cpu_time_ms,
            )

        except Exception as e:
            with self._lock:
                self._status = PluginStatus.READY
            return WasmExecutionResult(
                status="error",
                error=str(e),
                duration_ms=(time.time() - start_time) * 1000,
            )

    def unload(self) -> bool:
        """Unload the plugin and release resources."""
        with self._lock:
            if self._status == PluginStatus.UNLOADED:
                return True

            try:
                if self._plugin is not None:
                    self._plugin.close()
                    self._plugin = None
                self._status = PluginStatus.UNLOADED
                _log.info("Unloaded Wasm plugin: %s", self.metadata.name)
                return True
            except Exception as e:
                _log.error("Failed to unload plugin %s: %s", self.metadata.name, e)
                return False


class WasmPluginManager:
    """Manager for Wasm plugins with lifecycle management."""

    def __init__(self, plugin_dir: Path | None = None) -> None:
        self._runtime = ExtismRuntime()
        self._plugins: dict[str, WasmPlugin] = {}
        self._plugin_dir = plugin_dir
        self._lock = threading.Lock()

    @property
    def is_available(self) -> bool:
        """Check if the Wasm runtime is available."""
        return self._runtime.is_available

    def initialize(self) -> bool:
        """Initialize the Wasm runtime."""
        return self._runtime.initialize()

    def register_plugin(self, plugin: WasmPlugin) -> bool:
        """Register a plugin with the manager.

        Args:
            plugin: The plugin to register.

        Returns:
            True if registration was successful, False otherwise.
        """
        with self._lock:
            if plugin.metadata.name in self._plugins:
                _log.warning("Plugin %s already registered", plugin.metadata.name)
                return False

            self._plugins[plugin.metadata.name] = plugin
            _log.info("Registered plugin: %s", plugin.metadata.name)
            return True

    def get_plugin(self, name: str) -> WasmPlugin | None:
        """Get a registered plugin by name.

        Args:
            name: The plugin name.

        Returns:
            The plugin if found, None otherwise.
        """
        with self._lock:
            return self._plugins.get(name)

    def load_plugin(self, name: str) -> bool:
        """Load a registered plugin.

        Args:
            name: The plugin name.

        Returns:
            True if loading was successful, False otherwise.
        """
        with self._lock:
            plugin = self._plugins.get(name)
            if plugin is None:
                _log.error("Plugin not found: %s", name)
                return False
            return plugin.load()

    def execute_plugin(self, name: str, input_data: str | bytes) -> WasmExecutionResult | None:
        """Execute a registered plugin.

        Args:
            name: The plugin name.
            input_data: Input data to pass to the plugin.

        Returns:
            Execution result if successful, None if plugin not found.
        """
        with self._lock:
            plugin = self._plugins.get(name)
            if plugin is None:
                _log.error("Plugin not found: %s", name)
                return None
        return plugin.execute(input_data)

    def unload_plugin(self, name: str) -> bool:
        """Unload a registered plugin.

        Args:
            name: The plugin name.

        Returns:
            True if unloading was successful, False otherwise.
        """
        with self._lock:
            plugin = self._plugins.get(name)
            if plugin is None:
                return False
        return plugin.unload()

    def list_plugins(self) -> list[str]:
        """List all registered plugin names.

        Returns:
            List of plugin names.
        """
        with self._lock:
            return list(self._plugins.keys())

    def remove_plugin(self, name: str) -> bool:
        """Remove and unload a plugin.

        Args:
            name: The plugin name.

        Returns:
            True if removal was successful, False otherwise.
        """
        with self._lock:
            plugin = self._plugins.pop(name, None)
            if plugin is None:
                return False

        plugin.unload()
        _log.info("Removed plugin: %s", name)
        return True

    def clear(self) -> None:
        """Remove and unload all plugins."""
        with self._lock:
            plugin_names = list(self._plugins.keys())

        for name in plugin_names:
            self.remove_plugin(name)


def create_plugin_from_manifest(manifest_path: Path) -> ExtismPlugin | None:
    """Create a plugin from a manifest file.

    Args:
        manifest_path: Path to the plugin manifest JSON file.

    Returns:
        An ExtismPlugin instance if successful, None otherwise.
    """
    try:
        manifest_data = json.loads(manifest_path.read_text())

        # Parse metadata
        metadata = WasmPluginMetadata(
            name=manifest_data.get("name", "unnamed"),
            version=manifest_data.get("version", "0.1.0"),
            description=manifest_data.get("description", ""),
            author=manifest_data.get("author", ""),
            entry_point=manifest_data.get("entry_point", "run"),
        )

        # Parse capabilities
        capabilities = manifest_data.get("capabilities", [])
        metadata.capabilities = [WasmCapability(c) for c in capabilities if isinstance(c, str)]

        # Get wasm path
        wasm_path = manifest_data.get("wasm")
        if wasm_path:  # noqa: SIM108 -- explicit branching preferred for readability
            plugin_path = manifest_path.parent / wasm_path
        else:
            # Try to find .wasm file in the same directory
            plugin_path = manifest_path.parent / f"{metadata.name}.wasm"

        if not plugin_path.exists():
            _log.error("Wasm file not found: %s", plugin_path)
            return None

        # Parse resource limits
        limits_data = manifest_data.get("limits", {})
        limits = ResourceLimits(
            max_memory_mb=limits_data.get("max_memory_mb", 128),
            max_cpu_time_ms=limits_data.get("max_cpu_time_ms", 5000),
            max_execution_time_ms=limits_data.get("max_execution_time_ms", 30000),
            max_output_size_bytes=limits_data.get("max_output_size_bytes", 1024 * 1024),
        )

        # Parse config
        config = manifest_data.get("config", {})

        return ExtismPlugin(
            plugin_path=plugin_path,
            metadata=metadata,
            limits=limits,
            config=config,
        )

    except json.JSONDecodeError as e:
        _log.error("Invalid manifest JSON: %s", e)
        return None
    except Exception as e:
        _log.error("Failed to create plugin from manifest: %s", e)
        return None


# Global singleton for the plugin manager
_plugin_manager: WasmPluginManager | None = None
_manager_lock = threading.Lock()


def get_plugin_manager() -> WasmPluginManager:
    """Get the global plugin manager instance.

    Returns:
        The global WasmPluginManager instance.
    """
    global _plugin_manager
    with _manager_lock:
        if _plugin_manager is None:
            _plugin_manager = WasmPluginManager()
        return _plugin_manager
