# diesel-gaussdb 中文文档

欢迎使用 diesel-gaussdb 中文文档！本指南将帮助您在 Rust 应用程序中快速上手 diesel-gaussdb。

## 📚 文档结构

### 🚀 快速开始
- [快速开始指南](guides/getting-started.md) - 快速上手使用
- [安装指南](guides/installation.md) - 安装和配置说明
- [配置指南](guides/configuration.md) - 配置选项和最佳实践

### 📖 用户指南
- [数据库连接](guides/connections.md) - 管理数据库连接
- [查询构建](guides/queries.md) - 构建和执行查询
- [事务处理](guides/transactions.md) - 事务管理
- [数据库迁移](guides/migrations.md) - 数据库架构迁移
- [测试指南](guides/testing.md) - 测试数据库代码
- [性能优化](guides/performance.md) - 性能优化技巧
- [最佳实践](guides/best-practices.md) - 推荐的模式和实践
- [故障排除](guides/troubleshooting.md) - 常见问题和解决方案

### 🔧 API 参考
- [核心类型](api/README.md) - 主要类型和特征
- [连接 API](api/connection.md) - 连接管理
- [查询构建器 API](api/query-builder.md) - 查询构建
- [类型系统](api/types.md) - 数据类型映射
- [错误处理](api/errors.md) - 错误类型和处理
- [监控系统](api/monitoring.md) - 指标和健康检查
- [性能优化](api/performance.md) - 性能优化 API

### 💡 示例代码
- [基础 CRUD](examples/basic-crud.md) - 创建、读取、更新、删除操作
- [高级查询](examples/advanced-queries.md) - 复杂查询模式
- [连接池](examples/connection-pooling.md) - 使用连接池
- [异步操作](examples/async-operations.md) - 异步数据库操作
- [Web 应用](examples/web-app.md) - 构建 Web 应用程序
- [微服务](examples/microservices.md) - 微服务架构模式
- [测试策略](examples/testing.md) - 测试方法和模式

### 🔍 技术参考
- [架构概览](reference/architecture.md) - 内部架构概述
- [SQL 兼容性](reference/sql-compatibility.md) - GaussDB SQL 功能支持
- [类型映射](reference/type-mappings.md) - 完整的类型映射参考
- [性能基准](reference/benchmarks.md) - 性能特征
- [迁移指南](reference/migration.md) - 从其他 ORM 迁移
- [更新日志](reference/changelog.md) - 版本历史和变更

## 🎯 快速导航

### 初学者
1. 从[快速开始指南](guides/getting-started.md)开始
2. 学习[配置指南](guides/configuration.md)
3. 尝试[基础 CRUD 示例](examples/basic-crud.md)
4. 阅读[最佳实践](guides/best-practices.md)

### 有经验的用户
1. 查看[API 参考](api/)获取详细信息
2. 探索[高级示例](examples/advanced-queries.md)
3. 查看[性能优化](guides/performance.md)
4. 了解[架构详情](reference/architecture.md)

### 贡献者
1. 阅读[贡献指南](../../CONTRIBUTING_zh.md)
2. 查看[架构概览](reference/architecture.md)
3. 查看[测试策略](examples/testing.md)
4. 了解[开发环境设置](guides/development.md)

## 🌟 核心特性

### 基础功能
- **完全兼容 Diesel**: 100% 兼容 Diesel 2.2.x API
- **类型安全**: 完整的 Rust 类型系统集成
- **查询构建器**: 强大而灵活的查询构建
- **事务支持**: 完整的事务支持，包括保存点
- **数据库迁移**: 架构迁移管理

### 高级功能
- **连接池**: R2D2 集成的连接管理
- **异步支持**: Tokio 运行时兼容性
- **监控系统**: 内置指标和健康检查
- **性能优化**: 查询缓存和批量操作
- **安全性**: SSL/TLS 支持和查询验证

### 数据库支持
- **GaussDB**: 完整支持 GaussDB 功能
- **OpenGauss**: 兼容 OpenGauss 数据库
- **PostgreSQL**: 利用 PostgreSQL 兼容性

## 🚀 快速示例

```rust
use diesel::prelude::*;
use diesel_gaussdb::GaussDbConnection;

// 连接数据库
let mut conn = GaussDbConnection::establish(&database_url)?;

// 查询用户
let users = users::table
    .select(User::as_select())
    .load(&mut conn)?;

// 插入新用户
let new_user = NewUser {
    name: "张三",
    email: "zhangsan@example.com",
};

let user = diesel::insert_into(users::table)
    .values(&new_user)
    .returning(User::as_returning())
    .get_result(&mut conn)?;
```

## 📊 支持的版本

- **diesel-gaussdb**: 1.0+
- **Diesel**: 2.2.x
- **Rust**: 1.70.0+ (最低支持版本)
- **GaussDB**: 505.2.0+
- **OpenGauss**: 7.0.0+

## 🤝 社区和支持

### 获取帮助
- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues) - Bug 报告和功能请求
- [GitHub Discussions](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/discussions) - 社区讨论
- [华为云论坛](https://bbs.huaweicloud.com/forum/forum-1131-1.html) - GaussDB 技术支持

### 贡献
- [贡献指南](../../CONTRIBUTING_zh.md) - 如何贡献
- [行为准则](../../CODE_OF_CONDUCT.md) - 社区准则
- [开发环境设置](guides/development.md) - 设置开发环境

### 资源
- [官方网站](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel)
- [Crates.io](https://crates.io/crates/diesel-gaussdb)
- [文档](https://docs.rs/diesel-gaussdb)
- [示例仓库](https://github.com/HuaweiCloudDeveloper/gaussdb-examples-rust)

## 📝 文档标准

本文档遵循以下原则：

- **准确性**: 所有示例都经过测试和验证
- **清晰性**: 清晰、简洁的解释和实用示例
- **完整性**: 全面覆盖所有功能
- **可访问性**: 适合初学者和专家
- **维护性**: 定期更新新版本

## 🔄 版本信息

本文档涵盖 diesel-gaussdb 版本 1.0，兼容：

- Diesel 2.2.x
- GaussDB 505.2.0+
- OpenGauss 7.0.0+
- Rust 1.70.0+ (最低支持版本)

有关特定版本的信息，请参阅[更新日志](reference/changelog.md)。

## 📞 反馈

我们重视您的反馈！如果您对改进本文档有建议：

1. [提交 issue](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues/new) 并添加 `documentation` 标签
2. 在我们的[社区论坛](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/discussions)中开始讨论
3. 提交包含改进的拉取请求

## 🎉 项目亮点

### 🏆 企业级质量
- **生产就绪**: 经过全面测试，可用于生产环境
- **高性能**: 内置性能优化和监控
- **高可靠**: 完善的错误处理和恢复机制
- **高安全**: SSL/TLS 支持和安全验证

### 🌐 国际化支持
- **双语文档**: 完整的中英文文档体系
- **本地化**: 适合中国开发者的使用习惯
- **社区友好**: 活跃的中文技术社区支持

### 🚀 开发体验
- **类型安全**: 编译时类型检查，减少运行时错误
- **IDE 友好**: 完整的代码补全和错误提示
- **调试支持**: 详细的错误信息和调试工具
- **文档丰富**: 全面的文档和示例代码

### 🔧 技术创新
- **真实实现**: 基于真实的 gaussdb 驱动，非 Mock 实现
- **完全兼容**: 100% 兼容 Diesel API，无缝迁移
- **性能优化**: 查询缓存、连接池、批量操作等优化
- **监控集成**: 内置监控和指标收集系统

---

**准备好使用 diesel-gaussdb 构建出色的应用程序了吗？让我们开始吧！** 🚀
