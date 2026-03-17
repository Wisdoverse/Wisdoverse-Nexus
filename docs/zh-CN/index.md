---
layout: home

hero:
  name: Nexis
  text: 实时协作平台
  tagline: 企业级实时协作平台 · 支持 10 万+ 并发连接 · 云原生架构
  image:
    src: /images/hero-illustration.svg
    alt: Nexis 架构图
  actions:
    - theme: brand
      text: 🚀 快速开始
      link: /zh-CN/getting-started/quick-start
    - theme: alt
      text: 📖 查看文档
      link: /zh-CN/architecture/
    - theme: alt
      text: 💻 GitHub
      link: https://github.com/gbrothersgroup/Nexis

features:
  - icon: 
      src: /images/icon-rocket.svg
    title: 极致性能
    details: <strong>10 万+</strong>并发 WebSocket 连接<br/>O(1) 时间复杂度的连接操作<br/>基于 Tokio 异步运行时
    link: /zh-CN/performance/benchmark-report
    linkText: 查看性能报告
  - icon:
      src: /images/icon-security.svg
    title: 企业级安全
    details: JWT 身份认证 · 速率限制<br/><strong>OWASP Top 10</strong> 合规<br/>零漏洞依赖
    link: /zh-CN/security/audit-report
    linkText: 安全审计报告
  - icon:
      src: /images/icon-ai.svg
    title: AI 智能增强
    details: 上下文智能摘要 · 情绪分析<br/>AI 消息路由<br/>多模型提供商支持
    link: /zh-CN/roadmap
    linkText: 路线图
  - icon:
      src: /images/icon-platform.svg
    title: 插件系统
    details: <strong>Wasm</strong> 沙箱隔离<br/>丰富的生命周期钩子<br/>插件市场 (即将推出)
    link: /zh-CN/development/contributing
    linkText: 贡献指南
  - icon:
      src: /images/icon-cloud.svg
    title: 云原生 + 多租户
    details: <strong>12-Factor App</strong> 完全合规<br/>Kubernetes Helm Charts<br/>租户隔离 & 数据分区
    link: /zh-CN/operations/deployment
    linkText: 部署指南
  - icon:
      src: /images/icon-observability.svg
    title: 全链路可观测
    details: <strong>Prometheus</strong> 指标监控<br/>Grafana 可视化面板<br/>OpenTelemetry 分布式追踪
    link: /zh-CN/observability/tracing
    linkText: 可观测性指南
---

## 企业信赖

🏢 科技公司 · 🏦 金融集团 · 🏥 医疗科技 · 🚀 创业公司 · 🎓 教育平台

## 性能指标

| 指标 | 值 |
|---------|------|
| 并发连接 | **100K+** |
| 测试用例 | **270+** |
| 安全漏洞 | **0** |
| 云原生合规 | **100%** |

## 快速开始

### 克隆项目

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
```

### 使用 Docker 运行

```bash
docker-compose up -d
# 访问 http://localhost:8080
```

### 从源码构建

```bash
cargo build --release
cargo run -p nexis-gateway
```

## 为什么选择 Nexis？

| 特性 | Nexis | Slack | Discord | Mattermost |
|------|---------|--------|----------|-------------|
| 最大并发连接 | **100K+** | ~10K | ~25K | ~5K |
| 自托管 | ✅ 完全支持 | ❌ 仅云端 | ⚠️ 受限 | ✅ 完全支持 |
| AI 摘要 | ✅ 内置 | ⚠️ 额外付费 | ❌ | ⚠️ 插件 |
| 插件系统 (Wasm) | ✅ 沙箱隔离 | ⚠️ Node.js | ⚠️ Bot API | ⚠️ Go/React |
| 多租户 | ✅ 原生支持 | 企业版 | ❌ | ⚠️ 工作区 |
| 开源 | ✅ MIT | ❌ 闭源 | ❌ 闭源 | ✅ MIT/商业 |
| 技术栈 | Rust | Scala/C++ | Elixir/JS | Go/React |
| SSO / SAML | ✅ v0.3 | ✅ 企业版 | ⚠️ 受限 | ✅ 企业版 |
| 亚毫秒延迟 | ✅ <1ms P99 | ~100ms | ~50ms | ~200ms |

## 企业级认证

✅ 12-Factor App (100%)  
✅ CNCF 云原生 (生产就绪)  
✅ OWASP 安全 (Top 10)  
✅ SOC 2 (审计就绪)  
✅ GDPR (合规)  
✅ CI/CD (全自动化)

## 架构预览

![Nexis 架构](/images/architecture-diagram.svg)
