"""Autonomous learning surface map."""

from typing import Any


class AutonomousLearningSurfaceMap:
    """Surface map for autonomous learning."""

    def __init__(self) -> None:
        """Initialize learning surface map."""
        self.surfaces: dict[str, Any] = {}

    def register_surface(self, surface_id: str, capabilities: list[str]) -> None:
        """Register a learning surface.

        Args:
            surface_id: Surface identifier
            capabilities: List of capabilities
        """
        self.surfaces[surface_id] = {
            "id": surface_id,
            "capabilities": capabilities,
        }

    def get_learning_opportunities(self) -> list[dict[str, Any]]:
        """Get available learning opportunities.

        Returns:
            List of learning opportunities
        """
        opportunities = []
        for surface_id, surface in self.surfaces.items():
            opportunities.append(
                {
                    "surface": surface_id,
                    "capabilities": surface["capabilities"],
                }
            )
        return opportunities
