# 贡献指南

感谢你对 Nexis 的关注！本文档帮助你参与项目开发。

## 行为准则

- 尊重所有贡献者
- 保持建设性讨论
- 接受建设性批评

## 如何贡献

### 报告 Bug

1. 搜索 [现有 Issues](https://github.com/gbrothersgroup/Nexis/issues) 确认未重复
2. 创建新 Issue，包含：
   - 复现步骤
   - 预期行为
   - 实际行为
   - 环境信息 (OS, Rust 版本)

### 提交功能建议

1. 先在 Discussions 中讨论你的想法
2. 达成共识后创建 Feature Request Issue
3. 等待维护者分配或自行实现

### 提交代码

```bash
# 1. Fork 并克隆
git clone https://github.com/YOUR_USERNAME/Nexis.git
cd Nexis

# 2. 创建分支
git checkout -b feature/your-feature

# 3. 开发和测试
cargo test --workspace
cargo clippy -- -D warnings
cargo fmt --all

# 4. 提交
git commit -m "feat: 添加某某功能"
git push origin feature/your-feature

# 5. 创建 Pull Request
```

## 代码规范

### 提交消息格式

```
<type>(<scope>): <subject>

[optional body]
```

**类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档
- `style`: 格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

### 代码风格

```bash
# 格式化
cargo fmt --all

# Clippy 检查
cargo clippy --workspace --all-targets -- -D warnings
```

## 开发设置

参见 [开发指南](/zh-CN/getting-started/development-guide)。

## 许可证

贡献的代码将采用 Apache-2.0 许可证。
