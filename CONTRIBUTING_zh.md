# 贡献指南

感谢您对 diesel-gaussdb 项目的关注！我们欢迎各种形式的贡献，包括代码、文档、测试、问题报告和功能建议。

[English](CONTRIBUTING.md) | **中文**

## 🤝 贡献方式

### 代码贡献
- 修复 bug
- 添加新功能
- 性能优化
- 代码重构

### 文档贡献
- 改进现有文档
- 添加新的示例
- 翻译文档
- 修正错误

### 测试贡献
- 编写单元测试
- 添加集成测试
- 性能测试
- 兼容性测试

### 社区贡献
- 回答问题
- 参与讨论
- 分享经验
- 推广项目

## 🚀 开始贡献

### 1. 环境准备

```bash
# 克隆仓库
git clone https://github.com/HuaweiCloudDeveloper/gaussdb-diesel.git
cd gaussdb-diesel

# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装必要工具
cargo install diesel_cli --no-default-features --features postgres
cargo install cargo-audit
cargo install cargo-tarpaulin  # 代码覆盖率工具
```

### 2. 设置开发环境

```bash
# 启动测试数据库
docker-compose -f docker-compose.test.yml up -d

# 等待数据库启动
sleep 10

# 运行测试确保环境正常
cargo test
```

### 3. 创建功能分支

```bash
# 从主分支创建新分支
git checkout -b feature/your-feature-name

# 或者修复 bug
git checkout -b fix/issue-number-description
```

## 📋 开发流程

### 1. 代码开发

#### 编码规范
- 遵循 Rust 官方编码规范
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 进行代码检查
- 添加适当的注释和文档

```bash
# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy -- -D warnings

# 构建项目
cargo build
```

#### 提交规范
使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<类型>[可选的作用域]: <描述>

[可选的正文]

[可选的脚注]
```

类型包括：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

示例：
```
feat(connection): 添加连接池支持

- 实现 R2D2 连接池集成
- 添加连接池配置选项
- 更新相关文档和示例

Closes #123
```

### 2. 测试

#### 运行测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_connection

# 运行集成测试
cargo test --test diesel_integration

# 生成代码覆盖率报告
cargo tarpaulin --out Html
```

#### 编写测试
- 为新功能编写单元测试
- 添加集成测试验证端到端功能
- 确保测试覆盖率不低于 80%
- 测试边界条件和错误情况

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_establishment() {
        let database_url = "host=localhost port=5432 user=test password=test dbname=test";
        let result = GaussDbConnection::establish(database_url);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_connection_url() {
        let result = GaussDbConnection::establish("invalid_url");
        assert!(result.is_err());
    }
}
```

### 3. 文档

#### 代码文档
- 为公共 API 添加文档注释
- 使用 `cargo doc` 生成文档
- 包含使用示例

```rust
/// 建立到 GaussDB 数据库的连接
/// 
/// # 参数
/// 
/// * `database_url` - 数据库连接字符串
/// 
/// # 示例
/// 
/// ```rust
/// use diesel_gaussdb::GaussDbConnection;
/// 
/// let database_url = "host=localhost port=5432 user=gaussdb password=pass dbname=mydb";
/// let connection = GaussDbConnection::establish(&database_url)?;
/// ```
/// 
/// # 错误
/// 
/// 如果连接失败，返回 `ConnectionError`
pub fn establish(database_url: &str) -> ConnectionResult<Self> {
    // 实现代码
}
```

#### 用户文档
- 更新相关的用户指南
- 添加新功能的使用示例
- 更新 API 参考文档
- 保持中英文文档同步

## 🔍 代码审查

### 提交 Pull Request

1. **确保代码质量**
   ```bash
   # 运行完整的检查流程
   ./scripts/validate-workflows.sh
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

2. **创建 Pull Request**
   - 使用清晰的标题和描述
   - 引用相关的 issue
   - 添加测试结果截图（如适用）
   - 标记需要审查的特定部分

3. **PR 模板**
   ```markdown
   ## 变更描述
   简要描述此 PR 的变更内容
   
   ## 变更类型
   - [ ] Bug 修复
   - [ ] 新功能
   - [ ] 文档更新
   - [ ] 性能优化
   - [ ] 代码重构
   
   ## 测试
   - [ ] 添加了新的测试
   - [ ] 所有测试通过
   - [ ] 手动测试完成
   
   ## 检查清单
   - [ ] 代码遵循项目规范
   - [ ] 自我审查完成
   - [ ] 添加了必要的注释
   - [ ] 更新了相关文档
   
   ## 相关 Issue
   Closes #issue_number
   ```

### 审查过程

1. **自动检查**
   - CI/CD 流水线自动运行
   - 代码格式和质量检查
   - 测试套件执行
   - 安全扫描

2. **人工审查**
   - 代码逻辑审查
   - 架构设计讨论
   - 性能影响评估
   - 文档完整性检查

3. **反馈处理**
   - 及时回应审查意见
   - 进行必要的修改
   - 解释设计决策
   - 更新 PR 描述

## 🐛 问题报告

### 报告 Bug

使用 [Bug 报告模板](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues/new?template=bug_report.md)：

```markdown
## Bug 描述
清晰简洁地描述 bug

## 重现步骤
1. 执行 '...'
2. 点击 '....'
3. 滚动到 '....'
4. 看到错误

## 期望行为
描述您期望发生的行为

## 实际行为
描述实际发生的行为

## 环境信息
- OS: [例如 macOS 12.0]
- Rust 版本: [例如 1.70.0]
- diesel-gaussdb 版本: [例如 1.0.0]
- GaussDB 版本: [例如 505.2.0]

## 附加信息
添加任何其他相关信息
```

### 功能请求

使用 [功能请求模板](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues/new?template=feature_request.md)：

```markdown
## 功能描述
清晰简洁地描述您想要的功能

## 问题描述
描述当前的问题或限制

## 建议的解决方案
描述您希望如何实现这个功能

## 替代方案
描述您考虑过的其他解决方案

## 附加信息
添加任何其他相关信息或截图
```

## 📚 文档贡献

### 文档类型
- **用户指南**: 面向最终用户的使用说明
- **API 文档**: 详细的 API 参考
- **示例代码**: 实际可运行的示例
- **技术参考**: 深入的技术细节

### 文档标准
- 遵循 [文档样式指南](docs/STYLE_GUIDE.md)
- 保持中英文版本同步
- 包含可运行的代码示例
- 使用清晰的结构和格式

### 翻译贡献
- 翻译现有英文文档到中文
- 翻译中文文档到英文
- 保持技术术语的一致性
- 考虑文化差异和表达习惯

## 🏆 认可贡献者

我们重视每一个贡献，并通过以下方式认可贡献者：

### 贡献者列表
- 在 README 中列出主要贡献者
- 在发布说明中感谢贡献者
- 在项目网站上展示贡献者

### 贡献统计
- 代码提交统计
- 文档贡献统计
- 问题解决统计
- 社区参与统计

## 📞 获取帮助

如果您在贡献过程中遇到问题：

### 技术问题
- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues) - 技术问题和 bug 报告
- [GitHub Discussions](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/discussions) - 一般讨论和问题

### 社区支持
- [华为云 GaussDB 技术支持论坛](https://bbs.huaweicloud.com/forum/forum-1131-1.html) - 官方技术支持
- [Rust 中文社区](https://rustcc.cn/) - Rust 语言相关问题

### 直接联系
- 项目维护者邮箱
- 华为云开源社区联系方式

## 📄 许可证

通过贡献代码，您同意您的贡献将在与项目相同的许可证下发布（MIT 或 Apache-2.0）。

## 🙏 致谢

感谢所有为 diesel-gaussdb 项目做出贡献的开发者、文档作者、测试人员和社区成员！

您的贡献让这个项目变得更好，让更多的开发者能够受益。

---

**让我们一起构建更好的 diesel-gaussdb！** 🚀
