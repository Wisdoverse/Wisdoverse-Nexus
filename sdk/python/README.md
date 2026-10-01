# Wisdoverse Nexus Python SDK

The Python SDK is an async client for the Nexus collaboration gateway. Its M1
session flow accepts a JWT issued by your application; the gateway validates
the token and returns its member identity and expiry. The SDK does not create
accounts or issue, refresh, or revoke credentials.

## Installation

```bash
pip install wisdoverse-nexus-sdk
```

## HTTP usage

Create a token through your application's trusted authentication system and
keep it out of source control and logs. The gateway's `GET /v1/auth/session`
validates the supplied HS256 JWT. A successful result includes `member_id`,
`member_type`, `expires_at` (Unix time in milliseconds), and
`refresh_supported=False`. M1 supports only the default single-tenant,
in-memory, single-process configuration. Multi-tenant core flows are explicitly
disabled with HTTP 503 until M3 acceptance.

```python
import asyncio

from nexis import NexisClient
from nexis.models import CreateRoomData


async def main() -> None:
    token = "<JWT_FROM_YOUR_APPLICATION>"
    async with NexisClient("http://localhost:8080") as client:
        session = await client.authenticate(token)
        print(f"Authenticated member {session.member_id} ({session.member_type})")

        room = await client.create_room(CreateRoomData(name="General", topic="Updates"))
        message = await client.send_message(room.id, "Hello from Python")
        print(f"Created {room.name}; message {message.id}")

        history = await client.get_messages(room.id)
        print(f"History contains {len(history)} messages")


asyncio.run(main())
```

`NexisClient` also provides `list_rooms(limit=100, offset=0)`, `get_room`,
`delete_room`, and `invite_member`. Messages use `id`, `room_id`, `sender`,
`text`, and optional `reply_to`; rooms use `id`, `name`, optional `topic`,
`messages`, and `member_count`. Only a room's creator or invited members may
read it, write messages, or subscribe to it. Only the creator may invite
members or delete the room.

For retry-safe message sends, provide the optional `client_message_id` on
WebSocket sends or `clientMessageId` on HTTP sends. The key must be non-empty
and at most 128 bytes. The service retains at most 10,000 keys for 60 minutes,
scoped by sender, room, and key; if capacity is exhausted it returns HTTP 503.
Retrying the same content returns the original message ID, while reusing the
key with different content returns HTTP 409. Use a new key for a new message.

## WebSocket usage

The WebSocket endpoint is `/ws`. It does not take a token or room ID in the
URL. Connect with a token and optional room ID; the first protocol messages
authenticate and subscribe before `connect()` succeeds. Read events with
`async for`, and close explicitly when finished:

```python
from nexis.websocket import WebSocketConnection

ws = WebSocketConnection("ws://localhost:8080/ws", token, room_id=room.id)
await ws.connect()
await ws.send({
    "type": "send_message",
    "room_id": room.id,
    "content": "Hello over WebSocket",
})

async for event in ws.messages():
    print(event)
    if event.get("type") == "new_message":
        break

await ws.close()
```

Events use flat `snake_case` fields. Clients authenticate with `auth` and
receive `auth_success`; subscribe with `join_room` and receive `room_joined`;
send `send_message` with `room_id`, `content`, and optional `reply_to`;
receive `new_message` with `message_id`, `sender_id`, and a millisecond
`timestamp`. Heartbeats use `heartbeat` and `heartbeat_ack`. Reconnecting
repeats authentication and room subscription. Fetch HTTP message history after
reconnect to fill any gap while disconnected.

## M1 scope and migration

Earlier examples that call `login(email, password)` or
`register(RegisterData(...))` describe an API that is not part of M1. Replace
those calls with `authenticate(token)` using an externally issued JWT, and
handle expiry in your identity provider because refresh is unsupported. The
old user/email-oriented auth result and room/message model fields are also
removed; use the member identity and models listed above. The default preview
uses in-memory storage, one tenant, and one process. Durable persistence and
multi-tenant acceptance belong to M3. Complete message history is available
only during the current process lifetime; a restart clears this recovery
window.

The planned M1 acceptance command is `bash scripts/m1_smoke.sh`. It will build
and start an isolated real gateway and exercise the TypeScript and Python SDKs
and Web/mobile APIs, writing `artifacts/m1/summary.json` and
`artifacts/m1/gateway.log`. The log must not contain tokens or secrets. This
command describes the acceptance plan; its results must be read from the
generated report.
