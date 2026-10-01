# Wisdoverse Nexus Python SDK

Python client library for the Wisdoverse Nexus API.

## Installation

```bash
pip install wisdoverse-nexus-sdk
```

## Usage

Examples use a local gateway and synthetic account data. Substitute the URL and
credentials in your local environment, and redact them before sharing logs.

```python
import asyncio
from nexis import NexisClient

async def main():
    async with NexisClient("http://localhost:8080") as client:
        result = await client.login("user@example.com", "password")
        room = await client.create_room(name="General")
        await client.join_room(room.id)
        msg = await client.send_message(room.id, "Hello!")
        print(msg.content)

asyncio.run(main())
```

## WebSocket

```python
from nexis.websocket import WebSocketConnection

ws = WebSocketConnection("ws://localhost:8080/ws?room_id=xxx", token)
await ws.connect()
ws.on_message(lambda m: print(m))
asyncio.create_task(ws.listen())
```
