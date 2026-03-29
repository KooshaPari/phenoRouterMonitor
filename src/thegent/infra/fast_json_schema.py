"""Fast JSON schema validator with optimized backends.

This module provides a high-performance abstraction layer for JSON schema validation
that automatically selects the fastest available backend:
- fastjsonschema: 2-3x faster than jsonschema
- jsonschema: Standard fallback

Performance improvements:
- fastjsonschema compiles schemas to Python code (2-3x faster)
- Automatic backend selection based on availability
- Cached compiled schemas for repeated validation
"""

from collections.abc import Callable
from typing import Any, cast

# Library-first (LIBRARY_FIRST_POLICY.md): Using cachetools.LRUCache
from cachetools import LRUCache

try:
    import fastjsonschema

    FASTJSONSCHEMA_AVAILABLE = True
except ImportError:
    FASTJSONSCHEMA_AVAILABLE = False

try:
    import jsonschema

    JSONSCHEMA_AVAILABLE = True
except ImportError:
    JSONSCHEMA_AVAILABLE = False


class FastJSONSchemaValidator:
    """High-performance JSON schema validator with automatic backend selection.

    Backend priority (fastest first):
    1. fastjsonschema (if installed) - 2-3x faster, compiles schemas to Python
    2. jsonschema (standard fallback) - baseline performance
    """

    def __init__(self, schema: dict[str, Any]) -> None:
        """Initialize validator with a schema.

        Args:
            schema: JSON schema dictionary
        """
        self.schema = schema
        self._backend = None
        self._validator = None
        self._compiled_validator: Callable[[Any], None] | None = None

        # Select backend based on availability
        if FASTJSONSCHEMA_AVAILABLE:
            self._backend = "fastjsonschema"
            # Compile schema for fast validation
            self._compiled_validator = cast("Callable[[Any], None]", fastjsonschema.compile(schema))
        elif JSONSCHEMA_AVAILABLE:
            self._backend = "jsonschema"
            self._validator = jsonschema.Draft202012Validator(schema)
        else:
            raise ImportError("No JSON schema validator available. Install fastjsonschema or jsonschema")

    def validate(self, instance: Any) -> None:
        """Validate instance against schema.

        Args:
            instance: Data to validate

        Raises:
            ValidationError: If validation fails
        """
        if self._backend == "fastjsonschema":
            # fastjsonschema raises ValueError on validation failure
            assert self._compiled_validator is not None
            try:
                self._compiled_validator(instance)
            except ValueError as e:
                # Convert to jsonschema-like exception for compatibility
                if JSONSCHEMA_AVAILABLE:
                    raise jsonschema.ValidationError(str(e)) from e
                raise
        elif self._backend == "jsonschema":
            self._validator.validate(instance)
        else:
            raise RuntimeError(f"Unknown backend: {self._backend}")

    def is_valid(self, instance: Any) -> bool:
        """Check if instance is valid without raising exception.

        Args:
            instance: Data to validate

        Returns:
            True if valid, False otherwise
        """
        try:
            self.validate(instance)
            return True
        except Exception:
            return False

    @property
    def backend(self) -> str:
        """Get current backend name."""
        return self._backend or "unknown"


# Global schema cache (Library-first: using cachetools.LRUCache for LRU eviction)
_schema_cache: LRUCache[str, FastJSONSchemaValidator] = LRUCache(maxsize=50)


def get_schema_validator(schema: dict[str, Any], cache_key: str | None = None) -> FastJSONSchemaValidator:
    """Get or create a schema validator (with caching).

    Args:
        schema: JSON schema dictionary
        cache_key: Optional cache key (uses schema hash if not provided)

    Returns:
        FastJSONSchemaValidator instance
    """
    if cache_key is None:
        import hashlib

        from thegent.infra.runtime_dispatcher import get_json_dumps

        json_dumps = get_json_dumps()
        schema_str = json_dumps(schema, sort_keys=True)
        cache_key = hashlib.sha256(schema_str.encode()).hexdigest()

    if cache_key not in _schema_cache:
        _schema_cache[cache_key] = FastJSONSchemaValidator(schema)

    return _schema_cache[cache_key]


def validate_json_schema(instance: Any, schema: dict[str, Any], cache_key: str | None = None) -> None:
    """Validate instance against schema using fastest available backend.

    Args:
        instance: Data to validate
        schema: JSON schema dictionary
        cache_key: Optional cache key for schema caching

    Raises:
        ValidationError: If validation fails
    """
    validator = get_schema_validator(schema, cache_key)
    validator.validate(instance)


def is_valid_json_schema(instance: Any, schema: dict[str, Any], cache_key: str | None = None) -> bool:
    """Check if instance is valid against schema.

    Args:
        instance: Data to validate
        schema: JSON schema dictionary
        cache_key: Optional cache key for schema caching

    Returns:
        True if valid, False otherwise
    """
    validator = get_schema_validator(schema, cache_key)
    return validator.is_valid(instance)
