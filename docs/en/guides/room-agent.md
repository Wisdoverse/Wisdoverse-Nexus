# Governed room assistant evaluation

M2 implementation supports explicit human invocation, source references, ordered progress/answer events, reported usage, retries and cancellation in the default single-tenant, single-process gateway. It is an engineering preview. The automated local report has **158 passing cases/groups**, including 50 synthetic cases × three repetitions and external MCP/SDK/frontend consumers. Real-model answer quality and human pilot acceptance remain unqualified.

## Configuration and privacy

The assistant is disabled by default. Configure:

```sh
NEXIS_AGENT_ENABLED=true
OPENAI_API_BASE=http://127.0.0.1:11434/v1
OPENAI_DEFAULT_MODEL=<your-configured-model>
# Inject OPENAI_API_KEY through the deployment secret mechanism.
```

The loopback URL illustrates a local OpenAI-compatible server, not a tested model. Hosted endpoints require HTTPS. The supported provider must stream `/chat/completions`, include final `prompt_tokens`/`completion_tokens`, and end with `[DONE]`. Redirects, automatic model retries and model-selected tool calls are disabled. Set `NEXIS_AGENT_ENABLED=false` and restart to disable new runs; in-flight process-local runs are interrupted by restart.

The provider receives the explicit question and selected room messages, including their identifiers and authors. Web/mobile show this disclosure before invocation. Use synthetic data until the model host, retention policy and cost profile are approved for an opt-in workspace. Private content and credentials must not enter public reports.

## Contract

Verify an externally issued JWT as in the [M1 guide](core-collaboration.md). All endpoints require current room access. Only human tokens can invoke; the invoker or room creator can cancel; current room members can observe the same run.

| Endpoint under `/v1/rooms/{room_id}` | Behavior |
| --- | --- |
| GET `/agents` | Named identity, enabled state, limits and provider data disclosure |
| POST `/agent-runs` | 202 acknowledgement for an explicit invocation; `clientRunId` is required |
| GET `/agent-runs/{run_id}` | Current answer, sources, usage, limits, trace and terminal failure code |
| GET `/agent-runs/{run_id}/events?after=0` | Ordered JSON event batches after the exclusive cursor; stop on terminal status |
| POST `/agent-runs/{run_id}/cancel` | Idempotent terminal cancellation; completed/failed runs remain terminal |
| POST `/mcp` | Stateless MCP `2025-03-26` initialize/list/read; only `room_history`, empty arguments |

Example request:

```json
{"clientRunId":"synthetic-request-1","prompt":"Summarize these messages","sourceMessageIds":[],"tool":"room_history","maxOutputTokens":1024,"deadlineMs":60000}
```

Empty source IDs select the latest 20 messages. Explicit IDs must belong to this room without duplicates. An unsupported tool or foreign source is rejected before model dispatch. Same payload/key replays the same run for the process's one-hour retry window; changing the payload returns 409. Keep the key after a lost acknowledgement. Neither answers nor tool calls write room messages, documents, tasks or external resources.

See the served `/openapi.json`, canonical JSON in `crates/nexis-gateway/src/router/openapi.json`, and [ADR-009](../architecture/adr/009-governed-room-agent.md) for budgets, failure behavior and privacy limitations. TypeScript uses `invokeAgent`/`observeAgentRun`; Python uses `invoke_agent`/`observe_agent_run`. A terminal event gives the state; read the run for the complete result. SDK observation budgets stop observation, not backend execution.

## Reproduction and evidence

```sh
python -m venv .venv-smoke
.venv-smoke/bin/pip install -r tests/smoke/m2_requirements.txt
pnpm install --frozen-lockfile --ignore-scripts
NEXIS_SMOKE_PYTHON=.venv-smoke/bin/python bash scripts/m2_smoke.sh
cargo test --locked -p nexis-gateway agent_runs --lib
pnpm --filter @wisdoverse/nexus-web exec vitest run src/entities/agent-run
pnpm --filter @wisdoverse/nexus-mobile exec vitest run src/entities/agent-run
```

The isolated run creates synthetic identities and ephemeral credentials, starts a separate loopback HTTP/SSE provider, validates run schemas and executes both SDKs and actual Web/mobile API/controller code. `artifacts/m2/summary.json` retains case counts, repeats, tested base revision, binary/source digests and pass/fail outcomes. Public CI must qualify the final proposed revision; local worktree evidence alone does not qualify a merge/release. Frontend transport tests do not replace native-device or browser accessibility acceptance.

`tests/evaluation/room-agent-v1.json` pins the synthetic rubric and categories. The fixture's fixed response and usage verify delivery and enforcement, not natural-language quality. Additional checks cover provider EOF/missing usage, forbidden tool requests, lost acknowledgement, revocation during pending headers, cancellation, cursor ordering and an independent `mcp==1.12.4` HTTP consumer. MCP support is the documented subset, not a claim of full conformance.

For real-provider smoke, configure the actual gateway model at runtime, obtain a scoped evaluation JWT, and explicitly opt in to `tests/smoke/real_provider.py` using its documented runtime variables. No real provider or paid request is part of CI. Do not mark M2 complete until real-provider configuration, scoped answer evaluation and reviewer/pilot decisions have linked evidence. M3 storage/recovery and M4 approved writes remain separate work packages.
