# 开发指南

## 环境搭建

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
cargo build --workspace
```

## 本地常用命令

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p nexis-gateway
cargo run -p nexis-cli -- --help
```

## 质量门禁

- 不新增 lint 警告。
- 受影响 crate 的测试必须通过。
- 行为变化必须同步更新文档。

## 分支与 PR

- 分支命名：`feat/*`、`fix/*`、`docs/*`
- 提交规范：Conventional Commits
- PR 需包含改动范围与测试证据
