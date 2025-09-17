# 配置指南

本指南详细介绍了 diesel-gaussdb 的各种配置选项，帮助您优化应用程序的性能和行为。

## 🔧 数据库连接配置

### 基本连接字符串

```rust
// 基本连接
let database_url = "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=my_app";

// 带SSL的连接
let database_url = "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=my_app sslmode=require";

// 连接超时设置
let database_url = "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=my_app connect_timeout=10";
```

### 连接参数详解

| 参数 | 描述 | 默认值 | 示例 |
|------|------|--------|------|
| `host` | 数据库服务器地址 | localhost | `host=192.168.1.100` |
| `port` | 数据库端口 | 5432 | `port=5433` |
| `user` | 用户名 | - | `user=gaussdb` |
| `password` | 密码 | - | `password=MyPassword123` |
| `dbname` | 数据库名称 | - | `dbname=production_db` |
| `sslmode` | SSL模式 | prefer | `sslmode=require` |
| `connect_timeout` | 连接超时(秒) | 无限制 | `connect_timeout=30` |
| `application_name` | 应用程序名称 | - | `application_name=MyApp` |

### SSL 配置

```rust
use diesel_gaussdb::{GaussDbConnection, SslMode};

// 禁用SSL
let url = "host=localhost user=gaussdb password=pass dbname=db sslmode=disable";

// 要求SSL
let url = "host=localhost user=gaussdb password=pass dbname=db sslmode=require";

// SSL证书验证
let url = "host=localhost user=gaussdb password=pass dbname=db sslmode=verify-full sslcert=client.crt sslkey=client.key sslrootcert=ca.crt";
```

## 🏊 连接池配置

### R2D2 连接池

```rust
use diesel::r2d2::{self, ConnectionManager, Pool, PoolBuilder};
use diesel_gaussdb::GaussDbConnection;
use std::time::Duration;

pub fn create_pool() -> Pool<ConnectionManager<GaussDbConnection>> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let manager = ConnectionManager::<GaussDbConnection>::new(database_url);
    
    Pool::builder()
        .max_size(15)                           // 最大连接数
        .min_idle(Some(5))                      // 最小空闲连接数
        .max_lifetime(Some(Duration::from_secs(3600))) // 连接最大生存时间
        .idle_timeout(Some(Duration::from_secs(600)))  // 空闲超时时间
        .connection_timeout(Duration::from_secs(30))   // 获取连接超时时间
        .test_on_check_out(true)                // 检出时测试连接
        .build(manager)
        .expect("Failed to create pool")
}
```

### 连接池参数说明

| 参数 | 描述 | 推荐值 | 注意事项 |
|------|------|--------|----------|
| `max_size` | 最大连接数 | 10-20 | 根据并发需求调整 |
| `min_idle` | 最小空闲连接 | max_size的30% | 保持足够的预热连接 |
| `max_lifetime` | 连接最大生存时间 | 1小时 | 防止长时间连接问题 |
| `idle_timeout` | 空闲超时 | 10分钟 | 释放不活跃连接 |
| `connection_timeout` | 获取连接超时 | 30秒 | 避免长时间等待 |

## ⚡ 性能配置

### 查询缓存

```rust
use diesel_gaussdb::performance::QueryCache;
use std::time::Duration;

// 启用查询缓存
let cache = QueryCache::builder()
    .max_entries(1000)                    // 最大缓存条目数
    .ttl(Duration::from_secs(300))        // 缓存生存时间
    .enable_metrics(true)                 // 启用指标收集
    .build();

// 在连接中使用缓存
let mut conn = establish_connection();
conn.set_query_cache(cache);
```

### 批量操作配置

```rust
use diesel_gaussdb::performance::BatchConfig;

let batch_config = BatchConfig {
    batch_size: 1000,           // 批量大小
    parallel_batches: 4,        // 并行批次数
    timeout: Duration::from_secs(60), // 批量操作超时
};

// 应用批量配置
conn.set_batch_config(batch_config);
```

### 预编译语句

```rust
use diesel_gaussdb::performance::PreparedStatements;

// 启用预编译语句缓存
let prepared_config = PreparedStatements::builder()
    .max_statements(100)        // 最大预编译语句数
    .enable_auto_prepare(true)  // 自动预编译频繁查询
    .build();

conn.set_prepared_statements(prepared_config);
```

## 📊 监控配置

### 启用监控

```rust
use diesel_gaussdb::monitoring::{MonitoringConfig, MetricsCollector};

let monitoring_config = MonitoringConfig {
    enable_query_metrics: true,     // 启用查询指标
    enable_connection_metrics: true, // 启用连接指标
    enable_error_tracking: true,    // 启用错误跟踪
    metrics_interval: Duration::from_secs(60), // 指标收集间隔
};

// 应用监控配置
diesel_gaussdb::monitoring::configure(monitoring_config);
```

### 自定义指标收集器

```rust
use diesel_gaussdb::monitoring::{MetricsCollector, QueryMetrics};

struct CustomMetricsCollector;

impl MetricsCollector for CustomMetricsCollector {
    fn record_query(&self, metrics: &QueryMetrics) {
        // 自定义指标记录逻辑
        println!("Query executed in {}ms", metrics.duration.as_millis());
    }
    
    fn record_connection_event(&self, event: &str) {
        // 记录连接事件
        println!("Connection event: {}", event);
    }
}

// 注册自定义收集器
diesel_gaussdb::monitoring::set_collector(Box::new(CustomMetricsCollector));
```

## 🔒 安全配置

### 连接安全

```rust
use diesel_gaussdb::security::SecurityConfig;

let security_config = SecurityConfig {
    require_ssl: true,              // 要求SSL连接
    verify_certificates: true,      // 验证SSL证书
    min_tls_version: "1.2".to_string(), // 最小TLS版本
    allowed_ciphers: vec![          // 允许的加密套件
        "ECDHE-RSA-AES256-GCM-SHA384".to_string(),
        "ECDHE-RSA-AES128-GCM-SHA256".to_string(),
    ],
};

// 应用安全配置
conn.configure_security(security_config)?;
```

### 查询安全

```rust
use diesel_gaussdb::security::QuerySecurity;

let query_security = QuerySecurity {
    enable_sql_injection_detection: true, // 启用SQL注入检测
    max_query_length: 10000,              // 最大查询长度
    blocked_keywords: vec![               // 阻止的关键词
        "DROP".to_string(),
        "TRUNCATE".to_string(),
    ],
};

conn.set_query_security(query_security);
```

## 🌍 环境配置

### 开发环境

```toml
# .env.development
DATABASE_URL=host=localhost port=5432 user=gaussdb password=dev123 dbname=myapp_dev
RUST_LOG=diesel_gaussdb=debug,diesel=debug
ENABLE_QUERY_LOGGING=true
CONNECTION_POOL_SIZE=5
```

### 生产环境

```toml
# .env.production
DATABASE_URL=host=prod-db.example.com port=5432 user=gaussdb password=SecurePass123 dbname=myapp_prod sslmode=require
RUST_LOG=diesel_gaussdb=info,diesel=warn
ENABLE_QUERY_LOGGING=false
CONNECTION_POOL_SIZE=20
MAX_CONNECTION_LIFETIME=3600
```

### 测试环境

```toml
# .env.test
DATABASE_URL=host=localhost port=5433 user=test_user password=test123 dbname=myapp_test
RUST_LOG=diesel_gaussdb=trace
ENABLE_QUERY_LOGGING=true
CONNECTION_POOL_SIZE=2
TEST_TIMEOUT=30
```

## 📝 日志配置

### 启用详细日志

```rust
use log::LevelFilter;
use env_logger::Builder;

fn init_logging() {
    let mut builder = Builder::from_default_env();
    
    builder
        .filter_level(LevelFilter::Info)
        .filter_module("diesel_gaussdb", LevelFilter::Debug)
        .filter_module("diesel", LevelFilter::Info)
        .init();
}
```

### 结构化日志

```rust
use serde_json::json;
use diesel_gaussdb::logging::StructuredLogger;

let logger = StructuredLogger::new()
    .with_format("json")
    .with_fields(json!({
        "service": "my-app",
        "version": "1.0.0",
        "environment": "production"
    }));

diesel_gaussdb::logging::set_logger(logger);
```

## 🔧 高级配置

### 自定义类型映射

```rust
use diesel_gaussdb::types::TypeMapping;

let type_mapping = TypeMapping::builder()
    .map_custom_type("my_enum", "TEXT")
    .map_array_type("my_array", "TEXT[]")
    .build();

conn.set_type_mapping(type_mapping);
```

### 查询优化器配置

```rust
use diesel_gaussdb::optimizer::QueryOptimizer;

let optimizer = QueryOptimizer::builder()
    .enable_index_hints(true)       // 启用索引提示
    .enable_join_optimization(true) // 启用连接优化
    .max_optimization_time(Duration::from_millis(100)) // 最大优化时间
    .build();

conn.set_query_optimizer(optimizer);
```

## 📋 配置验证

### 验证配置

```rust
use diesel_gaussdb::config::ConfigValidator;

fn validate_config() -> Result<(), Box<dyn std::error::Error>> {
    let validator = ConfigValidator::new();
    
    // 验证数据库连接
    validator.validate_connection(&database_url)?;
    
    // 验证连接池配置
    validator.validate_pool_config(&pool_config)?;
    
    // 验证性能配置
    validator.validate_performance_config(&perf_config)?;
    
    println!("✅ 所有配置验证通过");
    Ok(())
}
```

## 🚀 最佳实践

1. **连接池大小**: 根据应用并发量设置，通常为CPU核心数的2-4倍
2. **连接超时**: 设置合理的超时时间，避免长时间等待
3. **SSL配置**: 生产环境必须启用SSL
4. **监控指标**: 启用监控以便及时发现性能问题
5. **日志级别**: 生产环境使用INFO级别，开发环境使用DEBUG
6. **缓存策略**: 根据查询模式配置合适的缓存大小和TTL

---

**通过合理的配置，diesel-gaussdb 可以为您的应用提供最佳的性能和稳定性！** 🚀
