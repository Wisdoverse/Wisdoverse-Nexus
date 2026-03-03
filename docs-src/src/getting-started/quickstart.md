# Quick Start

## Prerequisites

- Rust 1.75+
- Git

## Clone and Build

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
cargo build --release
```

## Run Gateway

```bash
cargo run --release -p nexis-gateway
```

Gateway default address: `http://127.0.0.1:8080`

## Create a Room from CLI

```bash
cargo run --release -p nexis-cli -- create-room "general" --topic "Team discussion"
```

## Send a Message

```bash
cargo run --release -p nexis-cli -- \
  send-message "room_abc123" \
  "nexis:human:alice@example.com" \
  "Hello, Nexis!"
```

## Next

- [Architecture](../architecture/overview.md)
- [Rust API Reference](../api-reference.md)
