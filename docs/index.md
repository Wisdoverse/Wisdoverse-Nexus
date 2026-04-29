---
layout: home

hero:
  name: Wisdoverse Nexus
  text: Source-available AI-native collaboration infrastructure
  tagline: Rust-first real-time rooms, identity, WebSocket messaging, AI provider integration, and extensible collaboration workflows.
  image:
    src: /images/hero-illustration.svg
    alt: Wisdoverse Nexus architecture illustration
  actions:
    - theme: brand
      text: Quick Start
      link: /en/getting-started/quick-start
    - theme: alt
      text: Development Guide
      link: /en/getting-started/development-guide
    - theme: alt
      text: GitHub
      link: https://github.com/Wisdoverse/Wisdoverse-Nexus

features:
  - title: Real-time Gateway
    details: HTTP and WebSocket gateway implemented in Rust with Tokio and Axum.
    link: /en/api/websocket
    linkText: WebSocket API
  - title: Shared Protocol
    details: Identity, message, permission, and tenant primitives live in versioned Rust crates.
    link: /en/architecture/tenant-model
    linkText: Tenant Model
  - title: AI Integration Surface
    details: Provider abstraction and MCP integration surfaces for AI-assisted collaboration.
    link: /en/design/provider-system-design
    linkText: Provider Design
  - title: Web and Mobile Apps
    details: React + Vite web app and Expo-managed mobile app built from the same repository.
    link: /en/getting-started/development-guide
    linkText: Development Setup
  - title: Deployment Assets
    details: Docker Compose, Helm, Prometheus, and Grafana assets are kept under version control.
    link: /en/operations/deployment
    linkText: Deployment Guide
  - title: Open Governance
    details: Public contribution, security, release, and dependency update processes for GitHub collaboration.
    link: /en/development/contributing
    linkText: Contributing
---

## Project Status

Wisdoverse Nexus is currently pre-1.0. APIs, deployment manifests, and SDK
surfaces may change before a stable release. The `main` branch is expected to
remain buildable and audited, but production adopters should pin releases or
commit SHAs during evaluation.

## License Summary

Wisdoverse Nexus is source-available under the Wisdoverse Nexus Business Source
License 1.1. Personal learning, research, education, development/testing,
internal evaluation, and non-production use are permitted. Commercial production
use, SaaS or hosted services, managed services, resale, sublicensing, and
competitive products require a separate commercial license.

Each published version converts to Apache License 2.0 four years after that
version is first made publicly available.

## What Is in This Repository

| Area | Path |
| --- | --- |
| Rust workspace | `crates/` |
| Web application | `apps/web/` |
| Mobile application | `apps/mobile/` |
| TypeScript SDK | `sdk/typescript/` |
| Python SDK | `sdk/python/` |
| Documentation site | `docs/` |
| Deployment assets | `deploy/` |
| GitHub automation | `.github/` |

## Quick Start

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

cargo check --workspace
cargo run -p nexis-gateway
```

Node workspace checks:

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## Quality Gates

The repository is set up for GitHub Actions coverage across:

| Gate | Coverage |
| --- | --- |
| Rust | format, clippy, build, tests, docs, audit, and license checks |
| Node workspace | frozen pnpm install, Expo compatibility, mobile checks, web build, SDK build, docs build |
| Security | private advisory flow, dependency audit, secret scanning, and container scanning |
| Dependencies | Dependabot coverage for Cargo, npm packages, Dockerfiles, and GitHub Actions |

## Documentation

- [Quick Start](/en/getting-started/quick-start)
- [Development Guide](/en/getting-started/development-guide)
- [Architecture Overview](/en/architecture)
- [API Reference](/en/api/reference)
- [Security Overview](/en/architecture/security/overview)
- [License](/en/license)
- [Roadmap](/en/roadmap)
