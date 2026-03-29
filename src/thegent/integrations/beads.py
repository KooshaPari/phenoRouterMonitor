"""
Beads Task Tracking Integration - Persistent dependency tracking.
"""

import logging
import os
import subprocess
from dataclasses import dataclass
from enum import Enum
from typing import cast

from thegent.infra.shim_subprocess import run as shim_run

from thegent.integrations.base import DataclassConfig

logger = logging.getLogger(__name__)


class BeadsStatus(Enum):
    UNAVAILABLE = "unavailable"
    AVAILABLE = "available"


@dataclass
class BeadsConfig(DataclassConfig):
    binary_path: str = ""


class BeadsWrapper:
    def __init__(self, config: BeadsConfig | None = None):
        self._config = config or self._load_config()
        self._status = BeadsStatus.UNAVAILABLE
        if self._config.enabled:
            self._check_availability()

    def _load_config(self) -> BeadsConfig:
        config = cast("BeadsConfig", BeadsConfig.from_env("BEADS_"))
        config.enabled = os.environ.get("THEGENT_ENABLE_BEADS", "").lower() in ("1", "true", "yes")
        return config

    def _check_availability(self):
        binary = self._config.binary_path or "bd"
        try:
            result = shim_run([binary, "version"], capture_output=True, timeout=5)
            if result.returncode == 0:
                self._status = BeadsStatus.AVAILABLE
        except (subprocess.SubprocessError, OSError):
            pass

    @property
    def is_enabled(self):
        return self._config.enabled and self._status == BeadsStatus.AVAILABLE

    async def get_ready_beads(self):
        if not self.is_enabled:
            return {"success": False, "beads": []}
        return {"success": True, "beads": []}


_beads = None


def get_beads_wrapper():
    global _beads
    if _beads is None:
        _beads = BeadsWrapper()
    return _beads
