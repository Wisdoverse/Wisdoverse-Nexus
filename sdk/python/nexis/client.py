"""Async HTTP client for the supported gateway contract."""
from __future__ import annotations
from urllib.parse import quote, urlsplit, urlunsplit
import httpx
from uuid import uuid4
from .models import AuthResult, CreateRoomData, Message, Room


class NexisClient:
    def __init__(self, base_url: str, timeout: int = 30):
        self._base_url = base_url.rstrip("/")
        self._token: str | None = None
        self._client = httpx.AsyncClient(base_url=self._base_url, timeout=timeout)

    @property
    def base_url(self) -> str:
        return self._base_url

    @property
    def ws_url(self) -> str:
        url = urlsplit(self._base_url)
        return urlunsplit(("wss" if url.scheme == "https" else "ws", url.netloc, "", "", ""))

    async def authenticate(self, token: str) -> AuthResult:
        response = await self._client.get("/v1/auth/session", headers={"Authorization": f"Bearer {token}"})
        response.raise_for_status()
        data = response.json()
        self._token = token
        self._client.headers["Authorization"] = f"Bearer {token}"
        return AuthResult(token, data["memberId"], data["memberType"], data["expiresAt"])

    def _require_auth(self) -> None:
        if not self._token:
            raise RuntimeError("Call authenticate(token) first")

    async def _request(self, method: str, path: str, **kwargs) -> httpx.Response:
        self._require_auth()
        response = await self._client.request(method, path, **kwargs)
        response.raise_for_status()
        return response

    async def create_room(self, data: CreateRoomData) -> Room:
        response = await self._request("POST", "/v1/rooms", json={"name": data.name, "topic": data.topic})
        return self._parse_room(response.json())

    async def get_room(self, room_id: str) -> Room:
        response = await self._request("GET", f"/v1/rooms/{quote(room_id, safe='')}")
        return self._parse_room(response.json())

    async def list_rooms(self, limit: int = 100, offset: int = 0) -> list[Room]:
        response = await self._request("GET", "/v1/rooms", params={"limit": limit, "offset": offset})
        return [self._parse_room(room) for room in response.json()["rooms"]]

    async def delete_room(self, room_id: str) -> None:
        await self._request("DELETE", f"/v1/rooms/{quote(room_id, safe='')}")

    async def invite_member(self, room_id: str, member_id: str) -> None:
        await self._request("POST", f"/v1/rooms/{quote(room_id, safe='')}/invite", json={"memberId": member_id})

    async def send_message(self, room_id: str, text: str, reply_to: str | None = None, client_message_id: str | None = None) -> Message:
        response = await self._request("POST", "/v1/messages", json={"roomId": room_id, "text": text, "replyTo": reply_to, "clientMessageId": client_message_id or str(uuid4())})
        return self._parse_message(response.json())

    async def get_messages(self, room_id: str) -> list[Message]:
        response = await self._request("GET", f"/v1/rooms/{quote(room_id, safe='')}/messages")
        return [self._parse_message(message) for message in response.json()]

    async def export_data(self) -> dict:
        return (await self._request("GET", "/v1/members/me/export")).json()

    async def delete_data(self, confirm: bool = True) -> dict:
        return (await self._request("DELETE", "/v1/members/me", json={"confirm": confirm})).json()

    def logout(self) -> None:
        self._token = None
        self._client.headers.pop("Authorization", None)

    async def close(self) -> None:
        await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.close()

    @staticmethod
    def _parse_message(data: dict) -> Message:
        return Message(data["id"], data["roomId"], data["sender"], data["text"], data.get("reply_to"))

    @classmethod
    def _parse_room(cls, data: dict) -> Room:
        return Room(data["id"], data["name"], data.get("topic"),
                    [cls._parse_message(message) for message in data.get("messages", [])], data.get("member_count"))
