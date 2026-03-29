"""WP-31003: Infra Drift Self-Correction Loop.
Monitors provisioned resources and automatically corrects deviations from the target spec.
Ensures agent infrastructure remains stable and compliant over time.
"""

import logging

from thegent.infra.provisioner import InfraProvisioner, ResourceSpec

_log = logging.getLogger(__name__)


class DriftCorrector:
    """Orchestrates self-correction of agent infrastructure drift."""

    def __init__(self, provisioner: InfraProvisioner) -> None:
        self.provisioner = provisioner

    def check_drift(self, resource_id: str, target_spec: ResourceSpec) -> bool:
        """Check if a resource has drifted from its target specification."""
        current_spec = self.provisioner.active_resources.get(resource_id)

        if not current_spec:
            _log.warning("Resource %s not found. Drift detected (missing).", resource_id)
            return True

        # Compare key metrics
        drifted = (
            current_spec.cpu_cores != target_spec.cpu_cores
            or current_spec.memory_mb != target_spec.memory_mb
            or current_spec.resource_type != target_spec.resource_type
        )

        if drifted:
            _log.warning("Drift detected for resource %s.", resource_id)

        return drifted

    def correct_drift(self, resource_id: str, target_spec: ResourceSpec):
        """Automatically correct detected infrastructure drift."""
        _log.info("Correcting drift for resource: %s", resource_id)

        # 1. Decommission drifting resource
        self.provisioner.decommission(resource_id)

        # 2. Re-provision to target spec
        success = self.provisioner.provision(resource_id, target_spec)

        if success:
            _log.info("Drift correction successful for %s.", resource_id)
        else:
            _log.error("Drift correction FAILED for %s.", resource_id)
