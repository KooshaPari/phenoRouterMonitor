"""Worker Node for Multi-Runtime Bridge.

Runs on specialized runtimes (PyPy, CPython 3.14) and executes dispatched tasks.
"""

import asyncio
import importlib
import json
import os
import sys
import time
from pathlib import Path
import typer

# Ensure we can import thegent
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from thegent.infra.ipc import MaildirQueue

try:
    from thegent_shm import init_shm, record_resource_usage  # type: ignore[reportMissingImports]

    HAS_SHM = True
except ImportError:
    HAS_SHM = False

app = typer.Typer(
    name="worker-node",
    help="Worker node runtime entrypoint.",
    add_completion=False,
    no_args_is_help=False,
)


async def worker_loop(mesh_root: Path, runtime_name: str):
    queue_path = mesh_root / "queue" / runtime_name.lower()

    queue = MaildirQueue(queue_path)

    # Initialize SHM if available
    if HAS_SHM:
        init_shm(str(mesh_root / "state.shm"))

    # Platform-specific heartbeat tuning
    is_wifi = os.environ.get("THEGENT_NETWORK_TIER") == "wifi"
    heartbeat_interval = 2.0 if is_wifi else 1.0
    last_heartbeat = 0.0

    while True:
        # 1. Emit Heartbeat
        now = time.time()
        if now - last_heartbeat > heartbeat_interval:
            last_heartbeat = now

            # Record health metrics to SHM
            if HAS_SHM:
                import psutil

                try:
                    proc = psutil.Process()
                    record_resource_usage(os.getpid(), proc.cpu_percent(), proc.memory_info().rss)
                except Exception:
                    pass

        # 2. Check for new tasks
        result = queue.receive()
        if result:
            _msg_id, message = result
            if message.get("type") == "task":
                task_id = message["task_id"]
                module_name = message["module"]
                func_name = message["function"]
                args = message.get("args", [])
                kwargs = message.get("kwargs", {})

                try:
                    # Dynamic Import and Execution
                    module = importlib.import_module(module_name)
                    func = getattr(module, func_name)

                    start_time = time.perf_counter()
                    output = func(*args, **kwargs)
                    duration = time.perf_counter() - start_time

                    # Store result back in mesh (e.g., results/task_id.json)
                    result_path = mesh_root / "results" / f"{task_id}.json"
                    result_path.parent.mkdir(parents=True, exist_ok=True)
                    result_path.write_text(
                        json.dumps(
                            {
                                "status": "success",
                                "output": output,
                                "duration": duration,
                                "runtime": runtime_name,
                                "implementation": sys.implementation.name,
                                "version": sys.version,
                            }
                        ),
                        encoding="utf-8",
                    )

                except Exception as e:
                    result_path = mesh_root / "results" / f"{task_id}.json"
                    result_path.write_text(
                        json.dumps({"status": "error", "error": str(e), "runtime": runtime_name}),
                        encoding="utf-8",
                    )

        # 3. Sleep with low-latency responsiveness
        await asyncio.sleep(0.1)


@app.callback(invoke_without_command=True)
def cli(
    mesh_root: Path = typer.Option(..., "--mesh-root", help="Mesh root directory."),
    runtime: str = typer.Option(..., "--runtime", help="Runtime name."),
) -> None:
    asyncio.run(worker_loop(mesh_root, runtime))


if __name__ == "__main__":
    app()
