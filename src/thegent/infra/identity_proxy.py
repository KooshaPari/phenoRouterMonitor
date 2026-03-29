"""SSH Identity Proxy Bridge.

Allows isolated L2 agents to use host SSH keys for Git operations
without exposing private keys to the guest environment.
"""

import logging
import os
import select
import socket
import threading
from pathlib import Path

from thegent.agents.identity import verify_actor_signature

logger = logging.getLogger(__name__)


class SSHIdentityProxy:
    """
    Acts as a secure bridge between the host's SSH agent and isolated L2 agents.
    Uses Unix Domain Sockets to forward signing requests.
    """

    def __init__(self, proxy_socket_path: Path) -> None:
        self.proxy_socket_path = proxy_socket_path
        self.host_ssh_auth_sock = os.environ.get("SSH_AUTH_SOCK")
        self._running = False
        self._thread: threading.Thread | None = None

    def start(self) -> None:
        """Start the proxy server."""
        if not self.host_ssh_auth_sock:
            logger.warning("SSH_AUTH_SOCK not set on host. Identity proxy disabled.")
            return

        if self.proxy_socket_path.exists():
            self.proxy_socket_path.unlink()

        self.proxy_socket_path.parent.mkdir(parents=True, exist_ok=True)

        self._running = True
        self._thread = threading.Thread(target=self._server_loop, daemon=True)
        self._thread.start()
        logger.info(f"SSH Identity Proxy started at {self.proxy_socket_path}")

    def stop(self) -> None:
        """Stop the proxy server."""
        self._running = False
        if self.proxy_socket_path.exists():
            self.proxy_socket_path.unlink()
        logger.info("SSH Identity Proxy stopped.")

    def _server_loop(self) -> None:
        """Listen for connections from isolated L2 agents."""
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
            s.bind(str(self.proxy_socket_path))
            s.listen(5)
            s.settimeout(1.0)

            while self._running:
                try:
                    conn, _addr = s.accept()
                    threading.Thread(target=self._handle_client, args=(conn,), daemon=True).start()
                except TimeoutError:  # noqa: PERF203 -- socket accept loop, timeout handling required
                    continue
                except Exception as e:
                    if self._running:
                        logger.error(f"Proxy server error: {e}")

    def _handle_client(self, client_conn: socket.socket) -> None:
        """Forward requests to the host SSH agent."""
        assert self.host_ssh_auth_sock is not None  # guaranteed by start() guard
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as host_conn:
                host_conn.connect(self.host_ssh_auth_sock)

                # Bidirectional forwarding loop
                client_conn.setblocking(False)
                host_conn.setblocking(False)

                def _forward_recv(src: socket.socket, dst: socket.socket) -> bool:
                    """Forward data from src to dst. Returns False if connection closed."""
                    try:
                        data = src.recv(4096)
                        if not data:
                            return False
                        dst.sendall(data)
                    except BlockingIOError:
                        pass
                    return True

                sockets = [client_conn, host_conn]
                while self._running:
                    readable, _, exceptional = select.select(sockets, [], sockets, 1.0)
                    if exceptional:
                        break
                    for sock in readable:
                        src, dst = (client_conn, host_conn) if sock is client_conn else (host_conn, client_conn)
                        if not _forward_recv(src, dst):
                            return
        except Exception as e:
            logger.error(f"Proxy handler error: {e}")
        finally:
            client_conn.close()

    def get_env(self) -> dict[str, str]:
        """Return the environment variable for L2 agents to use this proxy."""
        return {"SSH_AUTH_SOCK": str(self.proxy_socket_path), "THEGENT_IDENTITY_PROXY": "1"}

    @staticmethod
    def require_actor_identity(
        *,
        actor_id: str,
        signature: str,
        payload: str,
        signing_key: str,
    ) -> None:
        """Require and validate actor identity metadata for write operations."""
        if not actor_id.strip():
            raise ValueError("actor_id must be non-empty")
        if not signature.strip():
            raise ValueError("signature must be non-empty")
        if not signing_key.strip():
            raise ValueError("signing_key must be non-empty")
        if not verify_actor_signature(
            actor_id=actor_id,
            payload=payload,
            signing_key=signing_key,
            signature=signature,
        ):
            raise ValueError("invalid actor signature")
