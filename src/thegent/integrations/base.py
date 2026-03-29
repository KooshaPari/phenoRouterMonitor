"""Base classes and utilities for integrations.

Provides standard patterns for:
- Configuration loading
- Status tracking
- Enable/disable toggles
- Feature flags
"""

from __future__ import annotations

import json
import logging
import os
from abc import ABC, abstractmethod
from dataclasses import Field, dataclass, fields
from enum import StrEnum
from pathlib import Path
from typing import Any, cast

_log = logging.getLogger(__name__)


def _dataclass_field_list(instance_or_cls: object) -> list[Field[Any]]:
    """Return dataclass fields with a concrete typed list for pyright."""
    return list(fields(cast("Any", instance_or_cls)))


class IntegrationStatus(StrEnum):
    """Standard integration status values."""

    UNKNOWN = "unknown"
    DISABLED = "disabled"
    ENABLED = "enabled"
    AVAILABLE = "available"
    UNAVAILABLE = "unavailable"
    ERROR = "error"


@dataclass
class IntegrationInfo:
    """Basic integration metadata."""

    name: str
    description: str = ""
    version: str = "1.0.0"
    status: IntegrationStatus = IntegrationStatus.UNKNOWN
    enabled: bool = False
    error: str | None = None


# ---------------------------------------------------------------------------
# Feature Flag System
# ---------------------------------------------------------------------------


class FeatureFlag:
    """Simple feature flag with environment variable support.

    Usage:
        FLAG = FeatureFlag("MY_FEATURE", default=False)

        if FLAG.enabled:
            ...
    """

    def __init__(self, name: str, default: bool = False, env_prefix: str = "THEGENT_"):
        self.name = name
        self._default = default
        self._env_key = f"{env_prefix}ENABLE_{name}"

    @property
    def enabled(self) -> bool:
        """Check if feature is enabled via environment variable."""
        val = os.environ.get(self._env_key, "")
        if val:
            return val.lower() in ("1", "true", "yes", "on")
        return self._default

    def __bool__(self) -> bool:
        return self.enabled


class FeatureRegistry:
    """Registry for all feature flags."""

    _flags: dict[str, FeatureFlag] = {}  # noqa: RUF012

    @classmethod
    def register(cls, flag: FeatureFlag) -> None:
        cls._flags[flag.name] = flag

    @classmethod
    def get(cls, name: str) -> FeatureFlag | None:
        return cls._flags.get(name)

    @classmethod
    def all_enabled(cls) -> dict[str, bool]:
        return {name: flag.enabled for name, flag in cls._flags.items()}


def feature(name: str, default: bool = False) -> FeatureFlag:
    """Create and register a feature flag."""
    flag = FeatureFlag(name, default)
    FeatureRegistry.register(flag)
    return flag


# ---------------------------------------------------------------------------
# Serializable Mixin
# ---------------------------------------------------------------------------


class SerializableMixin:  # noqa: PLW1641
    """Mixin providing to_dict/from_dict for dataclasses.

    Automatically handles:
    - Enum values → serialized as .value
    - datetime objects → serialized as .isoformat()
    - Path objects → serialized as str()
    - Nested SerializableMixin objects → .to_dict()
    - Nested dicts/lists → recursive serialization

    Usage:
        @dataclass
        class MyModel(SerializableMixin):
            name: str
            value: int = 0

        m = MyModel(name="test", value=42)
        d = m.to_dict()  # {"name": "test", "value": 42}
        m2 = MyModel.from_dict(d)  # MyModel(name="test", value=42)
    """

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary with automatic type serialization."""
        from enum import Enum
        from datetime import datetime
        from pathlib import Path

        def _serialize(val: Any) -> Any:
            if val is None:
                return None
            if isinstance(val, Enum):
                return val.value
            if isinstance(val, datetime):
                return val.isoformat()
            if isinstance(val, Path):
                return str(val)
            if isinstance(val, SerializableMixin):
                return val.to_dict()
            if isinstance(val, dict):
                return {k: _serialize(v) for k, v in val.items()}
            if isinstance(val, (list, tuple)):
                return [_serialize(v) for v in val]
            return val

        if hasattr(self, "__dataclass_fields__"):
            result = {}
            for f in _dataclass_field_list(self):
                val = getattr(self, f.name, None)
                result[f.name] = _serialize(val)
            return result
        # Fallback for non-dataclass
        result = {}
        for key, val in self.__dict__.items():
            result[key] = _serialize(val)
        return result

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "SerializableMixin":
        """Create instance from dictionary with type-aware deserialization.

        Automatically converts:
        - ISO strings → datetime (when field type is datetime)
        - Strings → Path (when field type is Path)
        - Values → Enum (when field type is Enum)
        - Dicts → nested SerializableMixin (when field type is SerializableMixin subclass)
        """
        import inspect

        if not hasattr(cls, "__dataclass_fields__"):
            # Non-dataclass fallback
            return cast("SerializableMixin", cls(**data))

        # Get field types
        converted: dict[str, Any] = {}

        for f in _dataclass_field_list(cls):
            field_name = f.name
            if field_name not in data:
                continue

            val = data[field_name]
            field_type = f.type

            # Resolve string type annotations
            if isinstance(field_type, str):
                # Try to resolve from module globals
                frame = inspect.currentframe()
                try:
                    # Walk up frames to find the class module
                    for _ in range(5):
                        if frame is None:
                            break
                        if frame.f_globals.get("__name__") == cls.__module__:
                            field_type = frame.f_globals.get(field_type, field_type)
                            break
                        frame = frame.f_back
                finally:
                    del frame

            # Deserialize based on type
            converted[field_name] = cls._deserialize_value(val, field_type)

        return cls(**converted)

    @classmethod
    def _deserialize_value(cls, val: Any, target_type: Any) -> Any:
        """Deserialize a value to the target type."""
        from datetime import datetime
        from enum import Enum
        from pathlib import Path
        from types import UnionType
        from typing import get_origin, get_args, Union

        if val is None:
            return None

        # Handle None/Optional (both typing.Union and types.UnionType)
        origin = get_origin(target_type)
        if origin is Union or isinstance(target_type, UnionType):
            # Extract non-None type from Optional[X] or X | None
            args = get_args(target_type)
            non_none_types = [a for a in args if a is not type(None)]
            if non_none_types:
                # Try each type in order until one works
                for candidate_type in non_none_types:
                    if isinstance(candidate_type, type):
                        # Check if it's a SerializableMixin subclass
                        if issubclass(candidate_type, SerializableMixin):
                            if isinstance(val, dict):
                                try:
                                    return candidate_type.from_dict(val)
                                except Exception:
                                    continue
                        # Check if it's an Enum
                        if issubclass(candidate_type, Enum):
                            if isinstance(val, candidate_type):
                                return val
                            try:
                                return candidate_type(val)
                            except (ValueError, KeyError):
                                continue
                    # Use the first non-None type for other cases
                    target_type = non_none_types[0]
                    break

        # Handle Path
        if target_type is Path or (isinstance(target_type, type) and issubclass(target_type, Path)):
            if isinstance(val, str):
                return Path(val)
            return val

        # Handle datetime
        if target_type is datetime or (isinstance(target_type, type) and issubclass(target_type, datetime)):
            if isinstance(val, str):
                # Handle ISO format with or without timezone
                try:
                    if "T" in val:
                        return datetime.fromisoformat(val.replace("Z", "+00:00"))
                    return datetime.fromisoformat(val)
                except ValueError:
                    return val
            return val

        # Handle Enum
        if isinstance(target_type, type) and issubclass(target_type, Enum):
            if isinstance(val, target_type):
                return val
            try:
                return target_type(val)
            except ValueError:
                # Try by name
                try:
                    if isinstance(val, str):
                        return target_type[val]
                    return val
                except KeyError:
                    return val

        # Handle nested SerializableMixin
        if isinstance(target_type, type) and issubclass(target_type, SerializableMixin):
            if isinstance(val, dict):
                return target_type.from_dict(val)
            return val

        # Handle lists with typed elements
        origin = get_origin(target_type)
        if origin is list:
            args = get_args(target_type)
            if args and isinstance(val, list):
                elem_type = args[0]
                return [cls._deserialize_value(v, elem_type) for v in val]

        # Handle dicts with typed values
        if origin is dict:
            args = get_args(target_type)
            if args and len(args) >= 2 and isinstance(val, dict):
                val_type = args[1]
                return {k: cls._deserialize_value(v, val_type) for k, v in val.items()}

        return val

    def __eq__(self, other: object) -> bool:
        """Equality based on serialized dict comparison."""
        if not isinstance(other, type(self)):
            return False
        return self.to_dict() == other.to_dict()

    def __serializable_hash__(self) -> int:
        """Hash based on serialized dict."""
        data = self.to_dict()

        def _make_hashable(val: Any) -> Any:
            if val is None:
                return None
            if isinstance(val, dict):
                return tuple(sorted((k, _make_hashable(v)) for k, v in val.items()))
            if isinstance(val, (list, tuple)):
                return tuple(_make_hashable(v) for v in val)
            return val

        try:
            return hash(_make_hashable(data))
        except TypeError:
            # Fall back to id-based hash if unhashable
            return id(self)

    def __repr__(self) -> str:
        """Readable repr showing class name and key fields."""
        cls_name = type(self).__name__
        data = self.to_dict()

        # Show first 3 fields in repr for readability
        if hasattr(self, "__dataclass_fields__"):
            field_list = _dataclass_field_list(self)
            shown_fields = []
            for f in field_list[:3]:  # Show first 3 fields
                if f.name in data:
                    val = getattr(self, f.name, None)  # Get original value
                    # Format value for display
                    if isinstance(val, str) and len(val) > 30:
                        val_repr = f"'{val[:27]}...'"
                    elif isinstance(val, dict):
                        val_repr = f"{{...{len(val)} keys}}"
                    elif isinstance(val, (list, tuple)) and len(val) > 3:
                        val_repr = f"[...{len(val)} items]"
                    else:
                        val_repr = repr(val)
                    shown_fields.append(f"{f.name}={val_repr}")

            if len(field_list) > 3:
                shown_fields.append("...")

            return f"{cls_name}({', '.join(shown_fields)})"

        # Non-dataclass fallback
        items = list(data.items())[:3]
        parts = [f"{k}={v!r}" for k, v in items]
        if len(data) > 3:
            parts.append("...")
        return f"{cls_name}({', '.join(parts)})"

    def diff(self, other: "SerializableMixin") -> dict[str, tuple[Any, Any]]:
        """Compare this instance with another and return field differences.

        Args:
            other: Another instance to compare with

        Returns:
            Dict mapping field names to (self_value, other_value) tuples
            for fields that differ. Empty dict if instances are equal.

        Example:
            p1 = Person(name="Alice", age=30)
            p2 = Person(name="Alice", age=35)
            diff = p1.diff(p2)  # {"age": (30, 35)}
        """
        if not isinstance(other, type(self)):
            raise TypeError(f"Cannot diff {type(self).__name__} with {type(other).__name__}")

        self_dict = self.to_dict()
        other_dict = other.to_dict()

        differences = {}
        all_keys = set(self_dict.keys()) | set(other_dict.keys())

        for key in all_keys:
            self_val = self_dict.get(key, _MISSING)
            other_val = other_dict.get(key, _MISSING)

            if self_val != other_val:
                differences[key] = (
                    None if self_val is _MISSING else self_val,
                    None if other_val is _MISSING else other_val,
                )

        return differences

    def copy(self, **overrides: Any) -> "SerializableMixin":
        """Create a shallow copy with optional field overrides.

        Args:
            **overrides: Field values to override in the copy

        Returns:
            New instance with copied values and any overrides applied

        Example:
            p1 = Person(name="Alice", age=30)
            p2 = p1.copy(age=35)  # Person(name="Alice", age=35)
        """
        data = self.to_dict()
        data.update(overrides)
        return type(self).from_dict(data)

    def merge(self, other: "SerializableMixin", *, overwrite: bool = True) -> "SerializableMixin":
        """Merge fields from another instance into a new instance.

        Args:
            other: Instance to merge from
            overwrite: If True (default), other's non-None values overwrite self's.
                      If False, only fill in None fields from other.

        Returns:
            New merged instance

        Example:
            p1 = Person(name="Alice", age=None, city="NYC")
            p2 = Person(name="Bob", age=30, city=None)
            merged = p1.merge(p2)  # Person(name="Bob", age=30, city="NYC")
            merged = p1.merge(p2, overwrite=False)  # Person(name="Alice", age=30, city="NYC")
        """
        if not isinstance(other, type(self)):
            raise TypeError(f"Cannot merge {type(self).__name__} with {type(other).__name__}")

        self_dict = self.to_dict()
        other_dict = other.to_dict()

        merged = dict(self_dict)
        for key, value in other_dict.items():
            if overwrite:
                if value is not None:
                    merged[key] = value
            elif merged.get(key) is None and value is not None:
                merged[key] = value

        return type(self).from_dict(merged)

    def patch(self, **updates: Any) -> "SerializableMixin":
        """Apply updates to create a new instance (alias for copy).

        More explicit name for the copy operation when making targeted changes.

        Args:
            **updates: Field values to update

        Returns:
            New instance with updates applied
        """
        return self.copy(**updates)

    def to_json(self, *, indent: int | None = None, sort_keys: bool = False) -> str:
        """Serialize instance to JSON string.

        Args:
            indent: JSON indentation level (None for compact)
            sort_keys: Whether to sort dictionary keys

        Returns:
            JSON string representation

        Example:
            person.to_json()  # '{"name": "Alice", "age": 30}'
            person.to_json(indent=2)  # Pretty-printed
        """
        import json

        return json.dumps(self.to_dict(), indent=indent, sort_keys=sort_keys, default=str)

    @classmethod
    def from_json(cls, json_str: str) -> "SerializableMixin":
        """Create instance from JSON string.

        Args:
            json_str: JSON string to parse

        Returns:
            New instance from parsed JSON

        Raises:
            json.JSONDecodeError: If JSON is invalid

        Example:
            person = Person.from_json('{"name": "Alice", "age": 30}')
        """
        data = json.loads(json_str)
        return cls.from_dict(data)

    def to_json_file(self, path: str | Path, *, indent: int = 2) -> None:
        """Write instance to JSON file.

        Args:
            path: File path to write
            indent: JSON indentation level (default: 2 for readability)
        """
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(self.to_json(indent=indent))

    @classmethod
    def from_json_file(cls, path: str | Path) -> "SerializableMixin":
        """Create instance from JSON file.

        Args:
            path: File path to read

        Returns:
            New instance from parsed JSON file

        Raises:
            FileNotFoundError: If file doesn't exist
            json.JSONDecodeError: If JSON is invalid
        """
        path = Path(path)
        return cls.from_json(path.read_text())


class _Missing:
    """Sentinel for missing values."""

    def __repr__(self) -> str:
        return "<MISSING>"


_MISSING = _Missing()


def hashable_dataclass(cls: type) -> type:
    """Decorator to make a dataclass hashable using SerializableMixin hash.

    Also restores the SerializableMixin __repr__ for cleaner output.

    Usage:
        @hashable_dataclass
        @dataclass
        class MyModel(SerializableMixin):
            name: str
            value: int = 0

    Or:
        @dataclass
        @hashable_dataclass
        class MyModel(SerializableMixin):
            name: str
            value: int = 0
    """
    if hasattr(cls, "__dataclass_fields__"):
        if cls.__hash__ is None:
            # Restore the hash method from SerializableMixin
            cls.__hash__ = SerializableMixin.__serializable_hash__
        # Also restore the custom __repr__ for cleaner output
        setattr(cls, "__repr__", SerializableMixin.__repr__)
    return cls


# ---------------------------------------------------------------------------
# Singleton Mixin
# ---------------------------------------------------------------------------

import threading
from typing import ClassVar


class SingletonMixin:
    """Thread-safe singleton mixin for classes.

    Provides a consistent singleton pattern with:
    - Thread-safe initialization (double-checked locking)
    - Lazy instantiation
    - Reset capability for testing

    Usage:
        class MyService(SingletonMixin):
            def __init__(self, config: str = "default"):
                self.config = config

        # Get singleton instance
        service = MyService.get_instance()

        # Get with custom args (only used on first call)
        service = MyService.get_instance(config="custom")

        # Reset for testing
        MyService.reset_instance()

    Note:
        - First call to get_instance() creates the instance
        - Subsequent calls return the same instance
        - Args passed after first call are ignored
        - Use reset_instance() to clear for testing
    """

    _instances: ClassVar[dict[type, Any]] = {}
    _locks: ClassVar[dict[type, threading.Lock]] = {}
    _global_lock = threading.Lock()

    @classmethod
    def _get_lock(cls) -> threading.Lock:
        """Get or create lock for this class."""
        with SingletonMixin._global_lock:
            if cls not in SingletonMixin._locks:
                SingletonMixin._locks[cls] = threading.Lock()
            return SingletonMixin._locks[cls]

    @classmethod
    def get_instance(cls, *args, **kwargs) -> "SingletonMixin":
        """Get the singleton instance, creating it if necessary.

        Args:
            *args: Positional arguments for __init__ (only used on first call)
            **kwargs: Keyword arguments for __init__ (only used on first call)

        Returns:
            The singleton instance
        """
        if cls not in SingletonMixin._instances:
            lock = cls._get_lock()
            with lock:
                # Double-checked locking
                if cls not in SingletonMixin._instances:
                    instance = cls(*args, **kwargs)
                    SingletonMixin._instances[cls] = instance
        return SingletonMixin._instances[cls]

    @classmethod
    def reset_instance(cls) -> None:
        """Reset the singleton instance.

        Useful for testing to get a fresh instance.
        Warning: Not thread-safe during reset.
        """
        lock = cls._get_lock()
        with lock:
            if cls in SingletonMixin._instances:
                del SingletonMixin._instances[cls]

    @classmethod
    def has_instance(cls) -> bool:
        """Check if an instance exists."""
        return cls in SingletonMixin._instances


# ---------------------------------------------------------------------------
# Config Loading Utilities
# ---------------------------------------------------------------------------


def load_env_config(prefix: str, defaults: dict[str, Any] | None = None) -> dict[str, Any]:
    """Load configuration from environment variables with prefix.

    Args:
        prefix: Environment variable prefix (e.g., "MYAPP_")
        defaults: Default values for config keys

    Returns:
        Dict with config values from env (with type conversion)
    """
    config = dict(defaults) if defaults else {}

    for key, default_val in (defaults or {}).items():
        env_key = f"{prefix}{key.upper()}"
        env_val = os.environ.get(env_key)

        if env_val is not None:
            if isinstance(default_val, bool):
                config[key] = env_val.lower() in ("1", "true", "yes", "on")
            elif isinstance(default_val, int):
                config[key] = int(env_val)
            elif isinstance(default_val, float):
                config[key] = float(env_val)
            elif isinstance(default_val, list):
                config[key] = [s.strip() for s in env_val.split(",")]
            else:
                config[key] = env_val

    return config


def load_file_config(path: Path | str, defaults: dict[str, Any] | None = None) -> dict[str, Any]:
    """Load configuration from JSON or YAML file.

    Args:
        path: Path to config file (.json, .yaml, .yml)
        defaults: Default values

    Returns:
        Merged config dict
    """
    config = dict(defaults) if defaults else {}
    config_path = Path(path)

    if not config_path.exists():
        return config

    try:
        content = config_path.read_text()

        if config_path.suffix == ".json":
            data = json.loads(content)
        elif config_path.suffix in (".yaml", ".yml"):
            try:
                import yaml

                data = yaml.safe_load(content) or {}
            except ImportError:
                _log.warning("PyYAML not installed, skipping YAML config")
                return config
        else:
            return config

        if isinstance(data, dict):
            config.update(data)

    except (json.JSONDecodeError, OSError, ValueError) as e:
        _log.warning(f"Failed to load config from {path}: {e}")

    return config


# ---------------------------------------------------------------------------
# Dataclass Config Base
# ---------------------------------------------------------------------------


@dataclass
class DataclassConfig:
    """Base dataclass config with env loading support.

    Inherit from this for dataclass-based configs:
        @dataclass
        class MyConfig(DataclassConfig):
            base_url: str = "http://localhost"
            api_key: str = ""
    """

    enabled: bool = False

    @classmethod
    def from_env(cls, prefix: str = "") -> "DataclassConfig":
        """Load config from environment variables."""
        env_values: dict[str, Any] = {}

        for f in fields(cls):
            env_key = f"{prefix}{f.name.upper()}"
            env_val = os.environ.get(env_key)

            if env_val is not None and f.default is not None:
                field_type = type(f.default)
                if field_type == bool:
                    env_values[f.name] = env_val.lower() in ("1", "true", "yes")
                elif field_type == int:
                    env_values[f.name] = int(env_val)
                elif field_type == float:
                    env_values[f.name] = float(env_val)
                elif field_type == list:
                    env_values[f.name] = [s.strip() for s in env_val.split(",")]
                else:
                    env_values[f.name] = env_val

        return cls(**env_values)


# ---------------------------------------------------------------------------
# Base Integration Class
# ---------------------------------------------------------------------------


class BaseIntegration(ABC):
    """Base class for integrations with standard lifecycle."""

    def __init__(self, name: str, config: DataclassConfig | None = None) -> None:
        self.name = name
        self._config = config
        self._status = IntegrationStatus.UNKNOWN
        self._error: str | None = None

    @property
    def status(self) -> IntegrationStatus:
        """Current integration status."""
        return self._status

    @property
    def enabled(self) -> bool:
        """Whether integration is enabled."""
        return self._config.enabled if self._config else False

    @property
    def error(self) -> str | None:
        """Last error message, if any."""
        return self._error

    @abstractmethod
    def check_available(self) -> bool:
        """Check if integration is available."""
        ...

    @abstractmethod
    def connect(self) -> bool:
        """Connect to integration."""
        ...

    @abstractmethod
    def disconnect(self) -> None:
        """Disconnect from integration."""
        ...

    def enable(self) -> None:
        """Enable integration."""
        if self._config:
            self._config.enabled = True

    def disable(self) -> None:
        """Disable integration."""
        if self._config:
            self._config.enabled = False

    def get_info(self) -> IntegrationInfo:
        """Get integration metadata."""
        return IntegrationInfo(
            name=self.name,
            status=self._status,
            enabled=self.enabled,
            error=self._error,
        )


__all__ = [
    "BaseIntegration",
    "DataclassConfig",
    "FeatureFlag",
    "FeatureRegistry",
    "IntegrationInfo",
    "IntegrationStatus",
    "SerializableMixin",
    "SingletonMixin",
    "feature",
    "hashable_dataclass",
    "load_env_config",
    "load_file_config",
]


# Alias for backward compatibility
BaseIntegrationConfig = DataclassConfig
