//! Integration tests for real database testing
//!
//! This module provides comprehensive integration tests for diesel-gaussdb
//! with real database connections when available.

use diesel_gaussdb::prelude::*;
use diesel::connection::SimpleConnection;

/// Test configuration for database connections
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("GAUSSDB_TEST_URL")
                .unwrap_or_else(|_| "gaussdb://gaussdb:Gaussdb@123@localhost:5432/test_db".to_string()),
        }
    }
}

/// Helper to establish a test connection
pub fn establish_test_connection() -> Result<GaussDBConnection, diesel::result::ConnectionError> {
    let config = TestConfig::default();
    GaussDBConnection::establish(&config.database_url)
}

/// Test basic database connection
#[tokio::test]
async fn test_real_database_connection() {
    // Test connection establishment
    let mut connection = match establish_test_connection() {
        Ok(conn) => conn,
        Err(e) => {
            println!("⚠️  Failed to establish connection: {}", e);
            println!("💡 This is expected if no real GaussDB is available");
            println!("💡 Set GAUSSDB_TEST_URL environment variable to test with real database");
            return;
        }
    };

    // Test basic query execution
    let result = connection.batch_execute("SELECT 1");
    assert!(result.is_ok(), "Basic query should succeed: {:?}", result);

    println!("✅ Real database connection test passed");
}

/// Test table creation and basic CRUD operations
#[tokio::test]
async fn test_crud_operations() {
    let mut connection = match establish_test_connection() {
        Ok(conn) => conn,
        Err(_) => {
            println!("⚠️  Skipping CRUD test - no database available");
            return;
        }
    };

    // Create test table
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS test_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    "#;

    connection.batch_execute(create_table_sql)
        .expect("Failed to create test table");

    // Insert test data
    let insert_sql = r#"
        INSERT INTO test_users (name, email) VALUES
        ('Alice Johnson', 'alice@example.com'),
        ('Bob Smith', 'bob@example.com')
        ON CONFLICT (email) DO NOTHING
    "#;

    connection.batch_execute(insert_sql)
        .expect("Failed to insert test data");

    // Test SELECT query
    let select_sql = "SELECT COUNT(*) FROM test_users";
    connection.batch_execute(select_sql)
        .expect("Failed to execute SELECT query");

    // Clean up
    connection.batch_execute("DROP TABLE IF EXISTS test_users")
        .expect("Failed to clean up test table");

    println!("✅ CRUD operations test passed");
}

/// Test transaction management
#[tokio::test]
async fn test_transaction_management() {
    let mut connection = match establish_test_connection() {
        Ok(conn) => conn,
        Err(_) => {
            println!("⚠️  Skipping transaction test - no database available");
            return;
        }
    };

    // Test transaction operations
    connection.batch_execute("BEGIN")
        .expect("Failed to begin transaction");

    connection.batch_execute("CREATE TEMPORARY TABLE tx_test (id INT)")
        .expect("Failed to create temp table in transaction");

    connection.batch_execute("ROLLBACK")
        .expect("Failed to rollback transaction");

    println!("✅ Transaction management test passed");
}

/// Test type system with real database
#[tokio::test]
async fn test_type_system_integration() {
    let mut connection = match establish_test_connection() {
        Ok(conn) => conn,
        Err(_) => {
            println!("⚠️  Skipping type system test - no database available");
            return;
        }
    };

    // Test various PostgreSQL-compatible types
    let type_tests = vec![
        "SELECT 42::INTEGER",
        "SELECT 'hello'::TEXT",
        "SELECT 3.14::NUMERIC",
        "SELECT TRUE::BOOLEAN",
        "SELECT CURRENT_TIMESTAMP::TIMESTAMP",
        "SELECT ARRAY[1,2,3]::INTEGER[]",
    ];

    for sql in type_tests {
        connection.batch_execute(sql)
            .unwrap_or_else(|e| panic!("Failed to execute type test '{}': {}", sql, e));
    }

    println!("✅ Type system integration test passed");
}

/// Test performance with real database
#[tokio::test]
async fn test_performance_baseline() {
    let mut connection = match establish_test_connection() {
        Ok(conn) => conn,
        Err(_) => {
            println!("⚠️  Skipping performance test - no database available");
            return;
        }
    };

    // Create test table for performance testing
    connection.batch_execute(r#"
        CREATE TEMPORARY TABLE perf_test (
            id SERIAL PRIMARY KEY,
            data TEXT
        )
    "#).expect("Failed to create performance test table");

    // Measure basic query performance
    let start = std::time::Instant::now();

    for i in 0..100 {
        let sql = format!("INSERT INTO perf_test (data) VALUES ('test_data_{}')", i);
        connection.batch_execute(&sql)
            .expect("Failed to insert performance test data");
    }

    let duration = start.elapsed();
    println!("✅ Performance test: 100 inserts in {:?}", duration);

    // Basic performance assertion (should complete within reasonable time)
    assert!(duration.as_secs() < 10, "Performance test took too long: {:?}", duration);
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = TestConfig::default();
        assert!(config.database_url.contains("gaussdb://"));
        assert!(config.database_url.contains("localhost") || config.database_url.contains("127.0.0.1"));
    }

    #[test]
    fn test_database_url_format() {
        let config = TestConfig::default();

        // Test that our URL has the expected format
        assert!(config.database_url.starts_with("gaussdb://"));
        assert!(config.database_url.contains("@"));
        assert!(config.database_url.contains(":"));
    }
}
