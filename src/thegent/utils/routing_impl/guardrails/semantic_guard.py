from __future__ import annotations

"""GW-55: Semantic prompt guard — embedding similarity vs reference prompts.

Blocks requests whose embedding is too similar to known harmful/forbidden prompts.
Uses the same EmbeddingProvider protocol as semantic_cache.py.

# @trace FR-GUARD-055
"""

import logging
from dataclasses import dataclass, field

from thegent.utils.routing_impl.semantic_cache import EmbeddingProvider, NumpyEmbeddingProvider, cosine_similarity

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class SemanticGuardConfig:
    threshold: float = 0.85  # block if similarity >= threshold
    reference_prompts: list[str] = field(default_factory=list)
    enabled: bool = True


@dataclass
class SemanticGuardResult:
    blocked: bool
    similarity: float  # highest similarity found
    matched_reference: str  # which reference prompt matched (or "")


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def check_semantic_guard(
    text: str,
    config: SemanticGuardConfig,
    provider: EmbeddingProvider | None = None,
) -> SemanticGuardResult:
    """Check text similarity against reference prompts.

    If provider is None, uses NumpyEmbeddingProvider (deterministic seeded).
    Returns blocked=False if config.reference_prompts is empty.
    """
    if not config.enabled:
        _log.debug("check_semantic_guard: disabled — returning not blocked")
        return SemanticGuardResult(blocked=False, similarity=0.0, matched_reference="")

    if not config.reference_prompts:
        _log.debug("check_semantic_guard: no reference prompts — returning not blocked")
        return SemanticGuardResult(blocked=False, similarity=0.0, matched_reference="")

    active_provider: EmbeddingProvider = provider if provider is not None else NumpyEmbeddingProvider()

    text_embedding = active_provider.embed(text)

    best_similarity = -1.0
    best_reference = ""

    for ref_prompt in config.reference_prompts:
        ref_embedding = active_provider.embed(ref_prompt)
        sim = cosine_similarity(text_embedding, ref_embedding)
        _log.debug(
            "check_semantic_guard: similarity=%.4f threshold=%.4f ref=%r",
            sim,
            config.threshold,
            ref_prompt[:60],
        )
        if sim > best_similarity:
            best_similarity = sim
            best_reference = ref_prompt

    blocked = best_similarity >= config.threshold

    _log.debug(
        "check_semantic_guard: blocked=%s best_similarity=%.4f threshold=%.4f",
        blocked,
        best_similarity,
        config.threshold,
    )

    return SemanticGuardResult(
        blocked=blocked,
        similarity=best_similarity,
        matched_reference=best_reference if blocked else "",
    )
