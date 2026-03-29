"""Pure execution orchestration logic (no I/O, no subprocess).

This module contains the core business logic for task execution orchestration,
separated from I/O and subprocess management. Coordinates:
- Agent/model resolution
- Budget checking
- Routing decisions
- Policy evaluation
- Concurrency control
- Registry integration
- Error classification
"""

import hashlib
import time
import uuid
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import structlog

_log = structlog.get_logger(__name__)


class ExecutionOrchestrator:
    """Orchestrates agent execution without I/O operations."""

    @staticmethod
    def validate_timeout(timeout: int | None, agent: str, default: int) -> int:
        """Compute effective timeout with agent-specific minimums."""
        if timeout is None:
            return default
        if agent == "claude":
            return max(timeout, 300)
        return timeout

    @staticmethod
    def check_idempotency(
        idempotency_token: str | None,
        registry: Any,
    ) -> dict[str, Any] | None:
        """Check for existing run via idempotency token.

        Returns dict with cached result if replay detected, None otherwise.
        """
        if not idempotency_token:
            return None

        session_id_from_token = f"run_{hashlib.sha256(idempotency_token.encode()).hexdigest()[:8]}"
        if registry.session_exists(session_id_from_token):
            existing = registry.find_by_token(idempotency_token)
            if existing and existing.get("status") == "completed":
                _log.info("Replay detected for token %s; skipping execution.", idempotency_token)
                return {
                    "stdout": existing.get("stdout", ""),
                    "stderr": existing.get("stderr", ""),
                    "exit_code": existing.get("exit_code", 0),
                    "run_id": existing.get("run_id"),
                    "replayed": True,
                }
        return None

    @staticmethod
    def classify_error(result: Any) -> str | None:
        """Classify error from agent result.

        Returns error_class string or None if no error.
        """
        if result is None:
            return None
        if hasattr(result, "timed_out") and result.timed_out:
            return "timeout"
        if hasattr(result, "exit_code") and result.exit_code != 0:
            # Check for usage limit errors
            try:
                from thegent.agents.resilience import is_usage_limit
                if is_usage_limit(result):
                    return "usage_limit"
            except Exception:
                pass
            return "api_error"
        return None

    @staticmethod
    def build_run_metadata(
        run_id: str,
        agent: str,
        model: str | None,
        prompt: str,
        cwd: Path,
        mode: str,
        owner: str,
        **kwargs,
    ) -> dict[str, Any]:
        """Build run metadata dict for registry and auditing.

        Additional kwargs are merged into the metadata.
        """
        from thegent.execution import AgentSource, InteractivityMode, RunMeta

        resolved_domain_tag = kwargs.get("domain") or "default"

        metadata = RunMeta(
            run_id=run_id,
            correlation_id=kwargs.get("correlation_id"),
            source=(
                AgentSource.THEGENT_SUBAGENT
                if kwargs.get("task_id")
                else AgentSource.THEGENT_RUN
            ),
            interactivity=InteractivityMode.PTY,
            agent=agent or "unknown",
            model=model,
            mode=mode,
            prompt=prompt,
            cwd=str(cwd),
            owner=owner,
            task_id=kwargs.get("task_id"),
            task_metadata=kwargs.get("task_metadata"),
            route_contract=kwargs.get("route_contract"),
            route_request=kwargs.get("route_request"),
            lane=kwargs.get("lane", "standard"),
            confidence=kwargs.get("confidence"),
            idempotency_token=kwargs.get("idempotency_token"),
            override_reason=kwargs.get("override_reason"),
            override_by=owner if kwargs.get("override_reason") else None,
            domain_tag=str(resolved_domain_tag),
            contract_version=kwargs.get("contract_version"),
            arbitration=kwargs.get("arbitration"),
        )
        return metadata

    @staticmethod
    def record_execution_end(
        registry: Any,
        run_id: str,
        exit_code: int,
        status: str,
        error_class: str | None = None,
        cost_usd: float | None = None,
    ) -> None:
        """Record execution completion in registry."""
        registry.register_end(
            run_id=run_id,
            exit_code=exit_code,
            status=status,
            ended_at_utc=datetime.now(UTC).isoformat(),
            duration_s=0.0,  # Caller must compute
            error_class=error_class,
            cost_usd=cost_usd,
        )

    @staticmethod
    def compute_agents_to_try(
        primary_agent: str,
        model: str | None,
    ) -> list[str]:
        """Build fallback chain from primary agent and model routes."""
        from thegent.agents import get_fallback_agents
        from thegent.models import ModelCatalog, normalize_model_id

        agents = [primary_agent] if primary_agent else []

        if model:
            model_id = normalize_model_id(model)
            routes = ModelCatalog.routes_for(model_id)
            catalog_fallbacks = [r.provider for r in routes if r.provider != primary_agent]
            agents.extend(catalog_fallbacks)

        provider_fallbacks = get_fallback_agents(primary_agent or "unknown")
        for pf in provider_fallbacks:
            if pf not in agents:
                agents.append(pf)

        return agents
