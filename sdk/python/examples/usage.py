"""M1 usage example with an externally issued JWT and the Nexus gateway."""

import asyncio

from nexis import NexisClient
from nexis.models import CreateRoomData
from nexis.websocket import WebSocketConnection


async def main() -> None:
    # Supply a short-lived development token through your environment or secret
    # manager. Never commit a real token or print it in logs.
    token = "<JWT_FROM_YOUR_APPLICATION>"

    async with NexisClient("http://localhost:8080") as client:
        session = await client.authenticate(token)
        print(
            f"Authenticated member {session.member_id} "
            f"({session.member_type}); expires at {session.expires_at} ms"
        )

        rooms = await client.list_rooms(limit=100, offset=0)
        room = rooms[0] if rooms else await client.create_room(
            CreateRoomData(name="M1 demo", topic="SDK example")
        )
        print(f"Using room {room.id}: {room.name}")

        sent = await client.send_message(room.id, "Hello from the Python SDK")
        print(f"Sent message {sent.id}")

        history = await client.get_messages(room.id)
        print(f"Fetched {len(history)} messages from HTTP history")

        ws = WebSocketConnection("ws://localhost:8080/ws", token, room_id=room.id)
        await ws.connect()
        try:
            # The gateway confirms auth and room subscription before connect returns.
            await ws.send({
                "type": "send_message",
                "room_id": room.id,
                "content": "Hello over WebSocket",
            })
            async for event in ws.messages():
                print(f"WebSocket event: {event.get('type')}")
                if event.get("type") == "new_message":
                    break
        finally:
            await ws.close()


if __name__ == "__main__":
    asyncio.run(main())
