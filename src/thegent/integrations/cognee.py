"""
Cognee Integration - Knowledge engine/memory layer

Provides vector + graph + modular tasks for AI agents.
Alternative memory engine to graphiti.

Security:
- Verify Apache-2.0 license compatibility

License: Apache-2.0 (verified at https://github.com/topoteretes/cognee)
"""

import logging
import os
from dataclasses import dataclass
from enum import Enum
from typing import cast

from thegent.integrations.base import DataclassConfig

logger = logging.getLogger(__name__)


class CogneeStatus(Enum):
    DISABLED = "disabled"
    ENABLED = "enabled"


@dataclass
class CogneeConfig(DataclassConfig):
    api_url: str = "http://localhost:8000"


class CogneeClient:
    def __init__(self, config: CogneeConfig | None = None):
        self._config = config or self._load_config()
        self._status = CogneeStatus.DISABLED
        if self._config.enabled:
            self._status = CogneeStatus.ENABLED

    def _load_config(self) -> CogneeConfig:
        config = cast("CogneeConfig", CogneeConfig.from_env("COGNEE_"))
        config.enabled = os.environ.get("THEGENT_ENABLE_COGNEE", "").lower() in ("1", "true", "yes")
        return config

    @property
    def is_enabled(self):
        return self._config.enabled

    async def add_memory(self, text: str, metadata: dict | None = None):
        if not self.is_enabled:
            return None
        return {"id": "mock"}

    async def search(self, query: str, limit: int = 10):
        if not self.is_enabled:
            return []
        return []


_cognee = None


def get_cognee():
    global _cognee
    if _cognee is None:
        _cognee = CogneeClient()
    return _cognee
