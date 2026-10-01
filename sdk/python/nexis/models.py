"""Models for the supported gateway collaboration contract."""
from __future__ import annotations
from dataclasses import dataclass, field


@dataclass
class AuthResult:
    token: str
    member_id: str
    member_type: str
    expires_at: int
    refresh_supported: bool = False


@dataclass
class Message:
    id: str
    room_id: str
    sender: str
    text: str
    reply_to: str | None = None


@dataclass
class Room:
    id: str
    name: str
    topic: str | None = None
    messages: list[Message] = field(default_factory=list)
    member_count: int | None = None


@dataclass
class CreateRoomData:
    name: str
    topic: str | None = None
