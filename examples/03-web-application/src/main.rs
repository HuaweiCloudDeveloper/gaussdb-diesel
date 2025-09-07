//! Diesel-GaussDB Web 应用示例
//!
//! 这个示例展示了如何在 Web 应用中使用 diesel-gaussdb，包括：
//! - REST API 设计
//! - 数据库连接管理
//! - JSON 序列化/反序列化
//! - 错误处理

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
use std::net::SocketAddr;
use anyhow::{Result, Context};
use log::info;
use std::env;

/// 用户数据结构
#[derive(Debug, Serialize, Deserialize, diesel::QueryableByName)]
struct User {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
    age: Option<i32>,
}

/// 新用户数据结构
#[derive(Debug, Deserialize)]
struct NewUser {
    name: String,
    email: String,
    age: Option<i32>,
}

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

/// 初始化数据库
fn init_database() -> Result<()> {
    let mut conn = establish_connection()?;
    
    // 创建用户表
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE,
            age INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(&mut conn)?;

    info!("✅ 数据库初始化完成");
    Ok(())
}

/// 健康检查
async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("服务运行正常".to_string()))
}

/// 获取所有用户
async fn get_users() -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = diesel::sql_query(
        "SELECT id, name, email, age FROM users ORDER BY id"
    )
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(users)))
}

/// 根据 ID 获取用户
async fn get_user(Path(user_id): Path<i32>) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = diesel::sql_query(
        "SELECT id, name, email, age FROM users WHERE id = $1"
    )
    .bind::<diesel::sql_types::Integer, _>(user_id)
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match users.into_iter().next() {
        Some(user) => Ok(Json(ApiResponse::success(user))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 创建新用户
async fn create_user(Json(new_user): Json<NewUser>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query(
        "INSERT INTO users (name, email, age) VALUES ($1, $2, $3)"
    )
    .bind::<diesel::sql_types::Text, _>(&new_user.name)
    .bind::<diesel::sql_types::Text, _>(&new_user.email)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(new_user.age)
    .execute(&mut conn);

    match result {
        Ok(_) => Ok(Json(ApiResponse::success("用户创建成功".to_string()))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// 更新用户
async fn update_user(
    Path(user_id): Path<i32>,
    Json(update_data): Json<NewUser>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query(
        "UPDATE users SET name = $1, email = $2, age = $3 WHERE id = $4"
    )
    .bind::<diesel::sql_types::Text, _>(&update_data.name)
    .bind::<diesel::sql_types::Text, _>(&update_data.email)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(update_data.age)
    .bind::<diesel::sql_types::Integer, _>(user_id)
    .execute(&mut conn);

    match result {
        Ok(0) => Err(StatusCode::NOT_FOUND),
        Ok(_) => Ok(Json(ApiResponse::success("用户更新成功".to_string()))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// 删除用户
async fn delete_user(Path(user_id): Path<i32>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = diesel::sql_query("DELETE FROM users WHERE id = $1")
        .bind::<diesel::sql_types::Integer, _>(user_id)
        .execute(&mut conn);

    match result {
        Ok(0) => Err(StatusCode::NOT_FOUND),
        Ok(_) => Ok(Json(ApiResponse::success("用户删除成功".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 用户统计
async fn user_stats() -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(diesel::QueryableByName)]
    struct Stats {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_users: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        avg_age: Option<f64>,
    }

    let stats: Vec<Stats> = diesel::sql_query(
        "SELECT COUNT(*) as total_users, AVG(age) as avg_age FROM users"
    )
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(stats) = stats.into_iter().next() {
        let response = serde_json::json!({
            "total_users": stats.total_users,
            "average_age": stats.avg_age
        });
        Ok(Json(ApiResponse::success(response)))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// 创建路由
fn create_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/users", get(get_users))
        .route("/api/users", post(create_user))
        .route("/api/users/:id", get(get_user))
        .route("/api/users/:id", axum::routing::put(update_user))
        .route("/api/users/:id", axum::routing::delete(delete_user))
        .route("/api/users/search", get(search_users))
        .route("/api/users/batch", post(batch_create_users))
        .route("/api/stats", get(user_stats))
        .route("/api/stats/age-distribution", get(age_distribution))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    info!("🚀 启动 Diesel-GaussDB Web 应用示例");

    // 初始化数据库
    init_database()?;

    // 创建路由
    let app = create_routes();

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("🌐 服务器启动在 http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 搜索用户
async fn search_users(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sql = "SELECT id, name, email, age FROM users WHERE 1=1".to_string();

    if let Some(name) = params.get("name") {
        sql.push_str(&format!(" AND name ILIKE '%{}%'", name));
    }

    if let Some(email) = params.get("email") {
        sql.push_str(&format!(" AND email ILIKE '%{}%'", email));
    }

    if let Some(min_age) = params.get("min_age") {
        if let Ok(age) = min_age.parse::<i32>() {
            sql.push_str(&format!(" AND age >= {}", age));
        }
    }

    if let Some(max_age) = params.get("max_age") {
        if let Ok(age) = max_age.parse::<i32>() {
            sql.push_str(&format!(" AND age <= {}", age));
        }
    }

    sql.push_str(" ORDER BY id LIMIT 50");

    let users: Vec<User> = diesel::sql_query(sql)
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::success(users)))
}

/// 批量创建用户
async fn batch_create_users(
    Json(users): Json<Vec<NewUser>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if users.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if users.len() > 100 {
        return Err(StatusCode::BAD_REQUEST); // 限制批量大小
    }

    let values: Vec<String> = users
        .iter()
        .map(|user| {
            format!(
                "('{}', '{}', {})",
                user.name.replace("'", "''"), // 简单的 SQL 注入防护
                user.email.replace("'", "''"),
                user.age.map_or("NULL".to_string(), |a| a.to_string())
            )
        })
        .collect();

    let sql = format!(
        "INSERT INTO users (name, email, age) VALUES {}",
        values.join(", ")
    );

    let result = diesel::sql_query(sql).execute(&mut conn);

    match result {
        Ok(count) => Ok(Json(ApiResponse::success(format!("成功创建 {} 个用户", count)))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// 年龄分布统计
async fn age_distribution() -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(diesel::QueryableByName)]
    struct AgeGroup {
        #[diesel(sql_type = diesel::sql_types::Text)]
        age_group: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let distribution: Vec<AgeGroup> = diesel::sql_query(
        "SELECT
           CASE
             WHEN age < 18 THEN '未成年'
             WHEN age BETWEEN 18 AND 30 THEN '青年'
             WHEN age BETWEEN 31 AND 50 THEN '中年'
             WHEN age > 50 THEN '老年'
             ELSE '未知'
           END as age_group,
           COUNT(*) as count
         FROM users
         WHERE age IS NOT NULL
         GROUP BY age_group
         ORDER BY
           CASE
             WHEN age < 18 THEN 1
             WHEN age BETWEEN 18 AND 30 THEN 2
             WHEN age BETWEEN 31 AND 50 THEN 3
             WHEN age > 50 THEN 4
             ELSE 5
           END"
    )
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut result = serde_json::Map::new();
    for group in distribution {
        result.insert(group.age_group, serde_json::Value::Number(group.count.into()));
    }

    Ok(Json(ApiResponse::success(serde_json::Value::Object(result))))
}
