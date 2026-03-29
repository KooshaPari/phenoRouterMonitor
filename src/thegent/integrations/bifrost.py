"""
Bifrost Integration - Gateway claims validation

Provides gateway claims validation for LLM calls.
Requires local validation of claims.

Security:
- Verify Apache-2.0 license compatibility
- Local validation required

License: Apache-2.0 (verified at https://github.com/maximhq/bifrost)
"""

import logging
import os
import time
from dataclasses import dataclass
from enum import Enum
from typing import Any, cast

from thegent.integrations.base import DataclassConfig

logger = logging.getLogger(__name__)


class BifrostStatus(Enum):
    DISABLED = "disabled"
    ENABLED = "enabled"


@dataclass
class BifrostConfig(DataclassConfig):
    gateway_url: str = "http://localhost:8080"
    secret_key: str = ""
    api_key: str = ""
    rate_limit: int = 1000
    timeout: int = 30


class BifrostValidationError(Exception):
    """Raised when bifrost validation fails"""


class BifrostRateLimitError(BifrostValidationError):
    """Raised when rate limit exceeded"""


class BifrostAuthError(BifrostValidationError):
    """Raised when auth fails"""


class ClaimsValidator:
    """Validates claims for gateway access"""

    def __init__(self, config: BifrostConfig):
        self._config = config
        self._rate_limit_cache: dict[str, list[float]] = {}

    def validate_api_key(self, api_key: str) -> bool:
        """Validate API key format and presence"""
        if not api_key:
            return False
        # Basic format validation - key should be present
        return len(api_key) > 0

    def validate_rate_limit(self, identifier: str) -> tuple[bool, int]:
        """Check rate limit, returns (allowed, current_count)"""
        now = time.time()
        window = 3600  # 1 hour window

        # Clean old entries
        if identifier in self._rate_limit_cache:
            self._rate_limit_cache[identifier] = [t for t in self._rate_limit_cache[identifier] if now - t < window]
        else:
            self._rate_limit_cache[identifier] = []

        current_count = len(self._rate_limit_cache[identifier])
        allowed = current_count < self._config.rate_limit

        if allowed:
            self._rate_limit_cache[identifier].append(now)

        return allowed, current_count + 1

    def validate_claims(self, claims: dict[str, Any]) -> dict[str, Any]:
        """Main claims validation entry point"""
        errors = []

        # Validate API key
        api_key = claims.get("api_key", "")
        if not self.validate_api_key(api_key):
            errors.append("Invalid or missing api_key")

        # Validate rate limit
        identifier = claims.get("identifier", "default")
        allowed, count = self.validate_rate_limit(identifier)
        if not allowed:
            raise BifrostRateLimitError(f"Rate limit exceeded: {count} requests")

        # Validate required claims
        required = ["api_key", "identifier"]
        for req in required:
            if req not in claims:
                errors.append(f"Missing required claim: {req}")

        if errors:
            raise BifrostValidationError(", ".join(errors))

        return {"valid": True, "identifier": identifier}


class BifrostClient:
    def __init__(self, config: BifrostConfig | None = None):
        self._config = config or self._load_config()
        self._status = BifrostStatus.DISABLED
        self._validator = None
        if self._config.enabled:
            self._status = BifrostStatus.ENABLED
            self._validator = ClaimsValidator(self._config)

    def _load_config(self) -> BifrostConfig:
        config = cast("BifrostConfig", BifrostConfig.from_env("BIFROST_"))
        config.enabled = os.environ.get("THEGENT_ENABLE_BIFROST", "").lower() in ("1", "true", "yes")
        return config

    @property
    def is_enabled(self) -> bool:
        return self._config.enabled

    @property
    def status(self) -> BifrostStatus:
        return self._status

    def validate_claims(self, claims: dict[str, Any]) -> dict[str, Any]:
        """Validate claims - main entry point"""
        if not self.is_enabled:
            logger.debug("Bifrost disabled, skipping validation")
            return {"valid": True, "skipped": True}

        if not self._validator:
            raise BifrostValidationError("Bifrost enabled but validator not initialized")

        return self._validator.validate_claims(claims)

    def check_rate_limit(self, identifier: str) -> tuple[bool, int]:
        """Check rate limit without full validation"""
        if not self.is_enabled or not self._validator:
            return True, 0
        return self._validator.validate_rate_limit(identifier)


_bifrost = None


def get_bifrost() -> BifrostClient:
    global _bifrost
    if _bifrost is None:
        _bifrost = BifrostClient()
    return _bifrost
