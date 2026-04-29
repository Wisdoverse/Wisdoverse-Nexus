# 快速开始

本页使用与当前 GitHub 源码可用仓库一致的命令验证本地 Wisdoverse Nexus checkout。

## 前置要求

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- 如需容器化运行 gateway，安装 Docker 或 Docker Compose

## 克隆仓库

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus
```

## 方式一：从源码运行

```bash
cargo check --workspace
cargo run -p nexis-gateway
```

另开一个终端验证：

```bash
curl http://localhost:8080/health
# OK
```

gateway 同时提供：

- `GET /openapi.json`：OpenAPI 文档
- `GET /docs`：Swagger UI
- `GET /metrics`：Prometheus metrics

## 方式二：使用 Docker Compose

```bash
docker compose up -d
docker compose ps
curl http://localhost:8080/health
```

停止服务：

```bash
docker compose down
```

## 验证 Node Workspace

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## 本地构建文档

```bash
pnpm --dir docs docs:build
pnpm --dir docs docs:dev
```

开发服务器会输出本地 VitePress 访问地址。

## 下一步

- [开发指南](development-guide.md)
- [架构概览](../architecture/index.md)
- [贡献指南](../development/contributing.md)
