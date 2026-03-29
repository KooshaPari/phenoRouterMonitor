"""HTTP utilities for thegent.

Common HTTP client helpers.
"""

from __future__ import annotations

import httpx
from typing import Any


class HTTPClient:
    """Simple HTTP client wrapper."""

    def __init__(self, base_url: str = "", timeout: float = 30.0):
        self.base_url = base_url
        self.timeout = timeout
        self._client: httpx.Client | None = None

    @property
    def client(self) -> httpx.Client:
        if self._client is None:
            self._client = httpx.Client(
                base_url=self.base_url,
                timeout=self.timeout,
            )
        return self._client

    def get(self, url: str, **kwargs: Any) -> httpx.Response:
        return self.client.get(url, **kwargs)

    def post(self, url: str, **kwargs: Any) -> httpx.Response:
        return self.client.post(url, **kwargs)

    def close(self) -> None:
        if self._client:
            self._client.close()
            self._client = None

    def __enter__(self) -> "HTTPClient":
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()


def get(url: str, **kwargs: Any) -> httpx.Response:
    """Simple GET request."""
    return httpx.get(url, **kwargs)


def post(url: str, **kwargs: Any) -> httpx.Response:
    """Simple POST request."""
    return httpx.post(url, **kwargs)
