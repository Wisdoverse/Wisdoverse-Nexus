# 架构概述

Wisdoverse Nexus 是一个模块化的 AI 原生协作平台，采用 Rust 异步架构。

当前实现由 HTTP/WebSocket 网关和模块化领域 crate 构成；云原生微服务是必须遵循的演进约束，不代表服务已经拆分或生产部署能力已验证。前端代码按 FSD 层级单向依赖，后端按 DDD 隔离领域模型与传输、存储和框架层。演进为独立服务时，每个服务必须拥有自己的数据和迁移，并通过发布的契约访问其他服务数据。

容器与 Kubernetes 部署须使用可复现的锁定构建、非 root 运行、外部配置和密钥注入、资源请求与限制、启动/就绪/存活探针、优雅关闭及请求排空。服务还须提供结构化日志、指标、传播的 trace/correlation ID，定义超时、有限队列、重试退避和幂等边界，并记录发布、回滚和 schema 兼容方案。当前 M1 是单进程、单租户、内存存储预览；云原生约束不构成持久化、水平扩展或生产就绪的验收证明。详见[根目录 AGENTS.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/AGENTS.md)和[架构 ADR 008](../../en/architecture/adr/008-fsd-ddd-cloud-native-services.md)。

## 核心设计原则

1. **可扩展性能** — 基于 Tokio 异步运行时，并通过 benchmark 验证容量边界
2. **模块化** — 每个功能独立 crate，可单独使用
3. **可扩展** — 插件系统支持 Wasm 沙箱隔离
4. **云原生友好** — 通过容器、Helm 和可观测性配置支持部署演进

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                      客户端层                          │
│   Web Client  │  Mobile App  │  CLI  │  Third-party         │
└──────────────────────┬─────────────────────────────────┘
                     │ WebSocket / HTTP
┌──────────────────────▼─────────────────────────────────┐
│                   nexis-gateway                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ Auth     │ │ Router   │ │ Rate     │ │ Metrics  │        │
│  │ Middleware│ │          │ │ Limiter  │ │          │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
└──────────────────────┬─────────────────────────────────┘
                     │
┌──────────────────────▼─────────────────────────────────┐
│                     核心层                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ nexis-   │ │ nexis-   │ │ nexis-   │ │ nexis-   │        │
│  │ protocol │ │ context  │ │ memory   │ │ task     │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
└──────────────────────┬─────────────────────────────────┘
                     │
┌──────────────────────▼─────────────────────────────────┐
│                      扩展层                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ nexis-   │ │ nexis-   │ │ nexis-   │ │ nexis-   │        │
│  │ plugin   │ │ mcp      │ │ ai       │ │ skills   │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## 核心 Crates

| Crate | 职责 |
|-------|------|
| `nexis-gateway` | WebSocket 网关、HTTP API |
| `nexis-protocol` | 消息协议定义 |
| `nexis-context` | 会话上下文管理 |
| `nexis-plugin` | Wasm 插件系统 |
| `nexis-memory` | 长期记忆存储 |
| `nexis-ai` | AI 提供商抽象和协作能力 |

## 技术栈

- **运行时**: Tokio 1.x
- **HTTP/WebSocket**: Axum + axum-extra
- **序列化**: Serde
- **认证**: JWT (jsonwebtoken)
- **追踪**: OpenTelemetry + Tracing
- **测试**: 内置 test framework + tokio-test

## 下一步

- [开发指南](/zh-CN/getting-started/development-guide) — 本地开发环境
- [贡献指南](/zh-CN/development/contributing) — 参与项目开发
- [性能报告](/zh-CN/performance/benchmark-report) — 性能基准测试
