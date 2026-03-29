"""Helpers for WL-116 audio transcript input handling."""

from __future__ import annotations

import os
from pathlib import Path

_TEXT_TRANSCRIPT_SUFFIXES = {".txt", ".md", ".srt"}
_BINARY_AUDIO_SUFFIXES = {".mp3", ".wav", ".m4a", ".ogg", ".flac", ".webm", ".mp4"}


def _read_text_transcript(path: Path) -> str:
    """Read UTF-8 transcript files and normalize optional BOM prefix."""
    return path.read_text(encoding="utf-8").lstrip("\ufeff").strip()


def _load_srt_transcript(path: Path) -> str:
    """Parse SRT into plain text by dropping indices/timestamps."""
    chunks: list[str] = []
    for raw in path.read_text(encoding="utf-8").lstrip("\ufeff").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.isdigit():
            continue
        if "-->" in line:
            continue
        chunks.append(line)
    return "\n".join(chunks).strip()


def transcribe_audio_file(path: Path, model: str = "whisper-1") -> str:
    """Transcribe audio file using OpenAI Whisper API.

    # @trace WL-116

    Args:
        path: Path to the binary audio file to transcribe.
        model: Whisper model identifier (default: "whisper-1").

    Returns:
        Transcribed text from the audio file.

    Raises:
        FileNotFoundError: If the audio file does not exist.
        ValueError: If the file suffix is not a supported binary audio format.
        RuntimeError: If OPENAI_API_KEY is not set or the API call fails.
    """
    if not path.exists():
        raise FileNotFoundError(f"Audio file not found: {path}")
    if path.suffix.lower() not in _BINARY_AUDIO_SUFFIXES:
        raise ValueError(
            f"Unsupported audio file format: {path.suffix}. "
            f"Supported binary audio formats: {sorted(_BINARY_AUDIO_SUFFIXES)}"
        )
    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        raise RuntimeError("OPENAI_API_KEY is not set. Cannot call Whisper transcription API.")
    try:
        from openai import OpenAI
    except ImportError as exc:
        raise RuntimeError(
            "openai package is required for Whisper transcription. Install with: pip install openai"
        ) from exc

    client = OpenAI(api_key=api_key)
    with path.open("rb") as audio_file:
        response = client.audio.transcriptions.create(
            model=model,
            file=audio_file,
        )
    return response.text


def build_codex_audio_include() -> list[str]:
    """Return include list for Codex Responses API audio transcript passthrough.

    # @trace WL-116

    Returns:
        List of include strings to pass to the Codex Responses API so that
        input audio transcripts are surfaced in the response.
    """
    return ["item.input_audio.transcript"]


def inject_transcript_into_prompt(prompt: str, transcript: str) -> str:
    """Inject audio transcript into agent prompt.

    # @trace WL-116

    Args:
        prompt: The original agent prompt.
        transcript: The transcript text to prepend.

    Returns:
        Combined prompt with transcript block prepended.
    """
    return f"[AUDIO TRANSCRIPT]\n{transcript}\n[/AUDIO TRANSCRIPT]\n\n{prompt}"


def load_transcripts(paths: list[str]) -> tuple[str, list[str]]:
    """Load transcript text from provided paths.

    Accepts transcript text files (.txt, .md, .srt) and binary audio files
    (.mp3, .wav, .m4a, .ogg, .flac, .webm, .mp4). Binary audio files are
    transcribed via the OpenAI Whisper API (requires OPENAI_API_KEY).

    # @trace WL-116
    """
    snippets: list[str] = []
    used_paths: list[str] = []
    for raw in paths:
        p = Path(raw).expanduser()
        if not p.exists():
            raise FileNotFoundError(f"Audio input not found: {p}")
        suffix = p.suffix.lower()
        if suffix in _TEXT_TRANSCRIPT_SUFFIXES:
            if suffix == ".srt":
                text = _load_srt_transcript(p)
            else:
                text = _read_text_transcript(p)
        elif suffix in _BINARY_AUDIO_SUFFIXES:
            text = transcribe_audio_file(p)
        else:
            raise ValueError(
                f"Unsupported --audio input {p}. "
                f"Accepted formats: text ({sorted(_TEXT_TRANSCRIPT_SUFFIXES)}), "
                f"binary audio ({sorted(_BINARY_AUDIO_SUFFIXES)})."
            )
        if text:
            snippets.append(text)
            used_paths.append(str(p))
    return ("\n\n".join(snippets), used_paths)
