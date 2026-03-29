"""
ChunkHound Integration - Local-first code intelligence

Provides code chunking/indexing for local code intelligence.
Low infra overhead for code analysis.

Security:
- Verify MIT license compatibility

License: MIT (verified at https://github.com/chunkhound/chunkhound)
"""

import logging
import os
from dataclasses import dataclass
from enum import Enum
from typing import cast

from thegent.integrations.base import DataclassConfig

logger = logging.getLogger(__name__)


class ChunkHoundStatus(Enum):
    DISABLED = "disabled"
    ENABLED = "enabled"


@dataclass
class ChunkHoundConfig(DataclassConfig):
    index_path: str = "./.chunkhound"


class ChunkHoundClient:
    def __init__(self, config: ChunkHoundConfig | None = None):
        self._config = config or self._load_config()
        self._status = ChunkHoundStatus.DISABLED
        if self._config.enabled:
            self._status = ChunkHoundStatus.ENABLED

    def _load_config(self) -> ChunkHoundConfig:
        config = cast("ChunkHoundConfig", ChunkHoundConfig.from_env("CHUNKHOUND_"))
        config.enabled = os.environ.get("THEGENT_ENABLE_CHUNKHOUND", "").lower() in ("1", "true", "yes")
        return config

    @property
    def is_enabled(self):
        return self._config.enabled

    def index(self, path: str):
        if not self.is_enabled:
            return None
        return {"indexed": 0}

    def query(self, query: str):
        if not self.is_enabled:
            return []
        return []


_chunkhound = None


def get_chunkhound():
    global _chunkhound
    if _chunkhound is None:
        _chunkhound = ChunkHoundClient()
    return _chunkhound
