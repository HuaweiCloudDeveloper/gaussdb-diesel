//! Diesel-GaussDB 高级查询示例
//!
//! 这个示例展示了 diesel-gaussdb 的高级查询功能，包括：
//! - 窗口函数
//! - CTE (公共表表达式)
//! - 复杂子查询
//! - 聚合查询
//! - 联表查询

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

/// 查询结果结构体
#[derive(Debug, diesel::QueryableByName)]
struct UserPostStats {
    #[diesel(sql_type = diesel::sql_types::Text)]
    author: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    row_num: i32,
}

#[derive(Debug, diesel::QueryableByName)]
struct PostRank {
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    comment_count: i64,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    rank: i32,
}

#[derive(Debug, diesel::QueryableByName)]
struct UserActivity {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    post_count: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    comment_count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
struct TagStats {
    #[diesel(sql_type = diesel::sql_types::Text)]
    tag_name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    post_count: i64,
}

/// 创建示例数据
fn setup_sample_data(conn: &mut GaussDBConnection) -> Result<()> {
    info!("设置示例数据...");

    // 创建表
    create_tables(conn)?;

    // 清理现有数据
    diesel::sql_query("DELETE FROM post_tags").execute(conn)?;
    diesel::sql_query("DELETE FROM comments").execute(conn)?;
    diesel::sql_query("DELETE FROM posts").execute(conn)?;
    diesel::sql_query("DELETE FROM tags").execute(conn)?;
    diesel::sql_query("DELETE FROM users").execute(conn)?;

    // 创建用户
    diesel::sql_query(
        "INSERT INTO users (name, email, age) VALUES
         ('张三', 'zhangsan@example.com', 25),
         ('李四', 'lisi@example.com', 30),
         ('王五', 'wangwu@example.com', 28),
         ('赵六', 'zhaoliu@example.com', 35),
         ('钱七', 'qianqi@example.com', 22)"
    ).execute(conn)?;

    // 创建文章
    diesel::sql_query(
        "INSERT INTO posts (title, content, author_id, published) VALUES
         ('Rust 编程入门', 'Rust 是一门系统编程语言...', 1, true),
         ('Diesel ORM 指南', 'Diesel 是 Rust 的 ORM 框架...', 1, true),
         ('GaussDB 使用技巧', 'GaussDB 是华为云的数据库...', 2, true),
         ('数据库设计原则', '好的数据库设计需要遵循...', 2, false),
         ('Web 开发最佳实践', '现代 Web 开发需要考虑...', 3, true),
         ('性能优化技巧', '应用性能优化的关键在于...', 4, true),
         ('安全编程指南', '编写安全的代码需要注意...', 5, false)"
    ).execute(conn)?;

    // 创建标签
    diesel::sql_query(
        "INSERT INTO tags (name, color) VALUES
         ('Rust', '#f74c00'),
         ('数据库', '#336791'),
         ('Web开发', '#61dafb'),
         ('性能', '#ff6b6b'),
         ('安全', '#4ecdc4'),
         ('教程', '#45b7d1')"
    ).execute(conn)?;

    // 创建文章标签关联
    diesel::sql_query(
        "INSERT INTO post_tags (post_id, tag_id) VALUES
         (1, 1), (1, 6),
         (2, 1), (2, 2),
         (3, 2),
         (4, 2),
         (5, 3),
         (6, 4),
         (7, 5)"
    ).execute(conn)?;

    // 创建评论
    diesel::sql_query(
        "INSERT INTO comments (post_id, author_id, content) VALUES
         (1, 2, '很好的入门教程！'),
         (1, 3, '学到了很多，谢谢分享。'),
         (2, 1, 'Diesel 确实很强大。'),
         (3, 4, 'GaussDB 性能不错。'),
         (5, 5, '实用的建议！'),
         (6, 1, '性能优化很重要。')"
    ).execute(conn)?;

    info!("✅ 示例数据设置完成");

    Ok(())
}

/// 创建数据库表
fn create_tables(conn: &mut GaussDBConnection) -> Result<()> {
    info!("创建数据库表...");

    // 创建用户表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE,
            age INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(conn)?;

    // 创建文章表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            author_id INTEGER NOT NULL,
            published BOOLEAN DEFAULT FALSE,
            view_count INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (author_id) REFERENCES users(id)
        )"
    ).execute(conn)?;

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
    ).execute(conn)?;

    // 创建标签表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS tags (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL UNIQUE,
            color VARCHAR,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(conn)?;

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
    ).execute(conn)?;

    info!("✅ 所有表创建成功！");
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    info!("🚀 启动 Diesel-GaussDB 高级查询示例");

    let mut connection = establish_connection()?;
    info!("✅ 数据库连接成功！");

    // 设置示例数据
    setup_sample_data(&mut connection)?;

    // 演示各种高级查询
    demo_window_functions(&mut connection)?;
    demo_cte_queries(&mut connection)?;
    demo_subqueries(&mut connection)?;
    demo_aggregation_queries(&mut connection)?;

    info!("🎉 所有高级查询示例演示完成！");
    Ok(())
}

/// 演示窗口函数
fn demo_window_functions(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🪟 === 窗口函数演示 ===");

    // 1. ROW_NUMBER - 为每个用户的文章编号
    info!("1. ROW_NUMBER - 用户文章编号...");
    let results: Vec<UserPostStats> = diesel::sql_query(
        "SELECT u.name as author, p.title,
         ROW_NUMBER() OVER (PARTITION BY u.name ORDER BY p.created_at) as row_num
         FROM posts p
         JOIN users u ON p.author_id = u.id
         WHERE p.published = true
         ORDER BY u.name, row_num"
    ).load(conn)?;

    for result in &results {
        info!("  {}: {} (第{}篇)", result.author, result.title, result.row_num);
    }

    // 2. RANK - 按评论数排名文章
    info!("\n2. RANK - 文章评论数排名...");
    let rank_results: Vec<PostRank> = diesel::sql_query(
        "SELECT p.title,
         COUNT(c.id) as comment_count,
         RANK() OVER (ORDER BY COUNT(c.id) DESC) as rank
         FROM posts p
         LEFT JOIN comments c ON p.id = c.post_id
         WHERE p.published = true
         GROUP BY p.id, p.title
         ORDER BY rank"
    ).load(conn)?;

    for result in &rank_results {
        info!("  排名{}: 《{}》 - {} 条评论", result.rank, result.title, result.comment_count);
    }

    Ok(())
}

/// 演示 CTE 查询
fn demo_cte_queries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🔄 === CTE (公共表表达式) 演示 ===");

    // 1. 简单 CTE - 活跃用户
    info!("1. 简单 CTE - 活跃用户统计...");
    let active_users: Vec<UserActivity> = diesel::sql_query(
        "WITH active_users AS (
           SELECT u.name, COUNT(p.id) as post_count
           FROM users u
           LEFT JOIN posts p ON u.id = p.author_id
           GROUP BY u.id, u.name
           HAVING COUNT(p.id) > 0
         )
         SELECT name, post_count, 0 as comment_count
         FROM active_users
         ORDER BY post_count DESC"
    ).load(conn)?;

    for user in &active_users {
        info!("  活跃用户: {} - {} 篇文章", user.name, user.post_count);
    }

    // 2. 多个 CTE - 综合统计
    info!("\n2. 多个 CTE - 综合统计...");
    let comprehensive_stats: Vec<UserActivity> = diesel::sql_query(
        "WITH user_posts AS (
           SELECT u.id, u.name, COUNT(p.id) as post_count
           FROM users u
           LEFT JOIN posts p ON u.id = p.author_id
           GROUP BY u.id, u.name
         ),
         user_comments AS (
           SELECT u.id, COUNT(c.id) as comment_count
           FROM users u
           LEFT JOIN comments c ON u.id = c.author_id
           GROUP BY u.id
         )
         SELECT up.name, up.post_count,
                COALESCE(uc.comment_count, 0) as comment_count
         FROM user_posts up
         LEFT JOIN user_comments uc ON up.id = uc.id
         ORDER BY (up.post_count + COALESCE(uc.comment_count, 0)) DESC"
    ).load(conn)?;

    for stats in &comprehensive_stats {
        info!("  {}: {} 篇文章, {} 条评论", stats.name, stats.post_count, stats.comment_count);
    }

    Ok(())
}

/// 演示子查询
fn demo_subqueries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🔍 === 子查询演示 ===");

    // 1. EXISTS 子查询 - 有文章的用户
    info!("1. EXISTS 子查询 - 有文章的用户...");
    let authors: Vec<UserActivity> = diesel::sql_query(
        "SELECT u.name, 0 as post_count, 0 as comment_count
         FROM users u
         WHERE EXISTS (
           SELECT 1 FROM posts p WHERE p.author_id = u.id
         )
         ORDER BY u.name"
    ).load(conn)?;

    for author in &authors {
        info!("  作者: {}", author.name);
    }

    // 2. IN 子查询 - 有评论的文章
    info!("\n2. IN 子查询 - 有评论的文章...");
    let commented_posts: Vec<PostRank> = diesel::sql_query(
        "SELECT p.title, 0 as comment_count, 0 as rank
         FROM posts p
         WHERE p.id IN (
           SELECT DISTINCT c.post_id FROM comments c
         )
         ORDER BY p.title"
    ).load(conn)?;

    for post in &commented_posts {
        info!("  有评论的文章: 《{}》", post.title);
    }

    // 3. 标量子查询 - 文章及其评论数
    info!("\n3. 标量子查询 - 文章评论数...");
    let posts_with_comment_count: Vec<PostRank> = diesel::sql_query(
        "SELECT p.title,
         (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id) as comment_count,
         0 as rank
         FROM posts p
         WHERE p.published = true
         ORDER BY comment_count DESC"
    ).load(conn)?;

    for post in &posts_with_comment_count {
        info!("  《{}》: {} 条评论", post.title, post.comment_count);
    }

    Ok(())
}

/// 演示聚合查询
fn demo_aggregation_queries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n📊 === 聚合查询演示 ===");

    // 1. 基础统计
    info!("1. 基础统计信息...");

    #[derive(Debug, diesel::QueryableByName)]
    struct BasicStats {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        user_count: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        post_count: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        published_count: i64,
    }

    let stats: Vec<BasicStats> = diesel::sql_query(
        "SELECT
         (SELECT COUNT(*) FROM users) as user_count,
         (SELECT COUNT(*) FROM posts) as post_count,
         (SELECT COUNT(*) FROM posts WHERE published = true) as published_count"
    ).load(conn)?;

    if let Some(stats) = stats.first() {
        info!("  总用户数: {}", stats.user_count);
        info!("  总文章数: {}", stats.post_count);
        info!("  已发布文章数: {}", stats.published_count);
    }

    // 2. 按用户统计文章数
    info!("\n2. 按用户统计文章数...");
    let user_post_stats: Vec<UserActivity> = diesel::sql_query(
        "SELECT u.name, COUNT(p.id) as post_count, 0 as comment_count
         FROM users u
         LEFT JOIN posts p ON u.id = p.author_id
         GROUP BY u.id, u.name
         ORDER BY post_count DESC"
    ).load(conn)?;

    for stats in &user_post_stats {
        info!("  {}: {} 篇文章", stats.name, stats.post_count);
    }

    // 3. 按标签统计文章数
    info!("\n3. 按标签统计文章数...");
    let tag_stats: Vec<TagStats> = diesel::sql_query(
        "SELECT t.name as tag_name, COUNT(pt.post_id) as post_count
         FROM tags t
         LEFT JOIN post_tags pt ON t.id = pt.tag_id
         GROUP BY t.id, t.name
         ORDER BY post_count DESC"
    ).load(conn)?;

    for stats in &tag_stats {
        info!("  {}: {} 篇文章", stats.tag_name, stats.post_count);
    }

    Ok(())
}


