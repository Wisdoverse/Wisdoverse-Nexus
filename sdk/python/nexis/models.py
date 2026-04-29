"""Data models for Wisdoverse Nexus SDK."""

from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Optional


class MemberRole(str, Enum):
    OWNER = "owner"
    ADMIN = "admin"
    MEMBER = "member"


class MessageType(str, Enum):
    TEXT = "text"
    SYSTEM = "system"
    FILE = "file"


@dataclass
class User:
    id: str
    email: str
    display_name: str
    avatar_url: Optional[str] = None
    created_at: Optional[str] = None


@dataclass
class AuthResult:
    token: str
    refresh_token: str
    user: User


@dataclass
class Member:
    id: str
    user_id: str
    display_name: str
    role: MemberRole = MemberRole.MEMBER
    avatar_url: Optional[str] = None
    joined_at: Optional[str] = None


@dataclass
class Room:
    id: str
    name: str
    description: Optional[str] = None
    is_private: bool = False
    created_by: str = ""
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    members: list[Member] = field(default_factory=list)


@dataclass
class Message:
    id: str
    room_id: str
    sender_id: str
    sender_name: str
    content: str
    type: MessageType = MessageType.TEXT
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


@dataclass
class CreateRoomData:
    name: str
    description: Optional[str] = None
    is_private: bool = False


@dataclass
class RegisterData:
    email: str
    password: str
    display_name: Optional[str] = None
