---
layout: home

hero:
  name: Wisdoverse Nexus
  text: 源码可用 AI 原生协作基础设施
  tagline: Rust 优先的实时房间、身份、WebSocket 消息、AI provider 集成和可扩展协作工作流。
  image:
    src: /images/hero-illustration.svg
    alt: Wisdoverse Nexus 架构示意图
  actions:
    - theme: brand
      text: 快速开始
      link: /zh-CN/getting-started/quick-start
    - theme: alt
      text: 开发指南
      link: /zh-CN/getting-started/development-guide
    - theme: alt
      text: GitHub
      link: https://github.com/Wisdoverse/Wisdoverse-Nexus

features:
  - title: 实时网关
    details: 基于 Rust、Tokio 和 Axum 实现 HTTP 与 WebSocket 网关。
    link: /en/api/websocket
    linkText: WebSocket API
  - title: 统一协议
    details: 身份、消息、权限和租户基础类型保存在版本化 Rust crate 中。
    link: /en/architecture/tenant-model
    linkText: 租户模型
  - title: AI 集成表面
    details: 为 AI 协作提供 provider 抽象和 MCP 集成表面。
    link: /en/design/provider-system-design
    linkText: Provider 设计
  - title: Web 与 Mobile 应用
    details: 同一仓库内维护 React + Vite Web 应用和 Expo-managed Mobile 应用。
    link: /zh-CN/getting-started/development-guide
    linkText: 开发环境
  - title: 部署资源
    details: Docker Compose、Helm、Prometheus 和 Grafana 资源纳入版本管理。
    link: /en/operations/deployment
    linkText: 部署指南
  - title: 开放协作
    details: 面向 GitHub 协作的贡献、安全、发布和依赖更新流程。
    link: /zh-CN/development/contributing
    linkText: 贡献指南
---

## 项目状态

Wisdoverse Nexus 当前处于 `0.x` 阶段。API、部署清单和 SDK 表面在稳定版发布前
仍可能变化。`main` 分支预期保持可构建和可审计，但生产评估建议固定 release 或
commit SHA。

## 许可证摘要

Wisdoverse Nexus 采用 Wisdoverse Nexus Business Source License 1.1。未经商业
授权，允许个人学习、研究、教育、开发测试、内部评估和非生产环境使用。

未经商业授权，不允许商业生产环境使用、对外提供 SaaS / 托管服务、提供 managed
service、转售、二次授权或用于构建竞争性产品。

每个公开发布版本在首次公开满 4 年后自动转换为 Apache License 2.0。

## 仓库内容

| 区域 | 路径 |
| --- | --- |
| Rust workspace | `crates/` |
| Web 应用 | `apps/web/` |
| Mobile 应用 | `apps/mobile/` |
| TypeScript SDK | `sdk/typescript/` |
| Python SDK | `sdk/python/` |
| 文档站 | `docs/` |
| 部署资源 | `deploy/` |
| GitHub 自动化 | `.github/` |

## 快速开始

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

cargo check --workspace
cargo run -p nexis-gateway
```

Node workspace 检查：

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## 质量门禁

GitHub Actions 预期覆盖：

| 门禁 | 覆盖范围 |
| --- | --- |
| Rust | 格式化、clippy、构建、测试、文档、审计和许可证检查 |
| Node workspace | pnpm frozen install、Expo 兼容检查、mobile 检查、web build、SDK build、docs build |
| 安全 | 私有 advisory 流程、依赖审计、secret scanning 和容器扫描 |
| 依赖 | Cargo、npm、Dockerfile 和 GitHub Actions 的 Dependabot 覆盖 |

## 文档导航

- [快速开始](/zh-CN/getting-started/quick-start)
- [开发指南](/zh-CN/getting-started/development-guide)
- [架构概览](/zh-CN/architecture/)
- [安全审计](/zh-CN/security/audit-report)
- [路线图](/zh-CN/roadmap)
