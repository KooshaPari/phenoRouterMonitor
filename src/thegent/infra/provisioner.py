"""WP-31001: Self-Provisioning Infra Bridge.
Enables agents to provision their own compute, storage, and networking resources.
Provides a high-level API over Terraform/Cloud-init/Docker.
"""

import logging
import shutil

from pydantic import BaseModel

from thegent.errors import ConfigError, get_install_hint

_log = logging.getLogger(__name__)


class ResourceSpec(BaseModel):
    """Specification for an infra resource."""

    resource_type: str  # 'container', 'volume', 'vm'
    cpu_cores: float
    memory_mb: int
    labels: dict[str, str]


class InfraProvisioner:
    """Orchestrates self-provisioning of agent infrastructure."""

    def __init__(self, provider: str = "local") -> None:
        self.provider = provider
        self.active_resources: dict[str, ResourceSpec] = {}

    def provision(self, resource_id: str, spec: ResourceSpec) -> bool:
        """Provision a resource based on the spec."""
        _log.info(
            "Provisioning %s resource: %s (CPU: %.1f, Mem: %dMB)",
            spec.resource_type,
            resource_id,
            spec.cpu_cores,
            spec.memory_mb,
        )

        # Simulated provisioning logic
        if self.provider == "local":
            if spec.resource_type == "container":
                if not shutil.which("docker"):
                    raise ConfigError("Docker is required for container provisioning.", get_install_hint("docker"))
            # This would execute 'docker run' or similar
            _log.info("Executing local resource allocation for %s", resource_id)
            self.active_resources[resource_id] = spec
            return True

        return False

    def decommission(self, resource_id: str):
        """Release a previously provisioned resource."""
        if resource_id in self.active_resources:
            _log.info("Decommissioning resource: %s", resource_id)
            del self.active_resources[resource_id]
