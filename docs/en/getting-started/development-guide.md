# Development Guide

## Environment Setup

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
cargo build --workspace
```

## Local Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p nexis-gateway
cargo run -p nexis-cli -- --help
```

## Quality Gates

- Do not introduce new lint warnings.
- Keep tests green for affected crates.
- Update documentation when behavior changes.

## Branch and PR

- Branch naming: `feat/*`, `fix/*`, `docs/*`
- Commit style: Conventional Commits
- PRs should include scope and test evidence
