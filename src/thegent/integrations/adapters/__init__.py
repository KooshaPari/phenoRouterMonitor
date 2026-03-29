"""Adapters for workstream autosync.

This package contains extracted adapter modules from workstream_autosync.py
to reduce the main file size and improve maintainability.
"""

from thegent.integrations.adapters.checkpoint_adapter import CheckpointAdapter
from thegent.integrations.adapters.connector_config_adapter import ConnectorConfigAdapter
from thegent.integrations.adapters.encryption_adapter import (
    compute_artifact_key,
    xor_decrypt,
    xor_encrypt,
)
from thegent.integrations.adapters.metrics_adapter import MetricsAdapter
from thegent.integrations.adapters.sla_adapter import SLAAdapter
from thegent.integrations.adapters.state_adapter import StateAdapter
from thegent.integrations.adapters.sync_adapter import SyncAdapter

__all__ = [
    "CheckpointAdapter",
    "ConnectorConfigAdapter",
    "MetricsAdapter",
    "SLAAdapter",
    "StateAdapter",
    "SyncAdapter",
    "compute_artifact_key",
    "xor_decrypt",
    "xor_encrypt",
]
