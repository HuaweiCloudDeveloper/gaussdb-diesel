# 基础 CRUD 操作示例

本示例演示如何使用 diesel-gaussdb 执行基本的创建、读取、更新、删除操作。

## 📋 项目设置

### 1. 依赖配置

```toml
[dependencies]
diesel = { version = "2.2", features = ["postgres", "chrono", "serde_json"] }
diesel-gaussdb = "1.0"
gaussdb = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### 2. 数据库模式

```sql
-- migrations/2024-01-01-000000_create_users/up.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    full_name VARCHAR(100) NOT NULL,
    age INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    profile JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active);
```

## 🦀 Rust 代码实现

### 1. 模型定义

```rust
// src/models.rs
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use uuid::Uuid;
use serde_json::Value as JsonValue;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel_gaussdb::GaussDb))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub age: Option<i32>,
    pub is_active: bool,
    pub profile: Option<JsonValue>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub age: Option<i32>,
    pub profile: Option<JsonValue>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub age: Option<i32>,
    pub is_active: Option<bool>,
    pub profile: Option<JsonValue>,
    pub updated_at: NaiveDateTime,
}

impl NewUser {
    pub fn new(username: &str, email: &str, full_name: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            full_name: full_name.to_string(),
            age: None,
            profile: None,
        }
    }
    
    pub fn with_age(mut self, age: i32) -> Self {
        self.age = Some(age);
        self
    }
    
    pub fn with_profile(mut self, profile: JsonValue) -> Self {
        self.profile = Some(profile);
        self
    }
}
```

### 2. CRUD 操作实现

```rust
// src/crud.rs
use diesel::prelude::*;
use diesel_gaussdb::GaussDbConnection;
use crate::models::{User, NewUser, UpdateUser};
use crate::schema::users;
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;

pub struct UserCrud;

impl UserCrud {
    /// 创建新用户
    pub fn create(conn: &mut GaussDbConnection, new_user: NewUser) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    /// 根据ID查询用户
    pub fn find_by_id(conn: &mut GaussDbConnection, user_id: Uuid) -> QueryResult<User> {
        users::table
            .find(user_id)
            .select(User::as_select())
            .first(conn)
    }
    
    /// 根据用户名查询用户
    pub fn find_by_username(conn: &mut GaussDbConnection, username: &str) -> QueryResult<User> {
        users::table
            .filter(users::username.eq(username))
            .select(User::as_select())
            .first(conn)
    }
    
    /// 根据邮箱查询用户
    pub fn find_by_email(conn: &mut GaussDbConnection, email: &str) -> QueryResult<User> {
        users::table
            .filter(users::email.eq(email))
            .select(User::as_select())
            .first(conn)
    }
    
    /// 查询所有活跃用户
    pub fn find_active_users(conn: &mut GaussDbConnection) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::is_active.eq(true))
            .select(User::as_select())
            .order(users::created_at.desc())
            .load(conn)
    }
    
    /// 分页查询用户
    pub fn find_paginated(
        conn: &mut GaussDbConnection,
        page: i64,
        per_page: i64,
    ) -> QueryResult<Vec<User>> {
        users::table
            .select(User::as_select())
            .order(users::created_at.desc())
            .limit(per_page)
            .offset(page * per_page)
            .load(conn)
    }
    
    /// 更新用户信息
    pub fn update(
        conn: &mut GaussDbConnection,
        user_id: Uuid,
        mut update_data: UpdateUser,
    ) -> QueryResult<User> {
        // 自动更新 updated_at 字段
        update_data.updated_at = Utc::now().naive_utc();
        
        diesel::update(users::table.find(user_id))
            .set(&update_data)
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    /// 软删除用户（设置为非活跃状态）
    pub fn soft_delete(conn: &mut GaussDbConnection, user_id: Uuid) -> QueryResult<User> {
        let update_data = UpdateUser {
            username: None,
            email: None,
            full_name: None,
            age: None,
            is_active: Some(false),
            profile: None,
            updated_at: Utc::now().naive_utc(),
        };
        
        diesel::update(users::table.find(user_id))
            .set(&update_data)
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    /// 硬删除用户
    pub fn delete(conn: &mut GaussDbConnection, user_id: Uuid) -> QueryResult<usize> {
        diesel::delete(users::table.find(user_id))
            .execute(conn)
    }
    
    /// 批量创建用户
    pub fn create_batch(
        conn: &mut GaussDbConnection,
        new_users: Vec<NewUser>,
    ) -> QueryResult<Vec<User>> {
        diesel::insert_into(users::table)
            .values(&new_users)
            .returning(User::as_returning())
            .get_results(conn)
    }
    
    /// 统计用户数量
    pub fn count_all(conn: &mut GaussDbConnection) -> QueryResult<i64> {
        users::table.count().get_result(conn)
    }
    
    /// 统计活跃用户数量
    pub fn count_active(conn: &mut GaussDbConnection) -> QueryResult<i64> {
        users::table
            .filter(users::is_active.eq(true))
            .count()
            .get_result(conn)
    }
    
    /// 按年龄范围查询用户
    pub fn find_by_age_range(
        conn: &mut GaussDbConnection,
        min_age: i32,
        max_age: i32,
    ) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::age.ge(min_age))
            .filter(users::age.le(max_age))
            .select(User::as_select())
            .order(users::age.asc())
            .load(conn)
    }
    
    /// 搜索用户（按用户名或全名）
    pub fn search(conn: &mut GaussDbConnection, query: &str) -> QueryResult<Vec<User>> {
        let search_pattern = format!("%{}%", query);
        
        users::table
            .filter(
                users::username.ilike(&search_pattern)
                .or(users::full_name.ilike(&search_pattern))
            )
            .select(User::as_select())
            .order(users::created_at.desc())
            .load(conn)
    }
}
```

### 3. 使用示例

```rust
// src/main.rs
mod models;
mod crud;
mod schema;

use diesel_gaussdb::GaussDbConnection;
use crud::UserCrud;
use models::NewUser;
use serde_json::json;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立数据库连接
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let mut conn = GaussDbConnection::establish(&database_url)?;
    
    println!("🚀 diesel-gaussdb CRUD 操作示例");
    
    // 1. 创建用户
    println!("\n📝 创建用户...");
    let new_user = NewUser::new("zhangsan", "zhangsan@example.com", "张三")
        .with_age(25)
        .with_profile(json!({
            "bio": "软件工程师",
            "location": "北京",
            "interests": ["编程", "阅读", "旅行"]
        }));
    
    let created_user = UserCrud::create(&mut conn, new_user)?;
    println!("✅ 用户创建成功: {}", created_user.username);
    println!("   ID: {}", created_user.id);
    println!("   邮箱: {}", created_user.email);
    
    // 2. 查询用户
    println!("\n🔍 查询用户...");
    let found_user = UserCrud::find_by_username(&mut conn, "zhangsan")?;
    println!("✅ 找到用户: {} ({})", found_user.full_name, found_user.email);
    
    // 3. 更新用户
    println!("\n📝 更新用户信息...");
    let update_data = models::UpdateUser {
        full_name: Some("张三丰".to_string()),
        age: Some(30),
        profile: Some(json!({
            "bio": "高级软件工程师",
            "location": "上海",
            "interests": ["编程", "武术", "哲学"],
            "skills": ["Rust", "Python", "JavaScript"]
        })),
        username: None,
        email: None,
        is_active: None,
        updated_at: chrono::Utc::now().naive_utc(),
    };
    
    let updated_user = UserCrud::update(&mut conn, created_user.id, update_data)?;
    println!("✅ 用户更新成功: {}", updated_user.full_name);
    println!("   年龄: {:?}", updated_user.age);
    
    // 4. 批量创建用户
    println!("\n📝 批量创建用户...");
    let batch_users = vec![
        NewUser::new("lisi", "lisi@example.com", "李四").with_age(28),
        NewUser::new("wangwu", "wangwu@example.com", "王五").with_age(32),
        NewUser::new("zhaoliu", "zhaoliu@example.com", "赵六").with_age(24),
    ];
    
    let created_users = UserCrud::create_batch(&mut conn, batch_users)?;
    println!("✅ 批量创建 {} 个用户", created_users.len());
    
    // 5. 查询所有活跃用户
    println!("\n📋 查询所有活跃用户...");
    let active_users = UserCrud::find_active_users(&mut conn)?;
    println!("✅ 找到 {} 个活跃用户:", active_users.len());
    for user in &active_users {
        println!("   - {}: {} ({})", user.username, user.full_name, user.email);
    }
    
    // 6. 分页查询
    println!("\n📄 分页查询用户...");
    let page_users = UserCrud::find_paginated(&mut conn, 0, 2)?;
    println!("✅ 第1页用户 (每页2个):");
    for user in &page_users {
        println!("   - {}: {}", user.username, user.full_name);
    }
    
    // 7. 搜索用户
    println!("\n🔍 搜索用户...");
    let search_results = UserCrud::search(&mut conn, "张")?;
    println!("✅ 搜索结果 ({} 个):", search_results.len());
    for user in &search_results {
        println!("   - {}: {}", user.username, user.full_name);
    }
    
    // 8. 按年龄范围查询
    println!("\n🎂 按年龄范围查询用户 (25-30岁)...");
    let age_range_users = UserCrud::find_by_age_range(&mut conn, 25, 30)?;
    println!("✅ 找到 {} 个用户:", age_range_users.len());
    for user in &age_range_users {
        println!("   - {}: {} ({}岁)", user.username, user.full_name, user.age.unwrap_or(0));
    }
    
    // 9. 统计信息
    println!("\n📊 统计信息...");
    let total_count = UserCrud::count_all(&mut conn)?;
    let active_count = UserCrud::count_active(&mut conn)?;
    println!("✅ 总用户数: {}", total_count);
    println!("✅ 活跃用户数: {}", active_count);
    
    // 10. 软删除用户
    println!("\n🗑️ 软删除用户...");
    let soft_deleted = UserCrud::soft_delete(&mut conn, created_users[0].id)?;
    println!("✅ 用户 {} 已设置为非活跃状态", soft_deleted.username);
    
    // 11. 验证软删除
    let active_count_after = UserCrud::count_active(&mut conn)?;
    println!("✅ 软删除后活跃用户数: {}", active_count_after);
    
    println!("\n🎉 CRUD 操作示例完成！");
    
    Ok(())
}
```

## 🔧 高级功能

### 1. 事务处理

```rust
use diesel::prelude::*;

// 在事务中执行多个操作
let result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
    // 创建用户
    let user = UserCrud::create(conn, new_user)?;
    
    // 更新相关数据
    UserCrud::update(conn, user.id, update_data)?;
    
    // 如果任何操作失败，整个事务会回滚
    Ok(user)
})?;
```

### 2. 连接池使用

```rust
use diesel::r2d2::{ConnectionManager, Pool};

type DbPool = Pool<ConnectionManager<GaussDbConnection>>;

fn create_user_with_pool(pool: &DbPool, new_user: NewUser) -> QueryResult<User> {
    let mut conn = pool.get().unwrap();
    UserCrud::create(&mut conn, new_user)
}
```

### 3. 异步支持

```rust
use diesel_async::{AsyncConnection, AsyncPgConnection};

async fn create_user_async(
    conn: &mut AsyncPgConnection,
    new_user: NewUser,
) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
        .await
}
```

## 🚀 运行示例

```bash
# 设置环境变量
export DATABASE_URL="host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=crud_example"

# 运行迁移
diesel migration run

# 运行示例
cargo run
```

## 📝 注意事项

1. **错误处理**: 在生产环境中应该使用更完善的错误处理
2. **数据验证**: 在插入数据前应该进行适当的验证
3. **索引优化**: 根据查询模式创建合适的数据库索引
4. **连接池**: 在高并发环境中使用连接池
5. **事务管理**: 对于复杂操作使用事务确保数据一致性

---

**这个示例展示了 diesel-gaussdb 的基本 CRUD 操作，为您的应用开发提供了坚实的基础！** 🚀
