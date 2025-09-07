//! Diesel-GaussDB 性能测试示例
//!
//! 这个示例展示了如何对 diesel-gaussdb 进行性能测试，包括：
//! - 连接性能测试
//! - CRUD 操作性能测试
//! - 批量操作性能测试
//! - 复杂查询性能测试

use diesel::prelude::*;
use diesel_gaussdb::GaussDBConnection;
use anyhow::{Result, Context};
use log::info;
use std::env;
use std::time::{Duration, Instant};
// use rand::Rng;

/// 建立数据库连接
fn establish_connection() -> Result<GaussDBConnection> {
    let database_url = env::var("GAUSSDB_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres".to_string()
        });

    GaussDBConnection::establish(&database_url)
        .with_context(|| format!("Error connecting to {}", database_url))
}

/// 性能测试结果
#[derive(Debug)]
struct PerformanceResult {
    operation: String,
    total_time: Duration,
    operations_count: usize,
    ops_per_second: f64,
    avg_time_per_op: Duration,
}

impl PerformanceResult {
    fn new(operation: String, total_time: Duration, operations_count: usize) -> Self {
        let ops_per_second = operations_count as f64 / total_time.as_secs_f64();
        let avg_time_per_op = total_time / operations_count as u32;
        
        Self {
            operation,
            total_time,
            operations_count,
            ops_per_second,
            avg_time_per_op,
        }
    }

    fn print(&self) {
        info!("📊 性能测试结果: {}", self.operation);
        info!("  总时间: {:?}", self.total_time);
        info!("  操作数量: {}", self.operations_count);
        info!("  每秒操作数: {:.2}", self.ops_per_second);
        info!("  平均每操作时间: {:?}", self.avg_time_per_op);
        info!("");
    }
}

/// 初始化测试数据库
fn init_test_database(conn: &mut GaussDBConnection) -> Result<()> {
    info!("初始化测试数据库...");
    
    // 删除现有表
    let _ = diesel::sql_query("DROP TABLE IF EXISTS test_users CASCADE").execute(conn);
    let _ = diesel::sql_query("DROP TABLE IF EXISTS test_posts CASCADE").execute(conn);
    
    // 创建测试表
    diesel::sql_query(
        "CREATE TABLE test_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL,
            age INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(conn)?;

    diesel::sql_query(
        "CREATE TABLE test_posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            author_id INTEGER NOT NULL,
            published BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (author_id) REFERENCES test_users(id)
        )"
    ).execute(conn)?;

    // 创建索引
    diesel::sql_query("CREATE INDEX idx_test_users_email ON test_users(email)").execute(conn)?;
    diesel::sql_query("CREATE INDEX idx_test_posts_author ON test_posts(author_id)").execute(conn)?;
    diesel::sql_query("CREATE INDEX idx_test_posts_published ON test_posts(published)").execute(conn)?;

    info!("✅ 测试数据库初始化完成");
    Ok(())
}

/// 测试连接性能
fn test_connection_performance() -> Result<PerformanceResult> {
    info!("🔗 测试连接性能...");
    
    let connection_count = 100;
    let start_time = Instant::now();
    
    for _ in 0..connection_count {
        let _conn = establish_connection()?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("数据库连接".to_string(), total_time, connection_count))
}

/// 测试插入性能
fn test_insert_performance(conn: &mut GaussDBConnection) -> Result<PerformanceResult> {
    info!("📝 测试插入性能...");
    
    let insert_count = 1000;
    let start_time = Instant::now();
    
    for i in 0..insert_count {
        diesel::sql_query(&format!(
            "INSERT INTO test_users (name, email, age) VALUES ('用户{}', 'user{}@example.com', {})",
            i, i, 20 + (i % 50)
        )).execute(conn)?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("单条插入".to_string(), total_time, insert_count))
}

/// 测试批量插入性能
fn test_batch_insert_performance(conn: &mut GaussDBConnection) -> Result<PerformanceResult> {
    info!("📦 测试批量插入性能...");
    
    let batch_size = 100;
    let batch_count = 10;
    let total_inserts = batch_size * batch_count;
    
    let start_time = Instant::now();
    
    for batch in 0..batch_count {
        let mut values = Vec::new();
        for i in 0..batch_size {
            let idx = batch * batch_size + i;
            values.push(format!("('批量用户{}', 'batch{}@example.com', {})", 
                               idx, idx, 25 + (idx % 30)));
        }
        
        let sql = format!("INSERT INTO test_users (name, email, age) VALUES {}", 
                         values.join(", "));
        diesel::sql_query(sql).execute(conn)?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("批量插入".to_string(), total_time, total_inserts))
}

/// 测试查询性能
fn test_query_performance(conn: &mut GaussDBConnection) -> Result<PerformanceResult> {
    info!("🔍 测试查询性能...");
    
    #[derive(diesel::QueryableByName)]
    struct TestUser {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        id: i32,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
    }
    
    let query_count = 1000;
    let start_time = Instant::now();
    
    for i in 0..query_count {
        let _users: Vec<TestUser> = diesel::sql_query(&format!(
            "SELECT id, name, email FROM test_users WHERE age > {} LIMIT 10",
            20 + (i % 30)
        )).load(conn)?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("条件查询".to_string(), total_time, query_count))
}

/// 测试更新性能
fn test_update_performance(conn: &mut GaussDBConnection) -> Result<PerformanceResult> {
    info!("✏️ 测试更新性能...");
    
    let update_count = 500;
    let start_time = Instant::now();
    
    for i in 1..=update_count {
        diesel::sql_query(&format!(
            "UPDATE test_users SET age = {} WHERE id = {}",
            30 + (i % 20), i
        )).execute(conn)?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("单条更新".to_string(), total_time, update_count))
}

/// 测试复杂查询性能
fn test_complex_query_performance(conn: &mut GaussDBConnection) -> Result<PerformanceResult> {
    info!("🧮 测试复杂查询性能...");
    
    // 先插入一些文章数据
    for i in 1..=100 {
        diesel::sql_query(&format!(
            "INSERT INTO test_posts (title, content, author_id, published) VALUES ('文章标题{}', '这是文章{}的内容...', {}, {})",
            i, i, 1 + (i % 50), if i % 3 == 0 { "true" } else { "false" }
        )).execute(conn)?;
    }
    
    #[derive(diesel::QueryableByName)]
    struct UserStats {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        post_count: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        avg_age: Option<f64>,
    }
    
    let query_count = 100;
    let start_time = Instant::now();
    
    for _ in 0..query_count {
        let _stats: Vec<UserStats> = diesel::sql_query(
            "SELECT u.name, 
                    COUNT(p.id) as post_count,
                    AVG(u.age::float) as avg_age
             FROM test_users u
             LEFT JOIN test_posts p ON u.id = p.author_id
             WHERE u.age > 25
             GROUP BY u.id, u.name
             HAVING COUNT(p.id) > 0
             ORDER BY post_count DESC
             LIMIT 10"
        ).load(conn)?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("复杂联表查询".to_string(), total_time, query_count))
}

/// 测试事务性能
fn test_transaction_performance(conn: &mut GaussDBConnection) -> Result<PerformanceResult> {
    info!("🔄 测试事务性能...");
    
    let transaction_count = 100;
    let start_time = Instant::now();
    
    for i in 0..transaction_count {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // 在事务中执行多个操作
            diesel::sql_query(&format!(
                "INSERT INTO test_users (name, email, age) VALUES ('事务用户{}', 'tx{}@example.com', 25)",
                i, i
            )).execute(conn)?;

            diesel::sql_query(&format!(
                "UPDATE test_users SET age = age + 1 WHERE name = '事务用户{}'",
                i
            )).execute(conn)?;

            Ok(())
        })?;
    }
    
    let total_time = start_time.elapsed();
    Ok(PerformanceResult::new("事务操作".to_string(), total_time, transaction_count))
}

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🚀 启动 Diesel-GaussDB 性能测试");

    let mut connection = establish_connection()?;
    info!("✅ 数据库连接成功！");

    // 初始化测试数据库
    init_test_database(&mut connection)?;

    // 执行各项性能测试
    let mut results = Vec::new();

    // 连接性能测试
    results.push(test_connection_performance()?);

    // 插入性能测试
    results.push(test_insert_performance(&mut connection)?);

    // 批量插入性能测试
    results.push(test_batch_insert_performance(&mut connection)?);

    // 查询性能测试
    results.push(test_query_performance(&mut connection)?);

    // 更新性能测试
    results.push(test_update_performance(&mut connection)?);

    // 复杂查询性能测试
    results.push(test_complex_query_performance(&mut connection)?);

    // 事务性能测试
    results.push(test_transaction_performance(&mut connection)?);

    // 输出所有测试结果
    info!("🎯 === 性能测试总结 ===");
    for result in &results {
        result.print();
    }

    // 输出性能对比
    info!("📈 === 性能对比分析 ===");
    let insert_result = &results[1];
    let batch_insert_result = &results[2];
    
    let single_ops = insert_result.ops_per_second;
    let batch_ops = batch_insert_result.ops_per_second;
    let improvement = batch_ops / single_ops;
    
    info!("批量插入相比单条插入性能提升: {:.2}x", improvement);
    
    info!("🎉 性能测试完成！");
    Ok(())
}
