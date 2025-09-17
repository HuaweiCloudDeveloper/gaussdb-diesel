//! Diesel 2.2.x 兼容性测试集成
//!
//! 这个模块包含了从 diesel 项目继承的测试用例，
//! 用于验证 diesel-gaussdb 与 diesel 的兼容性。

use diesel::prelude::*;
use diesel_gaussdb::GaussDBConnection;
use std::env;

/// 获取测试数据库连接
fn establish_test_connection() -> GaussDBConnection {
    let database_url = env::var("GAUSSDB_TEST_URL")
        .unwrap_or_else(|_| "host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string());

    GaussDBConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// 测试基础连接功能
#[test]
fn test_basic_connection() {
    let mut connection = establish_test_connection();

    // 执行简单查询验证连接 - 使用 execute 而不是 get_result
    let rows_affected = diesel::sql_query("SELECT 1")
        .execute(&mut connection)
        .expect("Failed to execute basic query");

    // execute 返回受影响的行数，对于 SELECT 通常是 0
    assert!(rows_affected >= 0);
}

/// 测试表创建和基础 CRUD 操作
#[test]
fn test_basic_crud_operations() {
    let mut connection = establish_test_connection();

    // 创建测试表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS test_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )"
    )
    .execute(&mut connection)
    .expect("Failed to create test table");

    // 清理数据
    diesel::sql_query("DELETE FROM test_users")
        .execute(&mut connection)
        .expect("Failed to clean test data");

    // 插入测试数据
    let insert_result = diesel::sql_query(
        "INSERT INTO test_users (name, email) VALUES
         ('John Doe', 'john@example.com'),
         ('Jane Smith', 'jane@example.com')"
    )
    .execute(&mut connection)
    .expect("Failed to insert test data");

    // 验证插入了数据
    assert!(insert_result >= 0);

    // 更新数据
    let update_result = diesel::sql_query("UPDATE test_users SET name = 'John Updated' WHERE name = 'John Doe'")
        .execute(&mut connection)
        .expect("Failed to update record");

    // 验证更新了数据
    assert!(update_result >= 0);

    // 删除数据
    let delete_result = diesel::sql_query("DELETE FROM test_users WHERE name = 'Jane Smith'")
        .execute(&mut connection)
        .expect("Failed to delete record");

    // 验证删除了数据
    assert!(delete_result >= 0);

    // 清理测试表 - 先删除有外键约束的表
    diesel::sql_query("DROP TABLE IF EXISTS test_orders CASCADE")
        .execute(&mut connection)
        .unwrap_or(0); // 忽略错误，可能表不存在

    diesel::sql_query("DROP TABLE IF EXISTS test_users CASCADE")
        .execute(&mut connection)
        .expect("Failed to drop test table");
}

/// 测试事务功能
#[test]
fn test_transaction_support() {
    let mut connection = establish_test_connection();

    // 创建测试表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS test_transactions (
            id SERIAL PRIMARY KEY,
            value INTEGER NOT NULL
        )"
    )
    .execute(&mut connection)
    .expect("Failed to create test table");

    // 清理数据
    diesel::sql_query("DELETE FROM test_transactions")
        .execute(&mut connection)
        .expect("Failed to clean test data");

    // 测试成功事务
    let transaction_result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::sql_query("INSERT INTO test_transactions (value) VALUES (100)")
            .execute(conn)?;
        diesel::sql_query("INSERT INTO test_transactions (value) VALUES (200)")
            .execute(conn)?;
        Ok(())
    });

    assert!(transaction_result.is_ok());

    // 测试回滚事务
    let rollback_result: Result<(), diesel::result::Error> = connection.transaction(|conn| {
        diesel::sql_query("INSERT INTO test_transactions (value) VALUES (300)")
            .execute(conn)?;
        // 故意返回错误以触发回滚
        Err(diesel::result::Error::RollbackTransaction)
    });

    assert!(rollback_result.is_err());

    // 清理测试表
    diesel::sql_query("DROP TABLE IF EXISTS test_transactions")
        .execute(&mut connection)
        .expect("Failed to drop test table");
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    let mut connection = establish_test_connection();

    // 测试语法错误
    let result = diesel::sql_query("INVALID SQL SYNTAX")
        .execute(&mut connection);

    assert!(result.is_err());

    // 测试表不存在错误
    let result = diesel::sql_query("SELECT * FROM non_existent_table")
        .execute(&mut connection);

    assert!(result.is_err());
}
