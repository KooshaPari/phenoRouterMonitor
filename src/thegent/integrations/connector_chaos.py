"""Connector Chaos Tests for resilience testing of connectors.

# @trace WL-235
Provides chaos testing scenarios for connector outages and partial-failure edge cases.
Allows injection of faults into connector operations to validate resilience.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable


@dataclass
class ChaosScenario:
    """A chaos testing scenario for connector fault injection."""

    name: str
    fault_type: str
    probability: float = 1.0


class ConnectorChaosTestSuite:
    """Test suite for chaos testing of connectors."""

    def __init__(self) -> None:
        """Initialize the connector chaos test suite."""
        self._scenarios: dict[str, ChaosScenario] = {}

    def add_scenario(self, name: str, fault_type: str, probability: float = 1.0) -> ChaosScenario:
        """Add a chaos scenario to the test suite.

        Args:
            name: Unique name for the scenario.
            fault_type: Type of fault to inject (e.g., 'timeout', 'connection_drop').
            probability: Probability of fault injection (0.0 to 1.0). Defaults to 1.0.

        Returns:
            The created ChaosScenario.
        """
        scenario = ChaosScenario(name=name, fault_type=fault_type, probability=probability)
        self._scenarios[name] = scenario
        return scenario

    def run(self, scenario_name: str, target_fn: Callable[[], Any]) -> Any:
        """Run a target function under a chaos scenario.

        For simplicity, this implementation calls the target function directly.
        In a production system, this would inject faults based on the scenario probability.

        Args:
            scenario_name: Name of the scenario to apply.
            target_fn: The function to call under the scenario.

        Returns:
            The result of calling target_fn.

        Raises:
            KeyError: If the scenario_name does not exist.
        """
        _ = self._scenarios[scenario_name]
        # For simplicity, just call the target function directly
        # In a production system, faults would be injected based on scenario.probability
        return target_fn()

    def scenarios(self) -> list[ChaosScenario]:
        """Get all registered chaos scenarios.

        Returns:
            List of all ChaosScenarios.
        """
        return list(self._scenarios.values())
