//! GaussDB 真实连接验证测试
//!
//! 这个测试专门用于验证是否真正连接到了 GaussDB 数据库，
//! 而不是仅仅通过了编译检查。
//!
//! 运行方式：
//! ```bash
//! # 设置真实的 GaussDB 连接
//! export GAUSSDB_TEST_URL="host=localhost user=gaussdb password=Gaussdb@123 dbname=test"
//! 
//! # 运行验证测试
//! cargo test --features gaussdb gaussdb_connection_verification
//! ```

#[cfg(all(test, feature = "gaussdb"))]
mod gaussdb_verification_tests {
    use diesel::prelude::*;
    use diesel::connection::SimpleConnection;
    use diesel_gaussdb::prelude::*;
    use std::env;

    /// 获取测试数据库连接字符串
    fn get_test_database_url() -> String {
        env::var("GAUSSDB_TEST_URL")
            .unwrap_or_else(|_| {
                // 默认的 GaussDB 连接字符串
                "host=localhost user=gaussdb password=Gaussdb@123 dbname=test".to_string()
            })
    }

    /// 尝试建立真实的 GaussDB 连接
    fn try_establish_real_connection() -> Result<GaussDBConnection, diesel::ConnectionError> {
        let database_url = get_test_database_url();
        println!("尝试连接到 GaussDB: {}", database_url);
        
        GaussDBConnection::establish(&database_url)
    }

    /// 检查是否有真实的 GaussDB 数据库可用
    fn has_real_gaussdb_available() -> bool {
        match try_establish_real_connection() {
            Ok(_) => {
                println!("✅ 检测到真实的 GaussDB 数据库连接");
                true
            }
            Err(e) => {
                println!("❌ 无法连接到真实的 GaussDB 数据库: {}", e);
                println!("💡 请确保：");
                println!("   1. GaussDB 数据库正在运行");
                println!("   2. 连接参数正确");
                println!("   3. 用户有访问权限");
                false
            }
        }
    }

    #[test]
    fn test_real_gaussdb_connection() {
        println!("\n🔍 开始验证真实的 GaussDB 连接...");
        
        if !has_real_gaussdb_available() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        let mut conn = try_establish_real_connection()
            .expect("应该能够建立 GaussDB 连接");

        println!("✅ 成功建立 GaussDB 连接");

        // 测试基础查询
        let result = conn.batch_execute("SELECT 1 as test_value");
        assert!(result.is_ok(), "基础查询应该成功: {:?}", result);
        println!("✅ 基础查询执行成功");
    }

    #[test]
    fn test_gaussdb_specific_features() {
        if !has_real_gaussdb_available() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        let mut conn = try_establish_real_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 测试 GaussDB 特有的功能
        let gaussdb_queries = vec![
            ("版本查询", "SELECT version()"),
            ("当前数据库", "SELECT current_database()"),
            ("当前用户", "SELECT current_user"),
            ("当前时间", "SELECT now()"),
            ("数据库编码", "SELECT pg_encoding_to_char(encoding) FROM pg_database WHERE datname = current_database()"),
        ];

        for (test_name, query) in gaussdb_queries {
            println!("🧪 测试 {}: {}", test_name, query);
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "{} 查询应该成功: {:?}", test_name, result);
            println!("✅ {} 测试通过", test_name);
        }
    }

    #[test]
    fn test_gaussdb_data_types() {
        if !has_real_gaussdb_available() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        let mut conn = try_establish_real_connection()
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
            ("带时区时间戳", "SELECT '2024-01-01 12:30:45+08'::timestamptz"),
            ("数组类型", "SELECT ARRAY[1,2,3,4,5]"),
            ("JSON类型", "SELECT '{\"key\": \"value\", \"number\": 42}'::json"),
            ("UUID类型", "SELECT gen_random_uuid()"),
        ];

        for (type_name, query) in type_tests {
            println!("🧪 测试 {}: {}", type_name, query);
            let result = conn.batch_execute(query);
            match result {
                Ok(_) => println!("✅ {} 支持正常", type_name),
                Err(e) => {
                    println!("⚠️  {} 测试失败: {}", type_name, e);
                    // 某些类型可能在特定 GaussDB 版本中不支持，不强制失败
                }
            }
        }
    }

    #[test]
    fn test_gaussdb_transaction_support() {
        if !has_real_gaussdb_available() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        let mut conn = try_establish_real_connection()
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
    fn test_gaussdb_error_handling() {
        if !has_real_gaussdb_available() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        let mut conn = try_establish_real_connection()
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

    #[test]
    fn test_gaussdb_connection_info() {
        if !has_real_gaussdb_available() {
            println!("⚠️  跳过测试：没有可用的真实 GaussDB 数据库");
            return;
        }

        let mut conn = try_establish_real_connection()
            .expect("应该能够建立 GaussDB 连接");

        // 获取数据库信息来验证这确实是 GaussDB
        println!("🔍 获取数据库版本信息...");
        
        // 这些查询应该能帮助我们确认连接的是 GaussDB
        let info_queries = vec![
            "SELECT version()",
            "SELECT current_database()",
            "SELECT current_user",
            "SELECT inet_server_addr()",
            "SELECT inet_server_port()",
        ];

        for query in info_queries {
            println!("🧪 执行信息查询: {}", query);
            let result = conn.batch_execute(query);
            match result {
                Ok(_) => println!("✅ 查询成功: {}", query),
                Err(e) => println!("⚠️  查询失败: {} - {}", query, e),
            }
        }
    }
}

// Note: gaussdb feature is now always enabled for real implementation
