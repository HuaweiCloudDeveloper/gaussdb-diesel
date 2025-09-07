# 真实博客系统示例

这是一个基于 Diesel-GaussDB 的完整博客系统，展示了在真实 Web 应用中如何使用 diesel-gaussdb。

## 功能特性

### 核心功能
- ✅ 用户注册和登录
- ✅ 文章发布和管理
- ✅ 评论系统
- ✅ 标签管理
- ✅ 文章搜索
- ✅ 用户权限管理

### 技术特性
- ✅ RESTful API 设计
- ✅ 数据库连接池
- ✅ 事务处理
- ✅ 错误处理
- ✅ 日志记录
- ✅ 配置管理
- ✅ 模板渲染

## 技术栈

- **Web 框架**: Axum
- **数据库**: GaussDB/OpenGauss
- **ORM**: Diesel-GaussDB
- **模板引擎**: Askama
- **认证**: JWT + bcrypt
- **连接池**: R2D2

## 项目结构

```
src/
├── main.rs              # 应用入口
├── config.rs            # 配置管理
├── database.rs          # 数据库连接池
├── error.rs             # 错误处理
├── models/              # 数据模型
│   ├── mod.rs
│   ├── user.rs          # 用户模型
│   ├── post.rs          # 文章模型
│   ├── comment.rs       # 评论模型
│   └── tag.rs           # 标签模型
├── handlers/            # HTTP 处理器
│   ├── mod.rs
│   ├── auth.rs          # 认证相关
│   ├── posts.rs         # 文章相关
│   ├── comments.rs      # 评论相关
│   └── users.rs         # 用户相关
├── services/            # 业务逻辑
│   ├── mod.rs
│   ├── auth_service.rs  # 认证服务
│   ├── post_service.rs  # 文章服务
│   └── user_service.rs  # 用户服务
├── schema.rs            # 数据库表结构
└── templates/           # HTML 模板
    ├── base.html
    ├── index.html
    ├── post.html
    └── login.html
```

## 快速开始

### 1. 环境准备

```bash
# 启动 GaussDB/OpenGauss
docker run --name opengauss \
  -e GS_PASSWORD=Gaussdb@123 \
  -p 5432:5432 \
  -d opengauss/opengauss:7.0.0-RC1.B023

# 设置环境变量
export GAUSSDB_URL="host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres"
export RUST_LOG=info
```

### 2. 运行应用

```bash
cd examples/04-real-world-blog
cargo run
```

### 3. 访问应用

打开浏览器访问：http://localhost:8080

## API 接口

### 认证接口

```bash
# 用户注册
POST /api/auth/register
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}

# 用户登录
POST /api/auth/login
Content-Type: application/json

{
  "email": "test@example.com",
  "password": "password123"
}
```

### 文章接口

```bash
# 获取文章列表
GET /api/posts?page=1&limit=10

# 获取单篇文章
GET /api/posts/{id}

# 创建文章
POST /api/posts
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "文章标题",
  "content": "文章内容",
  "tags": ["Rust", "数据库"]
}

# 更新文章
PUT /api/posts/{id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "更新的标题",
  "content": "更新的内容"
}

# 删除文章
DELETE /api/posts/{id}
Authorization: Bearer {token}
```

### 评论接口

```bash
# 获取文章评论
GET /api/posts/{post_id}/comments

# 添加评论
POST /api/posts/{post_id}/comments
Authorization: Bearer {token}
Content-Type: application/json

{
  "content": "评论内容"
}
```

## 数据库设计

### 用户表 (users)
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL,
    avatar_url VARCHAR,
    bio TEXT,
    is_admin BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);
```

### 文章表 (posts)
```sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    slug VARCHAR UNIQUE NOT NULL,
    content TEXT NOT NULL,
    excerpt TEXT,
    author_id INTEGER NOT NULL REFERENCES users(id),
    published BOOLEAN DEFAULT FALSE,
    view_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);
```

### 评论表 (comments)
```sql
CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES posts(id),
    author_id INTEGER NOT NULL REFERENCES users(id),
    parent_id INTEGER REFERENCES comments(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 标签表 (tags)
```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    slug VARCHAR UNIQUE NOT NULL,
    description TEXT,
    color VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 文章标签关联表 (post_tags)
```sql
CREATE TABLE post_tags (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES posts(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(post_id, tag_id)
);
```

## 特色功能

### 1. 高级查询示例

```rust
// 获取热门文章（按评论数和浏览数排序）
pub async fn get_popular_posts(pool: &DbPool) -> Result<Vec<PostWithStats>> {
    let mut conn = pool.get()?;
    
    posts::table
        .left_join(comments::table)
        .group_by(posts::all_columns)
        .select((
            Post::as_select(),
            diesel::dsl::count(comments::id.nullable()),
        ))
        .order_by((
            diesel::dsl::count(comments::id.nullable()).desc(),
            posts::view_count.desc(),
        ))
        .limit(10)
        .load(&mut conn)
}

// 使用 CTE 查询用户活跃度
pub async fn get_user_activity_stats(pool: &DbPool) -> Result<Vec<UserStats>> {
    let mut conn = pool.get()?;
    
    diesel::sql_query(
        "WITH user_stats AS (
           SELECT u.id, u.username,
                  COUNT(DISTINCT p.id) as post_count,
                  COUNT(DISTINCT c.id) as comment_count,
                  SUM(p.view_count) as total_views
           FROM users u
           LEFT JOIN posts p ON u.id = p.author_id
           LEFT JOIN comments c ON u.id = c.author_id
           GROUP BY u.id, u.username
         )
         SELECT * FROM user_stats
         ORDER BY (post_count * 10 + comment_count * 2 + total_views / 100) DESC"
    ).load(&mut conn)
}
```

### 2. 事务处理示例

```rust
// 发布文章并添加标签（事务处理）
pub async fn publish_post_with_tags(
    pool: &DbPool,
    new_post: NewPost,
    tag_names: Vec<String>,
) -> Result<Post> {
    let mut conn = pool.get()?;
    
    conn.transaction(|conn| {
        // 插入文章
        let post: Post = diesel::insert_into(posts::table)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn)?;
        
        // 处理标签
        for tag_name in tag_names {
            let tag = find_or_create_tag(conn, &tag_name)?;
            
            diesel::insert_into(post_tags::table)
                .values(&NewPostTag {
                    post_id: post.id,
                    tag_id: tag.id,
                })
                .execute(conn)?;
        }
        
        Ok(post)
    })
}
```

### 3. 连接池配置

```rust
// 生产级连接池配置
pub fn create_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<GaussDBConnection>::new(database_url);
    
    Pool::builder()
        .max_size(20)                    // 最大连接数
        .min_idle(Some(5))               // 最小空闲连接数
        .connection_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(600)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .build(manager)
        .map_err(|e| anyhow::anyhow!("Failed to create pool: {}", e))
}
```

## 性能优化

### 1. 数据库索引

```sql
-- 文章查询优化
CREATE INDEX idx_posts_published_created ON posts(published, created_at DESC);
CREATE INDEX idx_posts_author_published ON posts(author_id, published);
CREATE INDEX idx_posts_slug ON posts(slug);

-- 评论查询优化
CREATE INDEX idx_comments_post_created ON comments(post_id, created_at);
CREATE INDEX idx_comments_author ON comments(author_id);

-- 标签查询优化
CREATE INDEX idx_post_tags_post ON post_tags(post_id);
CREATE INDEX idx_post_tags_tag ON post_tags(tag_id);
```

### 2. 查询优化

- 使用连接池减少连接开销
- 实现查询结果缓存
- 分页查询避免大量数据传输
- 使用 EXPLAIN 分析查询性能

## 部署指南

### 1. 生产环境配置

```toml
# config/production.toml
[database]
url = "host=prod-gaussdb port=5432 user=blog_user password=secure_password dbname=blog_prod"
pool_size = 20
connection_timeout = 30

[server]
host = "0.0.0.0"
port = 8080

[logging]
level = "info"
```

### 2. Docker 部署

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/blog_server /usr/local/bin/
EXPOSE 8080
CMD ["blog_server"]
```

## 测试

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration

# 性能测试
cargo test --release --test performance
```

---

**🎯 这个博客系统展示了 diesel-gaussdb 在真实 Web 应用中的完整使用方式！**
