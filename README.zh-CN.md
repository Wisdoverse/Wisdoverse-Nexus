# Wisdoverse Nexus

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/Wisdoverse/Wisdoverse-Nexus/ci.yml?branch=main&label=ci)](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions)
[![Security](https://img.shields.io/badge/security-policy-green)](SECURITY.md)
[![License](https://img.shields.io/badge/license-Wisdoverse%20BSL--1.1-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-wisdoverse.com-blue)](https://wisdoverse.com)

**面向 AI 原生协作场景的源码可用基础设施。**

</div>

Wisdoverse Nexus 是一个 Rust 优先的实时协作平台，覆盖房间协作、成员身份、
WebSocket 消息、AI 提供商集成以及可扩展的协作工作流。

本仓库是项目的 GitHub 源码可用仓库。产品与社区文档发布在
<https://wisdoverse.com>。

## 项目状态

Wisdoverse Nexus 目前处于 `0.x` 阶段。API、部署清单和 SDK 表面在稳定版发布前
仍可能变化。`main` 分支会保持可构建、可审计，但生产评估建议固定 release 或
commit SHA。

## 仓库结构

| 路径 | 说明 |
| --- | --- |
| `crates/` | Rust workspace，包含协议、网关、AI、上下文、插件和协作域模块 |
| `apps/web/` | React + Vite Web 应用 |
| `apps/mobile/` | Expo / React Native 移动应用 |
| `sdk/typescript/` | TypeScript SDK |
| `sdk/python/` | Python SDK |
| `docs/` | VitePress 文档站 |
| `deploy/` | Docker Compose、Helm、Prometheus、Grafana 部署资源 |
| `.github/` | GitHub Actions、Issue 模板、PR 模板和 Dependabot 配置 |

## 环境要求

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker 或 Docker Compose，用于容器化本地运行

## 快速开始

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

cargo check --workspace
cargo run -p nexis-gateway
```

Node workspace 验证：

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

Docker 运行：

```bash
docker compose up -d
curl http://localhost:8080/health
```

## 文档

- [官方文档站](https://wisdoverse.com)
- [本地文档首页](docs/index.md)
- [快速开始](docs/zh-CN/getting-started/quick-start.md)
- [开发指南](docs/zh-CN/getting-started/development-guide.md)
- [架构概览](docs/zh-CN/architecture/index.md)
- [安全审计](docs/zh-CN/security/audit-report.md)
- [英文 README](README.md)

## 质量门禁

GitHub workflow 预期覆盖：

- Rust 格式化、clippy、构建、测试、文档、审计、许可证检查和容器扫描
- Node workspace 安装、Expo 兼容检查、mobile 类型检查/测试、web 构建、SDK 构建和文档构建
- Dependabot 依赖更新自动化
- GitHub Security Advisories 私有漏洞报告

提交 PR 前建议至少运行：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build
```

## 参与贡献

欢迎贡献。请先阅读：

- [贡献指南](CONTRIBUTING.md)
- [行为准则](CODE_OF_CONDUCT.md)
- [适合新贡献者的 issue](https://github.com/Wisdoverse/Wisdoverse-Nexus/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

请保持 PR 聚焦，关联相关 issue；如果行为发生变化，同步更新文档；PR 描述中写明
已经执行的验证命令。

## 安全

请不要通过公开 issue 报告安全漏洞。私有报告渠道见 [SECURITY.md](SECURITY.md)。

## 许可证

Wisdoverse Nexus 使用 [Wisdoverse Nexus Business Source License 1.1](LICENSE)。

未经商业授权，允许个人学习、研究、教育、开发测试、内部评估和非生产环境使用。

未经商业授权，不允许商业生产环境使用、对外提供 SaaS / 托管服务、提供 managed
service、转售、二次授权或用于构建竞争性产品。

每个公开发布版本在首次公开满 4 年后自动转换为 Apache License 2.0。
