"""Multi-runtime diagnostics for PyPy, CPython, Rust, Go, and Mojo.

This module provides comprehensive health checks for all runtimes in the
polyglot architecture.
"""

import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from rich.console import Console
from rich.table import Table

from thegent.infra import run_subprocess_optimized

console = Console()


@dataclass
class RuntimeStatus:
    """Status of a runtime."""

    name: str
    available: bool
    version: str | None = None
    path: str | None = None
    performance_tier: str | None = None  # "optimal", "good", "degraded", "unavailable"
    issues: list[str] = field(default_factory=list)
    recommendations: list[str] = field(default_factory=list)

    def __post_init__(self):
        if self.performance_tier is None:
            self.performance_tier = "optimal" if self.available else "unavailable"


def check_pypy() -> RuntimeStatus:
    """Check PyPy runtime availability and health."""
    status = RuntimeStatus(name="PyPy", available=False)

    # Check if PyPy is available
    try:
        result = run_subprocess_optimized(
            ["uv", "run", "--python", "pypy-3.11", "python", "--version"],
            capture_output=True,
            timeout=10,
        )
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "optimal"
            status.path = "uv managed"
    except Exception as e:
        status.issues.append(f"PyPy check failed: {e}")

    # Check for ujson (PyPy optimization)
    if status.available:
        try:
            result = run_subprocess_optimized(
                ["uv", "run", "--python", "pypy-3.11", "--with", "ujson", "python", "-c", "import ujson; print('ok')"],
                capture_output=True,
                timeout=10,
            )
            if result.returncode != 0:
                status.issues.append("ujson not available (recommended for PyPy)")
                status.recommendations.append("Install ujson: uv pip install ujson")
        except Exception as exc:
            status.issues.append(f"ujson probe failed: {exc}")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install PyPy: uv python install pypy-3.11")

    return status


def check_cpython_313() -> RuntimeStatus:
    """Check CPython 3.13 runtime availability and health."""
    status = RuntimeStatus(name="CPython 3.13", available=False)

    try:
        result = run_subprocess_optimized(
            ["uv", "run", "--python", "3.13", "python", "--version"],
            capture_output=True,
            timeout=10,
        )
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "good"
            status.path = "uv managed"
    except Exception as e:
        status.issues.append(f"CPython 3.13 check failed: {e}")

    # Check for orjson (CPython optimization)
    if status.available:
        try:
            result = run_subprocess_optimized(
                ["uv", "run", "--python", "3.13", "--with", "orjson", "python", "-c", "import orjson; print('ok')"],
                capture_output=True,
                timeout=10,
            )
            if result.returncode != 0:
                status.issues.append("orjson not available (recommended for CPython)")
                status.recommendations.append("Install orjson: uv pip install orjson")
        except Exception as exc:
            status.issues.append(f"orjson probe failed: {exc}")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install CPython 3.13: uv python install 3.13")

    return status


def check_cpython_314() -> RuntimeStatus:
    """Check CPython 3.14 runtime availability and health."""
    status = RuntimeStatus(name="CPython 3.14", available=False)

    try:
        result = run_subprocess_optimized(
            ["uv", "run", "--python", "3.14", "python", "--version"],
            capture_output=True,
            timeout=10,
        )
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "optimal"
            status.path = "uv managed"
    except Exception as e:
        status.issues.append(f"CPython 3.14 check failed: {e}")

    # Check for orjson (CPython optimization)
    if status.available:
        try:
            result = run_subprocess_optimized(
                ["uv", "run", "--python", "3.14", "--with", "orjson", "python", "-c", "import orjson; print('ok')"],
                capture_output=True,
                timeout=10,
            )
            if result.returncode != 0:
                status.issues.append("orjson not available (recommended for CPython)")
                status.recommendations.append("Install orjson: uv pip install orjson")
        except Exception as exc:
            status.issues.append(f"orjson probe failed: {exc}")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install CPython 3.14: uv python install 3.14")

    return status


def check_rust() -> RuntimeStatus:
    """Check Rust runtime availability and health."""
    status = RuntimeStatus(name="Rust", available=False)

    # Check for cargo
    try:
        result = run_subprocess_optimized(["cargo", "--version"], capture_output=True, timeout=5)
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "optimal"
            status.path = "system"
    except Exception as e:
        status.issues.append(f"Rust check failed: {e}")

    # Check for thegent Rust crates
    if status.available:
        crates_path = Path("crates")
        if crates_path.exists():
            cargo_toml = crates_path / "Cargo.toml"
            if cargo_toml.exists():
                status.performance_tier = "optimal"
            else:
                status.issues.append("thegent Rust crates not found")
        else:
            status.issues.append("crates/ directory not found")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")

    return status


def check_go() -> RuntimeStatus:
    """Check Go runtime availability and health."""
    status = RuntimeStatus(name="Go", available=False)

    try:
        result = run_subprocess_optimized(["go", "version"], capture_output=True, timeout=5)
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "optimal"
            status.path = "system"
    except Exception as e:
        status.issues.append(f"Go check failed: {e}")

    # Check for cliproxyapi-plusplus
    if status.available:
        cliproxy_paths = [Path("../cliproxyapi-plusplus")]
        found = False
        for path in cliproxy_paths:
            if path.exists() and (path / "cmd" / "server" / "main.go").exists():
                found = True
                break
        if not found:
            status.issues.append("cliproxyapi-plusplus not found")
            status.recommendations.append("Clone cliproxyapi-plusplus repository")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install Go: https://go.dev/dl/")

    return status


def check_mojo() -> RuntimeStatus:
    """Check Mojo runtime availability and health."""
    status = RuntimeStatus(name="Mojo", available=False)

    try:
        result = run_subprocess_optimized(["mojo", "--version"], capture_output=True, timeout=5)
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "optimal"
            status.path = "system"
    except Exception as exc:
        status.issues.append(f"Mojo check failed: {exc}")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install Mojo: https://docs.modular.com/mojo/manual/get-started/")

    return status


def check_zig() -> RuntimeStatus:
    """Check Zig runtime availability and health."""
    status = RuntimeStatus(name="Zig", available=False)

    try:
        result = run_subprocess_optimized(["zig", "version"], capture_output=True, timeout=5)
        if result.returncode == 0 and result.stdout:
            stdout_text = (
                result.stdout if isinstance(result.stdout, str) else result.stdout.decode("utf-8", errors="replace")
            )
            status.available = True
            status.version = stdout_text.strip()
            status.performance_tier = "optimal"
            status.path = "system"
    except Exception as exc:
        status.issues.append(f"Zig check failed: {exc}")

    if not status.available:
        status.performance_tier = "unavailable"
        status.recommendations.append("Install Zig: https://ziglang.org/download/")

    return status


def check_hardware() -> dict[str, Any]:
    """Check hardware-specific performance features."""
    import platform

    hw_info = {
        "platform": platform.system(),
        "arch": platform.machine(),
        "amx_available": False,
        "cuda_available": False,
        "unified_memory": False,
    }

    if hw_info["platform"] == "Darwin" and hw_info["arch"] == "arm64":
        # M1/M2/M3 check
        hw_info["amx_available"] = True
        hw_info["unified_memory"] = True

    if hw_info["platform"] == "Linux":
        # Check for io_uring support
        try:
            if Path("/proc/sys/kernel/io_uring_disabled").exists():
                hw_info["io_uring_available"] = True
        except Exception as exc:
            hw_info["io_uring_available"] = False
            hw_info["io_uring_error"] = str(exc)

        # Check for LTO or other flags in /proc/config.gz if available
        # This is very distro-specific, so just a basic check
        hw_info["is_wsl"] = "microsoft" in platform.release().lower()

    return hw_info


def check_network_latency(target_host: str = "127.0.0.1") -> dict[str, float | list[str]]:
    """Check network latency to a target host."""
    import time

    latencies = []
    errors: list[str] = []
    for _ in range(5):
        start = time.perf_counter()
        try:
            # Simple ping-like check using subprocess
            if sys.platform == "win32":
                run_subprocess_optimized(["ping", "-n", "1", target_host], capture_output=True, timeout=2)
            else:
                run_subprocess_optimized(["ping", "-c", "1", target_host], capture_output=True, timeout=2)
            latencies.append((time.perf_counter() - start) * 1000)
        except Exception as exc:
            errors.append(str(exc))

    if not latencies:
        return {"avg_ms": -1.0, "jitter_ms": -1.0, "errors": errors}

    avg = sum(latencies) / len(latencies)
    jitter = max(latencies) - min(latencies)
    return {"avg_ms": avg, "jitter_ms": jitter, "errors": errors}


def check_ipc_mesh(mesh_root: Path) -> dict[str, Any]:
    """Verify IPC mesh connectivity and performance."""
    import time

    from thegent.infra.ipc import IPCMesh, MaildirQueue

    results = {
        "mesh_exists": mesh_root.exists(),
        "write_latency_ms": -1.0,
        "read_latency_ms": -1.0,
        "atomic_lock_works": False,
    }

    try:
        mesh = IPCMesh(mesh_root)

        # Test Atomic Lock
        if mesh.acquire_atomic_lock("doctor_test", ttl=5):
            results["atomic_lock_works"] = True
            mesh.release_atomic_lock("doctor_test")

        # Test Queue Latency
        queue = MaildirQueue(mesh_root / "doctor_queue")
        start = time.perf_counter()
        _msg_id = queue.send({"test": "data"})
        results["write_latency_ms"] = (time.perf_counter() - start) * 1000

        start = time.perf_counter()
        queue.receive()
        results["read_latency_ms"] = (time.perf_counter() - start) * 1000

    except Exception as e:
        results["error"] = str(e)

    return results


def check_all_runtimes(mesh_root: Path | None = None) -> dict[str, Any]:
    """Check all runtimes and hardware context."""
    if mesh_root is None:
        mesh_root = Path("/tmp/thegent-bridge-doctor")

    return {
        "pypy": check_pypy(),
        "cpython_313": check_cpython_313(),
        "cpython_314": check_cpython_314(),
        "rust": check_rust(),
        "go": check_go(),
        "mojo": check_mojo(),
        "zig": check_zig(),
        "hardware": check_hardware(),
        "ipc": check_ipc_mesh(mesh_root),
    }


def display_runtime_status(data: dict[str, Any]) -> None:
    """Display runtime status and hardware context in a formatted table."""
    statuses = {k: v for k, v in data.items() if isinstance(v, RuntimeStatus)}
    hw = data.get("hardware", {})
    ipc = data.get("ipc", {})

    # Hardware Summary
    console.print(f"\n[bold blue]Hardware Context:[/bold blue] {hw['platform']} ({hw['arch']})")
    feature_flags = []
    if hw.get("amx_available"):
        feature_flags.append("[green]AMX[/green]")
    if hw.get("cuda_available"):
        feature_flags.append("[green]CUDA[/green]")
    if hw.get("unified_memory"):
        feature_flags.append("[green]UMA[/green]")
    if feature_flags:
        console.print(f"Features: {' | '.join(feature_flags)}")

    # IPC Mesh Status
    if ipc:
        console.print("\n[bold blue]IPC Mesh Status:[/bold blue]")
        lock_status = "[green]OK[/green]" if ipc.get("atomic_lock_works") else "[red]FAIL[/red]"
        console.print(f"  Atomic Locks: {lock_status}")
        console.print(f"  Write Latency: [yellow]{ipc.get('write_latency_ms', -1):.2f}ms[/yellow]")
        console.print(f"  Read Latency:  [yellow]{ipc.get('read_latency_ms', -1):.2f}ms[/yellow]")
    table = Table(title="Multi-Runtime Status", show_header=True, header_style="bold cyan")
    table.add_column("Runtime", style="cyan")
    table.add_column("Status", style="green")
    table.add_column("Version", style="yellow")
    table.add_column("Performance", style="magenta")
    table.add_column("Issues", style="red")

    for status in statuses.values():
        status_icon = "✓" if status.available else "✗"
        status_text = "Available" if status.available else "Unavailable"
        performance = status.performance_tier or "unknown"
        issues = ", ".join(status.issues) if status.issues else "None"

        table.add_row(
            status.name,
            f"{status_icon} {status_text}",
            status.version or "N/A",
            performance,
            issues,
        )

    console.print(table)

    # Display recommendations
    all_recommendations = []
    for status in statuses.values():
        all_recommendations.extend(status.recommendations)

    if all_recommendations:
        console.print("\n[bold yellow]Recommendations:[/bold yellow]")
        for rec in set(all_recommendations):
            console.print(f"  • {rec}")
