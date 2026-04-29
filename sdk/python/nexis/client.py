"""HTTP client for Wisdoverse Nexus API."""

from __future__ import annotations
import httpx
from typing import Optional
from .models import (
    AuthResult, Room, Message, Member, CreateRoomData, RegisterData, User
)


class NexisClient:
    """Async HTTP client for the Wisdoverse Nexus API."""

    def __init__(self, base_url: str, timeout: int = 30):
        self._base_url = base_url.rstrip("/")
        self._token: Optional[str] = None
        self._client = httpx.AsyncClient(
            base_url=self._base_url,
            timeout=timeout,
            headers={"Content-Type": "application/json"},
        )

    @property
    def base_url(self) -> str:
        return self._base_url

    @property
    def ws_url(self) -> str:
        return self._base_url.replace("http", "ws")

    def _headers(self) -> dict[str, str]:
        headers = {}
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        return headers

    def _require_auth(self):
        if not self._token:
            raise RuntimeError("Not authenticated. Call login() or register() first.")

    async def login(self, email: str, password: str) -> AuthResult:
        resp = await self._client.post("/v1/auth/login", json={"email": email, "password": password})
        resp.raise_for_status()
        data = resp.json()
        self._token = data["token"]
        return self._parse_auth(data)

    async def register(self, data: RegisterData) -> AuthResult:
        payload = {"email": data.email, "password": data.password}
        if data.display_name:
            payload["display_name"] = data.display_name
        resp = await self._client.post("/v1/auth/register", json=payload)
        resp.raise_for_status()
        result = resp.json()
        self._token = result["token"]
        return self._parse_auth(result)

    # Rooms
    async def create_room(self, data: CreateRoomData) -> Room:
        self._require_auth()
        resp = await self._client.post("/v1/rooms", json={
            "name": data.name, "description": data.description, "is_private": data.is_private
        }, headers=self._headers())
        resp.raise_for_status()
        return self._parse_room(resp.json())

    async def get_room(self, room_id: str) -> Room:
        self._require_auth()
        resp = await self._client.get(f"/v1/rooms/{room_id}", headers=self._headers())
        resp.raise_for_status()
        return self._parse_room(resp.json())

    async def list_rooms(self) -> list[Room]:
        self._require_auth()
        resp = await self._client.get("/v1/rooms", headers=self._headers())
        resp.raise_for_status()
        return [self._parse_room(r) for r in resp.json()]

    async def join_room(self, room_id: str) -> None:
        self._require_auth()
        resp = await self._client.post(f"/v1/rooms/{room_id}/join", headers=self._headers())
        resp.raise_for_status()

    async def leave_room(self, room_id: str) -> None:
        self._require_auth()
        resp = await self._client.post(f"/v1/rooms/{room_id}/leave", headers=self._headers())
        resp.raise_for_status()

    # Messages
    async def send_message(self, room_id: str, content: str) -> Message:
        self._require_auth()
        resp = await self._client.post(f"/v1/rooms/{room_id}/messages", json={"content": content}, headers=self._headers())
        resp.raise_for_status()
        return self._parse_message(resp.json())

    async def get_messages(self, room_id: str, limit: int = 50, before: str | None = None) -> list[Message]:
        self._require_auth()
        params = {"limit": limit}
        if before:
            params["before"] = before
        resp = await self._client.get(f"/v1/rooms/{room_id}/messages", params=params, headers=self._headers())
        resp.raise_for_status()
        return [self._parse_message(m) for m in resp.json()]

    # GDPR
    async def export_data(self) -> dict:
        self._require_auth()
        resp = await self._client.get("/v1/members/me/export", headers=self._headers())
        resp.raise_for_status()
        return resp.json()

    async def delete_data(self, confirm: bool = True) -> dict:
        self._require_auth()
        resp = await self._client.delete("/v1/members/me", json={"confirm": confirm}, headers=self._headers())
        resp.raise_for_status()
        return resp.json()

    async def close(self):
        await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.close()

    # Parsers
    def _parse_auth(self, data: dict) -> AuthResult:
        return AuthResult(
            token=data["token"],
            refresh_token=data.get("refresh_token", ""),
            user=User(
                id=data["user"]["id"],
                email=data["user"]["email"],
                display_name=data["user"].get("display_name", ""),
                created_at=data["user"].get("created_at"),
            ),
        )

    def _parse_room(self, data: dict) -> Room:
        return Room(
            id=data["id"], name=data["name"],
            description=data.get("description"),
            is_private=data.get("is_private", False),
            created_by=data.get("created_by", ""),
            created_at=data.get("created_at"),
            updated_at=data.get("updated_at"),
        )

    def _parse_message(self, data: dict) -> Message:
        from .models import MessageType
        return Message(
            id=data["id"], room_id=data["room_id"],
            sender_id=data["sender_id"], sender_name=data.get("sender_name", ""),
            content=data["content"],
            type=MessageType(data.get("type", "text")),
            created_at=data.get("created_at"),
        )
