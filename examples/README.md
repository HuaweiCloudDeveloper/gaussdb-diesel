# Diesel-GaussDB 示例项目

这个目录包含了 diesel-gaussdb 的完整示例项目，展示了从基础使用到高级功能的各种场景。

## 📁 目录结构

```
examples/
├── README.md                    # 本文件
├── 01-basic-usage/             # 基础使用示例
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── main.rs             # 基础 CRUD 操作
│       ├── models.rs           # 数据模型定义
│       └── schema.rs           # 数据库表结构
├── 02-advanced-queries/        # 高级查询示例
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── main.rs             # 复杂查询演示
│       ├── window_functions.rs # 窗口函数示例
│       ├── cte_queries.rs      # CTE 查询示例
│       └── subqueries.rs       # 子查询示例
├── 03-web-application/         # Web 应用集成
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── main.rs             # Web 服务器
│       ├── handlers.rs         # HTTP 处理器
│       ├── database.rs         # 数据库连接池
│       └── models.rs           # API 模型
├── 04-real-world-blog/         # 真实博客系统
│   ├── Cargo.toml
│   ├── README.md
│   ├── migrations/             # 数据库迁移
│   └── src/
│       ├── main.rs             # 主程序
│       ├── models/             # 数据模型
│       ├── handlers/           # 业务逻辑
│       └── schema.rs           # 数据库结构
└── 05-performance-testing/     # 性能测试
    ├── Cargo.toml
    ├── README.md
    └── src/
        ├── main.rs             # 性能测试主程序
        ├── benchmarks.rs       # 基准测试
        └── load_testing.rs     # 负载测试
```

## 🎯 示例说明

### 1. 基础使用示例 (01-basic-usage)
- **目标用户**: 初学者
- **内容**: 连接数据库、基础 CRUD 操作、简单查询
- **学习时间**: 30 分钟

### 2. 高级查询示例 (02-advanced-queries)
- **目标用户**: 中级开发者
- **内容**: 窗口函数、CTE、子查询、复杂 JOIN
- **学习时间**: 1 小时

### 3. Web 应用集成 (03-web-application)
- **目标用户**: Web 开发者
- **内容**: Axum + Diesel-GaussDB 的完整 REST API
- **学习时间**: 2 小时

### 4. 真实博客系统 (04-real-world-blog)
- **目标用户**: 全栈开发者
- **内容**: 完整的博客系统，包含用户、文章、评论等功能
- **学习时间**: 4 小时

### 5. 性能测试 (05-performance-testing)
- **目标用户**: 性能工程师
- **内容**: 连接池优化、查询性能测试、负载测试
- **学习时间**: 1 小时

## 🚀 快速开始

### 环境准备

1. **安装 Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **启动 GaussDB/OpenGauss**
   ```bash
   docker run --name opengauss \
     -e GS_PASSWORD=Gaussdb@123 \
     -p 5432:5432 \
     -d opengauss/opengauss:7.0.0-RC1.B023
   ```

3. **设置环境变量**
   ```bash
   export GAUSSDB_URL="host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres"
   ```

### 运行示例

```bash
# 运行基础示例
cd examples/01-basic-usage
cargo run

# 运行高级查询示例
cd examples/02-advanced-queries
cargo run

# 运行 Web 应用示例
cd examples/03-web-application
cargo run
# 然后访问 http://localhost:3000

# 运行真实博客系统
cd examples/04-real-world-blog
cargo run
# 然后访问 http://localhost:8080

# 运行性能测试
cd examples/05-performance-testing
cargo run --release
```

## 📚 学习路径

### 初学者路径
1. 阅读项目 [README.md](../README.md)
2. 运行 [基础使用示例](01-basic-usage/)
3. 学习 [高级查询示例](02-advanced-queries/)

### 进阶路径
1. 完成初学者路径
2. 学习 [Web 应用集成](03-web-application/)
3. 研究 [真实博客系统](04-real-world-blog/)

### 专家路径
1. 完成进阶路径
2. 分析 [性能测试](05-performance-testing/)
3. 贡献代码到主项目

## 🔧 故障排除

### 常见问题

1. **连接失败**
   - 检查 GaussDB/OpenGauss 是否运行
   - 验证连接字符串是否正确
   - 确认防火墙设置

2. **编译错误**
   - 确保 Rust 版本 >= 1.70
   - 检查依赖版本兼容性
   - 清理并重新构建：`cargo clean && cargo build`

3. **运行时错误**
   - 检查数据库权限
   - 验证表结构是否正确
   - 查看详细错误日志

### 获取帮助

- 查看 [项目文档](../README.md)
- 提交 [GitHub Issue](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues)
- 访问 [华为云技术论坛](https://bbs.huaweicloud.com/forum/forum-1131-1.html)

## 🤝 贡献示例

如果您有好的示例想要分享：

1. Fork 本仓库
2. 在 `examples/` 目录下创建新的示例
3. 遵循现有的目录结构和命名规范
4. 添加详细的 README.md 和注释
5. 提交 Pull Request

### 示例要求

- 代码清晰易懂，有详细注释
- 包含完整的 README.md 说明
- 提供运行步骤和预期输出
- 遵循 Rust 代码规范
- 包含适当的错误处理

---

**🎯 通过这些示例，您将掌握 diesel-gaussdb 的所有核心功能和最佳实践！**
