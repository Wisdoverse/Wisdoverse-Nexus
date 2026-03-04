# ADR-000: Architecture Decision Records

## Status

Accepted

## Context

We need a way to document and track significant architectural decisions in the Nexis project.

## Decision

We will use Architecture Decision Records (ADRs) to document key decisions.

ADR format based on [Michael Nygard's template](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions):

- **Title**: ADR-NNN: Brief decision title
- **Status**: Proposed | Accepted | Deprecated | Superseded
- **Context**: What is the issue we're addressing?
- **Decision**: What is the change?
- **Consequences**: What are the trade-offs?

## ADR Index

- [ADR-000: Architecture Decision Records](000-record-architecture-decisions.md)
- [ADR-001: Rust as Primary Language](001-rust-as-primary-language.md)
- [ADR-002: Axum Web Framework](002-axum-web-framework.md)
- [ADR-003: AI Provider Abstraction](003-ai-provider-abstraction.md)
- [ADR-004: PostgreSQL for Message Persistence](004-postgresql-for-persistence.md)
- [ADR-005: Phase 6 Collaboration Domain Modules](005-phase6-collaboration-domain-modules.md)
- [ADR-006: Phase 6 Collaboration Input Validation](006-phase6-collaboration-input-validation.md)
- [ADR-007: Phase 6 CI Quality Gates for Performance and Coverage](007-phase6-ci-quality-gates.md)

## Consequences

- Decisions are documented and traceable
- New team members can understand why decisions were made
- We can revisit decisions when context changes
