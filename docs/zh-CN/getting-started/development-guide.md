# 开发指南

本指南用于搭建与 GitHub CI 模型一致的本地开发环境。

仓库架构约束要求前端采用 FSD、后端采用 DDD，并按云原生微服务原则设计服务和部署。当前 M1 网关仍是单进程、单租户、内存存储的预览模式；这些约束描述开发和演进要求，不表示微服务拆分或持久化已验收。详见[根目录 AGENTS.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/AGENTS.md)和[架构 ADR 008](../../en/architecture/adr/008-fsd-ddd-cloud-native-services.md)。

## 环境要求

- Rust 1.99.0，并安装 `rustfmt` 和 `clippy`（仓库工具链）
- Node.js `24.x`
- pnpm `>=10.30.0`
- Docker 或 Docker Compose，用于测试容器化服务

```bash
rustup toolchain install 1.99.0 --profile minimal --component rustfmt --component clippy
corepack enable
```

仓库的 `rust-toolchain.toml` 会自动选择 Rust 1.99.0，CI 也固定使用此版本。编译器升级需要单独验收。

## 克隆与安装

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

pnpm install --frozen-lockfile --ignore-scripts
cargo check --workspace
```

## 常用命令

### Rust workspace

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo run -p nexis-gateway
cargo run -p nexis-cli -- --help
```

### Web 应用

```bash
pnpm --filter @wisdoverse/nexus-web dev
pnpm --filter @wisdoverse/nexus-web build
```

### Mobile 应用

Mobile 应用由 Expo 管理。依赖升级时优先对齐当前 Expo SDK，而不是直接追 registry latest。

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

### SDK 与文档

```bash
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build
```

## 本地 Docker

根目录 `docker-compose.yml` 用于启动本地 gateway。Compose 挂载数据卷不代表 M1 房间和消息已经持久化：M1 当前使用单进程内存存储，进程重启后数据会丢失。持久化能力计划在 M3 验收。

```bash
docker compose up -d
docker compose ps
curl http://localhost:8080/health
```

`deploy/` 下包含开发基础设施：

```bash
docker compose -f deploy/docker-compose.yml up -d
```

## 仓库地图

| 路径 | 说明 |
| --- | --- |
| `crates/nexis-protocol` | 协议和身份基础类型 |
| `crates/nexis-gateway` | HTTP/WebSocket 网关 |
| `crates/nexis-ai` | AI provider 与 MCP 集成 |
| `crates/nexis-context` | 上下文管理 |
| `crates/nexis-plugin` | 插件运行模型 |
| `crates/nexis-meeting`, `nexis-doc`, `nexis-task`, `nexis-calendar` | 协作域模块 |
| `apps/web` | React + Vite 应用 |
| `apps/mobile` | Expo / React Native 应用 |
| `sdk/typescript`, `sdk/python` | 客户端 SDK |
| `docs` | VitePress 文档站 |
| `deploy` | Compose、Helm、Prometheus、Grafana 资源 |

## 常见问题

### `cc` linker not found

安装系统 C 工具链：

```bash
# Debian / Ubuntu
sudo apt install build-essential

# macOS
xcode-select --install
```

### Expo 依赖不匹配

运行兼容检查，并安装 Expo 推荐版本：

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
```

### 8080 端口被占用

停止冲突进程，或在本地 compose 配置中覆盖端口后再启动 gateway。

## 下一步

- [贡献指南](../development/contributing.md)
- [架构概览](../architecture/index.md)
- [路线图](../roadmap.md)
