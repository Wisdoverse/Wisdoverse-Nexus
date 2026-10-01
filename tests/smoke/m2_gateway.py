"""Governed agent transport/policy acceptance; the model is a synthetic HTTP fixture.

This is deliberately not a real-model quality evaluation. Fixture outputs have a
fixed rubric; no paid provider, production data, or credentials are contacted.
"""
from __future__ import annotations

import asyncio
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import threading
import time

import httpx
import jsonschema
from mcp import ClientSession
from mcp.client.streamable_http import streamablehttp_client
from nexis import CreateRoomData, NexisClient
from m1_gateway import token

ROOT = Path(__file__).resolve().parents[2]
ARTIFACTS = ROOT / "artifacts/m2"
ANSWER = "Synthetic response 🦀"
RESULTS: list[dict] = []


class ProviderFixture(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    calls: list[dict] = []
    call_lock = threading.Lock()

    def log_message(self, *_):
        # Never print provider request/authorization data.
        return

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        if self.path != "/v1/chat/completions" or not 0 < length <= 65536:
            self.send_error(400)
            return
        payload = json.loads(self.rfile.read(length))
        assert self.headers.get("Authorization") == "Bearer synthetic-provider-key"
        assert self.headers.get("x-correlation-id")
        assert payload["stream"] is True and payload["stream_options"]["include_usage"] is True
        assert payload["model"] == "synthetic-fixture-v1"
        assert payload["messages"][0]["role"] == "system"
        request = json.loads(payload["messages"][1]["content"])
        with self.call_lock:
            self.calls.append(request)
        prompt = request["task"]
        if prompt.startswith("provider_error"):
            self.send_error(503)
            return
        if prompt.startswith("timeout") or prompt.startswith("cancel"):
            time.sleep(.3)
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Connection", "close")
        self.end_headers()
        if prompt.startswith("tool_attempt"):
            events = [{"choices": [{"delta": {"tool_calls": [{"function": {"name": "file_write"}}]}}]}]
        elif prompt.startswith("malformed"):
            events = ["not-json"]
        else:
            answer = "x" * 80 if prompt.startswith("budget") else ANSWER
            events = [{"choices": [{"delta": {"content": answer[:10]}}]},
                      {"choices": [{"delta": {"content": answer[10:]}}]}]
            if not prompt.startswith("missing_usage"):
                events.append({"choices": [], "usage": {"prompt_tokens": 30, "completion_tokens": 8}})
        try:
            for event in events:
                frame = f"data: {json.dumps(event, ensure_ascii=False)}\r\n\r\n".encode()
                # Split even UTF-8/SSE boundaries to test an actual HTTP byte stream.
                for offset in range(0, len(frame), 7):
                    self.wfile.write(frame[offset:offset + 7])
                    self.wfile.flush()
                time.sleep(.005)
            if not prompt.startswith("early_eof"):
                self.wfile.write(b"data: [DONE]\n\n")
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            # Cancellation, deadline and revocation intentionally drop this stream.
            pass
        self.close_connection = True


def port() -> int:
    with socket.socket() as reserved:
        reserved.bind(("127.0.0.1", 0))
        return reserved.getsockname()[1]


def check(name: str, **evidence):
    RESULTS.append({"scenario": name, "passed": True, **evidence})
    print(f"PASS {name}", flush=True)


def schema_for_validation(schema):
    # OpenAPI 3.0 nullable has to be translated for JSON Schema validation.
    if isinstance(schema, list):
        return [schema_for_validation(value) for value in schema]
    if not isinstance(schema, dict):
        return schema
    result = {key: schema_for_validation(value) for key, value in schema.items() if key != "nullable"}
    if schema.get("nullable") and "type" in result:
        result["type"] = [result["type"], "null"]
    return result


async def terminal(http, path, headers):
    async with asyncio.timeout(3):
        while True:
            response = await http.get(path, headers=headers)
            response.raise_for_status()
            run = response.json()
            if run["status"] in {"completed", "failed", "cancelled"}:
                return run
            await asyncio.sleep(.02)


async def evaluate(base, credentials):
    cases = json.loads((ROOT / "tests/evaluation/room-agent-v1.json").read_text())
    assert cases["version"] == "1.0" and len(cases["cases"]) == 50
    owner = {"Authorization": f"Bearer {credentials['owner']}"}
    outsider = {"Authorization": f"Bearer {credentials['outsider']}"}
    async with httpx.AsyncClient(base_url=base, timeout=5, trust_env=False) as http:
        document = (await http.get('/openapi.json')).json()
        def validate(run):
            normalized = schema_for_validation(document)
            jsonschema.Draft7Validator({**normalized['components']['schemas']['AgentRun'], 'components': normalized['components']}).validate(run)
        room = (await http.post('/v1/rooms', headers=owner, json={'name': 'm2-policy'})).json()['id']
        other = (await http.post('/v1/rooms', headers=owner, json={'name': 'm2-other'})).json()['id']
        source = (await http.post('/v1/messages', headers=owner, json={'roomId': room, 'text': 'Synthetic scope: authorized only.'})).json()['id']
        foreign = (await http.post('/v1/messages', headers=owner, json={'roomId': other, 'text': 'FOREIGN_SECRET_SENTINEL'})).json()['id']
        path = f'/v1/rooms/{room}/agent-runs'
        for repeat in range(3):
            for case in cases['cases']:
                command = {'clientRunId': f"{case['id']}-{repeat}", 'prompt': case['prompt'], 'sourceMessageIds': [source], 'tool': 'room_history', 'maxOutputTokens': 1024, 'deadlineMs': 2000, **case.get('command', {})}
                if case['scenario'] == 'foreign_source':
                    command['sourceMessageIds'] = [foreign]
                headers = outsider if case['scenario'] == 'unauthorized' else owner
                before = len(ProviderFixture.calls)
                response = await http.post(path, headers=headers, json=command)
                assert response.status_code == case['http_status'], (case['id'], response.status_code)
                if response.status_code != 202:
                    assert response.json()['code'] == case['error_code']
                    assert len(ProviderFixture.calls) == before
                else:
                    run = await terminal(http, f"{path}/{response.json()['id']}", owner)
                    validate(run)
                    assert run['status'] == case['status'], (case['id'], run['status'], run['errorCode'])
                    assert run['errorCode'] == case.get('error_code')
                    assert run['sourceMessageIds'] == [source]
                    assert run['usage']['toolCalls'] == 1 and run['latencyMs'] is not None
                    assert len(run['answer'].encode()) <= run['maxOutputBytes']
                    assert run['events'][-1]['kind'] == run['status']
                    assert [event['sequence'] for event in run['events']] == list(range(1, len(run['events']) + 1))
                    if case['status'] == 'completed':
                        assert run['answer'] == ANSWER
                        assert run['usage']['inputTokens'] == 30 and run['usage']['outputTokens'] == 8
                    replay = await http.post(path, headers=owner, json=command)
                    assert replay.json()['id'] == run['id']
                    conflict = await http.post(path, headers=owner, json={**command, 'prompt': 'changed retry'})
                    assert conflict.status_code == 409
                check(case['id'], repetition=repeat + 1, category=case['category'])
        for request in ProviderFixture.calls:
            assert 'FOREIGN_SECRET_SENTINEL' not in json.dumps(request)
            assert all(message['roomId'] == room for message in request['untrustedRoomMessages'])
        # Room members can observe one result, but only invoker/owner can cancel it.
        await http.post(f'/v1/rooms/{room}/invite', headers=owner, json={'memberId': 'm2-peer'})
        peer = {'Authorization': f"Bearer {credentials['peer']}"}
        command = {'clientRunId': 'cancel-live', 'prompt': 'cancel stream', 'deadlineMs': 2000}
        run = (await http.post(path, headers=owner, json=command)).json()
        assert (await http.post(f"{path}/{run['id']}/cancel", headers=peer)).status_code == 403
        cancelled = (await http.post(f"{path}/{run['id']}/cancel", headers=owner)).json()
        assert cancelled['status'] == 'cancelled'
        await asyncio.sleep(.35)
        assert (await http.get(f"{path}/{run['id']}", headers=peer)).json()['status'] == 'cancelled'
        check('cancellation remains terminal after late provider bytes')
        revoked = (await http.post(path, headers=peer, json={'clientRunId': 'revoke-live', 'prompt': 'timeout revoked', 'deadlineMs': 2000})).json()
        deletion = await http.request('DELETE', '/v1/members/me', headers=peer, json={'confirm': True})
        assert deletion.status_code == 200
        assert (await http.get(f"{path}/{revoked['id']}", headers=peer)).status_code == 403
        run = await terminal(http, f"{path}/{revoked['id']}", owner)
        assert run['status'] == 'failed' and run['errorCode'] == 'AGENT_ACCESS_DENIED'
        check('revocation drops the running request and denies further reads')
        metrics = (await http.get('/metrics')).text
        assert 'nexis_agent_runs_total' in metrics and 'synthetic-provider-key' not in metrics
        check('terminal run metrics omit content and credentials')
        return room


async def external_mcp(base, room, credential):
    headers = {'Authorization': f'Bearer {credential}'}
    async with streamablehttp_client(f'{base}/v1/rooms/{room}/mcp', headers=headers, timeout=5, terminate_on_close=False) as (read, write, _):
        async with ClientSession(read, write) as session:
            initialized = await session.initialize()
            assert initialized.protocolVersion == '2025-03-26'
            tools = await session.list_tools()
            assert [tool.name for tool in tools.tools] == ['room_history']
            result = await session.call_tool('room_history', {})
            assert not result.isError and json.loads(result.content[0].text)['sources']
            try:
                await session.call_tool('file_write', {'path': '/tmp/denied'})
                raise AssertionError('write tool was allowed')
            except Exception as error:
                assert 'denied' in str(error)
            try:
                await session.call_tool('room_history', {'roomId': 'other-room'})
                raise AssertionError('foreign scope was allowed')
            except Exception as error:
                assert 'denied' in str(error)
    check('external mcp==1.12.4 initialize/list/read/deny over streamable HTTP JSON responses')


async def python_sdk(base, credential):
    async with NexisClient(base) as client:
        await client.authenticate(credential)
        room = await client.create_room(CreateRoomData('python-agent-smoke'))
        message = await client.send_message(room.id, 'SDK source')
        assert (await client.agent_capabilities(room.id))['enabled']
        run = await client.invoke_agent(room.id, 'Summarize', client_run_id='python-stable', source_message_ids=[message.id])
        events = [event async for event in client.observe_agent_run(room.id, run['id'], timeout=5, poll_interval=.02)]
        assert events[-1]['kind'] == 'completed'
        run = await client.get_agent_run(room.id, run['id'])
        assert run['answer'] == ANSWER and run['sourceMessageIds'] == [message.id]
        assert (await client.invoke_agent(room.id, 'Summarize', client_run_id='python-stable', source_message_ids=[message.id]))['id'] == run['id']
        assert (await client.cancel_agent_run(room.id, run['id']))['status'] == 'completed'
        check('Python SDK observes/replays the same terminal run with sources and reported usage')
        return room.id, run['id']


async def main():
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    secret = secrets.token_urlsafe(48)
    credentials = {name: token(secret, f'm2-{name}') for name in ('owner', 'peer', 'outsider')}
    server = ThreadingHTTPServer(('127.0.0.1', 0), ProviderFixture)
    server.daemon_threads = True
    threading.Thread(target=server.serve_forever, daemon=True).start()
    gateway_port = port()
    base = f'http://127.0.0.1:{gateway_port}'
    env = {**os.environ, 'JWT_SECRET': secret, 'JWT_ISSUER': 'nexis', 'JWT_AUDIENCE': 'nexis', 'NEXIS_ENV': 'development', 'NEXIS_BIND_ADDR': f'127.0.0.1:{gateway_port}', 'NEXIS_HTTPS_REDIRECT_ENABLED': 'false', 'RUST_LOG': 'warn', 'NEXIS_AGENT_ENABLED': 'true', 'OPENAI_API_BASE': f'http://127.0.0.1:{server.server_port}/v1', 'OPENAI_DEFAULT_MODEL': 'synthetic-fixture-v1', 'OPENAI_API_KEY': 'synthetic-provider-key', 'NEXIS_RATE_LIMIT_RPM': '100000', 'NEXIS_SMOKE_URL': base, 'NEXIS_SMOKE_TOKEN': credentials['owner'], 'NEXIS_AGENT_SMOKE': 'true', 'VITE_API_BASE_URL': base + '/v1', 'EXPO_PUBLIC_API_BASE_URL': base + '/v1'}
    for name in ('NEXIS_ENCRYPTION_KEY', 'NEXIS_DATABASE_PATH', 'DATABASE_URL'):
        env.pop(name, None)
    report = {'schema_version': 1, 'work_items': ['RD-005', 'RD-006', 'RD-007', 'RD-008'], 'evaluation_version': '1.0', 'provider': 'loopback synthetic OpenAI HTTP fixture; no real-model quality claim', 'base_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'binary_sha256': hashlib.sha256((ROOT / 'target/debug/nexis-gateway').read_bytes()).hexdigest(), 'source_diff_sha256': hashlib.sha256(subprocess.check_output(['git', 'diff', 'HEAD'], cwd=ROOT)).hexdigest(), 'results': RESULTS, 'passed': False}
    with (ARTIFACTS / 'gateway.log').open('w') as log:
        gateway = subprocess.Popen([str(ROOT / 'target/debug/nexis-gateway')], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            async with httpx.AsyncClient(trust_env=False) as client:
                async with asyncio.timeout(30):
                    while True:
                        if gateway.poll() is not None:
                            raise RuntimeError('Gateway exited before readiness')
                        try:
                            if (await client.get(base + '/health', timeout=1)).text == 'OK':
                                break
                        except httpx.TransportError:
                            # Startup is asynchronous; retry only until the bounded readiness deadline.
                            pass
                        await asyncio.sleep(.05)
            room = await evaluate(base, credentials)
            await external_mcp(base, room, credentials['owner'])
            sdk_room, sdk_run = await python_sdk(base, credentials['owner'])
            env.update(NEXIS_AGENT_SHARED_ROOM=sdk_room, NEXIS_AGENT_SHARED_RUN=sdk_run)
            commands = [('typescript-sdk', ['node', 'sdk/typescript/tests/live_agent.cjs']),
                        ('web', ['pnpm', '--filter', '@wisdoverse/nexus-web', 'exec', 'vitest', 'run', '--config', 'vitest.smoke.config.ts', 'smoke/agent.test.ts']),
                        ('mobile', ['pnpm', '--filter', '@wisdoverse/nexus-mobile', 'exec', 'vitest', 'run', '--config', 'vitest.smoke.config.ts', 'smoke/agent.test.ts'])]
            for name, command in commands:
                process = await asyncio.create_subprocess_exec(*command, cwd=ROOT, env=env, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.STDOUT)
                try:
                    output, _ = await asyncio.wait_for(process.communicate(), 45)
                except TimeoutError:
                    process.kill()
                    await process.wait()
                    raise RuntimeError(f'{name} smoke timed out') from None
                text = output.decode(errors='replace')
                for value in (secret, *credentials.values()):
                    text = text.replace(value, '[REDACTED]')
                (ARTIFACTS / f'{name}.log').write_text(text)
                if process.returncode:
                    raise RuntimeError(f'{name} failed; inspect sanitized artifact')
                check(f'{name} actual gateway agent transport')
            report['passed'] = True
        except Exception as error:
            report['failure_type'] = type(error).__name__
            raise
        finally:
            gateway.terminate()
            try:
                gateway.wait(timeout=5)
            except subprocess.TimeoutExpired:
                gateway.kill()
                gateway.wait()
            server.shutdown()
            server.server_close()
            report['sample_count'] = len(RESULTS)
            report['provider_calls'] = len(ProviderFixture.calls)
            (ARTIFACTS / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'M2 synthetic transport/policy acceptance passed: {len(RESULTS)} cases/groups', flush=True)


if __name__ == '__main__':
    if not __debug__:
        raise RuntimeError('Assertions are required')
    asyncio.run(main())
