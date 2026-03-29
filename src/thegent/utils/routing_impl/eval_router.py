"""GW-70: Online eval routing — route to highest-scoring model per task type.

Maintains per-model, per-task-type exponentially weighted moving average (EWMA)
scores. Routes to the model with the highest score for the given task type.

# @trace FR-EVAL-070
"""

from __future__ import annotations

import logging
import threading
import time
from dataclasses import dataclass

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Public data structures
# ---------------------------------------------------------------------------


@dataclass
class EvalScore:
    """Tracks the EWMA score for a (model, task_type) pair."""

    model: str
    task_type: str
    score: float  # current EWMA score (0.0-1.0)
    sample_count: int  # how many observations contributed
    last_updated: float  # time.time() timestamp


@dataclass
class EvalRouteResult:
    """Result of routing to the highest-scoring model for a task type."""

    selected_model: str
    score: float  # score of selected model
    task_type: str
    all_scores: dict  # model -> score for all candidates


# ---------------------------------------------------------------------------
# EvalRouter
# ---------------------------------------------------------------------------


class EvalRouter:
    """Routes requests to the highest-scoring model per task type using EWMA.

    Thread-safe. All state mutations are protected by a single lock.
    """

    def __init__(self, alpha: float = 0.3) -> None:
        if not (0 < alpha <= 1):
            raise ValueError(f"alpha must be in (0, 1], got {alpha!r}")
        self._alpha = alpha
        self._scores: dict = {}  # keyed by (model, task_type)
        self._lock: threading.Lock = threading.Lock()

    # ------------------------------------------------------------------
    # Mutations
    # ------------------------------------------------------------------

    def record_eval(self, model: str, task_type: str, score: float) -> None:
        """Update the EWMA score for (model, task_type).

        First observation sets the EWMA directly to the observed score.
        Subsequent: new_ewma = alpha * score + (1 - alpha) * old_ewma.
        """
        key = (model, task_type)
        now = time.time()
        with self._lock:
            existing = self._scores.get(key)
            if existing is None:
                new_score = score
                sample_count = 1
            else:
                new_score = self._alpha * score + (1 - self._alpha) * existing.score
                sample_count = existing.sample_count + 1
            self._scores[key] = EvalScore(
                model=model,
                task_type=task_type,
                score=new_score,
                sample_count=sample_count,
                last_updated=now,
            )
        _log.debug(
            "record_eval model=%r task_type=%r raw_score=%.4f ewma=%.4f",
            model,
            task_type,
            score,
            self._scores[key].score,
        )

    def reset(self) -> None:
        """Clear all scores. Useful for testing."""
        with self._lock:
            self._scores.clear()
        _log.debug("EvalRouter reset: all scores cleared")

    # ------------------------------------------------------------------
    # Queries
    # ------------------------------------------------------------------

    def get_score(self, model: str, task_type: str) -> float | None:
        """Return current EWMA score for (model, task_type), or None if no data."""
        with self._lock:
            entry = self._scores.get((model, task_type))
        return entry.score if entry is not None else None

    def route(
        self,
        task_type: str,
        available_models: list | None = None,
    ) -> EvalRouteResult | None:
        """Select the highest-scoring model for task_type.

        Parameters
        ----------
        task_type:
            The task category to route for.
        available_models:
            If provided, restrict candidates to this subset.

        Returns
        -------
        EvalRouteResult or None if no scored models are available.
        """
        with self._lock:
            candidates: dict = {}
            for (model, tt), entry in self._scores.items():
                if tt != task_type:
                    continue
                if available_models is not None and model not in available_models:
                    continue
                candidates[model] = entry.score

        if not candidates:
            _log.debug("route: no scored models for task_type=%r", task_type)
            return None

        best_model = max(candidates, key=lambda m: candidates[m])
        result = EvalRouteResult(
            selected_model=best_model,
            score=candidates[best_model],
            task_type=task_type,
            all_scores=dict(candidates),
        )
        _log.debug(
            "route: selected model=%r score=%.4f task_type=%r",
            best_model,
            candidates[best_model],
            task_type,
        )
        return result

    def list_scores(self, task_type: str | None = None) -> list:
        """Return all recorded EvalScore objects, optionally filtered by task_type."""
        with self._lock:
            entries = list(self._scores.values())
        if task_type is not None:
            entries = [e for e in entries if e.task_type == task_type]
        return entries


# ---------------------------------------------------------------------------
# Module-level singleton
# ---------------------------------------------------------------------------

_router: EvalRouter | None = None
_router_lock: threading.Lock = threading.Lock()


def get_eval_router() -> EvalRouter:
    """Return the module-level singleton EvalRouter, creating it if needed."""
    global _router
    with _router_lock:
        if _router is None:
            _router = EvalRouter()
    return _router


def reset_eval_router() -> None:
    """Replace the module-level singleton with a fresh EvalRouter instance."""
    global _router
    with _router_lock:
        _router = EvalRouter()
