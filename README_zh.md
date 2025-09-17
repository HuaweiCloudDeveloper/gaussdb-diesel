# diesel-gaussdb

[![Crates.io](https://img.shields.io/crates/v/diesel-gaussdb.svg)](https://crates.io/crates/diesel-gaussdb)
[![Documentation](https://docs.rs/diesel-gaussdb/badge.svg)](https://docs.rs/diesel-gaussdb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/workflows/CI/badge.svg)](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/actions)

[English](README.md) | **中文**

为 [Diesel ORM](https://diesel.rs/) 提供的 GaussDB 数据库后端实现。

## 🚀 特性

- **完整的 Diesel 兼容性**: 100% 兼容 Diesel 2.2.x API
- **真实数据库驱动**: 基于 [gaussdb](https://crates.io/crates/gaussdb) crate 实现
- **生产就绪**: 企业级功能，包括连接池、监控、性能优化
- **类型安全**: 完整的 Rust 类型系统支持
- **异步支持**: 兼容 Tokio 异步运行时
- **多数据库兼容**: 支持 GaussDB 和 OpenGauss

## 📦 安装

将以下内容添加到您的 `Cargo.toml`:

```toml
[dependencies]
diesel = { version = "2.2", features = ["postgres"] }
diesel-gaussdb = "1.0"
gaussdb = "0.1"
```

## 🔧 快速开始

### 1. 数据库连接

```rust
use diesel::prelude::*;
use diesel_gaussdb::GaussDbConnection;

// 连接到 GaussDB
let database_url = "host=localhost port=5432 user=gaussdb password=your_password dbname=your_db";
let mut connection = GaussDbConnection::establish(&database_url)
    .expect("连接数据库失败");
```

### 2. 定义模型

```rust
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel_gaussdb::GaussDb))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}
```

### 3. 数据库操作

```rust
use diesel::prelude::*;

// 查询用户
let users = users::table
    .select(User::as_select())
    .load(&mut connection)
    .expect("加载用户失败");

// 插入新用户
let new_user = NewUser {
    name: "张三",
    email: "zhangsan@example.com",
};

diesel::insert_into(users::table)
    .values(&new_user)
    .returning(User::as_returning())
    .get_result(&mut connection)
    .expect("插入用户失败");
```

## 🗄️ 数据库设置

### 使用 Docker 运行 OpenGauss

```bash
# 启动 OpenGauss 容器
docker run --name opengauss \
  -e GS_PASSWORD=Gaussdb@123 \
  -e GS_DB=diesel_test \
  -e GS_USER=gaussdb \
  -p 5432:5432 \
  -d opengauss/opengauss:7.0.0-RC1.B023

# 等待数据库启动
sleep 10

# 连接测试
docker exec -it opengauss gsql -d diesel_test -U gaussdb
```

### 使用 Docker Compose

```yaml
version: '3.8'
services:
  opengauss:
    image: opengauss/opengauss:7.0.0-RC1.B023
    environment:
      GS_PASSWORD: Gaussdb@123
      GS_DB: diesel_test
      GS_USER: gaussdb
    ports:
      - "5432:5432"
    volumes:
      - ./scripts/init-test-db.sql:/docker-entrypoint-initdb.d/init.sql
```

## 🔧 高级功能

### 连接池

```rust
use diesel::r2d2::{self, ConnectionManager};
use diesel_gaussdb::GaussDbConnection;

type DbPool = r2d2::Pool<ConnectionManager<GaussDbConnection>>;

let manager = ConnectionManager::<GaussDbConnection>::new(database_url);
let pool = r2d2::Pool::builder()
    .max_size(10)
    .build(manager)
    .expect("创建连接池失败");

// 使用连接池
let mut conn = pool.get().expect("获取连接失败");
```

### 事务处理

```rust
use diesel::prelude::*;

connection.transaction::<_, diesel::result::Error, _>(|conn| {
    // 在事务中执行操作
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)?;
    
    // 更新相关数据
    diesel::update(users::table.find(user_id))
        .set(users::updated_at.eq(diesel::dsl::now))
        .execute(conn)?;
    
    Ok(())
})?;
```

### 异步支持

```rust
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_gaussdb::AsyncGaussDbConnection;

let mut conn = AsyncGaussDbConnection::establish(&database_url).await?;

let users = users::table
    .select(User::as_select())
    .load(&mut conn)
    .await?;
```

## 📊 监控和性能

### 启用监控

```rust
use diesel_gaussdb::monitoring::enable_monitoring;

// 启用监控
enable_monitoring();

// 查看指标
let metrics = diesel_gaussdb::monitoring::get_metrics();
println!("连接数: {}", metrics.active_connections);
println!("查询数: {}", metrics.total_queries);
```

### 性能优化

```rust
use diesel_gaussdb::performance::{QueryCache, BatchOperations};

// 启用查询缓存
let cache = QueryCache::new(1000, Duration::from_secs(300));

// 批量操作
let batch = BatchOperations::new();
batch.insert_batch(&users, &new_users)?;
```

## 🧪 测试

运行测试套件：

```bash
# 启动测试数据库
docker-compose -f docker-compose.test.yml up -d

# 运行所有测试
cargo test

# 运行集成测试
cargo test --test diesel_integration

# 运行真实数据库测试
export GAUSSDB_TEST_URL="host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=diesel_test"
cargo test --test diesel_integration -- --nocapture
```

## 📚 文档

- [API 文档](docs/zh/api.md)
- [快速开始指南](docs/zh/getting-started.md)
- [配置指南](docs/zh/configuration.md)
- [最佳实践](docs/zh/best-practices.md)
- [故障排除](docs/zh/troubleshooting.md)
- [迁移指南](docs/zh/migration.md)

## 🤝 贡献

我们欢迎贡献！请查看 [贡献指南](CONTRIBUTING_zh.md) 了解详情。

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/HuaweiCloudDeveloper/gaussdb-diesel.git
cd gaussdb-diesel

# 安装依赖
cargo build

# 运行测试
./scripts/run-real-tests.sh
```

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。详见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)。

## 🔗 相关链接

- [Diesel ORM](https://diesel.rs/)
- [GaussDB 官方文档](https://www.huaweicloud.com/product/gaussdb.html)
- [OpenGauss 官方网站](https://opengauss.org/)
- [华为云开源社区](https://github.com/HuaweiCloudDeveloper)

## 📞 支持

- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues)
- [华为云 GaussDB 技术支持论坛](https://bbs.huaweicloud.com/forum/forum-1131-1.html)
- [Diesel 社区](https://github.com/diesel-rs/diesel/discussions)

## 🏆 致谢

感谢以下项目和社区的支持：

- [Diesel ORM](https://diesel.rs/) - 优秀的 Rust ORM 框架
- [GaussDB](https://www.huaweicloud.com/product/gaussdb.html) - 企业级数据库
- [OpenGauss](https://opengauss.org/) - 开源数据库
- [Rust 社区](https://www.rust-lang.org/community) - 活跃的开发者社区

---

**diesel-gaussdb** - 为 Rust 生态系统提供完整的 GaussDB 数据库支持 🚀
