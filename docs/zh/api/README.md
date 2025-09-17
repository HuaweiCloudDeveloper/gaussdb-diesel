# API 参考文档

diesel-gaussdb API 参考文档，包含所有公共接口的详细说明。

## 📚 模块概览

- [连接管理](connection.md) - 数据库连接和连接池
- [查询构建器](query-builder.md) - SQL查询构建和执行
- [类型系统](types.md) - 数据类型映射和转换
- [事务处理](transactions.md) - 事务管理和控制
- [性能优化](performance.md) - 缓存、批量操作等
- [监控系统](monitoring.md) - 指标收集和健康检查
- [错误处理](errors.md) - 错误类型和处理方法

## 🔧 核心类型

### GaussDbConnection

主要的数据库连接类型，实现了 Diesel 的 `Connection` trait。

```rust
pub struct GaussDbConnection {
    // 内部实现
}

impl Connection for GaussDbConnection {
    type Backend = GaussDb;
    type TransactionManager = AnsiTransactionManager;
    
    fn establish(database_url: &str) -> ConnectionResult<Self>;
    fn execute(&mut self, query: &str) -> QueryResult<usize>;
    fn query_by_index<T, U>(&mut self, source: T) -> QueryResult<Vec<U>>;
    fn query_by_name<T, U>(&mut self, source: T) -> QueryResult<Vec<U>>;
    fn execute_returning_count<T>(&mut self, source: &T) -> QueryResult<usize>;
    fn transaction_state(&mut self) -> &mut AnsiTransactionManager;
}
```

### GaussDb Backend

GaussDB 数据库后端实现。

```rust
pub struct GaussDb;

impl Backend for GaussDb {
    type QueryBuilder = GaussDbQueryBuilder;
    type RawValue<'a> = GaussDbValue<'a>;
    type BindCollector<'a> = GaussDbBindCollector<'a>;
}

impl diesel::sql_types::HasSqlType<diesel::sql_types::Integer> for GaussDb {
    fn metadata(_: &mut Self::MetadataLookup) -> Self::TypeMetadata {
        // 实现细节
    }
}
```

## 🔍 查询接口

### 基本查询

```rust
use diesel::prelude::*;
use diesel_gaussdb::GaussDbConnection;

// 查询所有记录
let results: Vec<User> = users::table
    .select(User::as_select())
    .load(&mut connection)?;

// 条件查询
let user: User = users::table
    .filter(users::id.eq(1))
    .select(User::as_select())
    .first(&mut connection)?;

// 插入记录
let new_user = NewUser {
    name: "张三",
    email: "zhangsan@example.com",
};

let inserted_user: User = diesel::insert_into(users::table)
    .values(&new_user)
    .returning(User::as_returning())
    .get_result(&mut connection)?;
```

### 高级查询

```rust
// 连接查询
let results: Vec<(User, Post)> = users::table
    .inner_join(posts::table)
    .select((User::as_select(), Post::as_select()))
    .load(&mut connection)?;

// 聚合查询
let count: i64 = users::table
    .count()
    .get_result(&mut connection)?;

// 分组查询
let results: Vec<(String, i64)> = posts::table
    .group_by(posts::category)
    .select((posts::category, diesel::dsl::count(posts::id)))
    .load(&mut connection)?;
```

## 🏊 连接池接口

### R2D2 集成

```rust
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_gaussdb::GaussDbConnection;

pub type DbPool = Pool<ConnectionManager<GaussDbConnection>>;

// 创建连接池
pub fn create_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<GaussDbConnection>::new(database_url);
    Pool::builder()
        .max_size(15)
        .build(manager)
        .expect("Failed to create pool")
}

// 使用连接池
pub fn get_user_by_id(pool: &DbPool, user_id: i32) -> QueryResult<User> {
    let mut conn = pool.get().unwrap();
    users::table
        .find(user_id)
        .select(User::as_select())
        .first(&mut conn)
}
```

## 🔄 事务接口

### 基本事务

```rust
use diesel::prelude::*;

// 自动事务
let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
    // 在事务中执行操作
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)?;
    
    diesel::update(users::table.find(user_id))
        .set(users::updated_at.eq(diesel::dsl::now))
        .execute(conn)?;
    
    Ok(())
})?;

// 手动事务控制
connection.begin_transaction()?;

match perform_operations(&mut connection) {
    Ok(_) => connection.commit_transaction()?,
    Err(e) => {
        connection.rollback_transaction()?;
        return Err(e);
    }
}
```

### 嵌套事务

```rust
// 保存点支持
connection.transaction::<_, diesel::result::Error, _>(|conn| {
    // 外层事务
    diesel::insert_into(users::table)
        .values(&user1)
        .execute(conn)?;
    
    // 内层事务（保存点）
    conn.transaction::<_, diesel::result::Error, _>(|inner_conn| {
        diesel::insert_into(posts::table)
            .values(&post1)
            .execute(inner_conn)?;
        
        // 如果这里出错，只回滚到保存点
        Ok(())
    })?;
    
    Ok(())
})?;
```

## 📊 类型系统

### 支持的数据类型

| Rust 类型 | SQL 类型 | 说明 |
|-----------|----------|------|
| `i16` | `SMALLINT` | 16位整数 |
| `i32` | `INTEGER` | 32位整数 |
| `i64` | `BIGINT` | 64位整数 |
| `f32` | `REAL` | 单精度浮点 |
| `f64` | `DOUBLE PRECISION` | 双精度浮点 |
| `String` | `VARCHAR`, `TEXT` | 字符串 |
| `bool` | `BOOLEAN` | 布尔值 |
| `Vec<u8>` | `BYTEA` | 二进制数据 |
| `chrono::NaiveDateTime` | `TIMESTAMP` | 时间戳 |
| `chrono::NaiveDate` | `DATE` | 日期 |
| `chrono::NaiveTime` | `TIME` | 时间 |
| `uuid::Uuid` | `UUID` | UUID |
| `serde_json::Value` | `JSON`, `JSONB` | JSON数据 |

### 自定义类型

```rust
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

impl ToSql<Text, GaussDb> for Status {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDb>) -> serialize::Result {
        match *self {
            Status::Active => out.write_all(b"active")?,
            Status::Inactive => out.write_all(b"inactive")?,
            Status::Pending => out.write_all(b"pending")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, GaussDb> for Status {
    fn from_sql(bytes: GaussDbValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_str()? {
            "active" => Ok(Status::Active),
            "inactive" => Ok(Status::Inactive),
            "pending" => Ok(Status::Pending),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}
```

## ⚡ 性能接口

### 查询缓存

```rust
use diesel_gaussdb::performance::QueryCache;

// 启用查询缓存
let cache = QueryCache::new(1000, Duration::from_secs(300));
connection.set_query_cache(cache);

// 缓存统计
let stats = connection.cache_stats();
println!("缓存命中率: {:.2}%", stats.hit_rate() * 100.0);
```

### 批量操作

```rust
use diesel_gaussdb::performance::BatchOperations;

// 批量插入
let users = vec![user1, user2, user3];
let batch = BatchOperations::new();
batch.insert_batch(&mut connection, users::table, &users)?;

// 批量更新
batch.update_batch(&mut connection, users::table, &updates)?;
```

## 📈 监控接口

### 指标收集

```rust
use diesel_gaussdb::monitoring::{enable_monitoring, get_metrics};

// 启用监控
enable_monitoring();

// 获取指标
let metrics = get_metrics();
println!("活跃连接数: {}", metrics.active_connections);
println!("总查询数: {}", metrics.total_queries);
println!("平均查询时间: {}ms", metrics.avg_query_time.as_millis());
```

### 健康检查

```rust
use diesel_gaussdb::monitoring::HealthCheck;

// 执行健康检查
let health = HealthCheck::new(&mut connection);
let status = health.check_all()?;

if status.is_healthy() {
    println!("✅ 数据库健康状态良好");
} else {
    println!("❌ 数据库健康检查失败: {:?}", status.issues());
}
```

## 🚨 错误处理

### 错误类型

```rust
use diesel_gaussdb::errors::{GaussDbError, ConnectionError, QueryError};

// 连接错误
match GaussDbConnection::establish(&invalid_url) {
    Ok(conn) => { /* 使用连接 */ },
    Err(ConnectionError::InvalidUrl(msg)) => {
        eprintln!("无效的数据库URL: {}", msg);
    },
    Err(ConnectionError::Timeout) => {
        eprintln!("连接超时");
    },
    Err(e) => {
        eprintln!("连接失败: {}", e);
    }
}

// 查询错误
match users::table.load::<User>(&mut connection) {
    Ok(users) => { /* 处理结果 */ },
    Err(QueryError::NotFound) => {
        println!("未找到用户");
    },
    Err(QueryError::UniqueViolation(field)) => {
        eprintln!("唯一约束违反: {}", field);
    },
    Err(e) => {
        eprintln!("查询失败: {}", e);
    }
}
```

## 🔧 配置接口

### 连接配置

```rust
use diesel_gaussdb::config::{ConnectionConfig, SslMode};

let config = ConnectionConfig::builder()
    .host("localhost")
    .port(5432)
    .user("gaussdb")
    .password("password")
    .database("myapp")
    .ssl_mode(SslMode::Require)
    .connect_timeout(Duration::from_secs(30))
    .build();

let connection = GaussDbConnection::establish_with_config(&config)?;
```

---

**完整的API文档请参考各个子模块的详细说明。** 📚
