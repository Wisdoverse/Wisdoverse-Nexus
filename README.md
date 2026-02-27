# Nexis

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/schorsch888/Nexis/ci.yml?branch=main&label=ci)](https://github.com/schorsch888/Nexis/actions)
[![Security](https://img.shields.io/badge/security-policy-green)](SECURITY.md)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-index-blue)](docs/index.md)

**AI-Native Team Communication Platform**

</div>

## Overview

Nexis is an AI-native collaboration platform where human members and AI members share a unified identity, messaging protocol, and runtime context.

## Features

- Unified identity model (`human`, `ai`, `agent`, `system`)
- Real-time gateway over WebSocket + JWT auth
- AI provider integration abstraction (OpenAI, Anthropic, Gemini)
- Rust workspace with modular crates
- Multi-language documentation (`en`, `zh-CN`)

## Quick Start

```bash
git clone https://github.com/schorsch888/Nexis.git
cd Nexis
cargo build --workspace
cargo run -p nexis-gateway
```

## Documentation

- [Documentation Home](docs/index.md)
- [English Docs](docs/en/getting-started/quickstart.md)
- [Chinese Docs](docs/zh-CN/getting-started/quickstart.md)
- [Architecture](docs/en/architecture/tenant-model.md)
- [API Reference](docs/en/api/reference.md)
- [Development Guide](docs/en/getting-started/development-guide.md)
- [Deployment Guide](docs/en/getting-started/deployment-guide.md)
- [Security Architecture](docs/en/architecture/security/overview.md)

## Contributing

- [Contributing Guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## Security

Please report vulnerabilities privately via [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
