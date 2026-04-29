# Roadmap

Wisdoverse Nexus is pre-1.0. The roadmap is directional and may change based on
maintainer capacity, security findings, and community feedback. Items listed as
planned are not release commitments.

## Current Focus

- Keep `main` buildable and reproducible.
- Stabilize the gateway HTTP/WebSocket contract.
- Keep SDK examples aligned with implemented endpoints.
- Improve source-available documentation and issue triage.
- Maintain dependency, audit, and CI hygiene.

## Near-Term Work

| Area | Intent |
| --- | --- |
| API contract | Clarify stable vs experimental endpoints and keep OpenAPI current |
| Gateway reliability | Expand integration tests for room, message, WebSocket, and search paths |
| SDKs | Align TypeScript and Python SDK behavior with gateway routes |
| Mobile | Keep Expo dependencies compatible with the active SDK and verify type/tests |
| Documentation | Make English docs the source of truth for first-run, deployment, and contribution flows |
| Security | Keep private advisory process, dependency audit, and secret scanning active |

## Experimental Areas

These areas exist in the repository but should be treated as evolving surfaces:

- AI provider and MCP integration
- Plugin and skills runtime
- Meeting, document, task, and calendar collaboration domains
- Federation and A2A design work
- Vector search and memory integrations

## Before 1.0

The project should not be considered 1.0-ready until the maintainers have:

- Published a stable API compatibility policy.
- Marked experimental crates and endpoints explicitly.
- Completed repeatable release automation.
- Documented production deployment requirements.
- Published migration guidance for breaking changes.
- Validated SDK examples against the gateway in CI.

## Community Feedback

Use GitHub for public planning:

- [Discussions](https://github.com/Wisdoverse/Wisdoverse-Nexus/discussions) for proposals and design questions
- [Issues](https://github.com/Wisdoverse/Wisdoverse-Nexus/issues) for bugs and scoped work items

Security issues must use the private advisory flow in
[SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md).
