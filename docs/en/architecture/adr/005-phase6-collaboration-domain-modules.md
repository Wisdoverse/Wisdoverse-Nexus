# ADR-005: Phase 6 Collaboration Domain Modules

## Status

Accepted

## Context

Phase 6 expands collaboration across meetings, documents, tasks, and calendar workflows.
We needed an architecture that supports:
- Independent domain evolution
- Strong type safety at domain boundaries
- Clear separation between HTTP transport and domain logic

Options considered:
- Keep all collaboration behavior inside `nexis-gateway`
- Introduce one large collaboration crate
- Split collaboration into focused crates and keep gateway as an API facade

## Decision

Use focused domain crates (`nexis-meeting`, `nexis-doc`, `nexis-task`, `nexis-calendar`) and keep `nexis-gateway` responsible for API routing, auth, and transport mapping.

## Consequences

### Positive
- Domain concerns are isolated and easier to test independently
- API surface can evolve without tightly coupling domain internals
- Team ownership can be aligned by collaboration domain

### Negative
- More crate boundaries increase coordination overhead
- Cross-domain workflows require explicit orchestration in gateway/application layers

### Mitigation
- Keep boundary contracts explicit and well-tested
- Use ADRs and architecture docs to track cross-domain integration patterns
