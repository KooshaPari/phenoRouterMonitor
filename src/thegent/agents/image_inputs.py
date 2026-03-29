"""Helpers for WL-114 image input handling.

# @trace WL-114
"""

from __future__ import annotations

import base64
import json
import mimetypes
from pathlib import Path

SUPPORTED_FORMATS: frozenset[str] = frozenset({".png", ".jpg", ".jpeg", ".webp", ".gif"})

_SUFFIX_TO_MEDIA_TYPE: dict[str, str] = {
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".webp": "image/webp",
    ".gif": "image/gif",
}


def encode_image_to_base64(path: Path) -> str:
    """Encode image file to base64 string.

    # @trace WL-114
    """
    return base64.standard_b64encode(path.read_bytes()).decode("ascii")


def _media_type_for(path: Path) -> str:
    """Return MIME type for the given image path.

    Falls back to mimetypes module if suffix not in explicit map.
    Raises ValueError for unknown types.

    # @trace WL-114
    """
    suffix = path.suffix.lower()
    if suffix in _SUFFIX_TO_MEDIA_TYPE:
        return _SUFFIX_TO_MEDIA_TYPE[suffix]
    guessed, _ = mimetypes.guess_type(str(path))
    if guessed and guessed.startswith("image/"):
        return guessed
    raise ValueError(f"Cannot determine image media type for: {path}")


def build_image_content_block(path: Path) -> dict:
    """Build an Anthropic/OpenAI image content block from a local file.

    Returns a dict in Anthropic content-block format:
    {"type": "image", "source": {"type": "base64", "media_type": "...", "data": "..."}}

    # @trace WL-114
    """
    media_type = _media_type_for(path)
    data = encode_image_to_base64(path)
    return {
        "type": "image",
        "source": {
            "type": "base64",
            "media_type": media_type,
            "data": data,
        },
    }


def build_codex_image_args(paths: list[str]) -> list[str]:
    """Build --image args for Codex CLI.

    Returns flat list: ["--image", path1, "--image", path2, ...]

    # @trace WL-114
    """
    args: list[str] = []
    for path in paths:
        if not isinstance(path, str) or not path.strip():
            raise ValueError("Image path values must be non-empty strings.")
        args.extend(["--image", path.strip()])
    return args


def build_claude_stdin_with_images(prompt: str, paths: list[str]) -> str:
    """Build JSONL stdin payload for Claude CLI with --input-format stream-json.

    Constructs a single user message with image content blocks followed by the
    text prompt. Returns a newline-terminated JSONL string.

    # @trace WL-114
    """
    content: list[dict] = []
    for raw in paths:
        p = Path(raw)
        block = build_image_content_block(p)
        content.append(block)
    content.append({"type": "text", "text": prompt})
    message = {
        "type": "user",
        "message": {
            "role": "user",
            "content": content,
        },
    }
    return json.dumps(message, separators=(",", ":")) + "\n"
