# Quick Start

## Prerequisites

- Rust stable
- Git

## Clone and Check

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus
cargo check --workspace
```

## Run Gateway

```bash
cargo run -p nexis-gateway
```

Validate from another terminal:

```bash
curl http://127.0.0.1:8080/health
# OK
```

## Next

- [Architecture](../architecture/overview.md)
- [Rust API Reference](../api-reference.md)
