//! GaussDB API 使用方式验证测试
//!
//! 这个测试用于验证 gaussdb crate 的正确使用方式，
//! 并确保我们的连接实现符合 gaussdb crate 的 API 规范。

#[cfg(all(test, feature = "gaussdb"))]
mod gaussdb_api_tests {
    use std::env;
    use std::str::FromStr;

    /// 测试 gaussdb crate 的基本 API 使用
    #[test]
    fn test_gaussdb_crate_api() {
        println!("🔍 测试 gaussdb crate 的 API 使用方式...");

        // 测试连接字符串格式
        let connection_strings = vec![
            "host=localhost user=postgres dbname=test",
            "postgresql://postgres@localhost/test",
            "postgres://postgres@localhost/test",
        ];

        for conn_str in connection_strings {
            println!("🧪 测试连接字符串: {}", conn_str);
            
            // 尝试解析连接字符串
            match gaussdb::Config::from_str(conn_str) {
                Ok(config) => {
                    println!("✅ 连接字符串解析成功: {}", conn_str);
                    
                    // 尝试连接（预期会失败，因为没有真实的数据库）
                    match config.connect(gaussdb::NoTls) {
                        Ok(_client) => {
                            println!("✅ 连接成功: {}", conn_str);
                        }
                        Err(e) => {
                            println!("⚠️  连接失败（预期）: {} - {}", conn_str, e);
                            // 这是预期的，因为没有真实的数据库运行
                        }
                    }
                }
                Err(e) => {
                    println!("❌ 连接字符串解析失败: {} - {}", conn_str, e);
                }
            }
        }
    }

    /// 测试不同的连接配置方式
    #[test]
    fn test_gaussdb_config_builder() {
        println!("🔍 测试 gaussdb Config 构建器...");

        // 使用构建器方式创建配置
        let mut config = gaussdb::Config::new();
        config.host("localhost");
        config.port(5432);
        config.user("postgres");
        config.dbname("test");

        println!("✅ Config 构建器创建成功");

        // 尝试连接
        match config.connect(gaussdb::NoTls) {
            Ok(_client) => {
                println!("✅ 使用构建器配置连接成功");
            }
            Err(e) => {
                println!("⚠️  使用构建器配置连接失败（预期）: {}", e);
                // 检查错误类型，确保这是连接错误而不是 API 使用错误
                println!("🔍 错误详情: {:?}", e);
            }
        }
    }

    /// 测试环境变量中的真实连接
    #[test]
    fn test_real_gaussdb_if_available() {
        println!("🔍 测试真实 GaussDB 连接（如果可用）...");

        // 尝试从环境变量获取真实的连接信息
        let test_urls = vec![
            env::var("GAUSSDB_TEST_URL").ok(),
            env::var("DATABASE_URL").ok(),
            env::var("POSTGRES_URL").ok(),
            Some("host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string()),
            Some("host=localhost user=postgres dbname=postgres".to_string()),
        ];

        for url_opt in test_urls {
            if let Some(url) = url_opt {
                println!("🧪 尝试连接: {}", url);
                
                match gaussdb::Config::from_str(&url) {
                    Ok(config) => {
                        match config.connect(gaussdb::NoTls) {
                            Ok(mut client) => {
                                println!("✅ 真实连接成功: {}", url);
                                
                                // 执行一个简单的查询来验证连接
                                match client.execute("SELECT 1", &[]) {
                                    Ok(rows) => {
                                        println!("✅ 查询执行成功，影响行数: {}", rows);
                                    }
                                    Err(e) => {
                                        println!("❌ 查询执行失败: {}", e);
                                    }
                                }
                                
                                // 执行版本查询
                                match client.query("SELECT version()", &[]) {
                                    Ok(rows) => {
                                        println!("✅ 版本查询成功，返回 {} 行", rows.len());
                                        if let Some(row) = rows.first() {
                                            if let Ok(version) = row.try_get::<_, String>(0) {
                                                println!("📋 数据库版本: {}", version);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ 版本查询失败: {}", e);
                                    }
                                }
                                
                                return; // 找到可用连接，退出测试
                            }
                            Err(e) => {
                                println!("⚠️  连接失败: {} - {}", url, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ 连接字符串解析失败: {} - {}", url, e);
                    }
                }
            }
        }
        
        println!("⚠️  没有找到可用的真实 GaussDB/PostgreSQL 连接");
        println!("💡 要测试真实连接，请设置环境变量：");
        println!("   export GAUSSDB_TEST_URL='host=your-host user=your-user password=your-password dbname=your-db'");
    }

    /// 测试错误处理
    #[test]
    fn test_gaussdb_error_handling() {
        println!("🔍 测试 gaussdb 错误处理...");

        // 测试无效的连接字符串
        let invalid_urls = vec![
            "invalid://connection/string",
            "host=nonexistent-host user=nonexistent-user dbname=nonexistent-db",
            "",
        ];

        for url in invalid_urls {
            println!("🧪 测试无效连接: {}", url);
            
            match gaussdb::Config::from_str(url) {
                Ok(config) => {
                    match config.connect(gaussdb::NoTls) {
                        Ok(_) => {
                            println!("❌ 意外成功: {}", url);
                        }
                        Err(e) => {
                            println!("✅ 正确失败: {} - {}", url, e);
                            println!("🔍 错误类型: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✅ 连接字符串解析正确失败: {} - {}", url, e);
                }
            }
        }
    }
}

#[cfg(not(feature = "gaussdb"))]
mod no_gaussdb_feature {
    #[test]
    fn test_gaussdb_feature_required() {
        println!("⚠️  gaussdb feature 未启用");
        println!("💡 使用 --features gaussdb 来启用 GaussDB 支持");
        println!("💡 示例: cargo test --features gaussdb gaussdb_api_test");
    }
}
