"""Fast WebSocket client with optimized backends.

This module provides optimized WebSocket client support:
- websockets library (modern, fast, async-first)
- websocket-client fallback (legacy support)
- Unified API for both sync and async operations

Performance improvements:
- websockets: Modern, faster, better async support
- Better resource management
- Automatic backend selection
"""

from types import TracebackType
from typing import Any

try:
    from websockets import connect as ws_connect

    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False

try:
    import websocket

    WEBSOCKET_CLIENT_AVAILABLE = True
except ImportError:
    WEBSOCKET_CLIENT_AVAILABLE = False


class FastWebSocket:
    """High-performance WebSocket client with automatic backend selection."""

    def __init__(self, url: str, **kwargs) -> None:
        """Initialize WebSocket client.

        Args:
            url: WebSocket URL (ws:// or wss://)
            **kwargs: Additional connection options
        """
        self.url = url
        self.options = kwargs
        self._ws: Any = None
        self._backend: str | None = None

    async def connect_async(self) -> None:
        """Connect asynchronously using websockets library.

        Performance:
            - websockets: Modern, fast, async-first
            - Better resource management
            - Non-blocking connection
        """
        import asyncio

        if WEBSOCKETS_AVAILABLE:
            self._backend = "websockets"
            self._ws = await ws_connect(self.url, **self.options)
        elif WEBSOCKET_CLIENT_AVAILABLE:
            # Fallback: run synchronous websocket-client in thread pool
            self._backend = "websocket-client-async"
            self._ws = await asyncio.to_thread(websocket.create_connection, self.url, **self.options)
        else:
            raise ImportError("No WebSocket library available. Install 'websockets' or 'websocket-client'.")

    def connect_sync(self) -> None:
        """Connect synchronously using websocket-client.

        Performance:
            - websocket-client: Legacy, sync-only
            - Fallback for compatibility
        """
        if WEBSOCKET_CLIENT_AVAILABLE:
            self._backend = "websocket-client"
            self._ws = websocket.create_connection(self.url, **self.options)
        else:
            raise ImportError("No WebSocket library available. Install 'websocket-client' for sync support.")

    async def send_async(self, data: str | bytes) -> None:
        """Send data asynchronously."""
        import asyncio

        if self._backend == "websockets":
            await self._ws.send(data)
        elif self._backend == "websocket-client-async":
            await asyncio.to_thread(self._ws.send, data)
        else:
            raise RuntimeError("Not connected or wrong backend")

    def send_sync(self, data: str | bytes) -> None:
        """Send data synchronously."""
        if self._backend == "websocket-client":
            self._ws.send(data)
        else:
            raise RuntimeError("Not connected or wrong backend")

    async def recv_async(self) -> str | bytes:
        """Receive data asynchronously."""
        import asyncio

        if self._backend == "websockets":
            return await self._ws.recv()
        if self._backend == "websocket-client-async":
            return await asyncio.to_thread(self._ws.recv)
        raise RuntimeError("Not connected or wrong backend")

    def recv_sync(self) -> str | bytes:
        """Receive data synchronously."""
        if self._backend == "websocket-client":
            return self._ws.recv()
        raise RuntimeError("Not connected or wrong backend")

    async def close_async(self) -> None:
        """Close connection asynchronously."""
        import asyncio

        if self._backend == "websockets":
            await self._ws.close()
        elif self._backend in ("websocket-client", "websocket-client-async"):
            await asyncio.to_thread(self._ws.close)

    def close_sync(self) -> None:
        """Close connection synchronously."""
        if self._backend == "websocket-client":
            self._ws.close()
        elif self._backend == "websockets":
            # Can't close async websocket synchronously
            raise RuntimeError("Use close_async() for websockets backend")

    async def __aenter__(self) -> "FastWebSocket":
        """Async context manager entry."""
        await self.connect_async()
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        """Async context manager exit."""
        await self.close_async()

    def __enter__(self) -> "FastWebSocket":
        """Sync context manager entry."""
        self.connect_sync()
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        """Sync context manager exit."""
        self.close_sync()


# Convenience functions
async def websocket_connect_async(url: str, **kwargs) -> FastWebSocket:
    """Create and connect WebSocket asynchronously."""
    ws = FastWebSocket(url, **kwargs)
    await ws.connect_async()
    return ws


def websocket_connect_sync(url: str, **kwargs) -> FastWebSocket:
    """Create and connect WebSocket synchronously."""
    ws = FastWebSocket(url, **kwargs)
    ws.connect_sync()
    return ws
