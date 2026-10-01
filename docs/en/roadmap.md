# Roadmap

Wisdoverse Nexus is a pre-1.0 engineering preview for development, evaluation,
and non-production environments. This public roadmap describes engineering
priorities and acceptance criteria. It does not commit to delivery dates or
stable compatibility.

English is the source of truth; the [Chinese roadmap](../zh-CN/roadmap.md)
tracks the same priorities and milestone status.

## Current Baseline

"Implemented" means code exists in the repository. "Configured" means a
workflow or deployment asset exists. Neither status establishes that a release
has passed its acceptance checks.

| Area | Current state | Remaining validation |
| --- | --- | --- |
| Gateway | Implemented room/message HTTP routes, WebSocket authentication, metrics, and OpenAPI endpoint | Contract consistency and a complete client-to-gateway workflow |
| Architecture | Implemented DDD boundaries in gateway domains and FSD layers in the Web app | Preserve domain boundaries and frontend dependency rules as features evolve |
| Clients and SDKs | Web, Expo mobile, TypeScript SDK, and Python SDK implementations exist | Align supported routes, payloads, authentication, and WebSocket behavior |
| CI | Configured Rust checks, mobile compatibility/type/tests, Web/SDK/docs builds, and security checks | Add client/SDK checks against a running gateway and retain reproducible results |
| Release and deployment | Configured container/TypeScript SDK release workflow and Compose/Helm assets | Verify published artifacts, persistence, upgrade, and recovery behavior |

See [Architecture](architecture.md), [Testing](guides/testing.md), and
[Release Process](guides/release-process.md) for the supporting repository
surfaces and verification commands.

## Prioritized Milestones

All milestones below are **planned**. Mark a milestone complete only when its
acceptance checks have passed and the evidence is linked from its public issue
or pull request. Priority indicates order, not a release date.

### M1 / P0: Core Collaboration Workflow

Align the gateway, API documentation, Web/mobile clients, and both SDKs around
the supported authentication, room, message, and WebSocket contracts.

Acceptance criteria:

- Document and test the supported authentication flow, including rejected and
  expired credentials.
- Demonstrate room creation/listing and message send/receive against a running
  gateway using synthetic data; verify WebSocket authentication and reconnect.
- Check client and SDK request/response shapes against gateway routes and the
  served OpenAPI document.
- Run client/SDK smoke checks against a running gateway in CI. Mock-only tests
  and successful builds do not satisfy this milestone.

### M2 / P1: Persistence and Tenant Reliability

Validate storage and isolation before expanding the supported deployment scope.

Acceptance criteria:

- Show that room/message data survives a gateway restart with the documented
  persistent storage configuration.
- Verify that HTTP and WebSocket workflows observe consistent room/message
  state.
- Test tenant access boundaries, including rejected cross-tenant requests, with
  the documented feature flags and storage configuration.
- Verify data export/deletion, backup/restore, and failure recovery paths.
- Publish reproducible benchmark methods and measured results with environment
  details and limitations.

### M3 / P2: AI Collaboration Example

Build one documented example connecting a model provider, room context, and MCP
tool use while keeping experimental capabilities explicit.

Acceptance criteria:

- Demonstrate the complete workflow with synthetic conversations and documented
  setup commands.
- Cover provider/tool failures, timeouts, and the configured permission boundary.
- Inject provider credentials at runtime and keep shared examples and results
  free of personal data and private deployment details.
- State which integrations are supported and which remain experimental.

## Ongoing Maintenance

- Keep `main` reproducible from a fresh checkout and retain Rust, mobile, Web,
  SDK, and documentation checks.
- Keep dependency audit, secret scanning, and private security reporting active.
- Preserve DDD/FSD boundaries and document breaking changes and migrations.
- Keep English and Chinese roadmap priorities and status aligned.

## Experimental Areas

These areas exist in the repository but should be treated as evolving surfaces:

- AI provider and MCP integration
- Plugin and skills runtime
- Meeting, document, task, and calendar collaboration domains
- Federation and A2A design work
- Vector search and memory integrations

## Before 1.0

Before a stable release, maintainers should record evidence for:

- Completion of M1 and M2; explicit scope for any supported M3 integrations.
- A stable HTTP/WebSocket and SDK compatibility policy, including deprecation
  timelines and migration guidance.
- Clearly marked experimental crates, endpoints, and integrations.
- A repeatable release workflow with verified container and SDK artifacts.
- Documented deployment requirements, persistence, backup/restore, and upgrades.
- Client and SDK examples validated against the gateway in CI.

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
