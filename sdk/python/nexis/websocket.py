"""First-message authentication and room subscription with bounded reconnect."""
from __future__ import annotations
import asyncio
import json
from typing import AsyncIterator
import websockets
from websockets.asyncio.client import ClientConnection
from websockets.protocol import State


class WebSocketConnection:
    def __init__(self, url: str, token: str, room_id: str | None = None,
                 max_reconnect: int = 5, reconnect_delay: float = 1.0):
        if max_reconnect < 0 or reconnect_delay < 0:
            raise ValueError("Reconnect settings must be nonnegative")
        self._url, self._token, self._room_id = url, token, room_id
        self._max_reconnect, self._reconnect_delay = max_reconnect, reconnect_delay
        self._ws: ClientConnection | None = None
        self._stop = asyncio.Event()
        self._handlers = []

    async def connect(self) -> None:
        await self.close()
        self._stop.clear()
        await self._do_connect()

    async def _do_connect(self) -> None:
        for attempt in range(self._max_reconnect + 1):
            if self._stop.is_set():
                raise ConnectionError("Connection was closed")
            socket = None
            try:
                socket = await websockets.connect(self._url, open_timeout=10)
                await socket.send(json.dumps({"type": "auth", "token": f"Bearer {self._token}"}))
                result = json.loads(await asyncio.wait_for(socket.recv(), 10))
                if result.get("type") != "auth_success":
                    raise PermissionError("Gateway rejected WebSocket authentication")
                if self._room_id:
                    await socket.send(json.dumps({"type": "join_room", "room_id": self._room_id}))
                    result = json.loads(await asyncio.wait_for(socket.recv(), 10))
                    if result.get("type") != "room_joined" or result.get("room_id") != self._room_id:
                        raise PermissionError("Gateway rejected room subscription")
                if self._stop.is_set():
                    await socket.close()
                    raise ConnectionError("Connection was closed")
                self._ws = socket
                return
            except PermissionError:
                if socket:
                    await socket.close()
                self._stop.set()
                raise
            except (OSError, TimeoutError, websockets.WebSocketException):
                if socket:
                    await socket.close()
                if attempt == self._max_reconnect:
                    raise ConnectionError("WebSocket reconnect attempts exhausted") from None
                try:
                    await asyncio.wait_for(self._stop.wait(), min(30, self._reconnect_delay * 2 ** attempt))
                except TimeoutError:
                    # The backoff elapsed; retry the connection unless stop was requested.
                    pass
        raise ConnectionError("Connection was closed")

    async def send(self, message: dict) -> None:
        if not self.is_connected:
            raise ConnectionError("WebSocket is not connected")
        await self._ws.send(json.dumps(message))

    async def messages(self) -> AsyncIterator[dict]:
        if not self._ws:
            raise ConnectionError("Call connect() first")
        while not self._stop.is_set():
            try:
                async for raw in self._ws:
                    message = json.loads(raw)
                    if message.get("type") in ("auth_error", "auth_required"):
                        self._stop.set()
                        raise PermissionError("WebSocket credentials expired or were rejected")
                    yield message
            except websockets.ConnectionClosed:
                # A dropped socket is recoverable; reconnect below and resume the stream.
                pass
            if not self._stop.is_set():
                await self._do_connect()

    async def listen(self) -> None:
        async for message in self.messages():
            for handler in self._handlers:
                handler(message)

    def on_message(self, handler) -> None:
        self._handlers.append(handler)

    async def close(self) -> None:
        self._stop.set()
        socket, self._ws = self._ws, None
        if socket:
            await socket.close()

    @property
    def is_connected(self) -> bool:
        return self._ws is not None and self._ws.state is State.OPEN
