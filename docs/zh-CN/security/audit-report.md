# 安全状态

本页总结源码可用仓库的公开安全状态。它不是第三方审计报告、认证报告或合规证明。

## 范围

覆盖内容：

- Rust workspace 依赖和编译检查
- Node workspace 依赖检查
- GitHub Actions 安全扫描
- 仓库中记录的 gateway 安全控制
- 公开漏洞报告流程

不覆盖：

- 第三方渗透测试
- SOC 2、ISO 27001、HIPAA 或 GDPR 认证
- 仓库之外的生产环境控制
- 下游用户自己的云基础设施

## 仓库控制项

| 控制项 | 状态 | 证据 |
| --- | --- | --- |
| 私有漏洞报告 | 已启用 | `SECURITY.md` 和 GitHub Security Advisories |
| Rust lint 门禁 | 已启用 | `cargo clippy --workspace --all-targets -- -D warnings` |
| Rust 依赖审计 | CI 中启用 | `cargo audit` |
| Node 依赖审计 | CI 中启用 | `pnpm audit` 与 package-local `npm audit` |
| Secret scanning | CI 中启用 | TruffleHog OSS workflow |
| 容器扫描 | CI 中启用 | Trivy workflow |
| Dependabot | 已启用 | Cargo、npm、Docker、GitHub Actions |

## 本地验证

```bash
cargo check --workspace
pnpm install --frozen-lockfile --ignore-scripts
pnpm audit --audit-level moderate
npm --prefix apps/mobile audit --audit-level=moderate
npm --prefix docs audit --audit-level=moderate
npm --prefix apps/web/e2e audit --audit-level=moderate
```

Rust advisory 检查需要安装 `cargo-audit`：

```bash
cargo audit
```

## 应用安全基线

安全相关行为记录在：

- [安全架构概览](/en/architecture/security/overview)
- [Baseline profile](/en/architecture/security/baseline-profile)
- [Enterprise profile](/en/architecture/security/enterprise-profile)
- [Key management](/en/architecture/security/key-management)

生产使用者需要为自己的环境配置 TLS 终止、密钥管理、CORS origins、rate limits、
数据库访问、日志保留和网络控制。

## 漏洞报告

不要通过公开 GitHub issue 报告安全漏洞。请使用根目录 [SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md)
中记录的 GitHub Security Advisory 私有渠道。

报告建议包含：

- 受影响组件和版本或 commit SHA
- 复现步骤
- 影响说明
- 在安全前提下可分享的日志、请求样例或 proof of concept
- 可选的修复建议

## 披露策略

维护者会私下分流安全报告，准备修复，发布 release 或补丁说明，并在适合协调披露时
发布 advisory 细节。
