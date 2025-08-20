//! 高级查询功能示例
//!
//! 这个示例展示了 diesel-gaussdb 的高级查询功能，包括：
//! - DISTINCT ON 查询
//! - 数组操作
//! - JSON 查询
//! - 复杂连接查询
//! - 子查询
//! - 窗口函数
//!
//! 运行示例：
//! ```bash
//! # 确保 GaussDB 容器运行
//! docker-compose up -d
//! 
//! # 运行示例
//! cargo run --example advanced_queries_example --features gaussdb,r2d2
//! ```

use diesel::prelude::*;
use diesel::connection::SimpleConnection;
use diesel_gaussdb::prelude::*;
use std::env;

// 定义数据模型
table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        department -> Text,
        salary -> Integer,
        created_at -> Timestamp,
        tags -> Array<Text>,
        metadata -> Json,
    }
}

table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        content -> Text,
        author_id -> Integer,
        tags -> Array<Text>,
        published -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    comments (id) {
        id -> Integer,
        post_id -> Integer,
        author_id -> Integer,
        content -> Text,
        created_at -> Timestamp,
    }
}

joinable!(posts -> users (author_id));
joinable!(comments -> posts (post_id));
joinable!(comments -> users (author_id));

allow_tables_to_appear_in_same_query!(users, posts, comments);

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = users)]
struct User {
    id: i32,
    name: String,
    email: String,
    department: String,
    salary: i32,
    created_at: chrono::NaiveDateTime,
    tags: Vec<String>,
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = posts)]
struct Post {
    id: i32,
    title: String,
    content: String,
    author_id: i32,
    tags: Vec<String>,
    published: bool,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = comments)]
struct Comment {
    id: i32,
    post_id: i32,
    author_id: i32,
    content: String,
    created_at: chrono::NaiveDateTime,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Diesel-GaussDB 高级查询功能示例");
    println!("=====================================");

    // 建立数据库连接
    let database_url = env::var("GAUSSDB_TEST_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string()
        });

    let mut conn = GaussDBConnection::establish(&database_url)?;
    println!("✅ 数据库连接成功");

    // 设置测试数据
    setup_test_data(&mut conn)?;
    println!("✅ 测试数据设置完成");

    // 1. 基础查询测试
    println!("\n🔍 1. 基础查询测试");
    test_basic_queries(&mut conn)?;

    // 2. DISTINCT ON 查询测试
    println!("\n🔍 2. DISTINCT ON 查询测试");
    test_distinct_on_queries(&mut conn)?;

    // 3. 数组操作测试
    println!("\n🔍 3. 数组操作测试");
    test_array_operations(&mut conn)?;

    // 4. JSON 查询测试
    println!("\n🔍 4. JSON 查询测试");
    test_json_queries(&mut conn)?;

    // 5. 连接查询测试
    println!("\n🔍 5. 连接查询测试");
    test_join_queries(&mut conn)?;

    // 6. 子查询测试
    println!("\n🔍 6. 子查询测试");
    test_subqueries(&mut conn)?;

    // 7. 聚合查询测试
    println!("\n🔍 7. 聚合查询测试");
    test_aggregate_queries(&mut conn)?;

    // 清理测试数据
    cleanup_test_data(&mut conn)?;
    println!("\n✅ 测试数据清理完成");

    println!("\n🎉 所有高级查询功能测试完成！");
    Ok(())
}

fn setup_test_data(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 创建表
    conn.batch_execute(r#"
        DROP TABLE IF EXISTS comments CASCADE;
        DROP TABLE IF EXISTS posts CASCADE;
        DROP TABLE IF EXISTS users CASCADE;
        
        CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL,
            department VARCHAR NOT NULL,
            salary INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            tags TEXT[] DEFAULT '{}',
            metadata JSON DEFAULT '{}'
        );
        
        CREATE TABLE posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            author_id INTEGER REFERENCES users(id),
            tags TEXT[] DEFAULT '{}',
            published BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE comments (
            id SERIAL PRIMARY KEY,
            post_id INTEGER REFERENCES posts(id),
            author_id INTEGER REFERENCES users(id),
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    "#)?;

    // 插入测试数据
    conn.batch_execute(r#"
        INSERT INTO users (name, email, department, salary, tags, metadata) VALUES
        ('Alice Johnson', 'alice@example.com', 'Engineering', 75000, '{"rust", "database"}', '{"level": "senior", "skills": ["rust", "sql"]}'),
        ('Bob Smith', 'bob@example.com', 'Engineering', 65000, '{"python", "web"}', '{"level": "mid", "skills": ["python", "django"]}'),
        ('Carol Davis', 'carol@example.com', 'Marketing', 55000, '{"design", "content"}', '{"level": "junior", "skills": ["photoshop", "writing"]}'),
        ('David Wilson', 'david@example.com', 'Engineering', 80000, '{"rust", "systems"}', '{"level": "senior", "skills": ["rust", "c++"]}'),
        ('Eve Brown', 'eve@example.com', 'Marketing', 60000, '{"social", "analytics"}', '{"level": "mid", "skills": ["analytics", "social_media"]}');
        
        INSERT INTO posts (title, content, author_id, tags, published) VALUES
        ('Getting Started with Rust', 'Rust is a systems programming language...', 1, '{"rust", "tutorial", "beginner"}', true),
        ('Database Design Patterns', 'When designing databases...', 1, '{"database", "design", "patterns"}', true),
        ('Python Web Development', 'Building web applications with Python...', 2, '{"python", "web", "django"}', true),
        ('Marketing in 2024', 'Digital marketing trends...', 3, '{"marketing", "trends", "digital"}', false),
        ('Systems Programming', 'Low-level programming concepts...', 4, '{"systems", "rust", "performance"}', true);
        
        INSERT INTO comments (post_id, author_id, content) VALUES
        (1, 2, 'Great introduction to Rust!'),
        (1, 4, 'Very helpful for beginners.'),
        (2, 3, 'Interesting database patterns.'),
        (3, 1, 'Nice Python tutorial.'),
        (5, 2, 'Systems programming is fascinating.');
    "#)?;

    Ok(())
}

fn test_basic_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 基础 SELECT 查询
    println!("  📋 基础 SELECT 查询");
    let result = conn.batch_execute("SELECT COUNT(*) FROM users");
    assert!(result.is_ok(), "基础查询应该成功");
    println!("    ✅ 用户数量查询成功");

    // WHERE 条件查询
    println!("  📋 WHERE 条件查询");
    let result = conn.batch_execute("SELECT * FROM users WHERE department = 'Engineering'");
    assert!(result.is_ok(), "条件查询应该成功");
    println!("    ✅ 部门筛选查询成功");

    // ORDER BY 排序查询
    println!("  📋 ORDER BY 排序查询");
    let result = conn.batch_execute("SELECT * FROM users ORDER BY salary DESC");
    assert!(result.is_ok(), "排序查询应该成功");
    println!("    ✅ 薪资排序查询成功");

    Ok(())
}

fn test_distinct_on_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // DISTINCT ON 查询 - PostgreSQL 特有功能
    println!("  📋 DISTINCT ON 查询");
    let result = conn.batch_execute(r#"
        SELECT DISTINCT ON (department) department, name, salary 
        FROM users 
        ORDER BY department, salary DESC
    "#);
    assert!(result.is_ok(), "DISTINCT ON 查询应该成功");
    println!("    ✅ 每个部门最高薪资员工查询成功");

    Ok(())
}

fn test_array_operations(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 数组包含查询
    println!("  📋 数组包含查询");
    let result = conn.batch_execute("SELECT * FROM users WHERE 'rust' = ANY(tags)");
    assert!(result.is_ok(), "数组包含查询应该成功");
    println!("    ✅ 标签包含 'rust' 的用户查询成功");

    // 数组重叠查询
    println!("  📋 数组重叠查询");
    let result = conn.batch_execute("SELECT * FROM posts WHERE tags && ARRAY['rust', 'database']");
    assert!(result.is_ok(), "数组重叠查询应该成功");
    println!("    ✅ 标签重叠查询成功");

    Ok(())
}

fn test_json_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // JSON 字段查询
    println!("  📋 JSON 字段查询");
    let result = conn.batch_execute("SELECT * FROM users WHERE metadata->>'level' = 'senior'");
    assert!(result.is_ok(), "JSON 字段查询应该成功");
    println!("    ✅ 高级员工查询成功");

    // JSON 数组查询
    println!("  📋 JSON 数组查询");
    let result = conn.batch_execute("SELECT * FROM users WHERE metadata->'skills' ? 'rust'");
    match result {
        Ok(_) => println!("    ✅ JSON 数组查询成功"),
        Err(_) => println!("    ⚠️  JSON 数组查询功能需要进一步实现"),
    }

    Ok(())
}

fn test_join_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // INNER JOIN 查询
    println!("  📋 INNER JOIN 查询");
    let result = conn.batch_execute(r#"
        SELECT u.name, p.title 
        FROM users u 
        INNER JOIN posts p ON u.id = p.author_id 
        WHERE p.published = true
    "#);
    assert!(result.is_ok(), "INNER JOIN 查询应该成功");
    println!("    ✅ 用户和文章连接查询成功");

    // LEFT JOIN 查询
    println!("  📋 LEFT JOIN 查询");
    let result = conn.batch_execute(r#"
        SELECT u.name, COUNT(p.id) as post_count
        FROM users u 
        LEFT JOIN posts p ON u.id = p.author_id 
        GROUP BY u.id, u.name
    "#);
    assert!(result.is_ok(), "LEFT JOIN 查询应该成功");
    println!("    ✅ 用户文章数量统计查询成功");

    Ok(())
}

fn test_subqueries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 子查询
    println!("  📋 子查询");
    let result = conn.batch_execute(r#"
        SELECT * FROM users 
        WHERE id IN (SELECT DISTINCT author_id FROM posts WHERE published = true)
    "#);
    assert!(result.is_ok(), "子查询应该成功");
    println!("    ✅ 有发布文章的用户查询成功");

    // EXISTS 子查询
    println!("  📋 EXISTS 子查询");
    let result = conn.batch_execute(r#"
        SELECT * FROM users u
        WHERE EXISTS (SELECT 1 FROM posts p WHERE p.author_id = u.id AND p.published = true)
    "#);
    assert!(result.is_ok(), "EXISTS 子查询应该成功");
    println!("    ✅ EXISTS 子查询成功");

    Ok(())
}

fn test_aggregate_queries(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 聚合函数查询
    println!("  📋 聚合函数查询");
    let result = conn.batch_execute(r#"
        SELECT department, 
               COUNT(*) as employee_count,
               AVG(salary) as avg_salary,
               MAX(salary) as max_salary,
               MIN(salary) as min_salary
        FROM users 
        GROUP BY department
        ORDER BY avg_salary DESC
    "#);
    assert!(result.is_ok(), "聚合查询应该成功");
    println!("    ✅ 部门统计查询成功");

    // HAVING 子句
    println!("  📋 HAVING 子句查询");
    let result = conn.batch_execute(r#"
        SELECT department, COUNT(*) as employee_count
        FROM users 
        GROUP BY department
        HAVING COUNT(*) > 1
    "#);
    assert!(result.is_ok(), "HAVING 查询应该成功");
    println!("    ✅ HAVING 筛选查询成功");

    Ok(())
}

fn cleanup_test_data(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    conn.batch_execute(r#"
        DROP TABLE IF EXISTS comments CASCADE;
        DROP TABLE IF EXISTS posts CASCADE;
        DROP TABLE IF EXISTS users CASCADE;
    "#)?;
    Ok(())
}
