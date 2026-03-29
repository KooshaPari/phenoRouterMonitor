"""Provider configuration constants and utilities (domain layer).

Shared constants for provider management that are safe for use-cases and agents layers.
Prevents use_cases layer from importing from agents layer.
"""

from typing import Final

# OAuth-only providers: no API key option. Claude and Codex require OAuth.
OAUTH_ONLY_PROVIDERS: Final[frozenset[str]] = frozenset({"claude", "codex"})
