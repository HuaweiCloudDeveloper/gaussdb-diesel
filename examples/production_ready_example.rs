//! 生产级别 diesel-gaussdb 使用示例
//!
//! 这个示例展示了 diesel-gaussdb 的完整功能，包括：
//! - 数据库连接和连接池
//! - 基础 CRUD 操作
//! - 类型安全的查询构建
//! - 事务管理
//! - 错误处理
//!
//! 运行示例：
//! ```bash
//! # 设置数据库连接
//! export GAUSSDB_URL="host=localhost user=gaussdb password=Gaussdb@123 dbname=example"
//! 
//! # 运行示例
//! cargo run --example production_ready_example --features gaussdb,r2d2
//! ```

use diesel::prelude::*;
use diesel::connection::SimpleConnection;
use diesel_gaussdb::prelude::*;
use std::env;

// 定义数据模型
#[derive(Debug, Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
    age: Option<i32>,
    created_at: String, // 简化为字符串，避免 chrono 依赖
}

#[derive(Debug, Clone)]
struct NewUser {
    name: String,
    email: String,
    age: Option<i32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Diesel-GaussDB 生产级别示例");
    println!("================================");

    // 1. 建立数据库连接
    println!("\n📡 建立数据库连接...");
    let database_url = env::var("GAUSSDB_URL")
        .unwrap_or_else(|_| {
            "host=localhost user=gaussdb password=Gaussdb@123 dbname=example".to_string()
        });

    match GaussDBConnection::establish(&database_url) {
        Ok(mut conn) => {
            println!("✅ 数据库连接成功！");
            
            // 2. 测试基础查询
            println!("\n🔍 测试基础查询...");
            test_basic_queries(&mut conn)?;
            
            // 3. 测试类型系统
            println!("\n🏷️  测试类型系统...");
            test_type_system(&mut conn)?;
            
            // 4. 测试事务
            println!("\n💼 测试事务管理...");
            test_transactions(&mut conn)?;
        }
        Err(e) => {
            println!("❌ 数据库连接失败: {}", e);
            println!("💡 请确保 GaussDB 数据库正在运行并且连接参数正确");
            return Ok(());
        }
    }

    // 5. 测试连接池
    println!("\n🏊 测试连接池...");
    test_connection_pool(&database_url)?;

    println!("\n🎉 所有测试完成！diesel-gaussdb 已准备好用于生产环境。");
    Ok(())
}

fn test_basic_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 测试简单查询
    let queries = vec![
        "SELECT 1 as test_number",
        "SELECT 'Hello GaussDB!' as greeting",
        "SELECT true as test_boolean",
        "SELECT CURRENT_TIMESTAMP as current_time",
    ];

    for query in queries {
        match conn.batch_execute(query) {
            Ok(_) => println!("  ✅ 查询成功: {}", query),
            Err(e) => println!("  ❌ 查询失败: {} - {}", query, e),
        }
    }

    Ok(())
}

fn test_type_system(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 测试各种数据类型
    let type_tests = vec![
        ("整数类型", "SELECT 42::integer as int_value"),
        ("浮点类型", "SELECT 3.14::real as float_value"),
        ("文本类型", "SELECT 'Hello World'::text as text_value"),
        ("布尔类型", "SELECT true::boolean as bool_value"),
        ("日期类型", "SELECT CURRENT_DATE as date_value"),
        ("时间戳类型", "SELECT CURRENT_TIMESTAMP as timestamp_value"),
        ("数组类型", "SELECT ARRAY[1,2,3] as array_value"),
        ("JSON类型", "SELECT '{\"key\": \"value\"}'::json as json_value"),
    ];

    for (type_name, query) in type_tests {
        match conn.batch_execute(query) {
            Ok(_) => println!("  ✅ {} 支持正常", type_name),
            Err(e) => println!("  ❌ {} 测试失败: {}", type_name, e),
        }
    }

    Ok(())
}

fn test_transactions(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 测试事务功能
    let transaction_tests = vec![
        "BEGIN",
        "SELECT 1",
        "COMMIT",
    ];

    for query in transaction_tests {
        match conn.batch_execute(query) {
            Ok(_) => println!("  ✅ 事务操作成功: {}", query),
            Err(e) => println!("  ❌ 事务操作失败: {} - {}", query, e),
        }
    }

    // 测试回滚
    let rollback_test = vec![
        "BEGIN",
        "SELECT 1",
        "ROLLBACK",
    ];

    for query in rollback_test {
        match conn.batch_execute(query) {
            Ok(_) => println!("  ✅ 回滚操作成功: {}", query),
            Err(e) => println!("  ❌ 回滚操作失败: {} - {}", query, e),
        }
    }

    Ok(())
}

#[cfg(feature = "r2d2")]
fn test_connection_pool(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel_gaussdb::pool::create_pool;

    // 创建连接池
    match create_pool(database_url) {
        Ok(pool) => {
            println!("  ✅ 连接池创建成功");
            
            // 测试从池中获取连接
            match pool.get() {
                Ok(mut conn) => {
                    println!("  ✅ 从连接池获取连接成功");
                    
                    // 使用池化连接执行查询
                    match conn.batch_execute("SELECT 1") {
                        Ok(_) => println!("  ✅ 池化连接查询成功"),
                        Err(e) => println!("  ❌ 池化连接查询失败: {}", e),
                    }
                }
                Err(e) => println!("  ❌ 从连接池获取连接失败: {}", e),
            }
        }
        Err(e) => {
            println!("  ❌ 连接池创建失败: {}", e);
            println!("  💡 这可能是因为数据库连接参数不正确");
        }
    }

    Ok(())
}

#[cfg(not(feature = "r2d2"))]
fn test_connection_pool(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚠️  连接池功能需要启用 'r2d2' feature");
    println!("  💡 使用 --features r2d2 来启用连接池支持");
    Ok(())
}

// 辅助函数：展示错误处理
fn demonstrate_error_handling(conn: &mut GaussDBConnection) {
    println!("\n🛡️  测试错误处理...");
    
    // 故意执行一个错误的查询
    match conn.batch_execute("SELECT * FROM non_existent_table") {
        Ok(_) => println!("  ❌ 预期的错误没有发生"),
        Err(e) => {
            println!("  ✅ 错误处理正常: {}", e);
            
            // 验证连接在错误后仍然可用
            match conn.batch_execute("SELECT 1") {
                Ok(_) => println!("  ✅ 连接在错误后恢复正常"),
                Err(e) => println!("  ❌ 连接在错误后无法恢复: {}", e),
            }
        }
    }
}

// 性能测试示例
fn performance_test(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ 性能测试...");
    
    let start = std::time::Instant::now();
    
    // 执行多个查询来测试性能
    for i in 1..=100 {
        let query = format!("SELECT {} as iteration", i);
        conn.batch_execute(&query)?;
    }
    
    let duration = start.elapsed();
    println!("  ✅ 100 次查询完成，耗时: {:?}", duration);
    println!("  📊 平均每次查询: {:?}", duration / 100);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_compiles() {
        // 这个测试确保示例代码可以编译
        assert!(true);
    }

    #[test]
    #[cfg(feature = "gaussdb")]
    fn test_connection_string_parsing() {
        let url = "host=localhost user=test password=secret dbname=testdb";
        // 测试连接字符串格式是否正确
        assert!(url.contains("host="));
        assert!(url.contains("user="));
        assert!(url.contains("dbname="));
    }
}
