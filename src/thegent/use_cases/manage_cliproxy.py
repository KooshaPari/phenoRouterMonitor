"""Business logic for cliproxy provider lifecycle and credentials management.

Pure orchestration for:
- Provider configuration
- OAuth vs API-key routing
- Credentials lifecycle
- Factory config lookup
- Auto-restart on credential changes
"""

import logging
import os
from pathlib import Path
from typing import Any

import httpx

from thegent.config import ThegentSettings
from thegent.domain.provider_config import OAUTH_ONLY_PROVIDERS
from thegent.infra.fast_yaml_parser import yaml_load, yaml_dumps

_LOG = logging.getLogger(__name__)


class ProviderConfigManager:
    """Manages provider configuration and credentials."""

    def __init__(self, settings: ThegentSettings):
        self.settings = settings
        self._provider_defs_cache: dict[str, Any] | None = None

    def get_provider_definitions(self) -> dict[str, Any]:
        """Load provider definitions from internal JSON (cached)."""
        if self._provider_defs_cache is not None:
            return self._provider_defs_cache

        from thegent.agents.cliproxy_manager import _get_provider_definitions

        self._provider_defs_cache = _get_provider_definitions()
        return self._provider_defs_cache

    def get_provider_login_config(self, provider: str) -> dict[str, Any] | None:
        """Get login configuration for provider."""
        from thegent.agents.cliproxy_manager import PROVIDER_LOGIN_CONFIG

        provider_lower = provider.lower()
        return PROVIDER_LOGIN_CONFIG.get(provider_lower)

    def build_provider_login_configs(self) -> dict[str, dict[str, Any]]:
        """Build login config dict from provider definitions."""
        defs_ = self.get_provider_definitions()
        out: dict[str, dict[str, Any]] = {}

        for name, cfg in defs_.items():
            if not isinstance(cfg, dict) or "login" not in cfg:
                continue
            login = cfg.get("login", {})
            base_url = cfg.get("base_url", "")
            if cfg.get("base_url_env"):
                base_url = os.environ.get(cfg["base_url_env"], base_url)

            out[name] = {
                "url": login.get("url", ""),
                "base_url": base_url,
                "display_name": login.get("display_name", name),
                "model": cfg.get("model", name),
                "instructions": login.get("instructions", []),
            }
        return out

    def inject_api_key(
        self,
        config: dict[str, Any],
        provider: str,
        api_key: str,
        cfg: dict[str, Any],
    ) -> None:
        """Inject API key into cliproxy config."""
        from thegent.agents.cliproxy_manager import (
            _inject_api_key_into_cliproxy,
        )

        _inject_api_key_into_cliproxy(config, provider, api_key, cfg)

    def inject_cursor_config(self, config: dict[str, Any]) -> None:
        """Inject cursor configuration if available."""
        from thegent.agents.cliproxy_manager import (
            _inject_cursor_into_cliproxy,
        )

        _inject_cursor_into_cliproxy(config, self.settings)

    def inject_kiro_config(self, config: dict[str, Any]) -> None:
        """Inject kiro configuration if available."""
        from thegent.agents.cliproxy_manager import (
            _inject_kiro_into_cliproxy,
        )

        _inject_kiro_into_cliproxy(config, self.settings)

    def load_config(self) -> dict[str, Any]:
        """Load cliproxy config file."""
        config_path = self.settings.cliproxy_config_path.expanduser().resolve()
        if not config_path.exists():
            return {}
        try:
            raw = yaml_load(config_path)
            return dict(raw) if isinstance(raw, dict) else {}
        except Exception as e:
            _LOG.warning("Failed to load cliproxy config: %s", e)
            return {}

    def save_config(self, config: dict[str, Any]) -> Path:
        """Save cliproxy config to file."""
        config_path = self.settings.cliproxy_config_path.expanduser().resolve()
        config_path.parent.mkdir(parents=True, exist_ok=True)
        config_path.write_text(yaml_dumps(config))
        return config_path

    def has_provider_credentials(self, provider: str) -> bool:
        """Check if provider has credentials in config."""
        from thegent.agents.cliproxy_manager import (
            _has_provider_credentials,
        )

        config = self.load_config()
        return _has_provider_credentials(config, provider)

    def has_oauth_credentials(self, provider: str) -> bool:
        """Check if provider has OAuth credentials in auth dir."""
        from thegent.agents.cliproxy_manager import (
            _has_oauth_credentials,
        )

        return _has_oauth_credentials(self.settings, provider)


class CredentialsResolver:
    """Resolves credentials from multiple sources."""

    @staticmethod
    def get_factory_api_key(provider: str) -> tuple[str | None, str]:
        """Look up API key in factory config.

        Returns (api_key, source_path) or (None, "").
        """
        from thegent.agents.cliproxy_manager import _get_factory_api_key

        return _get_factory_api_key(provider)

    @staticmethod
    def should_use_oauth(provider: str) -> bool:
        """Check if provider should use OAuth flow."""
        from thegent.agents.cliproxy_manager import _LOGIN_FLAGS

        return provider.lower() in _LOGIN_FLAGS

    @staticmethod
    def is_oauth_only(provider: str) -> bool:
        """Check if provider only supports OAuth."""
        return provider.lower() in OAUTH_ONLY_PROVIDERS


class ProxyHealthChecker:
    """Checks proxy health and readiness."""

    @staticmethod
    def is_reachable(base_url: str, timeout: float = 2.0) -> bool:
        """Check if proxy is reachable at base_url."""
        base = base_url.rstrip("/")
        paths = (
            ("/models", "/")
            if base.endswith("/v1")
            else ("/v1/models", "/models", "/")
        )

        for path in paths:
            try:
                resp = httpx.get(
                    f"{base}{path}",
                    headers={"Authorization": "Bearer sk-dummy"},
                    timeout=timeout,
                )
                if resp.is_success:
                    return True
            except Exception:
                continue
        return False

    @staticmethod
    def is_adapter_running(base_url: str) -> bool:
        """Check if server at base_url is the adapter (has /v1/responses)."""
        try:
            base = base_url.rstrip("/")
            url = (
                f"{base}/models" if base.endswith("/v1") else f"{base}/v1/models"
            )
            resp = httpx.get(url, timeout=2)
            if not resp.is_success:
                return False
            data = resp.json()
            if not isinstance(data, dict):
                return False
            # Adapter returns "models"; raw CLIProxy returns "data"
            return "models" in data
        except Exception:
            return False

    @staticmethod
    def is_openrouter_backend(backend_url: str) -> bool:
        """Check if backend is OpenRouter."""
        return "openrouter.ai" in backend_url
