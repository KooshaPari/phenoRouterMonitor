"""Configuration management."""

import logging
from pathlib import Path
from typing import Any

import orjson

logger = logging.getLogger(__name__)


class ConfigLoadError(ValueError):
    """Typed config load error captured during non-fatal startup parsing."""

    def __init__(self, path: Path, reason: str, *, cause: Exception | None = None) -> None:
        self.path = path
        self.reason = reason
        self.cause = cause
        super().__init__(f"{reason}: {path}")


class ConfigManager:
    """Configuration manager."""

    def __init__(self, config_path: Path | None = None) -> None:
        """Initialize config manager.

        Args:
            config_path: Config file path
        """
        self.config_path = config_path or Path("~/.thegent/config.json").expanduser()
        self.config_path.parent.mkdir(parents=True, exist_ok=True)
        self.last_load_error: ConfigLoadError | None = None
        self.config: dict[str, Any] = self._load_config()

    def _load_config(self) -> dict[str, Any]:
        """Load config from file.

        Returns:
            Config dictionary
        """
        if self.config_path.exists():
            try:
                data = orjson.loads(self.config_path.read_bytes())
            except orjson.JSONDecodeError as exc:
                self.last_load_error = ConfigLoadError(self.config_path, "invalid_json", cause=exc)
                logger.error("config_load_failed_invalid_json path=%s error=%s", self.config_path, exc)
                return {}
            except OSError as exc:
                self.last_load_error = ConfigLoadError(self.config_path, "read_error", cause=exc)
                logger.error("config_load_failed_io path=%s error=%s", self.config_path, exc)
                return {}

            if not isinstance(data, dict):
                self.last_load_error = ConfigLoadError(self.config_path, "invalid_shape")
                logger.error("config_load_failed_invalid_shape path=%s", self.config_path)
                return {}
            self.last_load_error = None
            return data
        self.last_load_error = None
        return {}

    def get(self, key: str, default: Any = None) -> Any:
        """Get config value.

        Args:
            key: Config key
            default: Default value

        Returns:
            Config value
        """
        return self.config.get(key, default)

    def set(self, key: str, value: Any) -> None:
        """Set config value.

        Args:
            key: Config key
            value: Config value
        """
        self.config[key] = value
        self._save_config()

    def _save_config(self) -> None:
        """Save config to file."""
        payload = orjson.dumps(self.config, option=orjson.OPT_INDENT_2)
        self.config_path.write_bytes(payload)
