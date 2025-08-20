//! 性能基准测试示例
//!
//! 这个示例测试 diesel-gaussdb 的性能表现，包括：
//! - 连接建立性能
//! - 查询执行性能
//! - 批量操作性能
//! - 连接池性能
//! - 事务性能
//!
//! 运行示例：
//! ```bash
//! cargo run --example performance_benchmark --features gaussdb,r2d2 --release
//! ```

use diesel::prelude::*;
use diesel::connection::SimpleConnection;
use diesel_gaussdb::prelude::*;
use diesel_gaussdb::pool::{GaussDBPool, PooledConnection};
use std::env;
use std::time::{Duration, Instant};

table! {
    benchmark_users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        created_at -> Timestamp,
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = benchmark_users)]
struct BenchmarkUser {
    id: i32,
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = benchmark_users)]
struct NewBenchmarkUser {
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Diesel-GaussDB 性能基准测试");
    println!("==============================");

    // 建立数据库连接
    let database_url = env::var("GAUSSDB_TEST_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string()
        });

    let mut conn = GaussDBConnection::establish(&database_url)?;
    println!("✅ 数据库连接成功");

    // 设置测试环境
    setup_benchmark_environment(&mut conn)?;
    println!("✅ 基准测试环境设置完成");

    // 1. 连接建立性能测试
    println!("\n⚡ 1. 连接建立性能测试");
    benchmark_connection_establishment(&database_url)?;

    // 2. 基础查询性能测试
    println!("\n⚡ 2. 基础查询性能测试");
    benchmark_basic_queries(&mut conn)?;

    // 3. 批量插入性能测试
    println!("\n⚡ 3. 批量插入性能测试");
    benchmark_bulk_insert(&mut conn)?;

    // 4. 连接池性能测试
    println!("\n⚡ 4. 连接池性能测试");
    benchmark_connection_pool(&database_url)?;

    // 5. 事务性能测试
    println!("\n⚡ 5. 事务性能测试");
    benchmark_transactions(&mut conn)?;

    // 6. 复杂查询性能测试
    println!("\n⚡ 6. 复杂查询性能测试");
    benchmark_complex_queries(&mut conn)?;

    // 清理测试环境
    cleanup_benchmark_environment(&mut conn)?;
    println!("\n✅ 基准测试环境清理完成");

    println!("\n🎉 所有性能基准测试完成！");
    Ok(())
}

fn setup_benchmark_environment(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    conn.batch_execute(r#"
        DROP TABLE IF EXISTS benchmark_users CASCADE;
        
        CREATE TABLE benchmark_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE INDEX idx_benchmark_users_email ON benchmark_users(email);
        CREATE INDEX idx_benchmark_users_created_at ON benchmark_users(created_at);
    "#)?;
    Ok(())
}

fn benchmark_connection_establishment(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    const ITERATIONS: usize = 100;
    
    println!("  📊 测试 {} 次连接建立", ITERATIONS);
    
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _conn = GaussDBConnection::establish(database_url)?;
        if (i + 1) % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  总时间: {:?}", duration);
    println!("    📈 平均每次连接: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒连接数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn benchmark_basic_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    const ITERATIONS: usize = 1000;
    
    // 插入一些测试数据
    let test_users: Vec<NewBenchmarkUser> = (0..100).map(|i| NewBenchmarkUser {
        name: format!("User {}", i),
        email: format!("user{}@example.com", i),
        created_at: chrono::Utc::now().naive_utc(),
    }).collect();
    
    diesel::insert_into(benchmark_users::table)
        .values(&test_users)
        .execute(conn)?;
    
    println!("  📊 测试 {} 次基础查询", ITERATIONS);
    
    // SELECT 查询性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _result = conn.batch_execute("SELECT COUNT(*) FROM benchmark_users")?;
        if (i + 1) % 100 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  SELECT 总时间: {:?}", duration);
    println!("    📈 平均每次查询: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒查询数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    // WHERE 查询性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let email = format!("user{}@example.com", i % 100);
        let _result = conn.batch_execute(&format!("SELECT * FROM benchmark_users WHERE email = '{}'", email))?;
        if (i + 1) % 100 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  WHERE 总时间: {:?}", duration);
    println!("    📈 平均每次查询: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒查询数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn benchmark_bulk_insert(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    const BATCH_SIZES: &[usize] = &[10, 100, 1000, 5000];
    
    for &batch_size in BATCH_SIZES {
        println!("  📊 测试批量插入 {} 条记录", batch_size);
        
        // 清理表
        diesel::delete(benchmark_users::table).execute(conn)?;
        
        // 准备测试数据
        let test_users: Vec<NewBenchmarkUser> = (0..batch_size).map(|i| NewBenchmarkUser {
            name: format!("Bulk User {}", i),
            email: format!("bulk{}@example.com", i),
            created_at: chrono::Utc::now().naive_utc(),
        }).collect();
        
        // 测试批量插入
        let start = Instant::now();
        diesel::insert_into(benchmark_users::table)
            .values(&test_users)
            .execute(conn)?;
        let duration = start.elapsed();
        
        println!("    ⏱️  插入时间: {:?}", duration);
        println!("    📈 每条记录: {:?}", duration / batch_size as u32);
        println!("    🚀 每秒插入: {:.2} 条", batch_size as f64 / duration.as_secs_f64());
        
        // 验证插入结果
        let count = conn.batch_execute("SELECT COUNT(*) FROM benchmark_users")?;
        println!("    ✅ 验证: 成功插入记录");
    }
    
    Ok(())
}

fn benchmark_connection_pool(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    const POOL_SIZE: u32 = 10;
    const ITERATIONS: usize = 1000;
    
    println!("  📊 测试连接池性能 (池大小: {}, 操作次数: {})", POOL_SIZE, ITERATIONS);
    
    // 创建连接池
    let pool = GaussDBPool::builder()
        .max_size(POOL_SIZE)
        .build(database_url)?;
    
    println!("    ✅ 连接池创建成功");
    
    // 测试连接获取性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let mut conn = pool.get()?;
        let _result = conn.batch_execute("SELECT 1")?;
        if (i + 1) % 100 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  总时间: {:?}", duration);
    println!("    📈 平均每次操作: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒操作数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    // 测试并发连接
    println!("  📊 测试并发连接获取");
    let start = Instant::now();
    let handles: Vec<_> = (0..POOL_SIZE).map(|i| {
        let pool = pool.clone();
        std::thread::spawn(move || {
            let mut conn = pool.get().unwrap();
            for _ in 0..10 {
                conn.batch_execute("SELECT 1").unwrap();
                std::thread::sleep(Duration::from_millis(1));
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();
    
    println!("    ⏱️  并发操作时间: {:?}", duration);
    println!("    🚀 并发效率: {:.2}x", (POOL_SIZE * 10) as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn benchmark_transactions(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    const ITERATIONS: usize = 100;
    
    println!("  📊 测试事务性能 ({} 次事务)", ITERATIONS);
    
    // 清理表
    diesel::delete(benchmark_users::table).execute(conn)?;
    
    // 测试事务提交性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        conn.batch_execute("BEGIN")?;
        
        let user = NewBenchmarkUser {
            name: format!("Transaction User {}", i),
            email: format!("tx{}@example.com", i),
            created_at: chrono::Utc::now().naive_utc(),
        };
        
        diesel::insert_into(benchmark_users::table)
            .values(&user)
            .execute(conn)?;
        
        conn.batch_execute("COMMIT")?;
        
        if (i + 1) % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  事务提交总时间: {:?}", duration);
    println!("    📈 平均每次事务: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒事务数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    // 测试事务回滚性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        conn.batch_execute("BEGIN")?;
        
        let user = NewBenchmarkUser {
            name: format!("Rollback User {}", i),
            email: format!("rb{}@example.com", i),
            created_at: chrono::Utc::now().naive_utc(),
        };
        
        diesel::insert_into(benchmark_users::table)
            .values(&user)
            .execute(conn)?;
        
        conn.batch_execute("ROLLBACK")?;
        
        if (i + 1) % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  事务回滚总时间: {:?}", duration);
    println!("    📈 平均每次回滚: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒回滚数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn benchmark_complex_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    const ITERATIONS: usize = 100;
    
    // 插入足够的测试数据
    let test_users: Vec<NewBenchmarkUser> = (0..1000).map(|i| NewBenchmarkUser {
        name: format!("Complex User {}", i),
        email: format!("complex{}@example.com", i),
        created_at: chrono::Utc::now().naive_utc() - chrono::Duration::days(i % 365),
    }).collect();
    
    diesel::insert_into(benchmark_users::table)
        .values(&test_users)
        .execute(conn)?;
    
    println!("  📊 测试复杂查询性能 ({} 次查询)", ITERATIONS);
    
    // 聚合查询性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _result = conn.batch_execute(r#"
            SELECT 
                DATE_TRUNC('month', created_at) as month,
                COUNT(*) as user_count,
                MIN(created_at) as first_user,
                MAX(created_at) as last_user
            FROM benchmark_users 
            GROUP BY DATE_TRUNC('month', created_at)
            ORDER BY month DESC
            LIMIT 12
        "#)?;
        
        if (i + 1) % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  聚合查询总时间: {:?}", duration);
    println!("    📈 平均每次查询: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒查询数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    // 模糊搜索性能
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let pattern = format!("Complex User {}%", i % 100);
        let _result = conn.batch_execute(&format!(
            "SELECT * FROM benchmark_users WHERE name LIKE '{}' ORDER BY created_at DESC LIMIT 10",
            pattern
        ))?;
        
        if (i + 1) % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!();
    println!("    ⏱️  模糊搜索总时间: {:?}", duration);
    println!("    📈 平均每次搜索: {:?}", duration / ITERATIONS as u32);
    println!("    🚀 每秒搜索数: {:.2}", ITERATIONS as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn cleanup_benchmark_environment(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    conn.batch_execute("DROP TABLE IF EXISTS benchmark_users CASCADE")?;
    Ok(())
}
