# Wisdoverse Nexus

Wisdoverse Nexus 是源码可用的 AI 原生协作基础设施：一个 Rust 优先的
gateway、共享协议 crate、Web/Mobile 应用、SDK 和部署资源，用于实时房间、
身份、WebSocket 消息和 AI 辅助工作流。

公开项目文档发布在 <https://wisdoverse.github.io/Wisdoverse-Nexus/>。

> [!WARNING]
> Wisdoverse Nexus 是 pre-1.0 工程预览版，适用于可信开发、评估和非生产环境。
> 在第一个稳定版本发布前，API、SDK、部署清单和 crate 边界都可能变化。

## 运行 Wisdoverse Nexus

### 环境要求

Wisdoverse Nexus 适合在全新 checkout 中复现 Rust、Node、Expo、Docker 和文档
检查。

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker 或 Docker Compose，用于容器化本地运行

### 选项 1. 从 spec 继续

让 coding agent 检查当前仓库，并从项目 spec 继续：

> Review Wisdoverse Nexus according to the following spec:
> <https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SPEC.md>

### 选项 2. 使用参考实现

从源码运行 Rust gateway：

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

cargo check --workspace
cargo run -p nexis-gateway
```

在另一个终端验证 gateway：

```bash
curl http://localhost:8080/health
# OK
```

也可以运行本地容器栈：

```bash
docker compose up -d
curl http://localhost:8080/health
```

### 选项 3. 验证 workspace

根据变更范围运行对应检查。对于较大范围的仓库变更：

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

---

## 文档

- Spec: [SPEC.md](SPEC.md)
- 公开文档：<https://wisdoverse.github.io/Wisdoverse-Nexus/>
- 快速开始：[docs/zh-CN/getting-started/quick-start.md](docs/zh-CN/getting-started/quick-start.md)
- 开发指南：[docs/zh-CN/getting-started/development-guide.md](docs/zh-CN/getting-started/development-guide.md)
- 架构：[docs/zh-CN/architecture/index.md](docs/zh-CN/architecture/index.md)
- 贡献：[CONTRIBUTING.md](CONTRIBUTING.md)
- 安全：[SECURITY.md](SECURITY.md)

## 许可证

Wisdoverse Nexus 使用 [Wisdoverse Nexus Business Source License
1.1](LICENSE)。该许可证允许在无需商业授权的情况下用于个人学习、研究、教育、
开发测试、内部评估和非生产环境。

商业生产环境使用、SaaS 或托管服务、managed service、转售、二次授权，以及用于
构建竞争性产品，都需要 Wisdoverse 的单独商业授权。

每个公开发布版本在 Wisdoverse 首次公开发布满 4 年后，自动转换为 Apache License
2.0。
