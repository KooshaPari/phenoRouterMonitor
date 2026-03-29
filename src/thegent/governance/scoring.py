"""Provider scoring system for economic governance (WP-5003).

Normalizes provider performance metrics (reliability, latency, cost) into
a composite score (0-10 scale) for routing decisions.

See: docs/changes/research-economic-governance/design.md § 2.1
"""

from __future__ import annotations

import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field


@dataclass
class ProviderMetrics:
    """Measured provider performance metrics.

    Attributes:
        provider_id: Unique provider identifier
        reliability: Uptime/success rate (0.0-1.0)
        latency_p99: 99th percentile latency in milliseconds
        cost_per_1m_tokens: USD cost per million tokens
        last_updated: Unix timestamp of last metric update
        sample_size: Number of measurements in this score
    """

    provider_id: str
    reliability: float  # 0.0-1.0
    latency_p99: float  # milliseconds
    cost_per_1m_tokens: float  # USD
    last_updated: float = field(default_factory=time.time)
    sample_size: int = 1000


@dataclass
class ProviderScore:
    """Normalized provider performance score.

    All component scores and composite are 0-10 scale.

    Attributes:
        provider_id: Provider identifier
        reliability_score: Normalized reliability (0-10)
        latency_score: Normalized latency; lower latency = higher score (0-10)
        cost_score: Normalized cost; lower cost = higher score (0-10)
        composite_score: Weighted average of components (0-10)
        timestamp: Unix timestamp of score calculation
    """

    provider_id: str
    reliability_score: float  # 0-10
    latency_score: float  # 0-10
    cost_score: float  # 0-10
    composite_score: float  # 0-10
    timestamp: float = field(default_factory=time.time)


class ProviderScorer(ABC):
    """Abstract base class for provider scoring strategies.

    Implementers must normalize metrics to 0-10 scale and compute
    composite scores for routing decisions.
    """

    @abstractmethod
    def score(self, metrics: ProviderMetrics) -> ProviderScore:
        """Compute normalized score from provider metrics.

        Args:
            metrics: Provider performance metrics

        Returns:
            Normalized provider score (0-10 scale)
        """

    @abstractmethod
    def normalize(self, raw_value: float, metric_type: str) -> float:
        """Normalize raw metric value to 0-10 scale.

        Args:
            raw_value: Raw metric value
            metric_type: Type of metric ("reliability", "latency", or "cost")

        Returns:
            Normalized score (0-10 scale)
        """


class DefaultProviderScorer(ProviderScorer):
    """Standard provider scorer with configurable weights.

    Weights (sum to 1.0):
        - Reliability: 0.4 (40%) — prioritizes uptime
        - Latency: 0.2 (20%) — response speed
        - Cost: 0.4 (40%) — economic efficiency

    Normalization baselines:
        - Baseline latency: 250ms = score 5.0
        - Baseline cost: $0.15/1M tokens = score 5.0

    Higher baseline = lower score. E.g., latency of 500ms (2× baseline)
    yields score ~3.3, while latency of 100ms (0.4× baseline) yields ~8.0.
    """

    # Weight configuration
    RELIABILITY_WEIGHT = 0.4
    LATENCY_WEIGHT = 0.2
    COST_WEIGHT = 0.4

    # Normalization baselines (metric values that correspond to score 5.0)
    BASELINE_LATENCY_MS = 250.0
    BASELINE_COST_PER_1M = 0.15

    def score(self, metrics: ProviderMetrics) -> ProviderScore:
        """Compute composite score from metrics.

        Normalizes each metric to 0-10 scale, then computes weighted average.

        Args:
            metrics: Provider metrics

        Returns:
            Provider score with reliability, latency, cost, and composite components
        """
        reliability_score = self._normalize_reliability(metrics.reliability)
        latency_score = self._normalize_latency(metrics.latency_p99)
        cost_score = self._normalize_cost(metrics.cost_per_1m_tokens)

        composite = (
            reliability_score * self.RELIABILITY_WEIGHT
            + latency_score * self.LATENCY_WEIGHT
            + cost_score * self.COST_WEIGHT
        )

        return ProviderScore(
            provider_id=metrics.provider_id,
            reliability_score=reliability_score,
            latency_score=latency_score,
            cost_score=cost_score,
            composite_score=composite,
            timestamp=time.time(),
        )

    def normalize(self, raw_value: float, metric_type: str) -> float:
        """Normalize metric to 0-10 scale.

        Args:
            raw_value: Raw metric value
            metric_type: "reliability", "latency", or "cost"

        Returns:
            Normalized score (0-10)

        Raises:
            ValueError: If metric_type is unknown
        """
        metric_type = metric_type.lower()
        if metric_type == "reliability":
            return self._normalize_reliability(raw_value)
        if metric_type == "latency":
            return self._normalize_latency(raw_value)
        if metric_type == "cost":
            return self._normalize_cost(raw_value)
        raise ValueError(f"Unknown metric type: {metric_type}")

    def _normalize_reliability(self, reliability: float) -> float:
        """Normalize reliability (0.0-1.0) to 0-10 scale.

        Linear mapping: score = reliability × 10

        Args:
            reliability: Uptime/success rate (0.0-1.0)

        Returns:
            Score (0-10), clamped to valid range
        """
        score = reliability * 10.0
        return max(0.0, min(10.0, score))

    def _normalize_latency(self, latency_ms: float) -> float:
        """Normalize latency (milliseconds) to 0-10 scale.

        Inverse relationship: lower latency = higher score.
        Uses sigmoid-like curve:
            score = 10 / (1 + (ratio - 1.0) × 0.5)
        where ratio = latency / baseline

        Examples:
            - 250ms (baseline): score 5.0
            - 100ms (0.4× baseline): score ~8.0
            - 500ms (2× baseline): score ~3.3

        Args:
            latency_ms: Latency in milliseconds

        Returns:
            Score (0-10), clamped to valid range
        """
        if latency_ms < 0:
            return 10.0

        ratio = latency_ms / self.BASELINE_LATENCY_MS
        score = 10.0 / (1.0 + (ratio - 1.0) * 0.5)
        return max(0.0, min(10.0, score))

    def _normalize_cost(self, cost: float) -> float:
        """Normalize cost (USD per 1M tokens) to 0-10 scale.

        Inverse relationship: lower cost = higher score.
        Uses sigmoid-like curve (same as latency):
            score = 10 / (1 + (ratio - 1.0) × 0.5)
        where ratio = cost / baseline

        Examples:
            - $0.15/1M (baseline): score 5.0
            - $0.06/1M (0.4× baseline): score ~8.0
            - $0.30/1M (2× baseline): score ~3.3

        Args:
            cost: Cost in USD per million tokens

        Returns:
            Score (0-10), clamped to valid range
        """
        if cost < 0:
            return 10.0

        ratio = cost / self.BASELINE_COST_PER_1M
        score = 10.0 / (1.0 + (ratio - 1.0) * 0.5)
        return max(0.0, min(10.0, score))
