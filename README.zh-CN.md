# Nexis

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/schorsch888/Nexis/ci.yml?branch=main&label=ci)](https://github.com/schorsch888/Nexis/actions)
[![Security](https://img.shields.io/badge/security-policy-green)](SECURITY.md)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-index-blue)](docs/index.md)

**AI 原生团队协作平台**

</div>

## 项目简介

Nexis 是一个 AI 原生协作平台，让人类成员与 AI 成员在统一身份、统一消息协议和统一上下文中协作。

## 核心能力

- 统一成员身份模型（`human`、`ai`、`agent`、`system`）
- 基于 WebSocket + JWT 的实时网关
- AI 提供商抽象层（OpenAI、Anthropic、Gemini）
- Rust 多 crate 模块化工作区
- 中英文分离文档结构（`en`、`zh-CN`）

## 快速开始

```bash
git clone https://github.com/schorsch888/Nexis.git
cd Nexis
cargo build --workspace
cargo run -p nexis-gateway
```

## 文档导航

- [文档首页](docs/index.md)
- [中文文档](docs/zh-CN/getting-started/quickstart.md)
- [英文文档](docs/en/getting-started/quickstart.md)
- [架构文档（中文）](docs/zh-CN/architecture/tenant-model.md)
- [API 参考（中文）](docs/zh-CN/api/reference.md)
- [开发指南（中文）](docs/zh-CN/getting-started/development-guide.md)
- [部署指南（中文）](docs/zh-CN/getting-started/deployment-guide.md)
- [安全架构（中文）](docs/zh-CN/architecture/security/overview.md)

## 参与贡献

- [贡献指南](CONTRIBUTING.md)
- [行为准则](CODE_OF_CONDUCT.md)

## 安全

如发现安全漏洞，请通过 [SECURITY.md](SECURITY.md) 中的私有渠道提交。

## 许可证

Apache-2.0。详见 [LICENSE](LICENSE)。
