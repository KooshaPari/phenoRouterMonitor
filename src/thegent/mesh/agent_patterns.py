"""Regex-based agent detection using agents.conf."""

import configparser
from pathlib import Path
from typing import Any

import structlog

from .process_detection import detect_agents

logger = structlog.get_logger(__name__)

DEFAULT_AGENTS_CONF = """
[agents]
claude = claude-code|clode
cursor = cursor-agent|cursor
thegent = thegent
codex = codex
copilot = copilot
roo = roo-code|roo
kilo = kilo-code|kilo
"""


def get_config_path() -> Path:
    """Get path to agents.conf."""
    # Priority: current dir, then ~/.heliosShield/agents.conf
    local_conf = Path.cwd() / "agents.conf"
    if local_conf.exists():
        return local_conf

    home_conf = Path.home() / ".heliosShield" / "agents.conf"
    if not home_conf.parent.exists():
        home_conf.parent.mkdir(parents=True, exist_ok=True)

    if not home_conf.exists():
        home_conf.write_text(DEFAULT_AGENTS_CONF.strip())

    return home_conf


def load_agent_patterns() -> dict[str, str]:
    """Load agent regex patterns from agents.conf."""
    config_path = get_config_path()
    config = configparser.ConfigParser()
    config.read(config_path)
    if "agents" not in config:
        raise RuntimeError(f"agents.conf at {config_path} is missing required [agents] section")
    return dict(config["agents"])


def run_detection() -> list[dict[str, Any]]:
    """Load patterns and detect agents."""
    patterns = load_agent_patterns()
    return detect_agents(patterns)


if __name__ == "__main__":
    # Handle direct execution without relative import issues
    import sys

    sys.path.append(str(Path(__file__).parent.parent))
    from thegent.mesh.agent_patterns import run_detection

    run_detection()
