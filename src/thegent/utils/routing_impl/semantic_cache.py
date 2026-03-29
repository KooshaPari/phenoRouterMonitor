"""GW-23: Semantic cache for LLM gateway using embedding similarity.

Caches LLM responses and retrieves them by semantic similarity of the
request prompt. Uses cosine similarity with a configurable threshold
(default: 0.95).

When the embedding model is unavailable, semantic cache degrades to
disabled (never hits) — the caller falls through to a live LLM call.

# @trace FR-CACHE-023
"""

from __future__ import annotations

import importlib.util
import logging
import threading
import time
import uuid
from dataclasses import dataclass
from typing import Any, Protocol, runtime_checkable

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# cosine_similarity helper
# ---------------------------------------------------------------------------


def cosine_similarity(a: list[float], b: list[float]) -> float:
    """Compute cosine similarity between two embedding vectors."""
    dot = sum(x * y for x, y in zip(a, b))
    norm_a = sum(x * x for x in a) ** 0.5
    norm_b = sum(x * x for x in b) ** 0.5
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return dot / (norm_a * norm_b)


# ---------------------------------------------------------------------------
# extract_prompt_text helper
# ---------------------------------------------------------------------------


def extract_prompt_text(messages: list[dict]) -> str:
    """Extract plain text from messages list for embedding."""
    parts = []
    for msg in messages:
        content = msg.get("content", "")
        if isinstance(content, str):
            parts.append(content)
        elif isinstance(content, list):
            for block in content:
                if isinstance(block, dict) and block.get("type") == "text":
                    parts.append(block.get("text", ""))
    return " ".join(parts).strip()


# ---------------------------------------------------------------------------
# EmbeddingProvider Protocol
# ---------------------------------------------------------------------------


@runtime_checkable
class EmbeddingProvider(Protocol):
    """Protocol for embedding providers."""

    def embed(self, text: str) -> list[float]:
        """Return the embedding vector for text."""
        ...

    def is_available(self) -> bool:
        """Return True if this provider can produce embeddings."""
        ...


# ---------------------------------------------------------------------------
# SentenceTransformerProvider
# ---------------------------------------------------------------------------


class SentenceTransformerProvider:
    """Embedding provider backed by sentence-transformers.

    Lazy-loads the sentence_transformers package. If the package is not
    installed, is_available() returns False and embed() raises ImportError.
    """

    def __init__(self, model_name: str = "all-MiniLM-L6-v2") -> None:
        self._model_name = model_name
        self._model: Any = None
        self._available: bool | None = None  # None = not yet checked

    def is_available(self) -> bool:
        """Return True if sentence_transformers is importable."""
        if self._available is None:
            self._available = importlib.util.find_spec("sentence_transformers") is not None
            if not self._available:
                _log.debug("sentence_transformers not available; SentenceTransformerProvider disabled")
        return self._available

    def _load_model(self) -> Any:
        """Lazy-load and cache the SentenceTransformer model."""
        if self._model is None:
            from sentence_transformers import SentenceTransformer  # type: ignore[import]

            self._model = SentenceTransformer(self._model_name)
            _log.debug("Loaded SentenceTransformer model=%s", self._model_name)
        return self._model

    def embed(self, text: str) -> list[float]:
        """Return the embedding vector for text as a list of floats."""
        model = self._load_model()
        result = model.encode(text, convert_to_numpy=True)
        return result.tolist()


# ---------------------------------------------------------------------------
# NumpyEmbeddingProvider (deterministic, for testing)
# ---------------------------------------------------------------------------


class NumpyEmbeddingProvider:
    """Deterministic embedding provider using numpy random unit vectors.

    Seeds the RNG from the hash of the input text, producing a consistent
    unit vector per unique string. Useful in tests when sentence_transformers
    is not installed.
    """

    def __init__(self, dim: int = 384) -> None:
        self._dim = dim

    def is_available(self) -> bool:
        """Return True if numpy is importable."""
        return importlib.util.find_spec("numpy") is not None

    def embed(self, text: str) -> list[float]:
        """Return a deterministic unit vector for text."""
        import numpy as np  # type: ignore[import]

        seed = abs(hash(text)) % (2**31)
        rng = np.random.default_rng(seed)
        vec = rng.standard_normal(self._dim)
        norm = np.linalg.norm(vec)
        if norm == 0:
            vec = np.ones(self._dim)
            norm = np.linalg.norm(vec)
        return (vec / norm).tolist()


# ---------------------------------------------------------------------------
# SemanticCacheConfig
# ---------------------------------------------------------------------------


@dataclass
class SemanticCacheConfig:
    """Configuration for SemanticCache."""

    similarity_threshold: float = 0.95
    max_entries: int = 500
    embedding_dim: int = 384  # all-MiniLM-L6-v2 output dim
    ttl: float = 3600.0
    namespace: str = "default"


# ---------------------------------------------------------------------------
# SemanticCacheEntry
# ---------------------------------------------------------------------------


@dataclass
class SemanticCacheEntry:
    """A single entry in the semantic cache."""

    key: str
    text: str
    embedding: list[float]
    response: dict[str, Any]
    created_at: float
    ttl: float
    namespace: str

    @property
    def is_expired(self) -> bool:
        """Return True if this entry has passed its TTL."""
        return (time.monotonic() - self.created_at) > self.ttl


# ---------------------------------------------------------------------------
# SemanticCache
# ---------------------------------------------------------------------------


class SemanticCache:
    """In-memory semantic cache keyed by namespace.

    Stores LLM responses indexed by their prompt embedding. Lookup
    computes cosine similarity and returns a hit when the nearest
    neighbor exceeds the configured threshold.

    Thread-safe via a single threading.Lock.
    """

    def __init__(
        self,
        config: SemanticCacheConfig | None = None,
        provider: EmbeddingProvider | None = None,
    ) -> None:
        self._config = config or SemanticCacheConfig()
        self._provider: EmbeddingProvider = provider or SentenceTransformerProvider()
        # Storage: namespace -> list of SemanticCacheEntry
        self._store: dict[str, list[SemanticCacheEntry]] = {}
        self._lock = threading.Lock()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get(self, text: str, namespace: str = "default") -> dict[str, Any] | None:
        """Look up a semantically similar cached response.

        Returns None on miss, on expired-only matches, or when the
        embedding provider is unavailable.
        """
        if not self._provider.is_available():
            _log.debug("Semantic cache: provider unavailable; returning None")
            return None

        query_embedding = self._provider.embed(text)

        with self._lock:
            entries = self._store.get(namespace, [])
            best_score = -1.0
            best_entry: SemanticCacheEntry | None = None

            for entry in entries:
                if entry.is_expired:
                    continue
                score = cosine_similarity(query_embedding, entry.embedding)
                if score > best_score:
                    best_score = score
                    best_entry = entry

        _log.debug(
            "Semantic cache lookup namespace=%s best_score=%.4f threshold=%.4f",
            namespace,
            best_score,
            self._config.similarity_threshold,
        )

        if best_entry is not None and best_score >= self._config.similarity_threshold:
            _log.debug("Semantic cache HIT namespace=%s score=%.4f", namespace, best_score)
            return best_entry.response

        return None

    def set(
        self,
        text: str,
        response: dict[str, Any],
        namespace: str = "default",
    ) -> None:
        """Store a response in the semantic cache.

        No-op when the embedding provider is unavailable.
        Evicts the oldest entries when max_entries is exceeded.
        """
        if not self._provider.is_available():
            _log.debug("Semantic cache: provider unavailable; skipping set")
            return

        embedding = self._provider.embed(text)
        entry = SemanticCacheEntry(
            key=str(uuid.uuid4()),
            text=text,
            embedding=embedding,
            response=response,
            created_at=time.monotonic(),
            ttl=self._config.ttl,
            namespace=namespace,
        )

        with self._lock:
            if namespace not in self._store:
                self._store[namespace] = []
            self._store[namespace].append(entry)

            # Evict oldest entries if over the limit
            overage = len(self._store[namespace]) - self._config.max_entries
            if overage > 0:
                self._store[namespace] = self._store[namespace][overage:]
                _log.debug(
                    "Semantic cache evicted %d oldest entries namespace=%s",
                    overage,
                    namespace,
                )

        _log.debug("Semantic cache SET namespace=%s key=%s", namespace, entry.key)

    def delete_expired(self, namespace: str | None = None) -> int:
        """Remove expired entries from the given namespace (or all namespaces).

        Returns the count of entries removed.
        """
        removed = 0
        with self._lock:
            namespaces = [namespace] if namespace is not None else list(self._store.keys())
            for ns in namespaces:
                if ns not in self._store:
                    continue
                before = len(self._store[ns])
                self._store[ns] = [e for e in self._store[ns] if not e.is_expired]
                removed += before - len(self._store[ns])
        _log.debug("Semantic cache delete_expired removed=%d", removed)
        return removed

    def clear(self, namespace: str | None = None) -> int:
        """Clear all entries from the given namespace (or all namespaces).

        Returns the count of entries removed.
        """
        removed = 0
        with self._lock:
            if namespace is not None:
                removed = len(self._store.get(namespace, []))
                self._store.pop(namespace, None)
            else:
                removed = sum(len(v) for v in self._store.values())
                self._store.clear()
        _log.debug("Semantic cache clear namespace=%s removed=%d", namespace, removed)
        return removed


# ---------------------------------------------------------------------------
# Singleton
# ---------------------------------------------------------------------------

_semantic_cache: SemanticCache | None = None
_semantic_cache_lock = threading.Lock()


def get_semantic_cache(config: SemanticCacheConfig | None = None) -> SemanticCache:
    """Return the process-global SemanticCache singleton, creating it if needed."""
    global _semantic_cache
    with _semantic_cache_lock:
        if _semantic_cache is None:
            _semantic_cache = SemanticCache(config=config)
    return _semantic_cache


def reset_semantic_cache() -> None:
    """Reset the process-global singleton (for testing)."""
    global _semantic_cache
    with _semantic_cache_lock:
        _semantic_cache = None


# ---------------------------------------------------------------------------
# Convenience functions
# ---------------------------------------------------------------------------


def semantic_cache_get(text: str, namespace: str = "default") -> dict[str, Any] | None:
    """Look up a semantically similar cached response. Returns None on miss."""
    return get_semantic_cache().get(text, namespace=namespace)


def semantic_cache_set(
    text: str,
    response: dict[str, Any],
    namespace: str = "default",
) -> None:
    """Store a response in the semantic cache."""
    get_semantic_cache().set(text, response, namespace=namespace)
