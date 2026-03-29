"""WP-16001/16002: Specialized teammate agents and delegation."""

import logging
from pathlib import Path

from thegent.config import ThegentSettings
from thegent.infra.fast_yaml_parser import yaml_load

_log = logging.getLogger(__name__)


class TeammateAgent:
    """A specialized teammate agent capable of handling delegated sub-tasks."""

    def __init__(
        self, id: str, description: str, expertise: list[str], role: str = "specialist", priority: int = 1
    ) -> None:
        self.id = id
        self.description = description
        self.expertise = expertise
        self.role = role
        self.priority = priority


class TeammateManager:
    """Manages teammate discovery and task delegation."""

    def __init__(self, settings: ThegentSettings) -> None:
        self.settings = settings
        self.teammates: list[TeammateAgent] = []
        self._discover_teammates()

    def _discover_teammates(self) -> None:
        """Discover teammate agents from markdown definitions in agents/ directory."""
        agents_dir = Path("agents")
        if not agents_dir.exists():
            _log.warning("Agents directory not found: %s", agents_dir)
            return

        for agent_file in agents_dir.glob("*.md"):
            self._load_teammate(agent_file)

        _log.info("Discovered %d teammates", len(self.teammates))

    def _load_teammate(self, agent_file: Path) -> None:
        """Load a single teammate from a markdown file."""
        try:
            content = agent_file.read_text()
            if "---" in content:
                parts = content.split("---")
                if len(parts) >= 3:
                    frontmatter = yaml_load(parts[1])
                    if frontmatter:
                        agent_id = frontmatter.get("name", agent_file.stem)
                        description = frontmatter.get("description", "")
                        # Extract expertise from description or tags if added later
                        # For now, we'll use a simple heuristic or explicit tags
                        expertise = frontmatter.get("expertise", [])
                        if not expertise and "testing" in description.lower():
                            expertise.append("testing")
                        if not expertise and "security" in description.lower():
                            expertise.append("security")
                        if (not expertise and "ui" in description.lower()) or "ux" in description.lower():
                            expertise.append("ui")

                        role = frontmatter.get("role", "teammate")
                        priority = frontmatter.get("priority", 1)

                        self.teammates.append(
                            TeammateAgent(
                                id=agent_id,
                                description=description,
                                expertise=expertise,
                                role=role,
                                priority=priority,
                            )
                        )
        except Exception as e:
            _log.error("Failed to parse agent file %s: %s", agent_file, e)

    def find_specialist(self, task_description: str) -> TeammateAgent | None:
        """Find the best teammate for a task based on description."""
        for t in self.teammates:
            if any(e in task_description.lower() for e in t.expertise):
                return t
        return None

    def delegate(self, teammate_id: str, prompt: str, parent_run_id: str | None = None) -> str:
        """Delegate a task to a teammate."""
        _log.info("Delegating to %s: %s", teammate_id, prompt)
        # WP-16002: This would spawn a new AgentRunner or call an external service
        return f"run_delegated_{teammate_id}_{parent_run_id}"
