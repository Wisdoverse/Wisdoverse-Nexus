# Wisdoverse Nexus Python SDK

Python client library for the Wisdoverse Nexus API.

## Installation

```bash
pip install wisdoverse-nexus-sdk
```

## Usage

```python
import asyncio
from nexis import NexisClient

async def main():
    async with NexisClient("https://api.wisdoverse.com") as client:
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

ws = WebSocketConnection("wss://api.wisdoverse.com/ws?room_id=xxx", token)
await ws.connect()
ws.on_message(lambda m: print(m))
asyncio.create_task(ws.listen())
```
