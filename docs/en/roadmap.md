# Roadmap

Wisdoverse Nexus is a pre-1.0 engineering preview for development, evaluation,
and non-production environments. This public roadmap describes engineering
priorities and acceptance criteria. It does not commit to delivery dates or
stable compatibility.

English is the source of truth; the [Chinese roadmap](../zh-CN/roadmap.md)
tracks the same priorities and milestone status.

## Product Goal

The [vision](design/nexis-vision-design.md) brings human and AI members into
one collaboration model spanning rooms, documents, meetings, and workflows.
The first supported product journey is a team discussing work in a room,
asking an identifiable agent for help, reviewing its evidence, and approving
the resulting document and tasks. Shared identity, context, permissions, and
audit trails must follow that journey across domains.

The Rust gateway, protocol crates, Web/mobile clients, and SDKs are the
foundation for this journey. Prioritize a small, verifiable workflow before
expanding the number of providers, collaboration domains, or deployment modes.

## Related Projects and Planning Decisions

Reviewed on **2026-10-01** using the projects' official READMEs, linked to the
reviewed revisions below. This is a documentation comparison; compatibility
and performance require separate validation. Licenses vary by project and
component; these references do not select dependencies for Nexus.

| Project / source | Documented overlap | Decision for Nexus |
| --- | --- | --- |
| [Mattermost](https://github.com/mattermost/mattermost/blob/95fc4743df54efd8bbaba148c0e8542d8158c7c1/README.md) | Self-hosted team chat, workflow automation, calls, AI integration, and extension APIs | Make the room/client workflow and deployment reproducible; keep extension contracts explicit (M1, M3, M5) |
| [Zulip](https://github.com/zulip/zulip/blob/a89b9036d989e6b7a89e7bbeb3f8639cde95ece0/README.md) | Topic-based threading for live and asynchronous team conversations | Preserve conversation scope and source-message references so agent results remain understandable after reconnect (M1, M2) |
| [Open WebUI](https://github.com/open-webui/open-webui/blob/3572e01a71c2809bf482771ff2809fb6c6778f91/README.md) | Shared human/AI channels, local and OpenAI-compatible models, access control, MCP/tools, and RAG | Treat the shared AI room as a baseline to validate; focus Nexus on a Rust collaboration foundation and governed work across domains (M2, M4) |
| [AppFlowy](https://github.com/AppFlowy-IO/AppFlowy/blob/bbe886fcdd5295ff49aaeec883181c67ca89ec29/README.md) | AI workspace, documents/project data, self-hosting, and Rust-based building blocks | Connect discussion to durable, reviewable documents and tasks with consistent permissions (M3, M4) |
| [Dify](https://github.com/langgenius/dify/blob/7b0660b45b127a44b6467533ec6a8cd949586770/README.md) | Model providers, agent/tool workflows, RAG pipelines, APIs, and observability | Start with one provider/tool path and measure its failures, latency, and usage; defer a general visual workflow builder (M2, M5) |
| [A2A](https://github.com/a2aproject/A2A/blob/28097dbcd29103a97959384b6485fa799dd794ec/README.md) (protocol reference) | Agent discovery and interoperability for long-running tasks | Validate an adapter against a named protocol version and an external implementation before claiming compatibility (M5) |

This leads to the implementation order: collaboration contracts, a governed
human/AI room pilot, durable tenant operation, a document/task workflow, then
external-agent interoperability. M2 moves AI collaboration into the first
product evaluation cycle; M3 gates broader deployment, and M4 validates the
cross-domain product goal.

## Current Baseline

"Implemented" means code exists in the repository. "Configured" means a
workflow or deployment asset exists. Neither status establishes that a release
has passed its acceptance checks.

| Area | Current state | Remaining validation |
| --- | --- | --- |
| Gateway | Implemented room/message HTTP routes, WebSocket authentication, metrics, and OpenAPI endpoint | Contract consistency and a complete client-to-gateway workflow |
| Architecture | Implemented DDD boundaries in gateway domains and FSD layers in the Web app | Preserve domain boundaries and frontend dependency rules as features evolve |
| AI and collaboration domains | Human/AI/agent member types, provider/MCP/context surfaces, and document/task domain modules exist | Verify shared permissions and a complete room-to-agent-to-artifact workflow; A2A primitives remain experimental |
| Clients and SDKs | Web, Expo mobile, TypeScript SDK, and Python SDK implementations exist | Align supported routes, payloads, authentication, and WebSocket behavior |
| CI | Configured Rust checks, mobile compatibility/type/tests, Web/SDK/docs builds, and security checks | Add client/SDK checks against a running gateway and retain reproducible results |
| Release and deployment | Configured container/TypeScript SDK release workflow and Compose/Helm assets | Verify published artifacts, persistence, upgrade, and recovery behavior |

See [Architecture](architecture.md), [Testing](guides/testing.md), and
[Release Process](guides/release-process.md) for the supporting repository
surfaces and verification commands.

## Prioritized Milestones

M1 is **in review**; M2–M5 remain **planned**. Mark a milestone complete only when its
acceptance checks have passed and the evidence is linked from its public issue
or pull request. Priority indicates order, not a release date. M1 precedes M2;
M3 must pass before expanding deployment scope, and M4 requires M2 and M3.
M5 follows a validated M4 workflow.

## Execution and Quality Gates

The [execution standard](guides/roadmap-execution.md) decomposes M1–M5 into
20 planned work packages (RD-001–RD-020), with dependencies, proposed
responsibility roles, readiness/completion rules, and required evidence.
Assign actual owners before Ready; publish dates only with reviewed estimates.

Proposed qualification targets include a fixed single-node load profile,
HTTP p95 ≤200 ms, a 99.9% gateway success objective, explicit error budgets,
crash/restore checks, and a 50-case AI evaluation set. Every target must be
reported with its sample count, environment, observed value, and tested commit.
The guide separates the current CI floor of 60% workspace line coverage from
planned changed-line coverage and live-gateway gates. These targets remain
unverified until the relevant measurement work and acceptance checks pass.

Trusted alpha, evaluation beta, and 1.0 candidate gates require progressively
stronger artifact, recovery, pilot, and operational evidence. Use the
roadmap work item template to track blockers and outcomes. Authorize every
agent action; require human approval for writes and external side effects.

## Milestone Acceptance

### M1 / P0: Core Collaboration Workflow

Implementation and local acceptance evidence are available in the
[M1 report](guides/m1-acceptance.md). M1 is **in review**: public CI and reviewer
acceptance remain required before marking it complete. Other milestones remain planned.


Align the gateway, API documentation, Web/mobile clients, and both SDKs around
the supported authentication, room, message, and WebSocket contracts.

Acceptance criteria:

- Document and test the supported authentication flow, including rejected and
  expired credentials, room membership, and denied room/tenant access in the
  documented single-tenant or multi-tenant mode.
- Demonstrate room creation/listing and message send/receive against a running
  gateway using synthetic data; verify WebSocket authentication and reconnect.
- Check client and SDK request/response shapes against gateway routes and the
  served OpenAPI document.
- Specify stable member/message identifiers, conversation or reply references,
  and reconnect/history behavior so humans and agents can reference the same
  work. Verify retries and reconnects do not duplicate accepted messages or
  silently lose acknowledged messages within the supported recovery window.
- Run client/SDK smoke checks against a running gateway in CI. Mock-only tests
  and successful builds do not satisfy this milestone.

### M2 / P0: Governed Human/AI Room Pilot

Validate the core AI-native experience after M1 with one named agent, one
OpenAI-compatible provider configuration, and one read-only MCP tool. Publish
the supported provider configuration, including whether it is local or hosted.
Keep the pilot within the existing trusted, non-production evaluation scope.

Acceptance criteria:

- A human explicitly invokes an agent in a shared room; its identity, progress,
  streamed answer, source-message references, and failure/cancel state are
  visible. Both SDKs can observe the same result through documented contracts.
- Assemble context from the invoking member's authorized room data; reject
  access to other rooms/tenants and test permission revocation. Treat room
  content and tool results as untrusted inputs that cannot expand permissions.
- Restrict the read-only tool to an explicit allowlist and permission scope;
  test denied calls, provider/tool failures, timeouts, and cancellation.
- Record a trace linking the request, context sources, tool call, and answer,
  with latency, token usage, and configured budget limits. Redact credentials
  and private content in public evidence; disclose data sent to the provider.
- Run deterministic synthetic scenarios against a real gateway in CI and
  document an opt-in real-provider smoke check using runtime-injected secrets.
  Publish pass/fail expectations for answers and permission failures; mock
  coverage alone does not establish provider interoperability.

### M3 / P1: Persistence and Tenant Reliability

Validate one documented storage/deployment path before broadening evaluation
to more teams or deployment modes. Carry M1/M2 identity and policy decisions
through restart, recovery, and retained data.

Acceptance criteria:

- Show that room/message data survives a gateway restart with the documented
  persistent storage configuration.
- Verify that HTTP and WebSocket workflows observe consistent room/message
  state.
- Test tenant access boundaries, including rejected cross-tenant requests, with
  the documented feature flags and storage configuration. Apply the same
  checks to agent context, search, and enabled stored artifacts.
- Verify retention, data export/deletion, backup/restore, and failure recovery
  paths, including enabled context/memory/search stores. Retain useful audit
  records without retaining deleted content or credentials.
- Reproduce startup, upgrade/migration, and rollback or restore from the
  documented Compose deployment path; identify supported limits and the
  differences from experimental Helm or multi-node deployments.
- Publish reproducible benchmark methods and measured results with environment
  details and limitations. Set workload and resource/latency targets before
  measuring the end-to-end room/agent path; report failures as well as throughput.

### M4 / P2: Discussion to Documents and Tasks

Demonstrate one complete cross-domain workflow: a synthetic room conversation
or supplied meeting transcript becomes an agent-authored draft, a human reviews
it, and approved tasks return their status to the room.

Acceptance criteria:

- Preserve source-message/transcript references, document revisions, task
  ownership, and workflow status under the same identity and permission model.
- Require human approval before an agent writes shared documents, creates or
  assigns tasks, or performs other external side effects. Test rejection,
  revoked permission, retry/idempotency, and the resulting audit record.
- Verify two-member document edits converge through the documented CRDT sync
  path and survive restart; test conflicts and unauthorized edits.
- Retrieve only authorized room/document context, include source references
  in answers, and verify deletion or permission changes remove stale access.
- Run the discussion-to-draft-to-approved-task-to-room-status scenario against
  the gateway in CI with synthetic data. Document manual transcript input;
  real-time meeting recording remains a separate future capability.

### M5 / P3: Extensions and External-Agent Interoperability

Extend a working collaboration journey through versioned MCP/skills/plugin
contracts and one A2A adapter. Keep external frameworks behind adapters so
Nexus domain rules remain in its DDD application/domain layers.

Acceptance criteria:

- Publish supported extension versions, capability/permission scopes, and
  compatibility tests; isolate credentials and failures between integrations.
- Map external agent discovery, identity, task status, cancellation, and result
  artifacts to Nexus members and room/task references.
- Pin the evaluated A2A specification version and verify a complete handoff
  with one external implementation, including authentication and timeout cases.
- State supported transports and limitations; repository primitives or naming
  alone do not establish protocol conformance.

## Ongoing Maintenance

- Keep `main` reproducible from a fresh checkout and retain Rust, mobile, Web,
  SDK, and documentation checks.
- Keep dependency audit, secret scanning, and private security reporting active.
- Preserve DDD/FSD boundaries and document breaking changes and migrations.
- Keep English and Chinese roadmap priorities and status aligned.
- Track each milestone with its dependencies, scoped work items, and linked
  acceptance evidence. Revisit priorities using reproducible pilot feedback.

## Experimental Areas

These areas exist in the repository but should be treated as evolving surfaces:

- AI provider and MCP integration
- Plugin and skills runtime
- Meeting, document, task, and calendar collaboration domains
- Federation and A2A design work
- Vector search and memory integrations

Broad feature expansion is deferred until the corresponding milestone gates
pass: full meeting audio/video, a general visual workflow builder, autonomous
multi-agent orchestration, cross-instance federation, a public plugin
marketplace, and commercial billing. Existing modules remain experimental
unless their supported workflow has its own evidence.

## Before 1.0

Before a stable release, maintainers should record evidence for:

- Completion of M1–M3 and one M4 workflow with linked end-to-end evidence.
- A documented supported provider/tool matrix and agent permission, approval,
  context, usage-limit, and audit behavior.
- Explicit scope and conformance evidence for any shipped M5 integrations;
  unevaluated extensions remain experimental.
- A stable HTTP/WebSocket and SDK compatibility policy, including deprecation
  timelines and migration guidance.
- Clearly marked experimental crates, endpoints, and integrations.
- A repeatable release workflow with verified container and SDK artifacts.
- Documented deployment requirements, persistence, backup/restore, and upgrades.
- Client and SDK examples validated against the gateway in CI.
- The applicable execution-standard gate reports, named owners/reviewers,
  capacity/support matrix, error-budget policy, and rehearsed rollout/recovery.

The existing release workflow and deployment documentation are a baseline;
successful release and recovery checks are still required.

## Community Feedback

Use GitHub for public planning:

- [Discussions](https://github.com/Wisdoverse/Wisdoverse-Nexus/discussions) for proposals and design questions
- [Issues](https://github.com/Wisdoverse/Wisdoverse-Nexus/issues) for bugs and scoped work items

Use synthetic or redacted examples in public proposals, logs, screenshots, and
benchmark results. Remove personal information, customer/tenant identifiers,
private infrastructure addresses, and credentials before posting. Each scoped
milestone issue should include acceptance criteria and verification evidence.

Security issues must use the private advisory flow in
[SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md).
