"""JetBrains IDE integration utilities for thegent.

Provides detection of installed JetBrains IDEs and configuration of the MCP
server endpoint so the JetBrains AI plugin can connect to thegent.

MCP config path per IDE:  ~/.config/JetBrains/<IDE>/mcp.json
MCP config format:
    {"mcpServers": {"thegent": {"url": "http://localhost:3847/mcp"}}}

FR traceability: FR-IDE-001 (JetBrains MCP integration)
"""

from __future__ import annotations

import logging
import platform
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal, get_args

import orjson

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

#: Default thegent MCP server URL (matches MCP server port in ThegentSettings)
DEFAULT_MCP_SERVER_URL: str = "http://localhost:3847/mcp"

IdeType = Literal[
    "intellij",
    "pycharm",
    "goland",
    "clion",
    "webstorm",
    "rider",
    "datagrip",
    "rubymine",
    "phpstorm",
    "fleet",
]

_VALID_IDE_TYPES: frozenset[str] = frozenset(get_args(IdeType))

#: JetBrains config directory name fragments keyed by IDE type.
#: The actual directory is usually ~/.config/JetBrains/<DirFragment><Version>/
#: We match on the prefix so we pick up any installed version.
_IDE_DIR_PREFIXES: dict[str, list[str]] = {
    "intellij": ["IntelliJIdea", "IdeaIC"],
    "pycharm": ["PyCharm", "PyCharmCE"],
    "goland": ["GoLand"],
    "clion": ["CLion"],
    "webstorm": ["WebStorm"],
    "rider": ["Rider"],
    "datagrip": ["DataGrip"],
    "rubymine": ["RubyMine"],
    "phpstorm": ["PhpStorm"],
    "fleet": ["Fleet"],
}


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


@dataclass
class JetBrainsConfig:
    """Configuration record for a detected JetBrains IDE installation.

    Attributes:
        ide_type:            Canonical IDE type identifier.
        config_dir:          Path to the IDE's user configuration directory
                             (e.g. ~/.config/JetBrains/PyCharm2024.2).
        mcp_server_url:      The MCP server URL to write into the IDE config.
        serena_project_root: Absolute path of the project root to scope
                             Serena semantic tools to.
    """

    ide_type: str
    config_dir: Path
    mcp_server_url: str = DEFAULT_MCP_SERVER_URL
    serena_project_root: str = ""

    # Derived from config_dir; not part of equality / repr by default.
    _mcp_config_path: Path = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        if self.ide_type not in _VALID_IDE_TYPES:
            raise ValueError(f"ide_type must be one of {sorted(_VALID_IDE_TYPES)}, got {self.ide_type!r}")
        self._mcp_config_path = self.config_dir / "mcp.json"

    @property
    def mcp_config_path(self) -> Path:
        """Absolute path to the mcp.json configuration file."""
        return self._mcp_config_path


# ---------------------------------------------------------------------------
# Detection helpers (platform-specific)
# ---------------------------------------------------------------------------


def _jetbrains_base_dirs() -> list[Path]:
    """Return candidate base directories that contain JetBrains IDE config dirs."""
    import os

    system = platform.system()
    candidates: list[Path] = []

    if system == "Darwin":
        # macOS: ~/Library/Application Support/JetBrains
        candidates.append(Path.home() / "Library" / "Application Support" / "JetBrains")
        # New-style XDG on macOS (rare but possible)
        candidates.append(Path.home() / ".config" / "JetBrains")

    elif system == "Linux":
        # Linux: ~/.config/JetBrains  (XDG) or legacy
        xdg_config = Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
        candidates.append(xdg_config / "JetBrains")

    elif system == "Windows":
        appdata = os.environ.get("APPDATA", "")
        if appdata:
            candidates.append(Path(appdata) / "JetBrains")

    return [c for c in candidates if c.exists()]


def _match_ide_type(dir_name: str) -> str | None:
    """Infer IDE type from a JetBrains config directory name.

    Returns the canonical IDE type string, or None if not recognised.
    """
    for ide_type, prefixes in _IDE_DIR_PREFIXES.items():
        for prefix in prefixes:
            if dir_name.startswith(prefix):
                return ide_type
    return None


# ---------------------------------------------------------------------------
# Main integration class
# ---------------------------------------------------------------------------


class JetBrainsIntegration:
    """Detect installed JetBrains IDEs and manage their MCP configuration.

    Usage::

        integration = JetBrainsIntegration()
        configs = integration.detect_installed_ides()
        for cfg in configs:
            path = integration.write_mcp_config(cfg)
            print(f"Wrote {path}")
    """

    def __init__(
        self,
        mcp_server_url: str = DEFAULT_MCP_SERVER_URL,
        serena_project_root: str = "",
    ) -> None:
        """Initialise with target MCP server URL and optional project root.

        Args:
            mcp_server_url:      URL of the thegent MCP server.
            serena_project_root: Absolute project root for Serena context.
                                 If empty, the IDE plugin will use its own
                                 project root.
        """
        self.mcp_server_url = mcp_server_url
        self.serena_project_root = serena_project_root

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def detect_installed_ides(self) -> list[JetBrainsConfig]:
        """Return a list of JetBrainsConfig for every installed JetBrains IDE.

        The method scans the platform-appropriate JetBrains config base
        directory (e.g. ~/Library/Application Support/JetBrains on macOS) for
        sub-directories whose names match known IDE patterns.

        Returns:
            List of JetBrainsConfig instances, one per detected IDE installation.
            Returns an empty list when no JetBrains IDEs are found.
        """
        found: list[JetBrainsConfig] = []
        base_dirs = _jetbrains_base_dirs()

        for base_dir in base_dirs:
            if not base_dir.is_dir():
                continue
            try:
                children = list(base_dir.iterdir())
            except PermissionError:
                logger.debug("Permission denied reading %s", base_dir)
                continue

            for child in sorted(children):
                if not child.is_dir():
                    continue
                ide_type = _match_ide_type(child.name)
                if ide_type is None:
                    continue
                cfg = JetBrainsConfig(
                    ide_type=ide_type,
                    config_dir=child,
                    mcp_server_url=self.mcp_server_url,
                    serena_project_root=self.serena_project_root,
                )
                found.append(cfg)
                logger.debug("Detected JetBrains IDE: %s at %s", ide_type, child)

        return found

    def write_mcp_config(self, config: JetBrainsConfig) -> Path:
        """Write (or merge) the thegent MCP server entry into the IDE mcp.json.

        If mcp.json already exists, the existing ``mcpServers`` entries are
        preserved and the ``thegent`` entry is added / updated.  Other entries
        are left untouched.

        Args:
            config: JetBrainsConfig describing the target IDE installation.

        Returns:
            Path to the written mcp.json file.

        Raises:
            OSError: If the config directory cannot be created or written.
        """
        config.config_dir.mkdir(parents=True, exist_ok=True)

        existing = self.read_existing_config(config) or {}
        mcp_servers: dict = existing.get("mcpServers", {})

        # Build the thegent entry
        thegent_entry: dict = {"url": config.mcp_server_url}
        if config.serena_project_root:
            thegent_entry["env"] = {"SERENA_PROJECT_ROOT": config.serena_project_root}

        mcp_servers["thegent"] = thegent_entry
        output = {**existing, "mcpServers": mcp_servers}

        config.mcp_config_path.write_text(
            orjson.dumps(output, option=orjson.OPT_INDENT_2).decode("utf-8") + "\n",
            encoding="utf-8",
        )
        logger.info("Wrote MCP config to %s", config.mcp_config_path)
        return config.mcp_config_path

    def read_existing_config(self, config: JetBrainsConfig) -> dict | None:
        """Read and parse an existing mcp.json for the given IDE.

        Args:
            config: JetBrainsConfig whose ``mcp_config_path`` will be read.

        Returns:
            Parsed JSON as a dict, or ``None`` if the file does not exist or
            cannot be parsed.
        """
        path = config.mcp_config_path
        if not path.exists():
            return None
        try:
            raw = path.read_text(encoding="utf-8")
            data = orjson.loads(raw)
            if not isinstance(data, dict):
                logger.warning("mcp.json at %s is not a JSON object; ignoring", path)
                return None
            return data
        except orjson.JSONDecodeError as exc:
            logger.warning("Failed to parse %s: %s", path, exc)
            return None

    def is_mcp_plugin_installed(self, config: JetBrainsConfig) -> bool:
        """Check whether the thegent MCP plugin entry exists in the IDE config.

        Args:
            config: JetBrainsConfig for the IDE to inspect.

        Returns:
            True if mcp.json exists and contains a ``thegent`` entry under
            ``mcpServers``; False otherwise.
        """
        existing = self.read_existing_config(config)
        if existing is None:
            return False
        servers = existing.get("mcpServers", {})
        return "thegent" in servers

    def setup_all(self) -> list[dict]:
        """Detect all JetBrains IDEs and write MCP config for each.

        Convenience method that combines :meth:`detect_installed_ides` and
        :meth:`write_mcp_config`.

        Returns:
            List of result dicts, one per detected IDE, each with keys:
            ``ide_type``, ``config_dir``, ``mcp_config_path``, ``success``,
            ``error`` (present only on failure).
        """
        configs = self.detect_installed_ides()
        return [self._write_config_result(cfg) for cfg in configs]

    def _write_config_result(self, cfg: JetBrainsConfig) -> dict:
        """Write MCP config for one IDE and return a status dict.

        Encapsulates the try/except so it does not appear inside a loop,
        avoiding PERF203 overhead warnings.

        Args:
            cfg: JetBrainsConfig for the target IDE.

        Returns:
            Status dict with keys ``ide_type``, ``config_dir``,
            ``mcp_config_path``, ``success`` (and ``error`` on failure).
        """
        try:
            path = self.write_mcp_config(cfg)
            return {
                "ide_type": cfg.ide_type,
                "config_dir": str(cfg.config_dir),
                "mcp_config_path": str(path),
                "success": True,
            }
        except OSError as exc:
            logger.error("Failed to write MCP config for %s: %s", cfg.ide_type, exc)
            return {
                "ide_type": cfg.ide_type,
                "config_dir": str(cfg.config_dir),
                "mcp_config_path": str(cfg.mcp_config_path),
                "success": False,
                "error": str(exc),
            }
