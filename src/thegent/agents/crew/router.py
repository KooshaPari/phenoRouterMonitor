"""RouterManager with multi-runtime optimized backends."""

import logging
import importlib
import tempfile
from pathlib import Path
from typing import Any

from thegent.infra.runtime_dispatcher import router_dispatcher

from .router_logic import PurePythonRouter, RouteMetrics, RoutingStrategy

logger = logging.getLogger(__name__)

_NATIVE_IMPORTER = importlib.import_module


def _register_native_router() -> None:
    """Register the native router implementation when available."""

    try:
        thegent_router = _NATIVE_IMPORTER("thegent_router")
        from thegent.config import get_settings

        settings = get_settings()
        router = thegent_router.PyParetoRouter.with_full_config(
            low_threshold=0.35,  # These could also be exposed if needed
            high_threshold=0.65,
            hysteresis_band=settings.router_hysteresis_band,
            hysteresis_dwell_s=settings.router_hysteresis_dwell,
            hysteresis_max_dwell_s=settings.router_hysteresis_max_dwell,
            hysteresis_override=settings.router_hysteresis_override,
        )
        router_dispatcher.register("native", router)
    except ImportError as exc:
        logger.info(
            "Native router extension unavailable; using PurePython fallback (%s)",
            exc,
            exc_info=True,
        )
    except AttributeError as exc:
        logger.warning(
            "Native router API mismatch while constructing native implementation; using PurePython fallback",
            exc_info=True,
            extra={"error": str(exc)},
        )


_register_native_router()

router_dispatcher.register("pypy", PurePythonRouter())
router_dispatcher.register("python", PurePythonRouter())


class RouterManager:
    """
    Unified routing interface that selects the best backend (Rust vs Pure Python).
    """

    def __init__(self, strategy: RoutingStrategy = RoutingStrategy.BALANCED) -> None:
        self.strategy = strategy
        # Get the optimized implementation from the dispatcher
        self._impl = router_dispatcher.get_impl()
        if hasattr(self._impl, "strategy"):
            self._impl.strategy = strategy

    def update_agent_metrics(self, agent_id: str, metrics: RouteMetrics) -> None:
        if hasattr(self._impl, "update_agent_metrics"):
            self._impl.update_agent_metrics(agent_id, metrics)
        elif hasattr(self._impl, "agent_metrics"):
            self._impl.agent_metrics[agent_id] = metrics

    def select_agent(self, task_description: str, available_agents: list) -> Any:
        # Route to the optimized implementation
        if hasattr(self._impl, "select_agent"):
            return self._impl.select_agent(task_description, available_agents)
        if hasattr(self._impl, "route"):
            # Map Rust ParetoRouter interface if needed
            # For now, we assume interfaces are compatible or handled here
            return self._impl.route(task_description)

        return None

    def refresh_from_mesh(self, mesh_root: Path | None = None) -> None:
        """Sync local routing metrics with the global IPC mesh."""
        from thegent.infra.ipc import SharedStateManager

        from .router_logic import RouteMetrics

        manager = SharedStateManager(mesh_root or (Path(tempfile.gettempdir()) / "thegent-bridge"))
        mesh_metrics = manager.get_all_metrics()

        for provider, data in mesh_metrics.items():
            metrics = RouteMetrics(
                cost_per_token=data.get("cost_per_1k_output", 0) / 1000,
                latency_ms=float(data.get("latency_p50_ms", 0)),
                success_rate=data.get("success_rate", 1.0),
            )
            self.update_agent_metrics(provider, metrics)

    @property
    def backend(self) -> str:
        return self._impl.__class__.__name__
