# Quickstart

This page is kept for compatibility with older links. Use the maintained
[Quick Start](quick-start.md) page for current setup commands.

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus
cargo check --workspace
cargo run -p nexis-gateway
```

Validate the gateway:

```bash
curl http://localhost:8080/health
# OK
```

Next:

- [Quick Start](quick-start.md)
- [Development Guide](development-guide.md)
- [Contributing](../development/contributing.md)
