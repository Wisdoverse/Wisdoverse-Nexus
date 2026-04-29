# 贡献指南

Wisdoverse Nexus 按公开 GitHub 项目方式协作：小 PR、可复现命令、准确文档、
清晰 issue 关联，以及安全漏洞私有披露。

仓库根目录的 [CONTRIBUTING.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/CONTRIBUTING.md) 是贡献规则的主文件。

## 开发要求

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker 或 Docker Compose，用于本地服务

## 首次设置

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

pnpm install --frozen-lockfile --ignore-scripts
cargo check --workspace
```

运行网关：

```bash
cargo run -p nexis-gateway
```

运行 Web 应用：

```bash
pnpm --filter @wisdoverse/nexus-web dev
```

运行移动端检查：

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## 项目结构

| 路径 | 说明 |
| --- | --- |
| `crates/nexis-protocol` | 身份、消息、权限和租户协议基础 |
| `crates/nexis-gateway` | HTTP/WebSocket 网关和请求处理 |
| `crates/nexis-ai` | AI provider 与 MCP 集成表面 |
| `crates/nexis-context` | 上下文窗口与摘要基础能力 |
| `crates/nexis-plugin` | 插件执行模型 |
| `crates/nexis-meeting`, `nexis-doc`, `nexis-task`, `nexis-calendar` | 协作域 crate |
| `apps/web` | React + Vite Web 应用 |
| `apps/mobile` | Expo / React Native 移动应用 |
| `sdk/typescript`, `sdk/python` | 客户端 SDK |
| `docs` | VitePress 文档站 |
| `deploy` | 本地和部署清单 |

## 本地质量门禁

小改动运行相关检查；跨模块改动建议运行完整门禁：

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

## Pull Request 标准

提交 PR 前请确认：

- PR 范围聚焦，并说明用户影响。
- 关联相关 issue。
- 行为变化有测试或验证。
- 命令、配置、公开 API 或部署行为变化时同步更新文档。
- 在 PR 描述中写明验证命令。
- 明确标注 breaking changes。

CI 必须通过后才能合并。维护者可能会要求拆分过大的 PR。

## 文档标准

文档必须描述当前仓库事实，而不是未来计划。

- 不写未经验证的 benchmark、认证、客户或兼容性声明。
- 首次运行流程变化时，同步更新英文和中文入口。
- 公开文档站使用 <https://wisdoverse.github.io/Wisdoverse-Nexus/>。
- 仓库链接统一使用 `https://github.com/Wisdoverse/Wisdoverse-Nexus`。
- 运行时名称必须保持准确，例如 `nexis-gateway`、`NEXIS_*`、crate 名和 Docker service 名。

## 安全报告

不要通过公开 issue 或 PR 报告漏洞。请使用 [SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md)
中的 GitHub Security Advisory 私有流程。
