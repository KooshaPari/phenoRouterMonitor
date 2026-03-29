"""WP-12007: Persona profiles and access constraints.

Defines role-based access limits and defaults for different operator personas.
"""

from pathlib import Path
from typing import Any


class PersonaManager:
    """Manages role-based constraints for operator personas."""

    def __init__(self, agents_dir: Path | None = None) -> None:
        self.agents_dir = agents_dir or (Path(__file__).parent.parent.parent.parent / "agents")
        self._personas = {
            "operator": {"max_lane": "standard", "can_override": False},
            "incident_commander": {"max_lane": "critical", "can_override": True},
            "compliance_officer": {"max_lane": "recovery", "can_override": False, "read_only": True},
            # Teammate personas (WP-16001)
            "teammate": {"max_lane": "standard", "can_override": False, "role": "specialist", "priority": 1},
            "team_lead": {"max_lane": "critical", "can_override": True, "role": "lead", "priority": 2},
        }
        self._discovered_teammates: dict[str, dict[str, Any]] = {}

    def discover_teammates(self) -> dict[str, dict[str, Any]]:
        """WP-16001: Auto-discovery of teammates from the agents/ directory."""
        if not self.agents_dir.exists():
            return {}

        teammates = {}
        for md_file in self.agents_dir.glob("*.md"):
            agent_id = md_file.stem
            # Simple parsing for now - in a real impl, we'd use frontmatter
            try:
                content = md_file.read_text(encoding="utf-8")
                # Heuristic: if it mentions 'teammate' or 'role', it's a teammate
                if "teammate" in content.lower() or "role:" in content.lower():
                    teammates[agent_id] = {
                        "id": agent_id,
                        "path": str(md_file),
                        "type": "teammate",
                        "max_lane": "standard",
                    }
            except Exception:
                continue

        self._discovered_teammates = teammates
        return teammates

    def list_teammates(self) -> list[dict[str, Any]]:
        """List all discovered teammates."""
        if not self._discovered_teammates:
            self.discover_teammates()
        return list(self._discovered_teammates.values())

    def check_access(self, persona: str, operation: str, lane: str) -> dict[str, Any]:
        """Verify if a persona can perform a specific operation in a specific lane."""
        config = self._personas.get(persona)
        if not config:
            return {"allowed": False, "reason": "Unknown persona."}

        if lane == "critical" and config["max_lane"] != "critical":
            return {"allowed": False, "reason": f"Persona {persona} restricted from critical lane."}

        return {"allowed": True, "config": config}
