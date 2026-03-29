import contextlib
import os
from pathlib import Path
from typing import Any

import thegent_shm


class SHMManager:
    """Python wrapper for the high-performance Rust SHM mesh."""

    def __init__(self, shm_path: Path | None = None) -> None:
        default_shm_path = Path("/", "tmp", "thegent-bridge", "state.shm")
        self.shm_path = shm_path or Path(os.environ.get("THEGENT_SHM_PATH", str(default_shm_path)))
        self._initialized = False
        self._ensure_init()

    def _ensure_init(self) -> None:
        if not self._initialized:
            self.shm_path.parent.mkdir(parents=True, exist_ok=True)
            thegent_shm.py_init_shm(str(self.shm_path))
            self._initialized = True

    def update_provider_metrics(self, provider: str, request_count: int, success_count: int, latency_ms: int) -> None:
        self._ensure_init()
        with contextlib.suppress(Exception):
            thegent_shm.update_provider(provider, request_count, success_count, latency_ms)

    def get_provider_metrics(self, provider: str) -> dict[str, Any] | None:
        self._ensure_init()
        return thegent_shm.get_provider_metrics(provider)

    def record_resource_usage(self, pid: int, cpu_percent: float, memory_kb: int) -> None:
        self._ensure_init()
        with contextlib.suppress(Exception):
            thegent_shm.record_resource_usage(pid, float(cpu_percent), int(memory_kb))

    def award_xp(self, amount: int) -> None:
        self._ensure_init()
        with contextlib.suppress(Exception):
            thegent_shm.award_xp(amount)

    def get_xp_state(self) -> dict[str, Any] | None:
        self._ensure_init()
        return thegent_shm.get_xp_state()

    def record_failure(self, target: str, category: int) -> None:
        self._ensure_init()
        with contextlib.suppress(Exception):
            thegent_shm.record_failure(target, category)

    def set_health_score(self, score: float) -> None:
        self._ensure_init()
        with contextlib.suppress(Exception):
            thegent_shm.set_health_score(score)

    def get_health_score(self) -> float:
        self._ensure_init()
        return thegent_shm.get_health_score()

    def update_router_metrics(
        self, lifecycle_inc: int = 0, thegent_inc: int = 0, changes_inc: int = 0, hysteresis_inc: int = 0
    ) -> None:
        self._ensure_init()
        with contextlib.suppress(Exception):
            thegent_shm.update_router_metrics(lifecycle_inc, thegent_inc, changes_inc, hysteresis_inc)

    def get_router_metrics(self) -> dict[str, Any] | None:
        self._ensure_init()
        return thegent_shm.get_router_metrics()
