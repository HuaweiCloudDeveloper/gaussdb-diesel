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
use chrono::{Utc, NaiveDateTime};
use anyhow::{Result, Context};
use log::{info, warn};
use std::env;

// 重用基础示例的模型
#[path = "../../01-basic-usage/src/schema.rs"]
mod schema;
#[path = "../../01-basic-usage/src/models.rs"]
mod models;

use schema::*;
use models::*;

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

/// 创建示例数据
fn setup_sample_data(conn: &mut GaussDBConnection) -> Result<()> {
    info!("设置示例数据...");

    // 清理现有数据
    diesel::delete(post_tags::table).execute(conn)?;
    diesel::delete(comments::table).execute(conn)?;
    diesel::delete(posts::table).execute(conn)?;
    diesel::delete(tags::table).execute(conn)?;
    diesel::delete(users::table).execute(conn)?;

    // 创建用户
    let users_data = vec![
        NewUser { name: "张三", email: "zhangsan@example.com", age: Some(25) },
        NewUser { name: "李四", email: "lisi@example.com", age: Some(30) },
        NewUser { name: "王五", email: "wangwu@example.com", age: Some(28) },
        NewUser { name: "赵六", email: "zhaoliu@example.com", age: Some(35) },
        NewUser { name: "钱七", email: "qianqi@example.com", age: Some(22) },
    ];

    let users: Vec<User> = diesel::insert_into(users::table)
        .values(&users_data)
        .returning(User::as_returning())
        .get_results(conn)?;

    // 创建文章
    let posts_data = vec![
        NewPost { title: "Rust 编程入门", content: "Rust 是一门系统编程语言...", author_id: users[0].id, published: true },
        NewPost { title: "Diesel ORM 指南", content: "Diesel 是 Rust 的 ORM 框架...", author_id: users[0].id, published: true },
        NewPost { title: "GaussDB 使用技巧", content: "GaussDB 是华为云的数据库...", author_id: users[1].id, published: true },
        NewPost { title: "数据库设计原则", content: "好的数据库设计需要遵循...", author_id: users[1].id, published: false },
        NewPost { title: "Web 开发最佳实践", content: "现代 Web 开发需要考虑...", author_id: users[2].id, published: true },
        NewPost { title: "性能优化技巧", content: "应用性能优化的关键在于...", author_id: users[3].id, published: true },
        NewPost { title: "安全编程指南", content: "编写安全的代码需要注意...", author_id: users[4].id, published: false },
    ];

    let posts: Vec<Post> = diesel::insert_into(posts::table)
        .values(&posts_data)
        .returning(Post::as_returning())
        .get_results(conn)?;

    // 创建标签
    let tags_data = vec![
        NewTag { name: "Rust", color: Some("#f74c00") },
        NewTag { name: "数据库", color: Some("#336791") },
        NewTag { name: "Web开发", color: Some("#61dafb") },
        NewTag { name: "性能", color: Some("#ff6b6b") },
        NewTag { name: "安全", color: Some("#4ecdc4") },
        NewTag { name: "教程", color: Some("#45b7d1") },
    ];

    let tags: Vec<Tag> = diesel::insert_into(tags::table)
        .values(&tags_data)
        .returning(Tag::as_returning())
        .get_results(conn)?;

    // 创建文章标签关联
    let post_tags_data = vec![
        NewPostTag { post_id: posts[0].id, tag_id: tags[0].id }, // Rust 编程入门 - Rust
        NewPostTag { post_id: posts[0].id, tag_id: tags[5].id }, // Rust 编程入门 - 教程
        NewPostTag { post_id: posts[1].id, tag_id: tags[0].id }, // Diesel ORM 指南 - Rust
        NewPostTag { post_id: posts[1].id, tag_id: tags[1].id }, // Diesel ORM 指南 - 数据库
        NewPostTag { post_id: posts[2].id, tag_id: tags[1].id }, // GaussDB 使用技巧 - 数据库
        NewPostTag { post_id: posts[3].id, tag_id: tags[1].id }, // 数据库设计原则 - 数据库
        NewPostTag { post_id: posts[4].id, tag_id: tags[2].id }, // Web 开发最佳实践 - Web开发
        NewPostTag { post_id: posts[5].id, tag_id: tags[3].id }, // 性能优化技巧 - 性能
        NewPostTag { post_id: posts[6].id, tag_id: tags[4].id }, // 安全编程指南 - 安全
    ];

    diesel::insert_into(post_tags::table)
        .values(&post_tags_data)
        .execute(conn)?;

    // 创建评论
    let comments_data = vec![
        NewComment { post_id: posts[0].id, author_id: users[1].id, content: "很好的入门教程！" },
        NewComment { post_id: posts[0].id, author_id: users[2].id, content: "学到了很多，谢谢分享。" },
        NewComment { post_id: posts[1].id, author_id: users[0].id, content: "Diesel 确实很强大。" },
        NewComment { post_id: posts[2].id, author_id: users[3].id, content: "GaussDB 性能不错。" },
        NewComment { post_id: posts[4].id, author_id: users[4].id, content: "实用的建议！" },
        NewComment { post_id: posts[5].id, author_id: users[0].id, content: "性能优化很重要。" },
    ];

    diesel::insert_into(comments::table)
        .values(&comments_data)
        .execute(conn)?;

    info!("✅ 示例数据设置完成");
    info!("  - 用户: {} 个", users.len());
    info!("  - 文章: {} 个", posts.len());
    info!("  - 标签: {} 个", tags.len());
    info!("  - 评论: {} 个", comments_data.len());

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
    demo_aggregation_queries(&mut connection)?;
    demo_join_queries(&mut connection)?;
    demo_window_functions(&mut connection)?;
    demo_cte_queries(&mut connection)?;
    demo_subqueries(&mut connection)?;
    demo_complex_analytics(&mut connection)?;

    info!("🎉 所有高级查询示例演示完成！");
    Ok(())
}

/// 演示聚合查询
fn demo_aggregation_queries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n📊 === 聚合查询演示 ===");

    // 1. 基础统计
    info!("1. 基础统计信息...");
    let user_count: i64 = users::table.count().get_result(conn)?;
    let post_count: i64 = posts::table.count().get_result(conn)?;
    let published_count: i64 = posts::table.filter(posts::published.eq(true)).count().get_result(conn)?;

    info!("  总用户数: {}", user_count);
    info!("  总文章数: {}", post_count);
    info!("  已发布文章数: {}", published_count);

    // 2. 按用户统计文章数
    info!("\n2. 按用户统计文章数...");
    let user_post_stats: Vec<(String, i64)> = users::table
        .left_join(posts::table)
        .group_by(users::name)
        .select((users::name, diesel::dsl::count(posts::id.nullable())))
        .order_by(diesel::dsl::count(posts::id.nullable()).desc())
        .load(conn)?;

    for (name, count) in &user_post_stats {
        info!("  {}: {} 篇文章", name, count);
    }

    // 3. 按标签统计文章数
    info!("\n3. 按标签统计文章数...");
    let tag_stats: Vec<(String, i64)> = tags::table
        .inner_join(post_tags::table)
        .group_by(tags::name)
        .select((tags::name, diesel::dsl::count(post_tags::post_id)))
        .order_by(diesel::dsl::count(post_tags::post_id).desc())
        .load(conn)?;

    for (tag_name, count) in &tag_stats {
        info!("  {}: {} 篇文章", tag_name, count);
    }

    Ok(())
}

/// 演示联表查询
fn demo_join_queries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🔗 === 联表查询演示 ===");

    // 1. 内连接 - 查询文章及其作者
    info!("1. 内连接 - 文章及作者信息...");
    let posts_with_authors: Vec<(Post, User)> = posts::table
        .inner_join(users::table)
        .filter(posts::published.eq(true))
        .select((Post::as_select(), User::as_select()))
        .load(conn)?;

    for (post, author) in &posts_with_authors {
        info!("  《{}》 - 作者: {}", post.title, author.name);
    }

    // 2. 左连接 - 查询用户及其文章（包括没有文章的用户）
    info!("\n2. 左连接 - 用户及其文章统计...");
    let users_with_post_count: Vec<(User, i64)> = users::table
        .left_join(posts::table)
        .group_by(users::all_columns)
        .select((User::as_select(), diesel::dsl::count(posts::id.nullable())))
        .load(conn)?;

    for (user, post_count) in &users_with_post_count {
        info!("  {}: {} 篇文章", user.name, post_count);
    }

    // 3. 多表连接 - 查询文章、作者和评论数
    info!("\n3. 多表连接 - 文章详细信息...");
    let post_details: Vec<(Post, User, i64)> = posts::table
        .inner_join(users::table)
        .left_join(comments::table)
        .group_by((posts::all_columns, users::all_columns))
        .select((
            Post::as_select(),
            User::as_select(),
            diesel::dsl::count(comments::id.nullable())
        ))
        .load(conn)?;

    for (post, author, comment_count) in &post_details {
        info!("  《{}》 - 作者: {} - 评论数: {}", post.title, author.name, comment_count);
    }

    Ok(())
}

/// 演示窗口函数
fn demo_window_functions(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🪟 === 窗口函数演示 ===");

    // 1. ROW_NUMBER - 为每个用户的文章编号
    info!("1. ROW_NUMBER - 用户文章编号...");
    let results: Vec<(String, String, i32)> = diesel::sql_query(
        "SELECT u.name as author, p.title,
         ROW_NUMBER() OVER (PARTITION BY u.name ORDER BY p.created_at) as row_num
         FROM posts p
         JOIN users u ON p.author_id = u.id
         WHERE p.published = true
         ORDER BY u.name, row_num"
    ).load(conn)?;

    for (author, title, row_num) in &results {
        info!("  {}: {} (第{}篇)", author, title, row_num);
    }

    // 2. RANK - 按评论数排名文章
    info!("\n2. RANK - 文章评论数排名...");
    let rank_results: Vec<(String, i64, i32)> = diesel::sql_query(
        "SELECT p.title,
         COUNT(c.id) as comment_count,
         RANK() OVER (ORDER BY COUNT(c.id) DESC) as rank
         FROM posts p
         LEFT JOIN comments c ON p.id = c.post_id
         WHERE p.published = true
         GROUP BY p.id, p.title
         ORDER BY rank"
    ).load(conn)?;

    for (title, comment_count, rank) in &rank_results {
        info!("  排名{}: 《{}》 - {} 条评论", rank, title, comment_count);
    }

    // 3. 累计统计
    info!("\n3. 累计统计 - 用户文章累计数...");
    let cumulative_results: Vec<(String, String, i64)> = diesel::sql_query(
        "SELECT u.name as author, p.title,
         COUNT(*) OVER (PARTITION BY u.name ORDER BY p.created_at
                       ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as cumulative_count
         FROM posts p
         JOIN users u ON p.author_id = u.id
         WHERE p.published = true
         ORDER BY u.name, p.created_at"
    ).load(conn)?;

    for (author, title, cumulative) in &cumulative_results {
        info!("  {}: 《{}》 (累计第{}篇)", author, title, cumulative);
    }

    Ok(())
}

/// 演示 CTE 查询
fn demo_cte_queries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🔄 === CTE (公共表表达式) 演示 ===");

    // 1. 简单 CTE - 活跃用户
    info!("1. 简单 CTE - 活跃用户统计...");
    let active_users: Vec<(String, i64)> = diesel::sql_query(
        "WITH active_users AS (
           SELECT u.name, COUNT(p.id) as post_count
           FROM users u
           LEFT JOIN posts p ON u.id = p.author_id
           GROUP BY u.id, u.name
           HAVING COUNT(p.id) > 0
         )
         SELECT name, post_count
         FROM active_users
         ORDER BY post_count DESC"
    ).load(conn)?;

    for (name, post_count) in &active_users {
        info!("  活跃用户: {} - {} 篇文章", name, post_count);
    }

    // 2. 多个 CTE - 综合统计
    info!("\n2. 多个 CTE - 综合统计...");
    let comprehensive_stats: Vec<(String, i64, i64, i64)> = diesel::sql_query(
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
                COALESCE(uc.comment_count, 0) as comment_count,
                (up.post_count + COALESCE(uc.comment_count, 0)) as total_activity
         FROM user_posts up
         LEFT JOIN user_comments uc ON up.id = uc.id
         ORDER BY total_activity DESC"
    ).load(conn)?;

    for (name, posts, comments, total) in &comprehensive_stats {
        info!("  {}: {} 篇文章, {} 条评论, 总活跃度: {}", name, posts, comments, total);
    }

    Ok(())
}

/// 演示子查询
fn demo_subqueries(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n🔍 === 子查询演示 ===");

    // 1. EXISTS 子查询 - 有文章的用户
    info!("1. EXISTS 子查询 - 有文章的用户...");
    let authors: Vec<User> = users::table
        .filter(diesel::dsl::exists(
            posts::table.filter(posts::author_id.eq(users::id))
        ))
        .select(User::as_select())
        .load(conn)?;

    for author in &authors {
        info!("  作者: {}", author.name);
    }

    // 2. IN 子查询 - 有评论的文章
    info!("\n2. IN 子查询 - 有评论的文章...");
    let commented_posts: Vec<Post> = posts::table
        .filter(posts::id.eq_any(
            comments::table.select(comments::post_id).distinct()
        ))
        .select(Post::as_select())
        .load(conn)?;

    for post in &commented_posts {
        info!("  有评论的文章: 《{}》", post.title);
    }

    // 3. 标量子查询 - 文章及其评论数
    info!("\n3. 标量子查询 - 文章评论数...");
    let posts_with_comment_count: Vec<(String, i64)> = diesel::sql_query(
        "SELECT p.title,
         (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id) as comment_count
         FROM posts p
         WHERE p.published = true
         ORDER BY comment_count DESC"
    ).load(conn)?;

    for (title, comment_count) in &posts_with_comment_count {
        info!("  《{}》: {} 条评论", title, comment_count);
    }

    Ok(())
}

/// 演示复杂分析查询
fn demo_complex_analytics(conn: &mut GaussDBConnection) -> Result<()> {
    info!("\n📈 === 复杂分析查询演示 ===");

    // 1. 用户活跃度分析
    info!("1. 用户活跃度分析...");
    let user_activity: Vec<(String, i64, i64, f64)> = diesel::sql_query(
        "WITH user_stats AS (
           SELECT u.id, u.name,
                  COUNT(DISTINCT p.id) as post_count,
                  COUNT(DISTINCT c.id) as comment_count
           FROM users u
           LEFT JOIN posts p ON u.id = p.author_id
           LEFT JOIN comments c ON u.id = c.author_id
           GROUP BY u.id, u.name
         )
         SELECT name, post_count, comment_count,
                CASE
                  WHEN post_count + comment_count = 0 THEN 0
                  ELSE ROUND((post_count * 2.0 + comment_count) / 3.0, 2)
                END as activity_score
         FROM user_stats
         ORDER BY activity_score DESC"
    ).load(conn)?;

    for (name, posts, comments, score) in &user_activity {
        info!("  {}: {} 篇文章, {} 条评论, 活跃度: {}", name, posts, comments, score);
    }

    // 2. 热门标签分析
    info!("\n2. 热门标签分析...");
    let tag_popularity: Vec<(String, i64, f64)> = diesel::sql_query(
        "WITH tag_stats AS (
           SELECT t.name,
                  COUNT(pt.post_id) as post_count,
                  COUNT(DISTINCT p.author_id) as author_count
           FROM tags t
           LEFT JOIN post_tags pt ON t.id = pt.tag_id
           LEFT JOIN posts p ON pt.post_id = p.id
           GROUP BY t.id, t.name
         )
         SELECT name, post_count,
                CASE
                  WHEN post_count = 0 THEN 0
                  ELSE ROUND(post_count * 1.0 / author_count, 2)
                END as posts_per_author
         FROM tag_stats
         WHERE post_count > 0
         ORDER BY post_count DESC, posts_per_author DESC"
    ).load(conn)?;

    for (tag_name, post_count, posts_per_author) in &tag_popularity {
        info!("  标签 '{}': {} 篇文章, 平均每作者 {} 篇", tag_name, post_count, posts_per_author);
    }

    // 3. 内容质量分析（基于评论数）
    info!("\n3. 内容质量分析...");
    let content_quality: Vec<(String, String, i64, String)> = diesel::sql_query(
        "WITH post_quality AS (
           SELECT p.title, u.name as author,
                  COUNT(c.id) as comment_count,
                  CASE
                    WHEN COUNT(c.id) >= 2 THEN '高质量'
                    WHEN COUNT(c.id) = 1 THEN '中等质量'
                    ELSE '待改进'
                  END as quality_level
           FROM posts p
           JOIN users u ON p.author_id = u.id
           LEFT JOIN comments c ON p.id = c.post_id
           WHERE p.published = true
           GROUP BY p.id, p.title, u.name
         )
         SELECT title, author, comment_count, quality_level
         FROM post_quality
         ORDER BY comment_count DESC, title"
    ).load(conn)?;

    for (title, author, comment_count, quality) in &content_quality {
        info!("  《{}》 - {}: {} 条评论 ({})", title, author, comment_count, quality);
    }

    Ok(())
}
