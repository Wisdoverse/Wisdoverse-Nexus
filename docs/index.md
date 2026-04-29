---
layout: page
title: Wisdoverse Nexus
description: Source-available AI-native collaboration infrastructure for real-time rooms, identity, WebSocket messaging, and extensible workflows.
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
    <p class="nexus-home__eyebrow">Wisdoverse Nexus documentation</p>
    <h1 class="nexus-home__title">AI collaboration infrastructure, composed for production teams.</h1>
    <p class="nexus-home__lead">
      A Rust-first gateway, shared protocols, SDKs, web and mobile surfaces, and deployment assets for real-time rooms, identity, messaging, and AI-assisted workflows.
    </p>
    <div class="nexus-home__actions">
      <a class="nexus-button nexus-button--primary" :href="withBase('/en/getting-started/quick-start')">Start building</a>
      <a class="nexus-button nexus-button--secondary" :href="withBase('/en/architecture')">View architecture</a>
    </div>
    <div class="nexus-home__visual" aria-label="Wisdoverse Nexus product surface preview">
      <div class="nexus-console">
        <aside class="nexus-console__rail">
          <div class="nexus-console__brand">
            <img class="nexus-console__mark" :src="withBase('/images/logo.svg')" alt="" />
            <span>Nexus</span>
          </div>
          <nav class="nexus-console__nav" aria-label="Preview navigation">
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
            <span>main branch preview</span>
            <span class="nexus-console__status">Buildable and audited</span>
          </div>
          <h2 class="nexus-console__headline">One repository for the gateway, clients, protocols, docs, and release gates.</h2>
          <div class="nexus-console__grid">
            <div class="nexus-metric">
              <strong>Rust</strong>
              <span>Gateway and shared domain crates</span>
            </div>
            <div class="nexus-metric">
              <strong>WebSocket</strong>
              <span>Realtime room and message paths</span>
            </div>
            <div class="nexus-metric">
              <strong>SDKs</strong>
              <span>TypeScript and Python clients</span>
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
      <p class="nexus-section__kicker">Start with the active paths</p>
      <h2 class="nexus-section__title">Everything important is one decision away.</h2>
      <p class="nexus-section__copy">
        The homepage now routes evaluators directly into the surfaces they need first: setup, runtime architecture, API contracts, deployment, and the license boundary.
      </p>
      <div class="nexus-tiles">
        <article class="nexus-tile">
          <h3>Run it locally</h3>
          <p>Use the current gateway commands and Node workspace checks from a fresh checkout.</p>
          <a :href="withBase('/en/getting-started/quick-start')">Quick Start</a>
        </article>
        <article class="nexus-tile">
          <h3>Understand the system</h3>
          <p>Read the Rust-first architecture, tenant model, provider design, and protocol boundaries.</p>
          <a :href="withBase('/en/architecture')">Architecture</a>
        </article>
        <article class="nexus-tile">
          <h3>Evaluate operations</h3>
          <p>Review deployment assets, monitoring, troubleshooting, tracing, release gates, and security notes.</p>
          <a :href="withBase('/en/operations/deployment')">Operations</a>
        </article>
      </div>
    </div>
  </section>

  <section class="nexus-section nexus-section--soft">
    <div class="nexus-section__inner">
      <p class="nexus-section__kicker">Repository map</p>
      <h2 class="nexus-section__title">A source tree that reads like a product system.</h2>
      <div class="nexus-map">
        <div class="nexus-map__list">
          <div class="nexus-map__row">
            <code>crates/</code>
            <span>Gateway, protocols, AI integration, context, plugins, and collaboration domain crates.</span>
          </div>
          <div class="nexus-map__row">
            <code>apps/</code>
            <span>React + Vite web app and Expo-managed mobile app for product workflows.</span>
          </div>
          <div class="nexus-map__row">
            <code>sdk/</code>
            <span>TypeScript and Python clients for external integration and testable examples.</span>
          </div>
          <div class="nexus-map__row">
            <code>deploy/</code>
            <span>Docker Compose, Helm, Prometheus, and Grafana assets kept with the code.</span>
          </div>
        </div>
        <figure class="nexus-map__image">
          <img :src="withBase('/images/architecture-diagram.svg')" alt="Wisdoverse Nexus architecture diagram" />
        </figure>
      </div>
    </div>
  </section>

  <section class="nexus-band">
    <div class="nexus-band__inner">
      <div>
        <h2>Clear status, clear boundaries.</h2>
        <p>
          Wisdoverse Nexus is pre-1.0 and source-available under the Wisdoverse Nexus Business Source License 1.1. The docs keep evaluation, contribution, and commercial-use expectations explicit.
        </p>
      </div>
      <div class="nexus-band__links">
        <a :href="withBase('/en/license')">License summary <span>Open</span></a>
        <a :href="withBase('/en/roadmap')">Roadmap <span>Open</span></a>
        <a :href="withBase('/en/development/contributing')">Contributing <span>Open</span></a>
        <a href="https://github.com/Wisdoverse/Wisdoverse-Nexus">GitHub repository <span>Open</span></a>
      </div>
    </div>
  </section>
</main>
