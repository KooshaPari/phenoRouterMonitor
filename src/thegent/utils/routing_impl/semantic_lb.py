"""GW-61: Semantic load balancing — route to model best matching prompt.

Computes embedding similarity between the request prompt and pre-registered
model capability descriptions. Routes to the highest-similarity model.

Uses the same EmbeddingProvider protocol as semantic_cache.py.

# @trace FR-AROUTE-061
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field

from thegent.utils.routing_impl.semantic_cache import EmbeddingProvider, NumpyEmbeddingProvider, cosine_similarity

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Public data structures
# ---------------------------------------------------------------------------


@dataclass
class ModelCapability:
    """A model capability description used for semantic routing."""

    model: str  # model identifier e.g. "gpt-4o"
    description: str  # capability description e.g. "Best for coding tasks..."
    provider: str = ""  # optional provider hint
    tags: list[str] = field(default_factory=list)  # e.g. ["coding", "reasoning"]


@dataclass
class SemanticLbResult:
    """Result of a semantic load balancer routing decision."""

    selected_model: str  # winning model
    similarity: float  # similarity score of winner
    provider: str  # provider of winning model (may be "")
    scores: dict[str, float]  # all model → similarity scores


# ---------------------------------------------------------------------------
# SemanticLoadBalancer
# ---------------------------------------------------------------------------


class SemanticLoadBalancer:
    """Maintains a registry of model capabilities and routes by similarity."""

    def __init__(
        self,
        capabilities: list[ModelCapability],
        provider: EmbeddingProvider | None = None,
        min_similarity: float = 0.0,
    ) -> None:
        self._capabilities: list[ModelCapability] = list(capabilities)
        self._provider: EmbeddingProvider = provider or NumpyEmbeddingProvider()
        self._min_similarity = min_similarity
        # Pre-compute embeddings for all capability descriptions
        self._desc_embeddings: dict[str, list[float]] = {}
        for cap in self._capabilities:
            self._desc_embeddings[cap.model] = self._provider.embed(cap.description)

    def route(self, prompt: str) -> SemanticLbResult | None:
        """Select the model most similar to the prompt.

        Returns None if capabilities list is empty or all scores below min_similarity.
        """
        if not self._capabilities:
            _log.debug("SemanticLoadBalancer: no capabilities registered, returning None")
            return None

        prompt_embedding = self._provider.embed(prompt)

        scores: dict[str, float] = {}
        for cap in self._capabilities:
            desc_emb = self._desc_embeddings.get(cap.model)
            if desc_emb is None:
                desc_emb = self._provider.embed(cap.description)
                self._desc_embeddings[cap.model] = desc_emb
            scores[cap.model] = cosine_similarity(prompt_embedding, desc_emb)

        best_model = max(scores, key=lambda m: scores[m])
        best_score = scores[best_model]

        if best_score < self._min_similarity:
            _log.debug(
                "SemanticLoadBalancer: best score %.4f below min_similarity %.4f, returning None",
                best_score,
                self._min_similarity,
            )
            return None

        best_cap = next(c for c in self._capabilities if c.model == best_model)
        _log.debug(
            "SemanticLoadBalancer: selected model=%r score=%.4f provider=%r",
            best_model,
            best_score,
            best_cap.provider,
        )
        return SemanticLbResult(
            selected_model=best_model,
            similarity=best_score,
            provider=best_cap.provider,
            scores=scores,
        )

    def add_capability(self, capability: ModelCapability) -> None:
        """Register a new model capability."""
        self._capabilities.append(capability)
        self._desc_embeddings[capability.model] = self._provider.embed(capability.description)
        _log.debug("SemanticLoadBalancer: added capability model=%r", capability.model)

    def get_capabilities(self) -> list[ModelCapability]:
        """Return current list of capabilities."""
        return list(self._capabilities)


# ---------------------------------------------------------------------------
# Convenience function
# ---------------------------------------------------------------------------


def semantic_route(
    prompt: str,
    capabilities: list[ModelCapability],
    provider: EmbeddingProvider | None = None,
    min_similarity: float = 0.0,
) -> SemanticLbResult | None:
    """Convenience: create a one-shot SemanticLoadBalancer and route."""
    lb = SemanticLoadBalancer(
        capabilities=capabilities,
        provider=provider,
        min_similarity=min_similarity,
    )
    return lb.route(prompt)
