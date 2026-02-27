# Contributing to Nexis

Thanks for your interest in contributing.

## Development Environment

### Prerequisites

- Rust `1.75+`
- Git `2.30+`
- Optional: Docker

### Setup

```bash
git clone https://github.com/schorsch888/Nexis.git
cd Nexis
cargo build --workspace
cargo test --workspace
```

## Workflow

1. Create a branch from `main`: `feat/<topic>` or `fix/<topic>`.
2. Keep changes focused with clear commit messages.
3. Run format, lint, and tests locally before pushing.
4. Open a PR and complete the template/checklist.

## Code Standards

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Documentation expectations:

- Keep links relative and valid.
- Update docs when behavior changes.
- Keep language-specific docs in `docs/en` and `docs/zh-CN`.

## Pull Request Process

1. Use the PR template and describe user impact.
2. Link related issues.
3. Ensure CI is green.
4. Request at least one reviewer.

## Commit Convention

Use Conventional Commits:

```text
feat(scope): short summary
fix(scope): short summary
docs: short summary
refactor(scope): short summary
test(scope): short summary
chore: short summary
```

## Community Rules

By participating, you agree to follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
