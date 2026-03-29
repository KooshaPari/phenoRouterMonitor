"""Health score calculator for thegent project governance."""

import json
from datetime import UTC
from pathlib import Path
from typing import TypedDict


class DimensionScore(TypedDict):
    """Individual dimension score."""

    dimension: str
    weight: float
    target: int | float
    actual: int | float
    direction: str
    score: float
    status: str


class HealthReport(TypedDict):
    """Overall health report."""

    version: str
    overall_score: float
    status: str
    dimensions: list[DimensionScore]
    timestamp: str


class HealthScorer:
    """Calculates project health based on defined targets."""

    def __init__(self, targets_file: Path | str) -> None:
        """Load health targets configuration.

        Args:
            targets_file: Path to health-targets.json
        """
        self.targets_file = Path(targets_file)
        if not self.targets_file.exists():
            raise FileNotFoundError(f"Health targets file not found: {targets_file}")

        with open(self.targets_file) as f:
            self.config = json.load(f)

        self.dimensions = self.config.get("dimensions", {})
        self.bands = self.config.get("bands", {})

    def normalize_score(self, actual: float, target: float, direction: str) -> float:
        """Normalize a score to 0-100 scale.

        Args:
            actual: Actual measured value
            target: Target value
            direction: "higher_is_better" or "lower_is_better"

        Returns:
            Normalized score (0-100)
        """
        if target == 0:
            return 100.0 if actual == 0 else 0.0

        if direction == "higher_is_better":
            # For higher-is-better: actual/target * 100, capped at 100
            return min(100.0, (actual / target) * 100) if actual >= 0 else 0.0
        # For lower-is-better: (1 - actual/target) * 100, capped at 100
        score = (1 - actual / target) * 100
        return min(100.0, max(0.0, score))

    def dimension_status(self, score: float) -> str:
        """Get status label for a score.

        Args:
            score: Score 0-100

        Returns:
            Status label (excellent, healthy, warning, critical)
        """
        bands = sorted(self.bands.items(), key=lambda x: x[1]["min"], reverse=True)
        for band_name, band_config in bands:
            if score >= band_config["min"]:
                return band_name
        return "critical"

    def score_dimension(self, dimension_key: str, actual: float) -> DimensionScore:
        """Score a single dimension.

        Args:
            dimension_key: Dimension ID (e.g., "test_coverage")
            actual: Actual measured value

        Returns:
            Dimension score details
        """
        if dimension_key not in self.dimensions:
            raise ValueError(f"Unknown dimension: {dimension_key}")

        dim = self.dimensions[dimension_key]
        target = dim.get("target", 0)
        direction = dim.get("direction", "higher_is_better")
        weight = dim.get("weight", 0.0)

        score = self.normalize_score(actual, target, direction)
        status = self.dimension_status(score)

        return DimensionScore(
            dimension=dimension_key,
            weight=weight,
            target=target,
            actual=actual,
            direction=direction,
            score=score,
            status=status,
        )

    def calculate_overall(self, scores: list[DimensionScore]) -> float:
        """Calculate weighted overall score.

        Args:
            scores: List of dimension scores

        Returns:
            Overall weighted score (0-100)
        """
        if not scores:
            return 0.0

        total_weight = sum(s["weight"] for s in scores)
        if total_weight == 0:
            return 0.0

        weighted_sum = sum(s["score"] * s["weight"] for s in scores)
        return weighted_sum / total_weight

    def generate_report(self, measurements: dict[str, float]) -> HealthReport:
        """Generate a full health report.

        Args:
            measurements: Dict mapping dimension keys to actual values

        Returns:
            Complete health report
        """
        from datetime import datetime

        scores = []
        for dim_key, actual_value in measurements.items():
            if dim_key in self.dimensions:
                scores.append(self.score_dimension(dim_key, actual_value))

        overall = self.calculate_overall(scores)
        status = self.dimension_status(overall)

        return HealthReport(
            version=self.config.get("version", "1.0.0"),
            overall_score=round(overall, 1),
            status=status,
            dimensions=scores,
            timestamp=datetime.now(UTC).isoformat(),
        )
