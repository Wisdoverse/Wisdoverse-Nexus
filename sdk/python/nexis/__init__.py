"""Wisdoverse Nexus SDK - Python client for Wisdoverse Nexus API."""

from nexis.client import NexisClient
from nexis.websocket import WebSocketConnection
from nexis.models import Room, Message, Member, AuthResult

__version__ = "0.1.0"
__all__ = ["NexisClient", "WebSocketConnection", "Room", "Message", "Member", "AuthResult"]
