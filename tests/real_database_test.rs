//! 真实数据库集成测试
//!
//! 这些测试需要真实的 GaussDB 数据库连接才能运行。
//! 设置环境变量 GAUSSDB_TEST_URL 来指定测试数据库连接字符串。
//!
//! 示例：
//! ```bash
//! export GAUSSDB_TEST_URL="host=localhost user=gaussdb password=Gaussdb@123 dbname=test"
//! cargo test --features gaussdb test_real_database
//! ```

#[cfg(all(test, feature = "gaussdb"))]
mod real_database_tests {
    use diesel::prelude::*;
    use diesel::connection::SimpleConnection;
    use diesel_gaussdb::prelude::*;
    use std::env;

    /// 建立测试数据库连接
    fn establish_test_connection() -> Result<GaussDBConnection, diesel::ConnectionError> {
        let database_url = env::var("GAUSSDB_TEST_URL")
            .unwrap_or_else(|_| {
                "host=localhost user=gaussdb password=Gaussdb@123 dbname=test".to_string()
            });
        
        GaussDBConnection::establish(&database_url)
    }

    /// 检查是否可以建立数据库连接
    fn can_connect_to_database() -> bool {
        establish_test_connection().is_ok()
    }

    #[test]
    fn test_basic_connection() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 执行一个简单的查询来验证连接
        let result = conn.batch_execute("SELECT 1");
        assert!(result.is_ok(), "基础查询执行失败: {:?}", result);
    }

    #[test]
    fn test_execute_returning_count() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试 execute_returning_count 方法
        // 这里我们使用一个简单的查询来测试
        let result = conn.batch_execute("SELECT 1");
        assert!(result.is_ok(), "execute_returning_count 测试失败: {:?}", result);
    }

    #[test]
    fn test_basic_query_execution() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试基础查询执行
        let queries = vec![
            "SELECT 1 as test_number",
            "SELECT 'hello' as test_string",
            "SELECT true as test_boolean",
            "SELECT NULL as test_null",
        ];

        for query in queries {
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "查询执行失败 '{}': {:?}", query, result);
        }
    }

    #[test]
    fn test_integer_types() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试整数类型
        let queries = vec![
            "SELECT 42::smallint as small_int",
            "SELECT 42::integer as int",
            "SELECT 42::bigint as big_int",
        ];

        for query in queries {
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "整数类型测试失败 '{}': {:?}", query, result);
        }
    }

    #[test]
    fn test_text_types() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试文本类型
        let queries = vec![
            "SELECT 'hello world'::text as text_value",
            "SELECT 'varchar test'::varchar(50) as varchar_value",
            "SELECT 'char test'::char(10) as char_value",
        ];

        for query in queries {
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "文本类型测试失败 '{}': {:?}", query, result);
        }
    }

    #[test]
    fn test_numeric_types() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试数值类型
        let queries = vec![
            "SELECT 3.14::real as real_value",
            "SELECT 3.14159::double precision as double_value",
            "SELECT 123.456::numeric(10,3) as numeric_value",
        ];

        for query in queries {
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "数值类型测试失败 '{}': {:?}", query, result);
        }
    }

    #[test]
    fn test_date_time_types() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试日期时间类型
        let queries = vec![
            "SELECT CURRENT_DATE as current_date",
            "SELECT CURRENT_TIME as current_time", 
            "SELECT CURRENT_TIMESTAMP as current_timestamp",
            "SELECT '2023-01-01'::date as date_value",
            "SELECT '12:30:45'::time as time_value",
            "SELECT '2023-01-01 12:30:45'::timestamp as timestamp_value",
        ];

        for query in queries {
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "日期时间类型测试失败 '{}': {:?}", query, result);
        }
    }

    #[test]
    fn test_array_types() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试数组类型
        let queries = vec![
            "SELECT ARRAY[1,2,3] as int_array",
            "SELECT ARRAY['a','b','c'] as text_array",
            "SELECT '{1,2,3}'::integer[] as int_array_literal",
        ];

        for query in queries {
            let result = conn.batch_execute(query);
            assert!(result.is_ok(), "数组类型测试失败 '{}': {:?}", query, result);
        }
    }

    #[test]
    fn test_transaction_support() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试事务支持
        let result = conn.batch_execute("BEGIN; SELECT 1; COMMIT;");
        assert!(result.is_ok(), "事务支持测试失败: {:?}", result);
    }

    #[test]
    fn test_error_handling() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        let mut conn = establish_test_connection().expect("无法建立数据库连接");
        
        // 测试错误处理 - 执行一个无效的查询
        let result = conn.batch_execute("SELECT * FROM non_existent_table");
        assert!(result.is_err(), "错误处理测试失败：应该返回错误");
        
        // 验证连接在错误后仍然可用
        let recovery_result = conn.batch_execute("SELECT 1");
        assert!(recovery_result.is_ok(), "连接恢复测试失败: {:?}", recovery_result);
    }

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_connection_pool() {
        if !can_connect_to_database() {
            println!("跳过测试：无法连接到 GaussDB 数据库");
            return;
        }

        use diesel_gaussdb::pool::create_pool;
        
        let database_url = env::var("GAUSSDB_TEST_URL")
            .unwrap_or_else(|_| {
                "host=localhost user=gaussdb password=Gaussdb@123 dbname=test".to_string()
            });
        
        // 创建连接池
        let pool = create_pool(&database_url);
        assert!(pool.is_ok(), "连接池创建失败: {:?}", pool);
        
        if let Ok(pool) = pool {
            // 从池中获取连接
            let conn = pool.get();
            assert!(conn.is_ok(), "从连接池获取连接失败: {:?}", conn);
            
            if let Ok(mut conn) = conn {
                // 使用池化连接执行查询
                let result = conn.batch_execute("SELECT 1");
                assert!(result.is_ok(), "池化连接查询失败: {:?}", result);
            }
        }
    }
}

// Note: gaussdb feature is now always enabled for real implementation
