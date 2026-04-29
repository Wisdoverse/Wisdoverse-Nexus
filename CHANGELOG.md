# Changelog

All notable public changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Wisdoverse Nexus is pre-1.0, so breaking changes may still ship in minor
versions. Breaking changes must be called out in release notes.

## [Unreleased]

No unreleased changes yet.

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
