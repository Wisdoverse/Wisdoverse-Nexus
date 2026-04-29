# Community Feedback Plan

This page replaces the earlier launch-marketing plan with a contributor-focused
feedback process suitable for a public GitHub repository.

## Goals

- Make early user feedback actionable for maintainers.
- Keep bug reports reproducible.
- Keep feature requests scoped and linked to implementation work.
- Avoid collecting unnecessary personal data.

## Feedback Channels

| Channel | Use |
| --- | --- |
| GitHub Issues | Bugs, scoped tasks, documentation gaps |
| GitHub Discussions | Product questions, design proposals, early feedback |
| Pull requests | Proposed fixes with tests or documentation updates |
| GitHub Security Advisories | Private vulnerability reports |

## Bug Report Requirements

Ask reporters to include:

- Repository commit SHA or release version
- Operating system and architecture
- Rust, Node.js, pnpm, Docker versions when relevant
- Reproduction steps
- Expected behavior
- Actual behavior
- Logs or screenshots when useful
- Minimal configuration needed to reproduce

## Feature Proposal Requirements

Feature requests should explain:

- User problem
- Current workaround
- Proposed behavior
- API, deployment, or compatibility impact
- Whether the feature belongs in core, SDK, plugin, or documentation

## Maintainer Triage

Suggested labels:

- `bug`
- `documentation`
- `good first issue`
- `help wanted`
- `security`
- `needs reproduction`
- `proposal`
- `breaking-change`

Close issues that cannot be reproduced after reasonable follow-up, and link
duplicates to the canonical issue.
