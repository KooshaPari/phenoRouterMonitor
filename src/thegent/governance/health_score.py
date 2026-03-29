"""Composite health score model (0-100) for autonomous codebase governance.

Replaces XP/gamification with a weighted, multi-dimensional health metric.
Each dimension (test coverage, lint violations, etc.) is normalized against
targets defined in contracts/health-targets.json and combined into a single
score that drives autonomous agent scheduling decisions.
"""

import json
import logging
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path

from pydantic import BaseModel, Field

from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)

# BKM-05: Native SHM support
HAS_NATIVE_SHM = False
if ThegentSettings().use_native_shm:
    try:
        import thegent_shm  # type: ignore[import-untyped]

        thegent_shm.py_init_shm()  # type: ignore[call-arg]
        HAS_NATIVE_SHM = True
    except ImportError:
        pass

_TREND_IMPROVING_THRESHOLD = 2.0
_TREND_DEGRADING_THRESHOLD = -2.0


class HealthBand(StrEnum):
    """Health classification bands derived from composite score."""

    EXCELLENT = "excellent"
    HEALTHY = "healthy"
    WARNING = "warning"
    CRITICAL = "critical"


def get_band(score: float) -> HealthBand:
    """Return the appropriate HealthBand for a numeric score (0-100)."""
    if score >= 90:
        return HealthBand.EXCELLENT
    if score >= 70:
        return HealthBand.HEALTHY
    if score >= 40:
        return HealthBand.WARNING
    return HealthBand.CRITICAL


def _band_from_normalized(normalized: float) -> HealthBand:
    """Derive a per-dimension status band from its normalized value (0.0-1.0)."""
    if normalized >= 0.9:
        return HealthBand.EXCELLENT
    if normalized >= 0.7:
        return HealthBand.HEALTHY
    if normalized >= 0.4:
        return HealthBand.WARNING
    return HealthBand.CRITICAL


class DimensionScore(BaseModel):
    """Score for a single health dimension with normalization against target."""

    name: str
    weight: float = Field(ge=0.0, le=1.0)
    raw_value: float
    normalized: float = Field(ge=0.0, le=1.0)
    target: float
    direction: str
    status: HealthBand


class HealthScore(BaseModel):
    """Composite health score aggregating all governance dimensions."""

    score: float = Field(ge=0.0, le=100.0)
    dimensions: dict[str, DimensionScore]
    band: HealthBand
    trend: str
    computed_at: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    cycle_id: str | None = None


class HealthScoreComputer:
    """Computes composite health scores from raw dimension measurements.

    Loads dimension definitions (weights, targets, directions) from the
    contracts/health-targets.json file and normalizes incoming raw values
    into a weighted 0-100 composite score.
    """

    def __init__(self, health_targets_path: Path) -> None:
        with open(health_targets_path) as fh:
            data = json.load(fh)
        self._dimensions: dict[str, dict] = data["dimensions"]
        _log.debug(
            "loaded %d health dimensions from %s",
            len(self._dimensions),
            health_targets_path,
        )

    def _normalize(self, raw_value: float, target: float, direction: str) -> float:
        """Normalize a raw measurement to 0.0-1.0 against its target."""
        if direction == "higher_is_better":
            if target <= 0:
                return 1.0 if raw_value >= 0 else 0.0
            return min(1.0, raw_value / target)

        # lower_is_better
        if target == 0:
            if raw_value == 0:
                return 1.0
            return max(0.0, 1.0 - raw_value / 10.0)
        return min(1.0, max(0.0, 1.0 - (raw_value / target)))

    def compute(self, dimension_values: dict[str, float]) -> HealthScore:
        """Compute a health score from raw dimension measurements.

        Args:
            dimension_values: mapping of dimension name to raw measured value.
                Dimensions not present in the dict default to their worst case
                (0 for higher_is_better, target*2 for lower_is_better).

        Returns:
            A fully populated HealthScore.
        """
        dimensions: dict[str, DimensionScore] = {}
        weighted_sum = 0.0

        for dim_name, dim_cfg in self._dimensions.items():
            weight = dim_cfg["weight"]
            target = dim_cfg["target"]
            direction = dim_cfg["direction"]

            raw_value = dimension_values.get(dim_name, self._default_raw(target, direction))
            normalized = self._normalize(raw_value, target, direction)

            dimensions[dim_name] = DimensionScore(
                name=dim_name,
                weight=weight,
                raw_value=raw_value,
                normalized=normalized,
                target=target,
                direction=direction,
                status=_band_from_normalized(normalized),
            )
            weighted_sum += weight * normalized

        score = round(weighted_sum * 100, 2)
        if HAS_NATIVE_SHM:
            try:
                import thegent_shm  # type: ignore[import-untyped]

                thegent_shm.set_health_score(score)
            except Exception as e:
                _log.warning("failed to write health score to SHM: %s", e)
        return HealthScore(
            score=score,
            dimensions=dimensions,
            band=get_band(score),
            trend="stable",
        )

    def compute_with_trend(
        self,
        dimension_values: dict[str, float],
        previous_score: float | None,
    ) -> HealthScore:
        """Compute health score and derive trend from comparison with previous score."""
        result = self.compute(dimension_values)
        if previous_score is not None:
            delta = result.score - previous_score
            if delta >= _TREND_IMPROVING_THRESHOLD:
                trend = "improving"
            elif delta <= _TREND_DEGRADING_THRESHOLD:
                trend = "degrading"
            else:
                trend = "stable"
            result = result.model_copy(update={"trend": trend})
        return result

    @staticmethod
    def _default_raw(target: float, direction: str) -> float:
        """Return worst-case default when a dimension value is missing."""
        if direction == "higher_is_better":
            return 0.0
        return target * 2 if target > 0 else 10.0
