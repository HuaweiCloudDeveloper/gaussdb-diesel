# 性能测试示例

这个示例展示了如何对 diesel-gaussdb 进行全面的性能测试和基准测试。

## 测试项目

### 核心性能测试
- ✅ 数据库连接性能
- ✅ 单条插入性能
- ✅ 批量插入性能
- ✅ 查询性能测试
- ✅ 更新操作性能
- ✅ 复杂查询性能
- ✅ 事务处理性能

### 测试指标
- **吞吐量**: 每秒操作数 (OPS)
- **延迟**: 平均每操作时间
- **总时间**: 完成所有操作的总时间
- **性能对比**: 不同操作方式的性能差异

## 快速开始

### 1. 环境准备

```bash
# 启动 GaussDB/OpenGauss
docker run --name opengauss \
  -e GS_PASSWORD=Gaussdb@123 \
  -p 5432:5432 \
  -d opengauss/opengauss:7.0.0-RC1.B023

# 设置环境变量
export GAUSSDB_URL="host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres"
export RUST_LOG=info
```

### 2. 运行性能测试

```bash
cd examples/05-performance-testing
cargo run --release
```

### 3. 运行基准测试

```bash
# 运行 Criterion 基准测试
cargo bench

# 查看 HTML 报告
open target/criterion/report/index.html
```

## 测试结果示例

```
🎯 === 性能测试总结 ===
📊 性能测试结果: 数据库连接
  总时间: 2.345s
  操作数量: 100
  每秒操作数: 42.65
  平均每操作时间: 23.45ms

📊 性能测试结果: 单条插入
  总时间: 5.678s
  操作数量: 1000
  每秒操作数: 176.18
  平均每操作时间: 5.68ms

📊 性能测试结果: 批量插入
  总时间: 1.234s
  操作数量: 1000
  每秒操作数: 810.37
  平均每操作时间: 1.23ms

📊 性能测试结果: 条件查询
  总时间: 3.456s
  操作数量: 1000
  每秒操作数: 289.35
  平均每操作时间: 3.46ms

📈 === 性能对比分析 ===
批量插入相比单条插入性能提升: 4.60x
```

## 性能优化建议

### 1. 连接管理优化

```rust
// 使用连接池
use diesel::r2d2::{ConnectionManager, Pool};

type DbPool = Pool<ConnectionManager<GaussDBConnection>>;

fn create_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<GaussDBConnection>::new(database_url);
    Pool::builder()
        .max_size(20)
        .min_idle(Some(5))
        .build(manager)
        .expect("Failed to create pool")
}
```

### 2. 批量操作优化

```rust
// 使用批量插入而不是单条插入
let values: Vec<String> = data.iter()
    .map(|item| format!("('{}', '{}', {})", item.name, item.email, item.age))
    .collect();

let sql = format!("INSERT INTO users (name, email, age) VALUES {}", 
                 values.join(", "));
diesel::sql_query(sql).execute(conn)?;
```

### 3. 查询优化

```rust
// 使用索引优化查询
diesel::sql_query("CREATE INDEX idx_users_email ON users(email)").execute(conn)?;

// 使用 LIMIT 限制结果集
let users: Vec<User> = diesel::sql_query(
    "SELECT * FROM users WHERE age > $1 ORDER BY created_at DESC LIMIT 100"
).bind::<diesel::sql_types::Integer, _>(18)
.load(conn)?;
```

### 4. 事务优化

```rust
// 批量操作使用事务
conn.transaction::<_, diesel::result::Error, _>(|conn| {
    for item in batch_data {
        diesel::sql_query("INSERT INTO users ...")
            .bind::<diesel::sql_types::Text, _>(&item.name)
            .execute(conn)?;
    }
    Ok(())
})?;
```

## 基准测试配置

### Criterion 配置

```toml
[dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "database_benchmarks"
harness = false
```

### 基准测试代码

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_insert(c: &mut Criterion) {
    let mut conn = establish_connection().unwrap();
    
    c.bench_function("single_insert", |b| {
        b.iter(|| {
            diesel::sql_query("INSERT INTO test_users (name, email) VALUES ('test', 'test@example.com')")
                .execute(black_box(&mut conn))
        })
    });
}

criterion_group!(benches, bench_insert);
criterion_main!(benches);
```

## 性能监控

### 1. 系统资源监控

```bash
# 监控 CPU 和内存使用
top -p $(pgrep gaussdb)

# 监控数据库连接
psql -h localhost -U gaussdb -d postgres -c "SELECT * FROM pg_stat_activity;"
```

### 2. 数据库性能监控

```sql
-- 查看慢查询
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;

-- 查看表统计信息
SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del 
FROM pg_stat_user_tables;
```

### 3. 应用性能监控

```rust
use std::time::Instant;

let start = Instant::now();
// 执行数据库操作
let duration = start.elapsed();
log::info!("操作耗时: {:?}", duration);
```

## 性能测试最佳实践

### 1. 测试环境

- 使用与生产环境相似的硬件配置
- 确保数据库有足够的内存和存储
- 关闭不必要的后台进程

### 2. 测试数据

- 使用真实规模的测试数据
- 测试不同数据分布情况
- 包含边界条件测试

### 3. 测试方法

- 预热数据库连接
- 多次运行取平均值
- 测试并发场景
- 监控系统资源使用

### 4. 结果分析

- 对比不同实现方案
- 分析性能瓶颈
- 制定优化策略
- 建立性能基线

## 常见性能问题

### 1. 连接池配置不当

```rust
// 问题：连接池太小
Pool::builder().max_size(5)  // 太小

// 解决：合理配置连接池
Pool::builder()
    .max_size(20)              // 根据并发需求调整
    .min_idle(Some(5))         // 保持最小空闲连接
    .connection_timeout(Duration::from_secs(30))
```

### 2. 缺少索引

```sql
-- 问题：查询缺少索引
SELECT * FROM users WHERE email = 'user@example.com';

-- 解决：添加索引
CREATE INDEX idx_users_email ON users(email);
```

### 3. 大量单条操作

```rust
// 问题：逐条插入
for user in users {
    diesel::sql_query("INSERT INTO users ...").execute(conn)?;
}

// 解决：批量操作
let values = users.iter().map(|u| format!("('{}', '{}')", u.name, u.email)).collect::<Vec<_>>();
diesel::sql_query(&format!("INSERT INTO users (name, email) VALUES {}", values.join(", "))).execute(conn)?;
```

---

**🎯 这个性能测试示例帮助您全面评估和优化 diesel-gaussdb 的性能！**
