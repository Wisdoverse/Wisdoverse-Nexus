# Archived Phase 5 Review Snapshot

> This page is historical. It records the quality bar used during a 2026-02-27
> review pass and must not be treated as the current project state.

The original review covered `crates/nexis-a2a/`, `crates/nexis-memory/`, and
`crates/nexis-skills/`. Since then, the repository layout has changed and the
current source tree should be verified directly before making release or merge
decisions.

## Current Review Rule

For current work, use the maintained gates in:

- [Testing Guide](../en/guides/testing.md)
- [Contributing Guide](../en/development/contributing.md)
- [Security Overview](../en/architecture/security/overview.md)

## Durable Lessons

- Review findings must cite current file paths and line numbers.
- Build, lint, and test commands are part of the finding evidence.
- Historical review documents need an archive label once the codebase changes.
- Coverage percentages should not be claimed unless a reproducible coverage
  command and report artifact exist.

## Re-Review Checklist

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
```

When a focused crate is under review, include its package-specific command and
the exact commit SHA in the review notes.
