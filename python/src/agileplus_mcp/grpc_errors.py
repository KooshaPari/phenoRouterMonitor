"""Shared gRPC client exceptions."""

from __future__ import annotations

from typing import Any


class GrpcConnectionError(Exception):
    """Raised when the gRPC channel cannot be established."""


class GrpcCallError(Exception):
    """Raised when a gRPC call fails with a non-retryable status."""

    def __init__(self, code: Any, message: str) -> None:
        self.code = code
        super().__init__(f"gRPC error {code}: {message}")
