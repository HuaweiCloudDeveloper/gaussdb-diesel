//! 基于 Diesel-GaussDB 的真实博客系统
//!
//! 这是一个完整的博客系统，展示了如何在生产环境中使用 diesel-gaussdb
//! 构建高性能的 Web 应用程序。

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use diesel::prelude::*;
use diesel_gaussdb::GaussDBConnection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use anyhow::{Result, Context};
use log::info;
use std::env;

/// 博客文章结构
#[derive(Debug, Serialize, Deserialize, diesel::QueryableByName)]
struct Post {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    content: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    author_id: i32,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    published: bool,
}

/// 用户结构
#[derive(Debug, Serialize, Deserialize, diesel::QueryableByName)]
struct User {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
}

/// 新文章结构
#[derive(Debug, Deserialize)]
struct NewPost {
    title: String,
    content: String,
    author_id: i32,
}

/// 新评论结构
#[derive(Debug, Deserialize)]
struct NewComment {
    content: String,
    author_id: i32,
}

/// 评论结构
#[derive(Debug, Serialize, Deserialize, diesel::QueryableByName)]
struct Comment {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    post_id: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    author_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    content: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    author_name: String,
}

/// 新评论结构 (重复定义已删除)

/// API 响应结构
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "操作成功".to_string(),
        }
    }

    fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message,
        }
    }
}

/// 建立数据库连接
fn establish_connection() -> Result<GaussDBConnection> {
    let database_url = env::var("GAUSSDB_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres".to_string()
        });

    GaussDBConnection::establish(&database_url)
        .with_context(|| format!("Error connecting to {}", database_url))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    info!("🚀 启动 Diesel-GaussDB 博客系统");

    // 初始化数据库
    initialize_database()?;

    // 构建路由
    let app = create_router();

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("🌐 服务器启动在 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 创建路由
fn create_router() -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health_check))

        // 博客 API
        .route("/api/posts", get(get_posts))
        .route("/api/posts", post(create_post))
        .route("/api/posts/:id", get(get_post))
        .route("/api/posts/:id", axum::routing::put(update_post))
        .route("/api/posts/:id", axum::routing::delete(delete_post))
        .route("/api/posts/search", get(search_posts))
        .route("/api/posts/:id/comments", get(get_post_comments))
        .route("/api/posts/:id/comments", post(add_comment))

        // 用户 API
        .route("/api/users", get(get_users))
        .route("/api/users/:id", get(get_user))
        .route("/api/users/:id/posts", get(get_user_posts))

        // 评论 API
        .route("/api/comments/:id", axum::routing::delete(delete_comment))

        // 统计信息
        .route("/api/stats", get(blog_stats))
        .route("/api/stats/popular-posts", get(popular_posts))
}

/// 健康检查
async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("博客系统运行正常".to_string()))
}

/// 获取所有文章
async fn get_posts() -> Result<Json<ApiResponse<Vec<Post>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let posts: Vec<Post> = diesel::sql_query(
        "SELECT id, title, content, author_id, published FROM posts WHERE published = true ORDER BY id DESC"
    )
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(posts)))
}

/// 获取单篇文章
async fn get_post(Path(post_id): Path<i32>) -> Result<Json<ApiResponse<Post>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let posts: Vec<Post> = diesel::sql_query(
        "SELECT id, title, content, author_id, published FROM posts WHERE id = $1"
    )
    .bind::<diesel::sql_types::Integer, _>(post_id)
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match posts.into_iter().next() {
        Some(post) => Ok(Json(ApiResponse::success(post))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 创建新文章
async fn create_post(Json(new_post): Json<NewPost>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query(
        "INSERT INTO posts (title, content, author_id, published) VALUES ($1, $2, $3, true)"
    )
    .bind::<diesel::sql_types::Text, _>(&new_post.title)
    .bind::<diesel::sql_types::Text, _>(&new_post.content)
    .bind::<diesel::sql_types::Integer, _>(new_post.author_id)
    .execute(&mut conn);

    match result {
        Ok(_) => Ok(Json(ApiResponse::success("文章创建成功".to_string()))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// 获取所有用户
async fn get_users() -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = diesel::sql_query(
        "SELECT id, username, email FROM users ORDER BY id"
    )
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(users)))
}

/// 获取单个用户
async fn get_user(Path(user_id): Path<i32>) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = diesel::sql_query(
        "SELECT id, username, email FROM users WHERE id = $1"
    )
    .bind::<diesel::sql_types::Integer, _>(user_id)
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match users.into_iter().next() {
        Some(user) => Ok(Json(ApiResponse::success(user))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 博客统计
async fn blog_stats() -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(diesel::QueryableByName)]
    struct BlogStats {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_posts: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        published_posts: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_users: i64,
    }

    let stats: Vec<BlogStats> = diesel::sql_query(
        "SELECT
         (SELECT COUNT(*) FROM posts) as total_posts,
         (SELECT COUNT(*) FROM posts WHERE published = true) as published_posts,
         (SELECT COUNT(*) FROM users) as total_users"
    )
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(stats) = stats.into_iter().next() {
        let response = json!({
            "total_posts": stats.total_posts,
            "published_posts": stats.published_posts,
            "total_users": stats.total_users
        });
        Ok(Json(ApiResponse::success(response)))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// 初始化数据库
fn initialize_database() -> Result<()> {
    let mut conn = establish_connection()?;

    info!("初始化数据库表...");

    // 创建用户表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR UNIQUE NOT NULL,
            email VARCHAR UNIQUE NOT NULL,
            password_hash VARCHAR NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&mut conn)?;

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
    ).execute(&mut conn)?;

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
    ).execute(&mut conn)?;

    // 创建示例数据
    create_sample_data(&mut conn)?;

    info!("✅ 数据库初始化完成");
    Ok(())
}

/// 创建示例数据
fn create_sample_data(conn: &mut GaussDBConnection) -> Result<()> {
    // 检查是否已有数据
    #[derive(diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let user_count: Vec<Count> = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .load(conn)?;

    if let Some(count) = user_count.first() {
        if count.count > 0 {
            info!("示例数据已存在，跳过创建");
            return Ok(());
        }
    }

    info!("创建示例数据...");

    // 创建示例用户
    diesel::sql_query(
        "INSERT INTO users (username, email, password_hash) VALUES
         ('admin', 'admin@blog.com', 'hashed_password_1'),
         ('author1', 'author1@blog.com', 'hashed_password_2'),
         ('user1', 'user1@blog.com', 'hashed_password_3')"
    ).execute(conn)?;

    // 创建示例文章
    diesel::sql_query(
        "INSERT INTO posts (title, content, author_id, published) VALUES
         ('欢迎来到我们的博客', '这是我们博客的第一篇文章，欢迎大家！', 1, true),
         ('Rust 编程语言介绍', 'Rust 是一门系统编程语言，专注于安全、速度和并发...', 2, true),
         ('数据库设计最佳实践', '本文介绍了数据库设计的一些最佳实践和常见模式...', 2, true),
         ('草稿文章', '这是一篇草稿文章，尚未发布...', 1, false)"
    ).execute(conn)?;

    // 创建示例评论
    diesel::sql_query(
        "INSERT INTO comments (post_id, author_id, content) VALUES
         (1, 2, '很棒的博客，期待更多内容！'),
         (1, 3, '感谢分享，学到了很多。'),
         (2, 1, 'Rust 确实是一门很有前途的语言。'),
         (3, 3, '数据库设计很重要，谢谢分享经验。')"
    ).execute(conn)?;

    info!("✅ 示例数据创建完成");
    Ok(())
}

/// 新评论结构 (已在文件开头定义)

/// 更新文章
async fn update_post(
    Path(post_id): Path<i32>,
    Json(update_data): Json<NewPost>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query(&format!(
        "UPDATE posts SET title = '{}', content = '{}' WHERE id = {} AND published = true",
        update_data.title.replace("'", "''"),
        update_data.content.replace("'", "''"),
        post_id
    )).execute(&mut conn);

    match result {
        Ok(0) => Err(StatusCode::NOT_FOUND),
        Ok(_) => Ok(Json(ApiResponse::success("文章更新成功".to_string()))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// 删除文章
async fn delete_post(Path(post_id): Path<i32>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 先删除相关评论
    let _ = diesel::sql_query(&format!("DELETE FROM comments WHERE post_id = {}", post_id))
        .execute(&mut conn);

    let result = diesel::sql_query(&format!("DELETE FROM posts WHERE id = {}", post_id))
        .execute(&mut conn);

    match result {
        Ok(0) => Err(StatusCode::NOT_FOUND),
        Ok(_) => Ok(Json(ApiResponse::success("文章删除成功".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 搜索文章
async fn search_posts(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Post>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sql = "SELECT id, title, content, author_id, published FROM posts WHERE published = true".to_string();

    if let Some(keyword) = params.get("q") {
        sql.push_str(&format!(" AND (title ILIKE '%{}%' OR content ILIKE '%{}%')", keyword, keyword));
    }

    if let Some(author_id) = params.get("author_id") {
        if let Ok(id) = author_id.parse::<i32>() {
            sql.push_str(&format!(" AND author_id = {}", id));
        }
    }

    sql.push_str(" ORDER BY id DESC LIMIT 20");

    let posts: Vec<Post> = diesel::sql_query(sql)
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(posts)))
}

/// 获取文章评论
async fn get_post_comments(Path(post_id): Path<i32>) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(Debug, diesel::QueryableByName)]
    struct CommentWithAuthor {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        id: i32,
        #[diesel(sql_type = diesel::sql_types::Text)]
        content: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        author_name: String,
    }

    let comments: Vec<CommentWithAuthor> = diesel::sql_query(&format!(
        "SELECT c.id, c.content, u.username as author_name
         FROM comments c
         JOIN users u ON c.author_id = u.id
         WHERE c.post_id = {}
         ORDER BY c.created_at ASC",
        post_id
    )).load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = comments
        .into_iter()
        .map(|c| serde_json::json!({
            "id": c.id,
            "content": c.content,
            "author_name": c.author_name
        }))
        .collect();

    Ok(Json(ApiResponse::success(result)))
}

/// 添加评论
async fn add_comment(
    Path(post_id): Path<i32>,
    Json(comment_data): Json<NewComment>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query(&format!(
        "INSERT INTO comments (post_id, author_id, content) VALUES ({}, {}, '{}')",
        post_id,
        comment_data.author_id,
        comment_data.content.replace("'", "''")
    )).execute(&mut conn);

    match result {
        Ok(_) => Ok(Json(ApiResponse::success("评论添加成功".to_string()))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// 获取用户文章
async fn get_user_posts(Path(user_id): Path<i32>) -> Result<Json<ApiResponse<Vec<Post>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let posts: Vec<Post> = diesel::sql_query(&format!(
        "SELECT id, title, content, author_id, published
         FROM posts
         WHERE author_id = {} AND published = true
         ORDER BY id DESC",
        user_id
    )).load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(posts)))
}

/// 删除评论
async fn delete_comment(Path(comment_id): Path<i32>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query(&format!("DELETE FROM comments WHERE id = {}", comment_id))
        .execute(&mut conn);

    match result {
        Ok(0) => Err(StatusCode::NOT_FOUND),
        Ok(_) => Ok(Json(ApiResponse::success("评论删除成功".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 热门文章
async fn popular_posts() -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(Debug, diesel::QueryableByName)]
    struct PopularPost {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        id: i32,
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        author_name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        comment_count: i64,
    }

    let popular: Vec<PopularPost> = diesel::sql_query(
        "SELECT p.id, p.title, u.username as author_name, COUNT(c.id) as comment_count
         FROM posts p
         JOIN users u ON p.author_id = u.id
         LEFT JOIN comments c ON p.id = c.post_id
         WHERE p.published = true
         GROUP BY p.id, p.title, u.username
         ORDER BY comment_count DESC, p.id DESC
         LIMIT 10"
    ).load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = popular
        .into_iter()
        .map(|p| serde_json::json!({
            "id": p.id,
            "title": p.title,
            "author_name": p.author_name,
            "comment_count": p.comment_count
        }))
        .collect();

    Ok(Json(ApiResponse::success(result)))
}


