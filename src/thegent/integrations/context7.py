"""
Context7 MCP Provider Integration - Documentation context lookup.
"""

import logging
import os
import re
from dataclasses import dataclass
from enum import Enum
from typing import cast

from thegent.integrations.base import DataclassConfig

logger = logging.getLogger(__name__)


class Context7Status(Enum):
    DISABLED = "disabled"
    ENABLED = "enabled"


@dataclass
class Context7Config(DataclassConfig):
    api_endpoint: str = "https://api.context7.io/v1"
    api_key: str = ""


class Context7Provider:
    SECRET_PATTERNS = [  # noqa: RUF012
        r"(?i)(api[_-]?key|secret)=[\w-]+",
        r"ghp_[a-zA-Z0-9]{36}",
    ]

    def __init__(self, config: Context7Config | None = None):
        self._config = config or self._load_config()
        self._status = Context7Status.DISABLED
        if self._config.enabled:
            self._status = Context7Status.ENABLED

    def _load_config(self) -> Context7Config:
        config = cast("Context7Config", Context7Config.from_env("CONTEXT7_"))
        config.enabled = os.environ.get("THEGENT_ENABLE_CONTEXT7", "").lower() in ("1", "true", "yes")
        return config

    @property
    def is_enabled(self):
        return self._config.enabled

    def validate_query(self, query: str) -> bool:
        return all(not re.search(p, query) for p in self.SECRET_PATTERNS)

    async def lookup(self, query: str, context: str = ""):
        if not self.is_enabled:
            return {"success": False}
        if not self.validate_query(query):
            return {"success": False, "error": "security"}
        return {"success": True, "content": f"[Context7: {query}]"}


_context7 = None


def get_context7_provider():
    global _context7
    if _context7 is None:
        _context7 = Context7Provider()
    return _context7
