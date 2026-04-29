"""Wisdoverse Nexus Python SDK usage example."""

import asyncio
from nexis import NexisClient
from nexis.models import RegisterData
from nexis.websocket import WebSocketConnection


async def main():
    async with NexisClient("https://api.example.com") as client:
        # Auth
        result = await client.register(RegisterData(
            email="user@example.com",
            password="securepassword",
            display_name="Alice"
        ))
        print(f"Logged in as {result.user.display_name}")

        # Rooms
        room = await client.create_room(name="General")
        print(f"Created room: {room.id} - {room.name}")
        await client.join_room(room.id)

        # Messages
        msg = await client.send_message(room.id, "Hello from Python SDK!")
        print(f"Sent: {msg.content}")

        messages = await client.get_messages(room.id, limit=20)
        print(f"History: {len(messages)} messages")

        # WebSocket
        ws = WebSocketConnection(f"{client.ws_url}/ws?room_id={room.id}", result.token)
        await ws.connect()

        ws.on_message(lambda m: print(f"Received: {m}"))
        asyncio.create_task(ws.listen())

        await asyncio.sleep(10)
        await ws.close()


if __name__ == "__main__":
    asyncio.run(main())
