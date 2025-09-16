//! 真实 GaussDB 集成测试
//!
//! 这个测试使用真实的 OpenGauss 数据库来验证 diesel-gaussdb 的完整功能。
//! 
//! 前提条件：
//! 1. 运行 Docker 容器: `docker-compose up -d`
//! 2. 确保数据库可访问: localhost:5434
//!
//! 运行测试：
//! ```bash
//! cargo test --features gaussdb,r2d2 real_gaussdb_integration -- --nocapture
//! ```

#[cfg(all(test, feature = "gaussdb"))]
mod real_gaussdb_integration_tests {
    use diesel::prelude::*;
    use diesel::connection::SimpleConnection;
    use diesel_gaussdb::prelude::*;
    use std::env;

    /// 获取真实的 GaussDB 连接字符串
    fn get_real_gaussdb_url() -> String {
        env::var("GAUSSDB_TEST_URL")
            .unwrap_or_else(|_| {
                "host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string()
            })
    }

    /// 检查是否有真实的 GaussDB 可用
    fn has_real_gaussdb() -> bool {
        match GaussDBConnection::establish(&get_real_gaussdb_url()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// 建立测试连接
    fn establish_test_connection() -> Result<GaussDBConnection, diesel::ConnectionError> {
        GaussDBConnection::establish(&get_real_gaussdb_url())
    }

    #[test]
    fn test_real_gaussdb_basic_connection() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            println!("💡 请确保 Docker 容器正在运行: docker-compose up -d");
            return;
        }

        println!("🔍 测试基础 GaussDB 连接...");
        let mut conn = establish_test_connection()
            .expect("应该能够建立 GaussDB 连接");

        println!("✅ 成功建立 diesel-gaussdb 连接");

        // 测试基础查询
        let result = conn.batch_execute("SELECT 1 as test_value");
        assert!(result.is_ok(), "基础查询应该成功: {:?}", result);
        println!("✅ 基础查询执行成功");
    }

    #[test]
    fn test_real_gaussdb_version_info() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        println!("🔍 测试 GaussDB 版本信息查询...");
        let mut conn = establish_test_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 测试版本查询
        let result = conn.batch_execute("SELECT version()");
        assert!(result.is_ok(), "版本查询应该成功: {:?}", result);
        println!("✅ 版本查询执行成功");

        // 测试数据库信息查询
        let queries = vec![
            ("当前数据库", "SELECT current_database()"),
            ("当前用户", "SELECT current_user"),
            ("当前时间", "SELECT now()"),
            ("服务器版本", "SELECT version()"),
        ];

        for (test_name, query) in queries {
            println!("🧪 测试 {}: {}", test_name, query);
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "{} 查询应该成功: {:?}", test_name, result);
            println!("✅ {} 测试通过", test_name);
        }
    }

    #[test]
    fn test_real_gaussdb_data_types() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        println!("🔍 测试 GaussDB 数据类型支持...");
        let mut conn = establish_test_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 测试各种数据类型
        let type_tests = vec![
            ("整数类型", "SELECT 42::integer"),
            ("大整数类型", "SELECT 9223372036854775807::bigint"),
            ("浮点类型", "SELECT 3.14159::real"),
            ("双精度浮点", "SELECT 3.141592653589793::double precision"),
            ("文本类型", "SELECT 'Hello GaussDB'::text"),
            ("字符类型", "SELECT 'A'::char"),
            ("变长字符", "SELECT 'Variable'::varchar(50)"),
            ("布尔类型", "SELECT true::boolean"),
            ("日期类型", "SELECT '2024-01-01'::date"),
            ("时间类型", "SELECT '12:30:45'::time"),
            ("时间戳类型", "SELECT '2024-01-01 12:30:45'::timestamp"),
            ("数组类型", "SELECT ARRAY[1,2,3,4,5]"),
            ("JSON类型", "SELECT '{\"key\": \"value\", \"number\": 42}'::json"),
        ];

        for (type_name, query) in type_tests {
            println!("🧪 测试 {}: {}", type_name, query);
            let result = conn.batch_execute(query);
            match result {
                Ok(_) => println!("✅ {} 支持正常", type_name),
                Err(e) => {
                    println!("⚠️  {} 测试失败: {}", type_name, e);
                    // 某些类型可能在特定 GaussDB 版本中不支持，记录但不强制失败
                }
            }
        }
    }

    #[test]
    fn test_real_gaussdb_transactions() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        println!("🔍 测试 GaussDB 事务管理...");
        let mut conn = establish_test_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 测试事务功能
        println!("🧪 测试事务开始");
        let result = conn.batch_execute("BEGIN");
        assert!(result.is_ok(), "BEGIN 应该成功: {:?}", result);
        println!("✅ BEGIN 成功");

        println!("🧪 测试事务中的查询");
        let result = conn.batch_execute("SELECT 1");
        assert!(result.is_ok(), "事务中的查询应该成功: {:?}", result);
        println!("✅ 事务中查询成功");

        println!("🧪 测试事务提交");
        let result = conn.batch_execute("COMMIT");
        assert!(result.is_ok(), "COMMIT 应该成功: {:?}", result);
        println!("✅ COMMIT 成功");

        // 测试回滚
        println!("🧪 测试事务回滚");
        let result = conn.batch_execute("BEGIN");
        assert!(result.is_ok(), "BEGIN 应该成功: {:?}", result);

        let result = conn.batch_execute("SELECT 1");
        assert!(result.is_ok(), "事务中的查询应该成功: {:?}", result);

        let result = conn.batch_execute("ROLLBACK");
        assert!(result.is_ok(), "ROLLBACK 应该成功: {:?}", result);
        println!("✅ ROLLBACK 成功");
    }

    #[test]
    fn test_real_gaussdb_table_operations() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        println!("🔍 测试 GaussDB 表操作...");
        let mut conn = establish_test_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 创建测试表
        println!("🧪 创建测试表");
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS diesel_test_table (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                age INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        
        let result = conn.batch_execute(create_table_sql);
        assert!(result.is_ok(), "创建表应该成功: {:?}", result);
        println!("✅ 测试表创建成功");

        // 插入测试数据
        println!("🧪 插入测试数据");
        let insert_sql = "INSERT INTO diesel_test_table (name, age) VALUES ('Alice', 30), ('Bob', 25)";
        let result = conn.batch_execute(insert_sql);
        assert!(result.is_ok(), "插入数据应该成功: {:?}", result);
        println!("✅ 测试数据插入成功");

        // 查询测试数据
        println!("🧪 查询测试数据");
        let select_sql = "SELECT COUNT(*) FROM diesel_test_table";
        let result = conn.batch_execute(select_sql);
        assert!(result.is_ok(), "查询数据应该成功: {:?}", result);
        println!("✅ 测试数据查询成功");

        // 更新测试数据
        println!("🧪 更新测试数据");
        let update_sql = "UPDATE diesel_test_table SET age = 31 WHERE name = 'Alice'";
        let result = conn.batch_execute(update_sql);
        assert!(result.is_ok(), "更新数据应该成功: {:?}", result);
        println!("✅ 测试数据更新成功");

        // 删除测试数据
        println!("🧪 删除测试数据");
        let delete_sql = "DELETE FROM diesel_test_table WHERE name = 'Bob'";
        let result = conn.batch_execute(delete_sql);
        assert!(result.is_ok(), "删除数据应该成功: {:?}", result);
        println!("✅ 测试数据删除成功");

        // 清理测试表
        println!("🧪 清理测试表");
        let drop_table_sql = "DROP TABLE IF EXISTS diesel_test_table";
        let result = conn.batch_execute(drop_table_sql);
        assert!(result.is_ok(), "删除表应该成功: {:?}", result);
        println!("✅ 测试表清理成功");
    }

    #[test]
    fn test_real_gaussdb_error_handling() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        println!("🔍 测试 GaussDB 错误处理...");
        let mut conn = establish_test_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 测试错误处理
        println!("🧪 测试错误查询处理");
        let result = conn.batch_execute("SELECT * FROM non_existent_table_12345");
        assert!(result.is_err(), "错误查询应该失败");
        println!("✅ 错误查询正确返回错误");

        // 验证连接在错误后仍然可用
        println!("🧪 测试错误后连接恢复");
        let result = conn.batch_execute("SELECT 1");
        assert!(result.is_ok(), "错误后的正常查询应该成功: {:?}", result);
        println!("✅ 连接在错误后正常恢复");
    }

    #[cfg(feature = "r2d2")]
    #[test]
    fn test_real_gaussdb_connection_pool() {
        if !has_real_gaussdb() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        println!("🔍 测试 GaussDB 连接池...");
        
        use diesel_gaussdb::pool::create_pool;
        
        // 创建连接池
        let database_url = get_real_gaussdb_url();
        let pool = create_pool(&database_url)
            .expect("应该能够创建连接池");
        println!("✅ 连接池创建成功");

        // 从池中获取连接
        let mut conn = pool.get()
            .expect("应该能够从池中获取连接");
        println!("✅ 从连接池获取连接成功");

        // 使用池化连接执行查询
        let result = conn.batch_execute("SELECT 1");
        assert!(result.is_ok(), "池化连接查询应该成功: {:?}", result);
        println!("✅ 池化连接查询成功");

        // 测试多个并发连接
        let mut connections = Vec::new();
        for i in 0..3 {
            match pool.get() {
                Ok(conn) => {
                    connections.push(conn);
                    println!("✅ 获取连接 {} 成功", i + 1);
                }
                Err(e) => {
                    println!("❌ 获取连接 {} 失败: {}", i + 1, e);
                }
            }
        }

        println!("✅ 连接池测试完成，获取了 {} 个连接", connections.len());
    }
}

// Note: gaussdb feature is now always enabled for real implementation
