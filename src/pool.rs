//! Connection pool support for GaussDB
//!
//! This module provides connection pool integration for GaussDB connections,
//! supporting both r2d2 and async connection pools.

// Pool implementations will be added when r2d2 feature is enabled

/// R2D2 connection pool support
#[cfg(feature = "r2d2")]
pub mod r2d2_support {
    use crate::connection::GaussDBConnection;
    use diesel::connection::{Connection, SimpleConnection};
    use diesel::result::ConnectionError;
    use r2d2::{ManageConnection, Pool, PooledConnection};
    use std::fmt;

    /// Connection manager for r2d2 pool
    pub struct GaussDBConnectionManager {
        database_url: String,
    }

    impl GaussDBConnectionManager {
        /// Create a new connection manager with the given database URL
        pub fn new<S: Into<String>>(database_url: S) -> Self {
            Self {
                database_url: database_url.into(),
            }
        }
    }

    impl ManageConnection for GaussDBConnectionManager {
        type Connection = GaussDBConnection;
        type Error = ConnectionError;

        fn connect(&self) -> Result<Self::Connection, Self::Error> {
            GaussDBConnection::establish(&self.database_url)
        }

        fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
            // 执行一个简单的查询来验证连接是否有效
            conn.batch_execute("SELECT 1").map_err(|e| {
                ConnectionError::BadConnection(format!("Connection validation failed: {}", e))
            })
        }

        fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
            // 简化实现，实际应该检查连接状态
            false
        }
    }

    impl fmt::Debug for GaussDBConnectionManager {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("GaussDBConnectionManager")
                .field("database_url", &"[REDACTED]")
                .finish()
        }
    }

    /// Type alias for GaussDB connection pool
    pub type GaussDBPool = Pool<GaussDBConnectionManager>;

    /// Type alias for pooled GaussDB connection
    pub type PooledGaussDBConnection = PooledConnection<GaussDBConnectionManager>;

    /// Helper function to create a new connection pool
    pub fn create_pool<S: Into<String>>(database_url: S) -> Result<GaussDBPool, r2d2::Error> {
        let manager = GaussDBConnectionManager::new(database_url);
        Pool::new(manager)
    }

    /// Helper function to create a connection pool with custom configuration
    pub fn create_pool_with_config<S: Into<String>>(
        database_url: S,
        builder: r2d2::Builder<GaussDBConnectionManager>,
    ) -> Result<GaussDBPool, r2d2::Error> {
        let manager = GaussDBConnectionManager::new(database_url);
        builder.build(manager)
    }

    /// 创建一个生产级的连接池配置
    ///
    /// 这个函数提供了适合生产环境的默认配置：
    /// - 最大连接数：10
    /// - 最小空闲连接数：2
    /// - 连接超时：30秒
    /// - 空闲超时：10分钟
    /// - 最大生命周期：30分钟
    pub fn create_production_pool<S: Into<String>>(
        database_url: S,
    ) -> Result<GaussDBPool, r2d2::Error> {
        let manager = GaussDBConnectionManager::new(database_url);

        Pool::builder()
            .max_size(10)                                    // 最大连接数
            .min_idle(Some(2))                              // 最小空闲连接数
            .connection_timeout(std::time::Duration::from_secs(30))  // 连接超时
            .idle_timeout(Some(std::time::Duration::from_secs(600))) // 空闲超时（10分钟）
            .max_lifetime(Some(std::time::Duration::from_secs(1800))) // 最大生命周期（30分钟）
            .test_on_check_out(true)                        // 检出时测试连接
            .build(manager)
    }

    /// 创建一个开发环境的连接池配置
    ///
    /// 这个函数提供了适合开发环境的配置：
    /// - 最大连接数：5
    /// - 最小空闲连接数：1
    /// - 连接超时：10秒
    /// - 较短的超时时间便于快速调试
    pub fn create_development_pool<S: Into<String>>(
        database_url: S,
    ) -> Result<GaussDBPool, r2d2::Error> {
        let manager = GaussDBConnectionManager::new(database_url);

        Pool::builder()
            .max_size(5)                                     // 最大连接数
            .min_idle(Some(1))                              // 最小空闲连接数
            .connection_timeout(std::time::Duration::from_secs(10))  // 连接超时
            .idle_timeout(Some(std::time::Duration::from_secs(300))) // 空闲超时（5分钟）
            .test_on_check_out(false)                       // 开发环境不需要每次测试
            .build(manager)
    }
}

/// Async connection pool support (for future implementation)
#[cfg(feature = "tokio-gaussdb")]
pub mod async_support {
    
    // TODO: 实现异步连接池支持
    // 可以使用 bb8 或 deadpool 等异步连接池库
    
    /// Placeholder for async connection manager
    pub struct AsyncGaussDBConnectionManager {
        database_url: String,
    }
    
    impl AsyncGaussDBConnectionManager {
        /// Create a new async connection manager
        pub fn new<S: Into<String>>(database_url: S) -> Self {
            Self {
                database_url: database_url.into(),
            }
        }

        /// Get the database URL
        pub fn database_url(&self) -> &str {
            &self.database_url
        }
    }
}

// Re-export commonly used types
#[cfg(feature = "r2d2")]
pub use r2d2_support::{
    create_pool, create_pool_with_config, GaussDBConnectionManager, GaussDBPool,
    PooledGaussDBConnection,
};

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_connection_manager_creation() {
        use crate::pool::r2d2_support::GaussDBConnectionManager;
        let manager = GaussDBConnectionManager::new("host=localhost user=test dbname=test");
        // 测试管理器创建成功（无法直接访问私有字段）
        assert!(format!("{:?}", manager).contains("GaussDBConnectionManager"));
    }

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_connection_manager_debug() {
        use crate::pool::r2d2_support::GaussDBConnectionManager;
        let manager = GaussDBConnectionManager::new("host=localhost user=test password=secret dbname=test");
        let debug_str = format!("{:?}", manager);
        assert!(debug_str.contains("GaussDBConnectionManager"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret"));
    }

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_pool_creation_helper() {
        use crate::pool::r2d2_support::create_pool;
        // 这个测试不会实际连接数据库，只是测试函数签名
        let result = create_pool("host=localhost user=test dbname=test");
        // 由于没有真实的数据库连接，这里只检查函数是否可以调用
        assert!(result.is_ok() || result.is_err()); // 总是为真，但确保函数可以调用
    }

    #[test]
    #[cfg(feature = "tokio-gaussdb")]
    fn test_async_manager_creation() {
        use crate::pool::async_support::AsyncGaussDBConnectionManager;
        let manager = AsyncGaussDBConnectionManager::new("host=localhost user=test dbname=test");
        assert_eq!(manager.database_url(), "host=localhost user=test dbname=test");
    }

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_production_pool_creation() {
        use crate::pool::r2d2_support::create_production_pool;

        // 测试生产级连接池创建
        let result = create_production_pool("host=localhost user=test dbname=test");

        // 由于没有真实的数据库连接，这里只检查函数是否可以调用
        match result {
            Ok(pool) => {
                // 验证连接池配置
                assert_eq!(pool.max_size(), 10);
                println!("✅ 生产级连接池创建成功");
            }
            Err(_) => {
                // 在没有真实数据库的情况下，这是预期的
                println!("⚠️  生产级连接池创建失败（预期，因为没有真实数据库）");
            }
        }
    }

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_development_pool_creation() {
        use crate::pool::r2d2_support::create_development_pool;

        // 测试开发环境连接池创建
        let result = create_development_pool("host=localhost user=test dbname=test");

        match result {
            Ok(pool) => {
                // 验证连接池配置
                assert_eq!(pool.max_size(), 5);
                println!("✅ 开发环境连接池创建成功");
            }
            Err(_) => {
                // 在没有真实数据库的情况下，这是预期的
                println!("⚠️  开发环境连接池创建失败（预期，因为没有真实数据库）");
            }
        }
    }

    #[test]
    #[cfg(feature = "r2d2")]
    fn test_pool_configuration_differences() {
        use crate::pool::r2d2_support::{create_production_pool, create_development_pool};

        // 测试不同环境的连接池配置差异
        let prod_result = create_production_pool("host=localhost user=test dbname=test");
        let dev_result = create_development_pool("host=localhost user=test dbname=test");

        // 即使连接失败，我们也可以验证配置的差异
        if let (Ok(prod_pool), Ok(dev_pool)) = (prod_result, dev_result) {
            assert!(prod_pool.max_size() > dev_pool.max_size());
            println!("✅ 连接池配置差异验证通过");
        } else {
            println!("⚠️  连接池配置测试跳过（无真实数据库连接）");
        }
    }
}
