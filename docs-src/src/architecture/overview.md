# Architecture Overview

Wisdoverse Nexus follows a layered architecture for protocol, runtime, and gateway responsibilities.

## Core Layers

- `nexis-protocol`: shared protocol and identity primitives
- `nexis-gateway`: HTTP/WebSocket gateway and service orchestration
- `nexis-ai` and `nexis-mcp`: AI provider and MCP integration surfaces
- `nexis-context` and `nexis-memory`: context and memory domain modules
- `nexis-plugin` and `nexis-skills`: extension surfaces

## Multi-Tenant Model

Entity hierarchy:

- Tenant
- Workspace
- Member
- Room
- Message

Tenant-aware boundaries are designed for strict data isolation and future enterprise controls.

## Data and Search

- Structured entities via Rust domain models
- Search services with vector integration points
- Extensible provider registry for AI backends

## Related Docs

- Existing model detail: [tenant-model.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/docs/en/architecture/tenant-model.md)
- ADRs: [docs/en/architecture/adr](https://github.com/Wisdoverse/Wisdoverse-Nexus/tree/main/docs/en/architecture/adr)
