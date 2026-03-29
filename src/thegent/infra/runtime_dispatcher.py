"""Performance-optimized dispatcher for multi-runtime support.

This module handles the 'splitting' of code into optimal paths for:
1. PyPy (JIT-optimized pure Python)
2. CPython 3.13/3.14 (Native extensions and freethreading support)
3. Compiled backends (Rust, Go, Zig via FFI/Wasm)
"""

import logging
import sys
from collections.abc import Callable
from typing import Any

logger = logging.getLogger(__name__)

# --- Runtime Identity ---
IS_PYPY = sys.implementation.name == "pypy"
PYTHON_VERSION = sys.version_info[:2]
HAS_FREETHREADING = getattr(sys, "_is_gil_enabled", lambda: True)() is False


class PerformanceModule:
    """Base class for performance-critical modules with multiple implementations."""

    def __init__(self, name: str) -> None:
        self.name = name
        self._implementations: dict[str, Any] = {}
        self._selected: Any | None = None

    def register(self, runtime: str, impl: Any):
        self._implementations[runtime] = impl

    def get_impl(self) -> Any:
        if self._selected:
            return self._selected

        # Selection Logic
        # 1. Try Native Extension (Rust/C) first on CPython
        if not IS_PYPY:
            if "native" in self._implementations:
                self._selected = self._implementations["native"]
                return self._selected

        # 2. Try PyPy optimized logic
        if IS_PYPY and "pypy" in self._implementations:
            self._selected = self._implementations["pypy"]
            return self._selected

        # 3. Fallback to generic python
        self._selected = self._implementations.get("python")
        return self._selected


# --- Performance-Critical Dispatchers ---

# 1. JSON (Massive difference between orjson and pypy-json)
try:
    import orjson

    HAS_ORJSON = True
except ImportError:
    HAS_ORJSON = False

try:
    import ujson

    HAS_UJSON = True
except ImportError:
    HAS_UJSON = False


def _pypy_json_dumps(obj: Any, **kwargs) -> str:
    import json

    if HAS_UJSON:
        return ujson.dumps(obj, **kwargs)
    return json.dumps(obj, **kwargs)


def _cpython_json_dumps(obj: Any, **kwargs) -> str:
    if HAS_ORJSON:
        # orjson.dumps returns bytes
        return orjson.dumps(obj, **kwargs).decode("utf-8")
    import json

    return json.dumps(obj, **kwargs)


def _pypy_json_loads(s: str | bytes, **kwargs) -> Any:
    import json

    if HAS_UJSON:
        return ujson.loads(s, **kwargs)
    return json.loads(s, **kwargs)


def _cpython_json_loads(s: str | bytes, **kwargs) -> Any:
    if HAS_ORJSON:
        return orjson.loads(s)
    import json

    return json.loads(s, **kwargs)


json_dumps_dispatcher = PerformanceModule("json_dumps")
json_dumps_dispatcher.register("pypy", _pypy_json_dumps)
json_dumps_dispatcher.register("native", _cpython_json_dumps)
json_dumps_dispatcher.register("python", _pypy_json_dumps)

json_loads_dispatcher = PerformanceModule("json_loads")
json_loads_dispatcher.register("pypy", _pypy_json_loads)
json_loads_dispatcher.register("native", _cpython_json_loads)
json_loads_dispatcher.register("python", _pypy_json_loads)

# 2. TOML (rtoml vs tomli)
try:
    import rtoml

    HAS_RTOML = True
except ImportError:
    HAS_RTOML = False

try:
    import tomli

    HAS_TOMLI = True
except ImportError:
    HAS_TOMLI = False


def _pypy_toml_loads(s: str) -> dict[str, Any]:
    if HAS_TOMLI:
        return tomli.loads(s)
    import tomlkit

    return tomlkit.parse(s).value


def _cpython_toml_loads(s: str) -> dict[str, Any]:
    if HAS_RTOML:
        return rtoml.loads(s)
    if HAS_TOMLI:
        return tomli.loads(s)
    import tomlkit

    return tomlkit.parse(s).value


toml_loads_dispatcher = PerformanceModule("toml_loads")
toml_loads_dispatcher.register("pypy", _pypy_toml_loads)
toml_loads_dispatcher.register("native", _cpython_toml_loads)
toml_loads_dispatcher.register("python", _pypy_toml_loads)

# 3. Routing (The Core Logic)
try:
    import thegent_router  # type: ignore[reportMissingImports]

    HAS_RUST_ROUTER = True
except ImportError:
    HAS_RUST_ROUTER = False


def _python_route_logic(task: str, agents: list) -> Any:
    """Pure python JIT-friendly routing logic - keyword-based task classification."""
    task_lower = task.lower()

    # Keywords for different agent types
    code_keywords = ("implement", "code", "write", "refactor", "fix", "debug", "patch")
    research_keywords = ("research", "search", "find", "explore", "analyze", "summarize")
    plan_keywords = ("plan", "design", "architect", "spec", "document")

    # Score each agent based on task affinity
    best_agent = None
    best_score = -1

    for agent in agents:
        agent_name = getattr(agent, "name", str(agent)).lower()
        score = 0

        if any(kw in task_lower for kw in code_keywords):
            if "implementer" in agent_name or "coder" in agent_name:
                score += 10
            elif "agent" in agent_name:
                score += 5

        if any(kw in task_lower for kw in research_keywords):
            if "research" in agent_name or "search" in agent_name:
                score += 10

        if any(kw in task_lower for kw in plan_keywords):
            if "planner" in agent_name or "architect" in agent_name:
                score += 10

        # Default fallback scoring
        if score == 0:
            score = 1

        if score > best_score:
            best_score = score
            best_agent = agent

    return best_agent


router_dispatcher = PerformanceModule("router")
if HAS_RUST_ROUTER:
    from thegent.config import get_settings

    settings = get_settings()
    try:
        router = thegent_router.PyParetoRouter.with_full_config(  # type: ignore[reportAttributeAccessIssue]
            low_threshold=0.35,
            high_threshold=0.65,
            hysteresis_band=settings.router_hysteresis_band,
            hysteresis_dwell_s=settings.router_hysteresis_dwell,
            hysteresis_max_dwell_s=settings.router_hysteresis_max_dwell,
            hysteresis_override=settings.router_hysteresis_override,
        )
    except AttributeError:
        try:
            router = thegent_router.ParetoRouter.with_full_config(  # type: ignore[reportAttributeAccessIssue]
                low_threshold=0.35,
                high_threshold=0.65,
                hysteresis_band=settings.router_hysteresis_band,
                hysteresis_dwell_s=settings.router_hysteresis_dwell,
                hysteresis_max_dwell_s=settings.router_hysteresis_max_dwell,
                hysteresis_override=settings.router_hysteresis_override,
            )
        except AttributeError:
            router = thegent_router.ParetoRouter()  # type: ignore[reportAttributeAccessIssue]
    router_dispatcher.register("native", router)
router_dispatcher.register("pypy", _python_route_logic)
router_dispatcher.register("python", _python_route_logic)

# 4. Wasm (Extism) for Zig/Rust Plugins
try:
    import extism

    HAS_EXTISM = True
except ImportError:
    HAS_EXTISM = False


class WasmDispatcher:
    """Portable performance modules using Extism/Wasm (Zig/Rust)."""

    @staticmethod
    def call_plugin(plugin_path: str, func_name: str, data: bytes) -> bytes:
        if not HAS_EXTISM:
            raise ImportError("extism not installed. Wasm modules unavailable.")

        with extism.Plugin(open(plugin_path, "rb").read(), wasi=True) as plugin:
            return plugin.call(func_name, data)


# 5. Mojo Accelerator (Phase 2)
try:
    # Mojo-to-Python interop usually involves 'import mojo' in certain builds
    # or calling a compiled Mojo SO via ctypes/FFI
    HAS_MOJO = False  # Placeholder for Phase 2
except ImportError:
    HAS_MOJO = False


class MojoDispatcher:
    """High-speed compute offloading to Mojo kernels."""


# --- Global Access ---


def get_json_dumps() -> Callable:
    return json_dumps_dispatcher.get_impl()


def get_json_loads() -> Callable:
    return json_loads_dispatcher.get_impl()


def get_toml_loads() -> Callable:
    return toml_loads_dispatcher.get_impl()


def get_router() -> Any:
    return router_dispatcher.get_impl()


def get_runtime_status() -> dict[str, Any]:
    return {
        "implementation": sys.implementation.name,
        "version": f"{PYTHON_VERSION[0]}.{PYTHON_VERSION[1]}",
        "freethreading": HAS_FREETHREADING,
        "jit": IS_PYPY,
        "active_extensions": {
            "orjson": HAS_ORJSON,
            "ujson": HAS_UJSON,
            "rtoml": HAS_RTOML,
            "rust_router": HAS_RUST_ROUTER,
        },
    }
