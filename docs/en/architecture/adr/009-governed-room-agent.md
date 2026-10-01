# ADR-009: Governed, explicit room agent runs

Status: accepted for the single-process evaluation profile; durable and distributed execution require M3 qualification.

## Context and decision

Legacy mention/provider primitives do not establish an authorized workflow. A human invokes the named `nexis:ai:room-assistant` through a versioned room-scoped run contract. The gateway `agent_runs` domain owns run states, sources, event cursors and budgets. Its application service uses the room application contract and an `AgentProvider` port. The OpenAI HTTP/SSE adapter and MCP JSON-RPC interface remain outside the domain.

Context and the allowlisted `room_history` tool share one application use case. MCP clients call it over HTTP; the agent invokes the same use case in process. This is not a remote MCP-server integration. The tool performs an actual authorized read, has no write capabilities and accepts no caller-selected room scope. Model-requested tool calls fail closed. Provider credentials remain in the adapter and are never exposed through capabilities, events or errors.

Run acknowledgement precedes asynchronous provider execution. A client supplies `clientRunId`; same-actor/same-room retries return the accepted identity, while changed payloads conflict. Current retry/run retention is one hour and process-local. There is no automatic retry of model requests. Four global active runs and one active run per room bound work; overflow is rejected, not queued without a bound.

## Enforcement and data flow

The human prompt and at most 20 explicitly selected or latest authorized messages go to the configured model as separate untrusted data. Selected context JSON is capped at 32 KiB. Permission and token expiry are checked before context assembly, before dispatch, on streamed events, and every 100 ms while waiting. Revocation drops the transport and denies further observation; it cannot recall data already received by a provider.

The deadline is at most 60 seconds. The tool budget is one. Requested output is at most 4,096 provider tokens and the gateway separately caps UTF-8 output bytes at the requested token count, a deliberately conservative independent output limit. At most 512 events are retained. Successful completion requires a nonempty answer, explicit SSE `[DONE]`, and actual provider-reported input/output usage. Missing usage or premature EOF is a visible failure. No monetary budget is inferred from token counts: paid evaluation requires explicit dated pricing and a separate cost budget.

HTTP event polling uses an exclusive sequence cursor. Both SDKs expose bounded observation, and Web/mobile controller lifetimes abort local observation on navigation. Backend work continues until its own terminal state/deadline unless an authorized user cancels it. Completed/cancelled/failed states cannot be overwritten by late bytes.

## Deployment and consequences

`NEXIS_AGENT_ENABLED=false` is the default and the disable path. Enabling requires `OPENAI_API_BASE`, `OPENAI_DEFAULT_MODEL` and runtime-injected `OPENAI_API_KEY`. HTTPS is required except for explicit loopback model servers. Redirects and implicit reconnect/retry are disabled. Invalid enabled configuration fails startup before readiness. The model must support streaming chat completions and `stream_options.include_usage`.

MCP supports the pinned `2025-03-26` stateless Streamable HTTP JSON-response subset: initialize, initialized notification, ping, tool list/call. GET/SSE sessions, subscriptions, resources, prompts and writes are unsupported. `mcp==1.12.4` is an independent consumer in acceptance tests; this evidence does not establish all MCP conformance or external A2A compatibility.

Single-process memory does not meet durable execution, tenant or independent service-deployment requirements. M3 must provide owned migrations, retained run/retry state, crash recovery, privacy invalidation and qualified service artifacts before deployment scope expands. The existing microservice architecture policy remains applicable; this gateway integration is an intermediate bounded-context implementation.

## Verification

Run `scripts/m2_smoke.sh` with `tests/smoke/m2_requirements.txt`, gateway agent unit tests and both frontend controller suites. The 50-case versioned synthetic evaluation runs three repetitions with the roadmap's 25/10/5/5/5 category counts. It verifies protocol/policy/budget behavior against an actual gateway and separate HTTP fixture. Synthetic answer matching does not prove real-model grounding, injection resistance or product acceptance. Real-provider smoke is opt-in and reported separately.
