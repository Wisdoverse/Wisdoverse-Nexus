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
| Frontend boundaries | Review slice ownership/public APIs and import direction; retain configured dependency checks | Static checker passes for the covered Web/mobile modules; it checks relative import direction and cross-slice public-index usage, but does not prove complete FSD compliance or runtime behavior |
| Backend boundaries | Review domain dependencies, adapters and cross-context contracts; test affected use cases | Static checker passes for the covered gateway domain modules; it flags selected transport, storage and runtime references. DDD still requires review of contracts/use cases; a Rust crate is not independent deployment evidence |
| M1 deployment controls | Verify locked image inputs, external secret injection, non-root execution, resource limits, probes and shutdown behavior | Helm and raw Kubernetes manifests specify one replica, external Secret injection, startup/readiness/liveness probes, non-root security settings, and a 30-second grace period. Helm rejects multiple replicas and HPA; `Recreate` prevents overlapping gateway instances during upgrades. These are configuration controls, not evidence of production qualification |
| Service state/operation | Verify service-owned data, restart/recovery, scaling, meaningful readiness, draining and rollback | Room/message state remains in memory and is lost on restart or upgrade. Durable recovery, multi-node and cluster runtime behavior, and WebSocket draining remain unqualified and gated by M3 evidence. The gateway waits for accepted in-flight HTTP responses during graceful shutdown; health probes bypass HTTPS redirect. The render-only deployment gate verifies Helm lint/render, image/Secret references, rejected unsafe profiles, raw Kubernetes controls and the Compose signing-key variable |

CI runs `pnpm test:architecture` and `pnpm check:architecture` in a Node job.
The checker currently covers 98 frontend modules and 4 gateway domain modules;
these counts describe its static scan scope, not a qualification of every
architectural property or production readiness.

Architecture changes update this decision or add a superseding ADR, their
contracts, developer/deployment documentation and PR evidence. Record existing
gaps with a migration scope instead of presenting planned requirements as
implemented capabilities. This decision introduces no runtime topology change.

See the [architecture overview](../../architecture.md),
[M1 acceptance report](../../guides/m1-acceptance.md), and
[roadmap execution standard](../../guides/roadmap-execution.md).
