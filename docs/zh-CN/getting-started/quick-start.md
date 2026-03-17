# 快速开始

5 分钟内启动 Nexis。

## 前置要求

- **Rust** 1.75+ (推荐使用 [rustup](https://rustup.rs/))
- **Docker** (可选，用于容器化部署)
- **Git**

## 方式一：从源码构建

```bash
# 克隆仓库
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis

# 构建项目
cargo build --release

# 运行网关服务
cargo run -p nexis-gateway --release
```

## 方式二：使用 Docker

```bash
# 克隆仓库
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f
```

服务启动后访问：
- **网关服务**: http://localhost:8080
- **指标端点**: http://localhost:9090/metrics

## 验证安装

```bash
# 健康检查
curl http://localhost:8080/health

# 预期输出
{"status":"healthy","version":"0.1.0"}
```

## 下一步

- [开发指南](/zh-CN/getting-started/development-guide) — 本地开发环境配置
- [架构概述](/zh-CN/architecture/) — 了解 Nexis 架构设计
- [API 参考](/zh-CN/api/metrics) — 查看可用 API
