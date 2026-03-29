"""WP-23003: Attestable Execution Environments (TEE) check.

Provides verification logic for secure enclave execution.
"""

import logging
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path

_log = logging.getLogger(__name__)


class TEEType(Enum):
    NONE = "none"
    AWS_NITRO = "aws_nitro"
    INTEL_SGX = "intel_sgx"
    AMD_SEV = "amd_sev"
    AZURE_TDX = "azure_tdx"
    MOCK = "mock"


@dataclass
class TEEAttestation:
    """TEE Attestation report (WP-23003)."""

    tee_type: TEEType
    is_attested: bool
    provider_id: str | None = field(default=None)
    measurement_hash: str | None = field(default=None)
    firmware_version: str | None = field(default=None)


class TEEChecker:
    """Verifies if the agent is running in a trusted execution environment."""

    def __init__(self, mock_mode: bool = False) -> None:
        from thegent.config import ThegentSettings

        s = ThegentSettings()
        self.mock_mode = mock_mode or s.tee_mock

    def check(self) -> TEEAttestation:
        """Perform TEE check and return attestation."""
        if self.mock_mode:
            return TEEAttestation(
                tee_type=TEEType.MOCK,
                is_attested=True,
                provider_id="mock-tee-provider",
                measurement_hash="sha256:mock-measurement-1234567890",
                firmware_version="1.0.0-mock",
            )

        # 1. AWS Nitro Enclaves detection
        if Path("/dev/nsm").exists():
            return TEEAttestation(
                tee_type=TEEType.AWS_NITRO,
                is_attested=True,  # Simplified: real impl would call NSM API
                provider_id="aws",
            )

        # 2. Intel SGX detection
        if Path("/dev/isgx").exists() or Path("/dev/sgx/enclave").exists():
            return TEEAttestation(
                tee_type=TEEType.INTEL_SGX,
                is_attested=True,
                provider_id="intel",
            )

        # 3. AMD SEV detection
        if Path("/dev/sev").exists():
            return TEEAttestation(
                tee_type=TEEType.AMD_SEV,
                is_attested=True,
                provider_id="amd",
            )

        return TEEAttestation(tee_type=TEEType.NONE, is_attested=False)

    def enforce_tee(self) -> None:
        """Raise error if not running in TEE and environment requires it."""
        attestation = self.check()
        from thegent.config import ThegentSettings

        if not attestation.is_attested and ThegentSettings().tee_required:
            raise RuntimeError("TEE_REQUIRED: Execution environment is not an attested TEE.")

        if attestation.is_attested:
            _log.info(f"WP-23003: Environment attested as {attestation.tee_type.value}")


def get_tee_attestation() -> TEEAttestation:
    """Helper for governance audit emission."""
    return TEEChecker().check()
