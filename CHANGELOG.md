# Changelog

All notable public changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Wisdoverse Nexus is pre-1.0, so breaking changes may still ship in minor
versions. Breaking changes must be called out in release notes.

## [Unreleased]

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

## [0.1.0] - 2026-02-14

### Added

- Initial Rust workspace and crate layout.
- Core protocols (NIP-001, NIP-002, NIP-003).
- WebSocket gateway with JWT authentication.
- CLI foundations and core workflows.
- CI/CD baseline with code quality and security checks.

### Security

- Initial `SECURITY.md` policy and private disclosure channel.
- Automated dependency and secret scanning in CI.

## Version Links

[Unreleased]: https://github.com/Wisdoverse/Wisdoverse-Nexus/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Wisdoverse/Wisdoverse-Nexus/releases/tag/v0.1.0
