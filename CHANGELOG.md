# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Documentation structure normalized into strict `docs/en` and `docs/zh-CN` trees.
- `CODE_OF_CONDUCT.md` based on Contributor Covenant.
- `docs/en/getting-started/development-guide.md` as the development handbook entry.

### Changed
- Root `README.md` is now English only.
- `README.zh-CN.md` is now Chinese only.
- Security policy now points to the new architecture security docs.

### Fixed
- Internal markdown links updated after docs reorganization.

### Removed
- Redundant root-level docs under `docs/` and completed plan/temp docs.
- Redundant root localized duplicates (`CONTRIBUTING.zh-CN.md`, `SECURITY.zh-CN.md`).

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

[Unreleased]: https://github.com/schorsch888/Nexis/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/schorsch888/Nexis/releases/tag/v0.1.0
