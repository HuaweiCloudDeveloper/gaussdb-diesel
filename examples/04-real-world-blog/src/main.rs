//! 基于 Diesel-GaussDB 的真实博客系统
//!
//! 这是一个完整的博客系统，展示了如何在生产环境中使用 diesel-gaussdb
//! 构建高性能的 Web 应用程序。

use axum::{
    extract::Extension,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

mod config;
mod database;
mod error;
mod models;
mod handlers;
mod services;
mod schema;

use config::Config;
use database::{create_pool, DbPool};
use error::AppError;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    log::info!("🚀 启动 Diesel-GaussDB 博客系统");

    // 加载配置
    let config = Config::from_env()?;
    log::info!("✅ 配置加载完成");

    // 创建数据库连接池
    let db_pool = create_pool(&config.database_url)?;
    log::info!("✅ 数据库连接池创建完成");

    // 初始化数据库表
    initialize_database(&db_pool).await?;
    log::info!("✅ 数据库初始化完成");

    // 创建应用状态
    let app_state = AppState {
        db_pool,
        config: config.clone(),
    };

    // 构建路由
    let app = create_router(app_state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    log::info!("🌐 服务器启动在 http://{}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// 创建路由
fn create_router(state: AppState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        
        // API 路由
        .nest("/api", api_routes())
        
        // Web 页面路由
        .nest("/", web_routes())
        
        // 静态文件
        .route("/static/*file", get(handlers::static_files))
        
        // 中间件
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(Extension(state))
        )
}

/// API 路由
fn api_routes() -> Router {
    Router::new()
        // 认证路由
        .nest("/auth", auth_routes())
        
        // 文章路由
        .nest("/posts", post_routes())
        
        // 用户路由
        .nest("/users", user_routes())
        
        // 评论路由
        .nest("/comments", comment_routes())
        
        // 标签路由
        .nest("/tags", tag_routes())
}

/// 认证路由
fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))
        .route("/me", get(handlers::auth::me))
}

/// 文章路由
fn post_routes() -> Router {
    Router::new()
        .route("/", get(handlers::posts::list_posts))
        .route("/", post(handlers::posts::create_post))
        .route("/:id", get(handlers::posts::get_post))
        .route("/:id", post(handlers::posts::update_post))
        .route("/:id", post(handlers::posts::delete_post))
        .route("/:id/comments", get(handlers::posts::get_post_comments))
        .route("/:id/comments", post(handlers::posts::add_comment))
        .route("/search", get(handlers::posts::search_posts))
        .route("/popular", get(handlers::posts::popular_posts))
}

/// 用户路由
fn user_routes() -> Router {
    Router::new()
        .route("/", get(handlers::users::list_users))
        .route("/:id", get(handlers::users::get_user))
        .route("/:id/posts", get(handlers::users::get_user_posts))
        .route("/:id/stats", get(handlers::users::get_user_stats))
}

/// 评论路由
fn comment_routes() -> Router {
    Router::new()
        .route("/:id", get(handlers::comments::get_comment))
        .route("/:id", post(handlers::comments::update_comment))
        .route("/:id", post(handlers::comments::delete_comment))
}

/// 标签路由
fn tag_routes() -> Router {
    Router::new()
        .route("/", get(handlers::tags::list_tags))
        .route("/", post(handlers::tags::create_tag))
        .route("/:id", get(handlers::tags::get_tag))
        .route("/:id/posts", get(handlers::tags::get_tag_posts))
        .route("/popular", get(handlers::tags::popular_tags))
}

/// Web 页面路由
fn web_routes() -> Router {
    Router::new()
        .route("/", get(handlers::web::index))
        .route("/posts/:id", get(handlers::web::post_detail))
        .route("/login", get(handlers::web::login_page))
        .route("/register", get(handlers::web::register_page))
        .route("/admin", get(handlers::web::admin_dashboard))
}

/// 健康检查
async fn health_check(Extension(state): Extension<AppState>) -> Result<Json<Value>, AppError> {
    // 检查数据库连接
    let mut conn = state.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // 执行简单查询验证连接
    use diesel::prelude::*;
    let result: i32 = diesel::sql_query("SELECT 1 as test")
        .get_result::<(i32,)>(&mut conn)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .0;
    
    if result != 1 {
        return Err(AppError::DatabaseError("Health check failed".to_string()));
    }

    Ok(Json(json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 初始化数据库
async fn initialize_database(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;
    
    log::info!("初始化数据库表...");
    
    // 创建用户表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR UNIQUE NOT NULL,
            email VARCHAR UNIQUE NOT NULL,
            password_hash VARCHAR NOT NULL,
            avatar_url VARCHAR,
            bio TEXT,
            is_admin BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        )"
    ).execute(&mut conn)?;

    // 创建文章表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            slug VARCHAR UNIQUE NOT NULL,
            content TEXT NOT NULL,
            excerpt TEXT,
            author_id INTEGER NOT NULL,
            published BOOLEAN DEFAULT FALSE,
            view_count INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP,
            FOREIGN KEY (author_id) REFERENCES users(id)
        )"
    ).execute(&mut conn)?;

    // 创建评论表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS comments (
            id SERIAL PRIMARY KEY,
            post_id INTEGER NOT NULL,
            author_id INTEGER NOT NULL,
            parent_id INTEGER,
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (post_id) REFERENCES posts(id),
            FOREIGN KEY (author_id) REFERENCES users(id),
            FOREIGN KEY (parent_id) REFERENCES comments(id)
        )"
    ).execute(&mut conn)?;

    // 创建标签表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS tags (
            id SERIAL PRIMARY KEY,
            name VARCHAR UNIQUE NOT NULL,
            slug VARCHAR UNIQUE NOT NULL,
            description TEXT,
            color VARCHAR,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&mut conn)?;

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
    ).execute(&mut conn)?;

    // 创建索引
    create_indexes(&mut conn)?;

    // 创建示例数据
    create_sample_data(&mut conn)?;

    log::info!("✅ 数据库初始化完成");
    Ok(())
}

/// 创建数据库索引
fn create_indexes(conn: &mut diesel_gaussdb::GaussDBConnection) -> Result<(), diesel::result::Error> {
    use diesel::prelude::*;
    
    // 文章索引
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_posts_published_created ON posts(published, created_at DESC)").execute(conn)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_posts_author_published ON posts(author_id, published)").execute(conn)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_posts_slug ON posts(slug)").execute(conn)?;
    
    // 评论索引
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_comments_post_created ON comments(post_id, created_at)").execute(conn)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_comments_author ON comments(author_id)").execute(conn)?;
    
    // 标签索引
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_post_tags_post ON post_tags(post_id)").execute(conn)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_post_tags_tag ON post_tags(tag_id)").execute(conn)?;
    
    Ok(())
}

/// 创建示例数据
fn create_sample_data(conn: &mut diesel_gaussdb::GaussDBConnection) -> Result<(), diesel::result::Error> {
    use diesel::prelude::*;
    
    // 检查是否已有数据
    let user_count: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result::<(i64,)>(conn)?
        .0;
    
    if user_count > 0 {
        log::info!("示例数据已存在，跳过创建");
        return Ok(());
    }
    
    log::info!("创建示例数据...");
    
    // 创建管理员用户
    diesel::sql_query(
        "INSERT INTO users (username, email, password_hash, is_admin, bio) VALUES 
         ('admin', 'admin@blog.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uIfa', true, '系统管理员'),
         ('author1', 'author1@blog.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uIfa', false, '技术博主'),
         ('user1', 'user1@blog.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uIfa', false, '普通用户')"
    ).execute(conn)?;
    
    // 创建标签
    diesel::sql_query(
        "INSERT INTO tags (name, slug, description, color) VALUES 
         ('Rust', 'rust', 'Rust 编程语言相关内容', '#f74c00'),
         ('数据库', 'database', '数据库技术和最佳实践', '#336791'),
         ('Web开发', 'web-dev', 'Web 开发技术和框架', '#61dafb'),
         ('教程', 'tutorial', '技术教程和指南', '#28a745')"
    ).execute(conn)?;
    
    // 创建示例文章
    diesel::sql_query(
        "INSERT INTO posts (title, slug, content, excerpt, author_id, published, view_count) VALUES 
         ('Rust 编程语言入门指南', 'rust-getting-started', 
          'Rust 是一门系统编程语言，专注于安全、速度和并发。本文将带你了解 Rust 的基础概念...', 
          'Rust 编程语言的完整入门指南', 2, true, 150),
         ('使用 Diesel 操作 GaussDB 数据库', 'diesel-gaussdb-guide',
          '本文介绍如何使用 Diesel ORM 框架操作 GaussDB 数据库，包括连接配置、模型定义等...', 
          '完整的 Diesel-GaussDB 使用指南', 2, true, 89),
         ('现代 Web 开发最佳实践', 'modern-web-dev-practices',
          '现代 Web 开发涉及众多技术栈，本文总结了一些最佳实践和常用模式...', 
          'Web 开发的最佳实践总结', 2, true, 67)"
    ).execute(conn)?;
    
    log::info!("✅ 示例数据创建完成");
    Ok(())
}
