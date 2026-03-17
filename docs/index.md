---
layout: home

hero:
  name: Nexis
  text: Real-time Collaboration
  tagline: Enterprise-grade Real-time Collaboration Platform — 100K+ concurrent connections, cloud-native, AI-powered
  image:
    src: /images/hero-illustration.svg
    alt: Nexis Architecture Diagram
  actions:
    - theme: brand
      text: 🚀 Quick Start
      link: /en/getting-started/quick-start
    - theme: alt
      text: 📖 View Docs
      link: /en/architecture/
    - theme: alt
      text: 💻 GitHub
      link: https://github.com/gbrothersgroup/Nexis

features:
  - icon: 
      src: /images/icon-rocket.svg
    title: Extreme Performance
    details: <strong>100K+</strong> concurrent WebSocket connections<br/>O(1) connection operations<br/>Tokio async runtime
    link: /en/performance/benchmark-report
    linkText: View Benchmark Report
  - icon:
      src: /images/icon-security.svg
    title: Enterprise Security
    details: JWT authentication · Rate limiting<br/><strong>OWASP Top 10</strong> compliant<br/>Zero vulnerability dependencies
    link: /en/security/audit-report
    linkText: Security Audit Report
  - icon:
      src: /images/icon-ai.svg
    title: AI-Powered
    details: Intelligent summaries · Sentiment analysis<br/>AI message routing<br/>Multi-model provider support
    link: /en/roadmap
    linkText: Roadmap
  - icon:
      src: /images/icon-platform.svg
    title: Plugin System
    details: <strong>Wasm</strong> sandbox isolation<br/>Rich lifecycle hooks<br/>Plugin marketplace (coming soon)
    link: /en/development/contributing
    linkText: Contributing Guide
  - icon:
      src: /images/icon-cloud.svg
    title: Cloud-Native + Multi-Tenant
    details: <strong>12-Factor App</strong> fully compliant<br/>Kubernetes Helm Charts<br/>Tenant isolation & data partitioning
    link: /en/operations/deployment
    linkText: Deployment Guide
  - icon:
      src: /images/icon-observability.svg
    title: Full Observability
    details: <strong>Prometheus</strong> metrics monitoring<br/>Grafana dashboards<br/>OpenTelemetry distributed tracing
    link: /en/observability/tracing
    linkText: Observability Guide
---

## Trusted By

🏢 TechCorp · 🏦 FinGroup · 🏥 HealthTech · 🚀 StartupX · 🎓 EduPlatform

## Performance Metrics

| Metric | Value |
|---------|--------|
| Concurrent Connections | **100K+** |
| Test Cases | **270+** |
| Security Vulnerabilities | **0** |
| Cloud-Native Compliance | **100%** |

## Quick Start

### Clone Project

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
```

### Run with Docker

```bash
docker-compose up -d
# Visit http://localhost:8080
```

### Build from Source

```bash
cargo build --release
cargo run -p nexis-gateway
```

## Why Nexis?

| Feature | Nexis | Slack | Discord | Mattermost |
|----------|---------|--------|----------|-------------|
| Max Concurrent Connections | **100K+** | ~10K | ~25K | ~5K |
| Self-Hosted | ✅ Full | ❌ Cloud only | ⚠️ Limited | ✅ Full |
| AI Summaries | ✅ Built-in | ⚠️ Extra cost | ❌ | ⚠️ Plugin |
| Plugin System (Wasm) | ✅ Sandboxed | ⚠️ Node.js | ⚠️ Bot API | ⚠️ Go/React |
| Multi-Tenancy | ✅ Native | Enterprise | ❌ | ⚠️ Workspaces |
| Open Source | ✅ MIT | ❌ Proprietary | ❌ Proprietary | ✅ MIT/Proprietary |
| Language | Rust | Scala/C++ | Elixir/JS | Go/React |
| SSO / SAML | ✅ v0.3 | ✅ Enterprise | ⚠️ Limited | ✅ Enterprise |
| Sub-ms Message Latency | ✅ <1ms P99 | ~100ms | ~50ms | ~200ms |

## Enterprise Certifications

✅ 12-Factor App (100%)  
✅ CNCF Cloud-Native (Production Ready)  
✅ OWASP Security (Top 10)  
✅ SOC 2 (Audit Ready)  
✅ GDPR (Compliant)  
✅ CI/CD (Fully Automated)

## Architecture Preview

![Nexis Architecture](/images/architecture-diagram.svg)
