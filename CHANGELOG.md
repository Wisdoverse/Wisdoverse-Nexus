# Changelog

All notable public changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Wisdoverse Nexus is pre-1.0, so breaking changes may still ship in minor
versions. Breaking changes must be called out in release notes.

## [Unreleased]

### Added

- Explicit room assistant runs with scoped context/read-only MCP tools, bounded
  OpenAI-compatible streaming, provider usage, idempotent invocation and cancellation.
  Web/mobile and both SDKs observe the shared run; synthetic live-gateway qualification
  is separate from real-provider and human pilot acceptance.

- Added static FSD/DDD boundary checks with behavior tests and CI enforcement for
  covered Web/mobile and gateway domain modules.
- Verified JWT session and room history endpoints, creator/member permission checks,
  stable retry keys and shared HTTP/WebSocket message acknowledgements.
- Real-gateway CI acceptance for both SDKs and Web/mobile adapters, with synthetic
  credentials, OpenAPI response validation and retained scenario evidence.

- Documented the M1 collaboration guide covering externally issued JWT
  sessions, HTTP and WebSocket message flows, reconnect recovery, and linked
  real-gateway acceptance evidence.

### Changed

- Redesigned the English and Chinese READMEs with badges, component icons, a client diagram, and linked development guides.
  The gateway image now includes curl for the documented Compose health check.

- Updated the repository toolchain and builder images to stable Rust 1.99.0 and integrated Cargo minor and patch updates.
  Updated Web React to 19.3 and Vite to 8.3.2 with matching lockfiles and GitHub Actions pins.
  Both runtime images install the current Bookworm PCRE2 security patch.

- Shortened the root agent instructions and linked task-specific development and
  writing guides. The writing guide applies an ASD-STE100-based project profile;
  simple bounded tasks use Luna when available.

- CI pins the qualified Rust 1.98.1 compiler instead of floating `stable`; compiler
  upgrades require a fresh qualification before changing the strict lint baseline.

- M1 Kubernetes/Helm profiles now use one instance and Recreate upgrades, inject
  JWT keys through an existing Secret, support image digests, and include startup
  probes. Helm rejects unqualified multi-replica or autoscaling configurations.
- Gateway HTTP shutdown drains accepted requests; local health probes remain
  reachable when HTTPS redirect is enabled. Cluster runtime, durable restart and
  WebSocket drain qualification remain future deployment gates.
- Production gateway startup now requires a nonblank `JWT_SECRET`. Mobile session
  restoration validates saved tokens before entering protected routes and keeps
  retryable sessions on network failures.
- Moved mobile session and Web room state into entities, moved mobile navigation
  types into shared, and removed frontend cross-slice and reverse-layer imports.
- Codified FSD for Web/mobile, DDD for backends, and cloud-native microservice
  requirements in AGENTS.md and ADR-008; developer docs distinguish the current
  M1 in-memory preview from qualified service/deployment targets.
- Updated bilingual roadmap tracking and M1 evidence with pinned successful CI
  revisions, browser checks, reproduction commands and remaining review gates.

- Aligned Web/mobile and SDK collaboration contracts; reconnect restores subscriptions
  and HTTP history recovers gaps in the documented in-memory support window.
- Consolidated pending compatible Rust, Web, SDK, docs, Playwright, Docker Rust
  builder and GitHub Actions updates on the current main baseline; preserved
  Expo 55 / React Native 0.83 constraints and newer existing dependency versions.
- Synchronized pnpm and tracked npm lockfiles, raised PostCSS override floors,
  and resolved React type and Expo DOM peer requirements. Docker builders use
  Bookworm to match the runtime; locked builds and health-check dependencies
  prevent unreviewed resolution and missing runtime probes.

- Reworked the English and Chinese roadmaps using a source-linked comparison
  of related collaboration/AI projects. Priorities now connect a governed
  human/AI room pilot, tenant reliability, document/task workflows, and
  versioned external-agent interoperability to the project vision.
- Added bilingual roadmap execution standards and a public work-item template:
  scoped packages, responsibility/dependency tracking, proposed quality and
  SLO targets, AI evaluation, release/recovery gates, and evidence requirements.
- Updated the Python SDK README and usage example for the M1 member session,
  room, message, and WebSocket APIs.
- **Breaking migration:** remove the fictional email/password `login` and
  `register` flows, `RegisterData`, and their email-oriented user and
  refresh-token models from integrations. Obtain an HS256 JWT from your own
  identity system and pass it to `authenticate(token)`. There is no login,
  registration, token issuance, or refresh endpoint in M1. Update room and
  message field access to the M1 `topic`, `text`, and `sender` model fields;
  the validated JWT subject determines the sender.
- Aligned the English and Chinese roadmaps around the implemented baseline,
  prioritized milestones, acceptance criteria, and evidence required before 1.0.
- Public examples now use local gateway URLs or reserved example domains in
  place of organization-specific service URLs and internal-looking hostnames.
- Contribution guides and issue/PR templates now describe privacy review and
  redaction requirements for public examples, logs, screenshots, and configuration.
- Aligned Expo 55 dependencies with the SDK compatibility recommendations,
  including React Native `0.83.10`, and refreshed the pnpm and npm lockfiles
  to resolve dependency audit findings.
- Updated the TypeScript SDK and apps to `axios` `^1.20.0`, and the SDK to
  `ws` `^8.22.0`. Consumers upgrading from the initial release also receive
  axios 1.16's behavior changes: the fetch
  adapter enforces `maxBodyLength` / `maxContentLength` (previously
  ignored), the proxy adapter preserves the original `Host` header,
  URL-encoded credentials in basic-auth are decoded before being sent,
  and `parseProtocol` is stricter about malformed schemes. Review your
  request size limits and proxy assumptions if you set them explicitly.

### Security

- Updated vulnerable Rust dependencies and replaced yanked locked versions.
- Documented a scoped maintenance-only exception for the transitive `smallstr`
  advisory `RUSTSEC-2026-0215`: `yrs` still requires it and no patched version
  exists. Revisit the exception when upstream migrates; vulnerability checks
  remain enabled.

## [0.1.0] - 2026-04-29

### Added

- Initial public GitHub baseline for Wisdoverse Nexus.
- Rust workspace, gateway, protocol, AI, context, memory, plugin, skills, and
  collaboration domain crates.
- React web app, Expo mobile app, TypeScript SDK, Python SDK, VitePress docs,
  and Docker/Helm deployment assets.
- GitHub issue templates, PR template, CI, release, security, and Dependabot
  configuration.

### Changed

- Public project identity updated to Wisdoverse Nexus.
- README, contribution guide, and documentation navigation revised for GitHub
  source-available collaboration.
- Documentation now favors English as the source of truth, with Chinese pages
  kept as companion material where maintained.
- VitePress documentation build now uses strict internal link validation.
- Release workflow now publishes the gateway image and TypeScript SDK only.
- Dependency update policy and automation were refreshed for the current
  workspace layout.

### Fixed

- Removed unverifiable customer, certification, pricing, and capacity claims
  from public documentation.
- Replaced stale module references with the current `nexis-*` crate layout.
- Clarified that benchmark pages describe methodology and commands, not
  guaranteed capacity.
- Clarified that security pages are project-maintained posture notes, not
  third-party audit or compliance attestations.

### Security

- Security reporting now uses GitHub private security advisories.
- CI security checks were updated for dependency and secret scanning coverage.

## Version Links

[Unreleased]: https://github.com/Wisdoverse/Wisdoverse-Nexus/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Wisdoverse/Wisdoverse-Nexus/releases/tag/v0.1.0
