"""Fast TOML parser with optimized backends.

This module provides a high-performance abstraction layer for TOML parsing
that automatically selects the fastest available backend:
- rtoml (Rust-based): 10-20x faster than tomlkit
- tomli/tomli-w (Python 3.11+): 3-5x faster for reading
- tomlkit: Standard fallback (good for editing)

Performance improvements:
- rtoml uses Rust implementation (10-20x faster)
- tomli optimized pure-Python (3-5x faster)
- Automatic backend selection based on availability and use case
"""

from pathlib import Path
from typing import Any

try:
    import rtoml

    RTOML_AVAILABLE = True
except ImportError:
    RTOML_AVAILABLE = False

try:
    import tomli

    TOMLI_AVAILABLE = True
except ImportError:
    TOMLI_AVAILABLE = False

try:
    import tomli_w

    TOMLI_W_AVAILABLE = True
except ImportError:
    TOMLI_W_AVAILABLE = False

try:
    import tomlkit

    TOMLKIT_AVAILABLE = True
except ImportError:
    TOMLKIT_AVAILABLE = False


class FastTOMLParser:
    """High-performance TOML parser with automatic backend selection.

    Backend priority (fastest first):
    1. rtoml (if installed) - 10-20x faster, Rust-based
    2. tomli/tomli-w (if installed) - 3-5x faster, pure-Python
    3. tomlkit (standard fallback) - good for editing, slower for reading
    """

    def __init__(self, edit_mode: bool = False) -> None:
        """Initialize TOML parser.

        Args:
            edit_mode: If True, prefer tomlkit for editing capabilities
        """
        self.edit_mode = edit_mode
        self._backend = None
        self._tomlkit_doc = None

        # Select backend based on availability and requirements
        if edit_mode:
            # tomlkit is the only option that preserves formatting for editing
            if TOMLKIT_AVAILABLE:
                self._backend = "tomlkit"
            else:
                raise ImportError("tomlkit required for edit_mode but not available")
        elif RTOML_AVAILABLE:
            self._backend = "rtoml"
        elif TOMLI_AVAILABLE:
            self._backend = "tomli"
        elif TOMLKIT_AVAILABLE:
            self._backend = "tomlkit"
        else:
            raise ImportError("No TOML parser available. Install rtoml, tomli, or tomlkit")

    def load(self, stream: str | Path | Any) -> dict[str, Any]:
        """Load TOML from string or file path.

        Args:
            stream: TOML string, Path object, or file-like object

        Returns:
            Parsed TOML as dictionary
        """
        if isinstance(stream, Path):
            content = stream.read_text()
        elif isinstance(stream, str) and Path(stream).exists():
            content = Path(stream).read_text()
        else:
            content = stream if isinstance(stream, str) else stream.read()

        if self._backend == "rtoml":
            return rtoml.loads(content)
        if self._backend == "tomli":
            return tomli.loads(content)
        if self._backend == "tomlkit":
            return tomlkit.parse(content).value
        raise RuntimeError(f"Unknown backend: {self._backend}")

    def loads(self, s: str) -> dict[str, Any]:
        """Load TOML from string.

        Args:
            s: TOML string

        Returns:
            Parsed TOML as dictionary
        """
        if self.edit_mode:
            import tomlkit

            return tomlkit.parse(s).value

        from thegent.infra.runtime_dispatcher import get_toml_loads

        toml_loads = get_toml_loads()
        return toml_loads(s)

    def dump(self, data: dict[str, Any], stream: Any | None = None, **kwargs) -> str | None:
        """Dump TOML to string or file.

        Args:
            data: Data to serialize
            stream: Optional file-like object or Path to write to
            **kwargs: Additional options

        Returns:
            TOML string if stream is None, else None
        """
        if self._backend == "rtoml":
            result = rtoml.dumps(data, **kwargs)
        elif self._backend == "tomli_w" and TOMLI_W_AVAILABLE:
            result = tomli_w.dumps(data, **kwargs)
        elif self._backend == "tomlkit" or TOMLKIT_AVAILABLE:
            doc = tomlkit.document()
            for key, value in data.items():
                doc[key] = value
            result = tomlkit.dumps(doc)
        else:
            raise RuntimeError("No TOML writer available")

        if stream:
            if isinstance(stream, Path):
                stream.write_text(result)
            else:
                stream.write(result)
            return None
        return result

    def dumps(self, data: dict[str, Any], **kwargs) -> str:
        """Dump TOML to string.

        Args:
            data: Data to serialize
            **kwargs: Additional options

        Returns:
            TOML string
        """
        result = self.dump(data, **kwargs)
        if result is None:
            raise ValueError("dump() returned None (should not happen for dumps)")
        return result

    @property
    def backend(self) -> str:
        """Get current backend name."""
        return self._backend or "unknown"


# Global instance for convenience
_toml_parser: FastTOMLParser | None = None


def get_toml_parser(edit_mode: bool = False) -> FastTOMLParser:
    """Get global fast TOML parser instance.

    Args:
        edit_mode: If True, prefer tomlkit for editing capabilities

    Returns:
        FastTOMLParser instance
    """
    global _toml_parser
    if _toml_parser is None or _toml_parser.edit_mode != edit_mode:
        _toml_parser = FastTOMLParser(edit_mode=edit_mode)
    return _toml_parser


# Convenience functions
def toml_load(stream: str | Path | Any) -> dict[str, Any]:
    """Load TOML using fastest available backend."""
    return get_toml_parser().load(stream)


def toml_loads(s: str) -> dict[str, Any]:
    """Load TOML string using fastest available backend."""
    return get_toml_parser().loads(s)


def toml_dump(data: dict[str, Any], stream: Any | None = None, **kwargs) -> str | None:
    """Dump TOML using fastest available backend."""
    return get_toml_parser().dump(data, stream, **kwargs)


def toml_dumps(data: dict[str, Any], **kwargs) -> str:
    """Dump TOML to string using fastest available backend."""
    return get_toml_parser().dumps(data, **kwargs)
