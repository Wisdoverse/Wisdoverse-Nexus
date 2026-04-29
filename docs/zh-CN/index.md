---
layout: page
title: Wisdoverse Nexus
description: 源码可用 AI 原生协作基础设施，覆盖实时房间、身份、WebSocket 消息、AI provider 集成和可扩展工作流。
aside: false
sidebar: false
editLink: false
lastUpdated: false
---

<script setup>
import { withBase } from 'vitepress'
</script>

<main class="nexus-home">
  <section class="nexus-home__hero">
    <p class="nexus-home__eyebrow">Wisdoverse Nexus 文档</p>
    <h1 class="nexus-home__title">为生产团队组合的 AI 协作基础设施。</h1>
    <p class="nexus-home__lead">
      Rust 优先的 gateway、共享协议、SDK、Web/Mobile 界面和部署资源，用于实时房间、身份、消息和 AI 辅助工作流。
    </p>
    <div class="nexus-home__actions">
      <a class="nexus-button nexus-button--primary" :href="withBase('/zh-CN/getting-started/quick-start')">开始构建</a>
      <a class="nexus-button nexus-button--secondary" :href="withBase('/zh-CN/architecture/')">查看架构</a>
    </div>
    <div class="nexus-home__visual" aria-label="Wisdoverse Nexus 产品界面预览">
      <div class="nexus-console">
        <aside class="nexus-console__rail">
          <div class="nexus-console__brand">
            <img class="nexus-console__mark" :src="withBase('/images/logo.svg')" alt="" />
            <span>Nexus</span>
          </div>
          <nav class="nexus-console__nav" aria-label="预览导航">
            <span>Gateway</span>
            <span>Rooms</span>
            <span>Identity</span>
            <span>AI Providers</span>
            <span>SDKs</span>
            <span>Deploy</span>
          </nav>
        </aside>
        <div class="nexus-console__main">
          <div class="nexus-console__bar">
            <span>main 分支预览</span>
            <span class="nexus-console__status">可构建，可审计</span>
          </div>
          <h2 class="nexus-console__headline">一个仓库承载 gateway、客户端、协议、文档和发布门禁。</h2>
          <div class="nexus-console__grid">
            <div class="nexus-metric">
              <strong>Rust</strong>
              <span>Gateway 与共享领域 crate</span>
            </div>
            <div class="nexus-metric">
              <strong>WebSocket</strong>
              <span>实时房间与消息链路</span>
            </div>
            <div class="nexus-metric">
              <strong>SDKs</strong>
              <span>TypeScript 与 Python 客户端</span>
            </div>
          </div>
          <pre class="nexus-code"><code>cargo check --workspace
cargo run -p nexis-gateway
curl http://localhost:8080/health</code></pre>
        </div>
      </div>
    </div>
  </section>

  <section class="nexus-section">
    <div class="nexus-section__inner">
      <p class="nexus-section__kicker">从当前有效路径开始</p>
      <h2 class="nexus-section__title">关键入口必须一眼可达。</h2>
      <p class="nexus-section__copy">
        首页直接把评估者带到最需要的内容：启动、运行时架构、API 契约、部署和许可证边界。
      </p>
      <div class="nexus-tiles">
        <article class="nexus-tile">
          <h3>本地运行</h3>
          <p>从全新 checkout 使用当前 gateway 命令和 Node workspace 检查。</p>
          <a :href="withBase('/zh-CN/getting-started/quick-start')">快速开始</a>
        </article>
        <article class="nexus-tile">
          <h3>理解系统</h3>
          <p>阅读 Rust-first 架构、租户模型、provider 设计和协议边界。</p>
          <a :href="withBase('/zh-CN/architecture/')">架构概览</a>
        </article>
        <article class="nexus-tile">
          <h3>评估运维</h3>
          <p>检查部署资源、监控、故障排查、追踪、发布门禁和安全说明。</p>
          <a :href="withBase('/en/operations/deployment')">部署文档</a>
        </article>
      </div>
    </div>
  </section>

  <section class="nexus-section nexus-section--soft">
    <div class="nexus-section__inner">
      <p class="nexus-section__kicker">仓库地图</p>
      <h2 class="nexus-section__title">源码树应该像产品系统一样可读。</h2>
      <div class="nexus-map">
        <div class="nexus-map__list">
          <div class="nexus-map__row">
            <code>crates/</code>
            <span>Gateway、协议、AI 集成、上下文、插件和协作领域 crate。</span>
          </div>
          <div class="nexus-map__row">
            <code>apps/</code>
            <span>React + Vite Web 应用和 Expo-managed Mobile 应用。</span>
          </div>
          <div class="nexus-map__row">
            <code>sdk/</code>
            <span>TypeScript 与 Python 客户端，用于外部集成和可测试示例。</span>
          </div>
          <div class="nexus-map__row">
            <code>deploy/</code>
            <span>Docker Compose、Helm、Prometheus 和 Grafana 资源与代码一起维护。</span>
          </div>
        </div>
        <figure class="nexus-map__image">
          <img :src="withBase('/images/architecture-diagram.svg')" alt="Wisdoverse Nexus 架构图" />
        </figure>
      </div>
    </div>
  </section>

  <section class="nexus-band">
    <div class="nexus-band__inner">
      <div>
        <h2>状态清楚，边界清楚。</h2>
        <p>
          Wisdoverse Nexus 当前处于 pre-1.0，并采用 Wisdoverse Nexus Business Source License 1.1。文档明确区分评估、贡献和商业使用边界。
        </p>
      </div>
      <div class="nexus-band__links">
        <a :href="withBase('/en/license')">许可证摘要 <span>打开</span></a>
        <a :href="withBase('/zh-CN/roadmap')">路线图 <span>打开</span></a>
        <a :href="withBase('/zh-CN/development/contributing')">贡献指南 <span>打开</span></a>
        <a href="https://github.com/Wisdoverse/Wisdoverse-Nexus">GitHub 仓库 <span>打开</span></a>
      </div>
    </div>
  </section>
</main>
