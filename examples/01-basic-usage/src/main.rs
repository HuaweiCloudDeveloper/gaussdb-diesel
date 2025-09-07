//! GaussDB Diesel 基础使用示例
//!
//! 这个示例展示了如何使用 diesel-gaussdb 进行基本的数据库操作

use diesel::prelude::*;
use diesel_gaussdb::GaussDBConnection;
use anyhow::{Result, Context};
use log::info;
use std::env;

/// 建立数据库连接
fn establish_connection() -> Result<GaussDBConnection> {
    let database_url = env::var("GAUSSDB_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres".to_string()
        });

    info!("连接到数据库: {}", database_url);

    GaussDBConnection::establish(&database_url)
        .with_context(|| format!("Error connecting to {}", database_url))
}

/// 用于查询结果的结构体
#[derive(Debug, diesel::QueryableByName)]
struct UserResult {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

/// 创建所有必要的表
fn create_tables(conn: &mut GaussDBConnection) -> Result<()> {
    info!("创建数据库表...");

    // 创建用户表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE,
            age INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        )"
    ).execute(conn).context("Failed to create users table")?;

    // 创建文章表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            author_id INTEGER NOT NULL,
            published BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP,
            FOREIGN KEY (author_id) REFERENCES users(id)
        )"
    ).execute(conn).context("Failed to create posts table")?;

    // 创建评论表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS comments (
            id SERIAL PRIMARY KEY,
            post_id INTEGER NOT NULL,
            author_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (post_id) REFERENCES posts(id),
            FOREIGN KEY (author_id) REFERENCES users(id)
        )"
    ).execute(conn).context("Failed to create comments table")?;

    // 创建标签表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS tags (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL UNIQUE,
            color VARCHAR,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(conn).context("Failed to create tags table")?;

    // 创建文章标签关联表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS post_tags (
            id SERIAL PRIMARY KEY,
            post_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (post_id) REFERENCES posts(id),
            FOREIGN KEY (tag_id) REFERENCES tags(id),
            UNIQUE(post_id, tag_id)
        )"
    ).execute(conn).context("Failed to create post_tags table")?;

    info!("✅ 所有表创建成功！");
    Ok(())
}

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    info!("🚀 启动 Diesel-GaussDB 基础使用示例");

    // 建立数据库连接
    let mut connection = establish_connection()?;
    info!("✅ 数据库连接成功！");

    // 创建表
    create_tables(&mut connection)?;

    // 演示基础 CRUD 操作
    demo_basic_crud(&mut connection)?;

    // 演示事务处理
    demo_transactions(&mut connection)?;

    info!("🎉 所有示例演示完成！");
    Ok(())
}

/// 演示基础 CRUD 操作
fn demo_basic_crud(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n📋 === 基础 CRUD 操作演示 ===");

    // 清理现有数据
    info!("清理现有数据...");
    diesel::sql_query("DELETE FROM users").execute(conn)?;
    info!("✅ 数据清理完成");

    // 1. 创建用户 (Create)
    info!("1. 创建用户...");
    diesel::sql_query(
        "INSERT INTO users (name, email, age) VALUES
         ('张三', 'zhangsan@example.com', 25),
         ('李四', 'lisi@example.com', 30),
         ('王五', 'wangwu@example.com', NULL)"
    ).execute(conn)?;

    info!("✅ 成功创建用户");

    // 2. 查询用户 (Read)
    info!("\n2. 查询用户...");
    let all_users: Vec<UserResult> = diesel::sql_query(
        "SELECT id, name, email FROM users ORDER BY id"
    ).load(conn)?;

    info!("✅ 查询到 {} 个用户", all_users.len());
    for user in &all_users {
        info!("  - ID: {}, 姓名: {}, 邮箱: {}", user.id, user.name, user.email);
    }

    // 条件查询
    let adult_users: Vec<UserResult> = diesel::sql_query(
        "SELECT id, name, email FROM users WHERE age >= 18"
    ).load(conn)?;

    info!("✅ 成年用户数量: {}", adult_users.len());

    // 3. 更新用户 (Update)
    info!("\n3. 更新用户信息...");
    let updated_count = diesel::sql_query(
        "UPDATE users SET name = '张三（已更新）' WHERE name = '张三'"
    ).execute(conn)?;

    info!("✅ 成功更新 {} 个用户", updated_count);

    // 验证更新
    let updated_users: Vec<UserResult> = diesel::sql_query(
        "SELECT id, name, email FROM users WHERE name LIKE '%已更新%'"
    ).load(conn)?;

    for user in &updated_users {
        info!("  更新后的用户: {}", user.name);
    }

    // 4. 删除用户 (Delete)
    info!("\n4. 删除用户...");

    // 先插入一个临时用户
    diesel::sql_query(
        "INSERT INTO users (name, email, age) VALUES ('临时用户', 'temp@example.com', 20)"
    ).execute(conn)?;

    // 删除临时用户
    let deleted_count = diesel::sql_query(
        "DELETE FROM users WHERE name = '临时用户'"
    ).execute(conn)?;

    info!("✅ 成功删除 {} 个用户", deleted_count);

    // 最终统计
    let final_count: CountResult = diesel::sql_query(
        "SELECT COUNT(*) as count FROM users"
    ).get_result(conn)?;

    info!("✅ 最终用户数量: {}", final_count.count);

    Ok(())
}

/// 演示事务处理
fn demo_transactions(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🔄 === 事务处理演示 ===");

    // 1. 成功的事务
    info!("1. 执行成功的事务...");
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::sql_query(
            "INSERT INTO users (name, email, age) VALUES ('事务用户1', 'transaction1@example.com', 28)"
        ).execute(conn)?;

        info!("  ✅ 事务中的用户创建成功");
        Ok(())
    })?;

    info!("✅ 事务提交成功");

    // 2. 回滚的事务
    info!("\n2. 执行会回滚的事务...");
    let result: Result<(), diesel::result::Error> = conn.transaction(|conn| {
        diesel::sql_query(
            "INSERT INTO users (name, email, age) VALUES ('事务用户2', 'transaction2@example.com', 30)"
        ).execute(conn)?;

        // 故意触发错误以回滚事务
        Err(diesel::result::Error::RollbackTransaction)
    });

    match result {
        Ok(_) => info!("⚠️  事务应该失败但却成功了"),
        Err(_) => info!("✅ 事务按预期回滚"),
    }

    // 验证事务结果
    let transaction_users: Vec<UserResult> = diesel::sql_query(
        "SELECT id, name, email FROM users WHERE name LIKE '事务用户%'"
    ).load(conn)?;

    info!("✅ 事务后用户数量: {}", transaction_users.len());
    for user in &transaction_users {
        info!("  事务用户: {}", user.name);
    }

    Ok(())
}
