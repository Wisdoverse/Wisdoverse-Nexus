"""WebSocket connection for Wisdoverse Nexus."""

from __future__ import annotations
import asyncio
import json
from typing import Any, AsyncIterator, Callable, Optional
import websockets
from websockets.asyncio.client import ClientConnection


class WebSocketConnection:
    """Async WebSocket connection to Wisdoverse Nexus gateway."""

    def __init__(
        self,
        url: str,
        token: str,
        max_reconnect: int = 5,
        reconnect_delay: float = 1.0,
    ):
        self._url = url
        self._token = token
        self._max_reconnect = max_reconnect
        self._reconnect_delay = reconnect_delay
        self._ws: Optional[ClientConnection] = None
        self._should_reconnect = True
        self._handlers: list[Callable[[dict], Any]] = []

    async def connect(self) -> None:
        self._should_reconnect = True
        await self._do_connect()

    async def _do_connect(self) -> None:
        attempt = 0
        while attempt < self._max_reconnect:
            try:
                headers = {"Authorization": f"Bearer {self._token}"}
                self._ws = await websockets.connect(self._url, additional_headers=headers)
                return
            except Exception:
                attempt += 1
                if attempt < self._max_reconnect:
                    delay = self._reconnect_delay * (2 ** (attempt - 1))
                    await asyncio.sleep(delay)
        raise ConnectionError(f"Failed to connect after {self._max_reconnect} attempts")

    async def send(self, message: dict) -> None:
        if self._ws and not self._ws.closed:
            await self._ws.send(json.dumps(message))
        else:
            raise ConnectionError("WebSocket is not connected")

    async def messages(self) -> AsyncIterator[dict]:
        """Yield incoming messages as dicts."""
        if not self._ws:
            raise ConnectionError("WebSocket is not connected")
        try:
            async for raw in self._ws:
                yield json.loads(raw)
        except websockets.ConnectionClosed:
            pass

    async def listen(self) -> None:
        """Listen and dispatch messages to handlers."""
        async for msg in self.messages():
            for handler in self._handlers:
                handler(msg)

    def on_message(self, handler: Callable[[dict], Any]) -> None:
        self._handlers.append(handler)

    async def close(self) -> None:
        self._should_reconnect = False
        if self._ws and not self._ws.closed:
            await self._ws.close()

    @property
    def is_connected(self) -> bool:
        return self._ws is not None and not self._ws.closed
