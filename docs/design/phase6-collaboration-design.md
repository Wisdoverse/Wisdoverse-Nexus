# Archived Phase 6 Collaboration Design Note

This page is historical planning material. It records a collaboration-product
direction and must not be treated as the current shipped feature surface,
release plan, or benchmark target.

Current source-of-truth documents:

- [Architecture Overview](../en/architecture.md)
- [Roadmap](../en/roadmap.md)
- [Core Capabilities Design](../en/design/core-capabilities-design.md)
- [Testing Guide](../en/guides/testing.md)

## Durable Direction

- Collaboration features should be decomposed into focused domain crates.
- Gateway code should stay responsible for transport, routing, authentication,
  and API boundary mapping.
- AI participation should use explicit identity, permissions, and audit trails.
- Performance or reliability targets require reproducible benchmark evidence
  before they are published as project claims.

## Revalidation Checklist

Before reviving any Phase 6 item, verify:

- current crate ownership
- route and protocol contracts
- test coverage and CI gates
- security and tenant-boundary impact
- documentation updates needed for public contributors
