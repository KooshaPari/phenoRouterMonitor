"""API helpers for thegent.

Common API utilities.
"""

from __future__ import annotations

import httpx
from typing import Any


class APIClient:
    """Simple API client wrapper."""

    def __init__(self, base_url: str = "", timeout: float = 30.0):
        self.base_url = base_url
        self.timeout = timeout

    def get(self, path: str, **kwargs: Any) -> dict[str, Any]:
        """GET request."""
        with httpx.Client() as client:
            return client.get(f"{self.base_url}{path}", timeout=self.timeout, **kwargs).json()

    def post(self, path: str, **kwargs: Any) -> dict[str, Any]:
        """POST request."""
        with httpx.Client() as client:
            return client.post(f"{self.base_url}{path}", timeout=self.timeout, **kwargs).json()

    def put(self, path: str, **kwargs: Any) -> dict[str, Any]:
        """PUT request."""
        with httpx.Client() as client:
            return client.put(f"{self.base_url}{path}", timeout=self.timeout, **kwargs).json()

    def delete(self, path: str, **kwargs: Any) -> dict[str, Any]:
        """DELETE request."""
        with httpx.Client() as client:
            return client.delete(f"{self.base_url}{path}", timeout=self.timeout, **kwargs).json()
