# 开发指南

本指南帮助你在本地搭建 Nexis 开发环境。

## 开发环境配置

### 1. 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装组件
rustup component add clippy rustfmt

# 安装 cargo-watch (热重载)
cargo install cargo-watch
```

### 2. 克隆项目

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
```

### 3. 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p nexis-gateway
```

### 4. 代码检查

```bash
# Clippy 检查
cargo clippy --workspace --all-targets -- -D warnings

# 格式化代码
cargo fmt --all -- --check
```

## 项目结构

```
Nexis/
├── crates/
│   ├── nexis-core/       # 核心类型和 traits
│   ├── nexis-gateway/    # WebSocket 网关服务
│   ├── nexis-context/    # 上下文管理
│   ├── nexis-protocol/   # 消息协议
│   ├── nexis-plugin/     # 插件系统 (Wasm)
│   └── ...               # 其他 crates
├── docs/                 # VitePress 文档
├── Cargo.toml            # Workspace 配置
└── README.md
```

## 开发工作流

### 热重载开发

```bash
# 监听文件变化自动重新编译
cargo watch -x "run -p nexis-gateway"
```

### 运行特定功能

```bash
# 只启用特定 features
cargo run -p nexis-gateway --features otel
```

## 调试

### 日志级别

```bash
# 设置日志级别
RUST_LOG=debug cargo run -p nexis-gateway
```

### 常见问题

**Q: 编译失败 "linker 'cc' not found"**

安装 C 编译器：
```bash
# Ubuntu/Debian
sudo apt install build-essential

# macOS
xcode-select --install
```

**Q: WebSocket 连接被拒绝**

检查防火墙设置，确保 8080 端口未被占用。

## 下一步

- [架构概述](/zh-CN/architecture/) — 了解系统设计
- [贡献指南](/zh-CN/development/contributing) — 参与项目开发
