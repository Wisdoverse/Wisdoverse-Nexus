"""M1 acceptance against a real isolated gateway, with synthetic identities only."""
from __future__ import annotations
import asyncio
import base64
import hashlib
import hmac
import json
import os
from pathlib import Path
import platform
import secrets
import socket
import subprocess
import time

import httpx
import jsonschema
import websockets
from nexis import CreateRoomData, NexisClient, WebSocketConnection

ROOT = Path(__file__).resolve().parents[2]
ARTIFACTS = ROOT / "artifacts/m1"
RESULTS: list[dict] = []


def token(secret: str, member: str, **overrides) -> str:
    def encode(value: bytes) -> str:
        return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")
    claims = {"sub": member, "exp": int(time.time()) + 900, "iat": int(time.time()),
              "iss": "nexis", "aud": "nexis", "member_type": "human", **overrides}
    data = ".".join(encode(json.dumps(value, separators=(",", ":")).encode())
                    for value in [{"alg": "HS256", "typ": "JWT"}, claims])
    return f"{data}.{encode(hmac.new(secret.encode(), data.encode(), hashlib.sha256).digest())}"


def passed(name: str) -> None:
    RESULTS.append({"scenario": name, "passed": True})
    print(f"PASS {name}", flush=True)


async def receive(ws, kind: str) -> dict:
    async with asyncio.timeout(5):
        while True:
            event = json.loads(await ws.recv())
            if event["type"] == kind:
                return event
            if event["type"] in ("error", "auth_error", "auth_required"):
                raise AssertionError(f"Expected {kind}, got {event['type']} ({event.get('code')})")


async def protocol_checks(base: str, ws_url: str, tokens: dict[str, str], secret: str) -> None:
    async with httpx.AsyncClient(base_url=base, timeout=10, trust_env=False) as http:
        owner = {"Authorization": f"Bearer {tokens['owner']}"}
        peer = {"Authorization": f"Bearer {tokens['peer']}"}
        document = (await http.get("/openapi.json")).json()
        for path, method in [("/v1/auth/session", "get"), ("/v1/rooms", "get"), ("/v1/rooms", "post"),
                             ("/v1/messages", "post"), ("/v1/rooms/{id}/messages", "get")]:
            assert method in document["paths"][path]
        def validate(path, method, status, payload):
            schema = document["paths"][path][method]["responses"][str(status)]["content"]["application/json"]["schema"]
            jsonschema.Draft7Validator({"allOf": [schema], "components": document["components"]}).validate(payload)
        for credential in [None, "invalid-token", tokens["expired"], token(secret, "bad-issuer", iss="other"),
                           token(secret, "bad-audience", aud="other"), token(secret, "", member_type="human"),
                           token(secret, "tenant-scoped", tenant_id="synthetic-tenant")]:
            headers = {} if credential is None else {"Authorization": f"Bearer {credential}"}
            assert (await http.get("/v1/auth/session", headers=headers)).status_code == 401
            assert (await http.get("/v1/rooms", headers=headers)).status_code == 401
        passed("HTTP rejects missing, invalid, expired, issuer/audience, empty-subject and tenant-scoped credentials")
        session = await http.get("/v1/auth/session", headers=owner)
        assert session.status_code == 200
        validate("/v1/auth/session", "get", 200, session.json())
        assert session.json()["memberId"] == "m1-owner"
        passed("verified session matches served OpenAPI schema")
        response = await http.post("/v1/rooms", headers=owner, json={"name": "protocol-smoke", "topic": "synthetic"})
        assert response.status_code == 201
        room_id = response.json()["id"]
        validate("/v1/rooms", "post", 201, response.json())
        listing = await http.get("/v1/rooms", headers=owner)
        validate("/v1/rooms", "get", 200, listing.json())
        assert listing.json()["rooms"][0]["id"] == room_id
        assert (await http.get("/v1/rooms", headers=peer)).json()["rooms"] == []
        passed("room create/list and member visibility match OpenAPI")
        for method, path, body in [("GET", f"/v1/rooms/{room_id}", None),
                                   ("GET", f"/v1/rooms/{room_id}/messages", None),
                                   ("POST", "/v1/messages", {"roomId": room_id, "text": "denied"}),
                                   ("POST", f"/v1/rooms/{room_id}/invite", {"memberId": "other"}),
                                   ("DELETE", f"/v1/rooms/{room_id}", None)]:
            assert (await http.request(method, path, headers=peer, json=body)).status_code == 403
        passed("HTTP denies nonmember reads, writes and administration")
        assert (await http.post(f"/v1/rooms/{room_id}/invite", headers=owner, json={"memberId": "m1-peer"})).status_code == 200
        assert (await http.delete(f"/v1/rooms/{room_id}", headers=peer)).status_code == 403
        passed("invited member can collaborate but cannot administer")
        for credential, expected in [("invalid-token", "AUTH_FAILED"), (tokens["expired"], "TOKEN_EXPIRED")]:
            async with websockets.connect(ws_url) as ws:
                await ws.send(json.dumps({"type": "auth", "token": credential}))
                event = json.loads(await asyncio.wait_for(ws.recv(), 5))
                assert event["type"] == "auth_error" and event["code"] == expected
        async with websockets.connect(ws_url) as ws:
            await ws.send(json.dumps({"type": "join_room", "room_id": room_id}))
            assert json.loads(await ws.recv())["type"] == "auth_required"
        passed("WebSocket rejects invalid/expired credentials and operations before auth")
        async with websockets.connect(ws_url) as ws:
            short = token(secret, "m1-owner", exp=int(time.time()) + 2)
            await ws.send(json.dumps({"type": "auth", "token": short}))
            await receive(ws, "auth_success")
            event = json.loads(await asyncio.wait_for(ws.recv(), 4))
            assert event["type"] == "auth_error" and event["code"] == "TOKEN_EXPIRED"
        passed("WebSocket terminates established sessions on credential expiry")
        async with websockets.connect(ws_url) as ws:
            event = json.loads(await asyncio.wait_for(ws.recv(), 12))
            assert event["type"] == "auth_error" and event["code"] == "AUTH_TIMEOUT"
        passed("unauthenticated WebSocket times out with a delivered error")
        async with websockets.connect(ws_url) as observer, websockets.connect(ws_url) as outsider:
            for ws, credential in [(observer, tokens["owner"]), (outsider, token(secret, "m1-outsider"))]:
                await ws.send(json.dumps({"type": "auth", "token": credential}))
                await receive(ws, "auth_success")
                await ws.send(json.dumps({"type": "join_room", "room_id": room_id}))
            await receive(observer, "room_joined")
            denial = json.loads(await outsider.recv())
            assert denial["type"] == "error" and denial["code"] == "FORBIDDEN"
            accepted = await http.post("/v1/messages", headers=peer,
                json={"roomId": room_id, "sender": "forged", "text": "HTTP to WS", "clientMessageId": "http-stable"})
            assert accepted.status_code == 201
            message = accepted.json()
            validate("/v1/messages", "post", 201, message)
            event = await receive(observer, "new_message")
            assert event["message_id"] == message["id"] and event["sender_id"] == "m1-peer"
            assert event["content"] == message["text"]
            passed("HTTP write delivers the same identity/content to subscribed WebSocket; sender cannot be forged")
            try:
                await asyncio.wait_for(outsider.recv(), .2)
                raise AssertionError("Unsubscribed member received a room event")
            except TimeoutError:
                pass
            passed("nonmember WebSocket receives no room message")
            payload = {"roomId": room_id, "text": "HTTP to WS", "clientMessageId": "http-stable"}
            retries = await asyncio.gather(*(http.post("/v1/messages", headers=peer, json=payload) for _ in range(4)))
            assert all(item.status_code == 201 and item.json()["id"] == message["id"] for item in retries)
            try:
                await asyncio.wait_for(observer.recv(), .2)
                raise AssertionError("Retry broadcast a duplicate")
            except TimeoutError:
                pass
            payload["text"] = "changed"
            assert (await http.post("/v1/messages", headers=peer, json=payload)).status_code == 409
            passed("concurrent HTTP retries return one stable message without duplicate broadcast; changed payload conflicts")
            ws_payload = {"type": "send_message", "room_id": room_id, "content": "WS to HTTP", "reply_to": message["id"], "client_message_id": "ws-stable"}
            await observer.send(json.dumps(ws_payload))
            ack = await receive(observer, "message_accepted")
            event = await receive(observer, "new_message")
            assert ack["message_id"] == event["message_id"] and event["reply_to"] == message["id"]
            await observer.send(json.dumps(ws_payload))
            assert (await receive(observer, "message_accepted"))["message_id"] == ack["message_id"]
            history = await http.get(f"/v1/rooms/{room_id}/messages", headers=owner)
            validate("/v1/rooms/{id}/messages", "get", 200, history.json())
            assert [item["id"] for item in history.json()] == [message["id"], ack["message_id"]]
            passed("WebSocket commit/retry acknowledgements match ordered HTTP history and reply references")
            await observer.send(json.dumps({"type": "heartbeat", "timestamp": 123}))
            assert (await receive(observer, "heartbeat_ack"))["timestamp"] == 123
            passed("heartbeat matches documented flat contract")
            await observer.send(json.dumps({"type": "leave_room", "room_id": room_id}))
            await receive(observer, "room_left")
            await http.post("/v1/messages", headers=owner, json={"roomId": room_id, "text": "offline history"})
            try:
                await asyncio.wait_for(observer.recv(), .2)
                raise AssertionError("Left room still received a message")
            except TimeoutError:
                pass
            passed("leave stops live delivery; HTTP retains recoverable history")
        for body in [{"roomId": room_id, "text": " "}, {"roomId": room_id, "text": "x" * 32769},
                     {"roomId": room_id, "text": "bad reply", "replyTo": "missing-message"},
                     {"roomId": room_id, "text": "bad key", "clientMessageId": " "}]:
            assert (await http.post("/v1/messages", headers=owner, json=body)).status_code == 400
        assert (await http.get("/v1/rooms/missing/messages", headers=owner)).status_code == 404
        passed("invalid text/reply/retry inputs and missing rooms fail explicitly")


async def python_sdk_checks(base: str, ws_url: str, tokens: dict[str, str]) -> None:
    async with NexisClient(base) as client, NexisClient(base) as peer:
        await client.authenticate(tokens["owner"])
        await peer.authenticate(tokens["peer"])
        room = await client.create_room(CreateRoomData("python-smoke", "synthetic"))
        assert room.id in [item.id for item in await client.list_rooms()]
        await client.invite_member(room.id, "m1-peer")
        ws = WebSocketConnection(ws_url, tokens["owner"], room_id=room.id, reconnect_delay=.05)
        try:
            await ws.connect()
            stream = ws.messages()
            message = await peer.send_message(room.id, "python event", client_message_id="python-stable")
            event = await asyncio.wait_for(anext(stream), 5)
            assert event["message_id"] == message.id and event["content"] == message.text
            assert (await peer.send_message(room.id, "python event", client_message_id="python-stable")).id == message.id
            assert len(await client.get_messages(room.id)) == 1
            passed("Python SDK auth/create/list/send/history/retry observe the live gateway")
            async with websockets.connect(ws_url) as replacement:
                await replacement.send(json.dumps({"type": "auth", "token": tokens["owner"]}))
                await receive(replacement, "auth_success")
            offline = await peer.send_message(room.id, "during disconnect")
            pending = asyncio.create_task(anext(stream))
            async with asyncio.timeout(5):
                while not ws.is_connected:
                    await asyncio.sleep(.02)
            # Wait for the old stream to detect its close and replace the transport.
            await asyncio.sleep(.1)
            after = await peer.send_message(room.id, "after reconnect", reply_to=message.id)
            event = await asyncio.wait_for(pending, 5)
            assert event["message_id"] == after.id
            history = await client.get_messages(room.id)
            assert [item.id for item in history] == [message.id, offline.id, after.id]
            assert after.reply_to == message.id
            passed("Python SDK reconnect reauthenticates/resubscribes and history fills the offline gap without duplicates")
            await stream.aclose()
        finally:
            await ws.close()
        assert not ws.is_connected
        passed("Python explicit close cancels reconnect")


async def main() -> None:
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    secret = secrets.token_urlsafe(48)
    tokens = {"owner": token(secret, "m1-owner"), "peer": token(secret, "m1-peer"), "expired": token(secret, "m1-owner", exp=int(time.time()) - 1)}
    with socket.socket() as reserved:
        reserved.bind(("127.0.0.1", 0))
        port = reserved.getsockname()[1]
    base, ws_url = f"http://127.0.0.1:{port}", f"ws://127.0.0.1:{port}/ws"
    env = {**os.environ, "JWT_SECRET": secret, "JWT_ISSUER": "nexis", "JWT_AUDIENCE": "nexis", "NEXIS_ENV": "development", "NEXIS_BIND_ADDR": f"127.0.0.1:{port}", "NEXIS_DEFAULT_PROVIDER": "mock", "RUST_LOG": "warn", "NEXIS_HTTPS_REDIRECT_ENABLED": "false", "NEXIS_SMOKE_URL": base, "NEXIS_SMOKE_WS_URL": ws_url,
           "NEXIS_SMOKE_TOKEN": tokens["owner"], "NEXIS_SMOKE_PEER_TOKEN": tokens["peer"], "NEXIS_SMOKE_EXPIRED_TOKEN": tokens["expired"], "VITE_API_BASE_URL": base + "/v1", "VITE_WS_URL": ws_url, "EXPO_PUBLIC_API_BASE_URL": base + "/v1"}
    # Do not inherit optional encryption or database configuration into the default-feature qualification.
    for name in ("NEXIS_ENCRYPTION_KEY", "NEXIS_DATABASE_PATH", "DATABASE_URL"):
        env.pop(name, None)
    report = {"schema_version": 1, "work_items": ["RD-001", "RD-002", "RD-003", "RD-004"], "gate": "QG-01", "commit_sha": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "working_tree_diff_sha256": hashlib.sha256(subprocess.check_output(["git", "diff", "HEAD"], cwd=ROOT)).hexdigest(), "platform": platform.platform(), "python": platform.python_version(), "configuration": "default features, single tenant, in-memory, isolated loopback gateway", "results": RESULTS, "passed": False}
    with (ARTIFACTS / "gateway.log").open("w") as gateway_log:
        gateway = subprocess.Popen([str(ROOT / "target/debug/nexis-gateway")], cwd=ROOT, env=env, stdout=gateway_log, stderr=subprocess.STDOUT)
        try:
            async with httpx.AsyncClient(trust_env=False) as client:
                async with asyncio.timeout(30):
                    while True:
                        if gateway.poll() is not None:
                            raise RuntimeError("Isolated gateway exited before readiness")
                        try:
                            if (await client.get(base + "/health", timeout=1)).text == "OK": break
                        except httpx.TransportError: pass
                        await asyncio.sleep(.05)
            await protocol_checks(base, ws_url, tokens, secret)
            await python_sdk_checks(base, ws_url, tokens)
            for name, command in [("typescript-sdk", ["node", "sdk/typescript/tests/live_gateway.cjs"]),
                                  ("web-client", ["pnpm", "--filter", "@wisdoverse/nexus-web", "exec", "vitest", "run", "--config", "vitest.smoke.config.ts"]),
                                  ("mobile-client", ["pnpm", "--filter", "@wisdoverse/nexus-mobile", "exec", "vitest", "run", "--config", "vitest.smoke.config.ts"])]:
                process = await asyncio.create_subprocess_exec(*command, cwd=ROOT, env=env, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.STDOUT)
                try:
                    output, _ = await asyncio.wait_for(process.communicate(), 60)
                except TimeoutError:
                    process.kill()
                    await process.wait()
                    raise RuntimeError(f"{name} timed out") from None
                text = output.decode(errors="replace")
                for credential in [secret, *tokens.values()]: text = text.replace(credential, "[REDACTED]")
                (ARTIFACTS / f"{name}.log").write_text(text)
                if process.returncode: raise RuntimeError(f"{name} failed; see its sanitized artifact log")
                passed(f"{name} real-gateway collaboration suite")
            report["passed"] = True
        except Exception as error:
            report["failure"] = f"{type(error).__name__}: {error}"
            raise
        finally:
            gateway.terminate()
            try: gateway.wait(timeout=5)
            except subprocess.TimeoutExpired:
                gateway.kill()
                gateway.wait()
            report["sample_count"] = len(RESULTS)
            (ARTIFACTS / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"M1 acceptance passed: {len(RESULTS)} scenario groups; artifacts/m1/summary.json", flush=True)


if __name__ == "__main__":
    asyncio.run(main())
