# ADR-008: FSD, DDD, and cloud-native services

- Date: 2026-10-01
- Status: Accepted architecture constraint; service migration remains to be qualified.
- Scope: Web/mobile, backend bounded contexts, and service/deployment changes.
- Governing instructions: [AGENTS.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/AGENTS.md).

## Context

The repository contains Web/mobile clients, a gateway, domain crates, and
container/Compose/Helm assets. M1 qualifies one single-tenant, in-memory gateway
process. The required architecture is FSD for frontends, DDD for backends, and
cloud-native microservices for service evolution. Deployment assets alone do
not establish independent services, durability, or horizontal scaling.

## Decision

Frontends use `app → pages → widgets → features → entities → shared`.
Imports point downward through explicit slice public APIs. Domain state belongs
in entities, user actions in features, route composition in pages, and reusable
transport/UI/configuration in shared. Shared code remains independent of app
and feature stores; authentication dependencies use explicit lower-layer contracts.

Backend domain models, aggregates, value objects, and domain errors remain
independent of Axum, SQL, and external providers. Application services orchestrate
use cases; interfaces/infrastructure translate transport and storage contracts.
Define bounded-context ownership and use explicit commands/events across contexts.

Define microservice boundaries around bounded contexts and operational ownership.
Each deployed service has independently buildable, versioned artifacts and owns
its data and migrations. Cross-service access uses published APIs/events with
versioning, authorization, timeouts, bounded queues, retry/backoff, and idempotency.
Qualified scalable profiles keep durable state outside replaceable instances.

Cloud-native deployment changes provide locked builds, compatible base/runtime
images, non-root processes, injected configuration/secrets, resource requests
and limits, startup/readiness/liveness probes, and graceful draining on shutdown.
Service telemetry includes structured redacted logs, metrics and propagated
trace/correlation IDs. Record rollout, rollback, schema compatibility and recovery
checks with the affected service and deployment profile.

## Consequences and qualification

| Area | Required review/evidence | Current qualification |
| --- | --- | --- |
| Frontend boundaries | Review slice ownership/public APIs and import direction; retain configured dependency checks | FSD is required; M1 browser/transport tests do not prove every repository import complies |
| Backend boundaries | Review domain dependencies, adapters and cross-context contracts; test affected use cases | DDD is required; a Rust crate is a module boundary, not independent deployment evidence |
| Service state/operation | Verify service-owned data, restart/recovery, scaling, meaningful readiness, draining and rollback | M1 supports one in-memory gateway; broader deployment remains gated by M3 evidence |

Architecture changes update this decision or add a superseding ADR, their
contracts, developer/deployment documentation and PR evidence. Record existing
gaps with a migration scope instead of presenting planned requirements as
implemented capabilities. This decision introduces no runtime topology change.

See the [architecture overview](../../architecture.md),
[M1 acceptance report](../../guides/m1-acceptance.md), and
[roadmap execution standard](../../guides/roadmap-execution.md).
