# 快速开始指南

本指南将帮助您快速开始使用 diesel-gaussdb，从安装到运行您的第一个查询。

## 📋 前提条件

在开始之前，请确保您已安装：

- **Rust 1.70.0+**: [安装 Rust](https://rustup.rs/)
- **GaussDB 或 OpenGauss**: 数据库服务器
- **Git**: 用于克隆示例项目

## 🚀 第一步：项目设置

### 1. 创建新项目

```bash
cargo new my-gaussdb-app
cd my-gaussdb-app
```

### 2. 添加依赖

编辑 `Cargo.toml` 文件：

```toml
[dependencies]
diesel = { version = "2.2", features = ["postgres", "r2d2", "chrono"] }
diesel-gaussdb = "1.0"
gaussdb = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 3. 安装 Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

## 🗄️ 第二步：数据库设置

### 1. 启动 OpenGauss 数据库

使用 Docker：

```bash
docker run --name opengauss \
  -e GS_PASSWORD=Gaussdb@123 \
  -e GS_DB=my_app \
  -e GS_USER=gaussdb \
  -p 5432:5432 \
  -d opengauss/opengauss:7.0.0-RC1.B023
```

### 2. 配置环境变量

创建 `.env` 文件：

```env
DATABASE_URL=host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=my_app
```

### 3. 初始化 Diesel

```bash
diesel setup
```

## 📊 第三步：创建数据模型

### 1. 创建迁移

```bash
diesel migration generate create_users
```

### 2. 编写迁移 SQL

编辑 `migrations/*/up.sql`：

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

编辑 `migrations/*/down.sql`：

```sql
DROP TABLE users;
```

### 3. 运行迁移

```bash
diesel migration run
```

## 🦀 第四步：编写 Rust 代码

### 1. 定义模型

创建 `src/models.rs`：

```rust
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel_gaussdb::GaussDb))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub updated_at: NaiveDateTime,
}
```

### 2. 创建数据库连接

创建 `src/database.rs`：

```rust
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_gaussdb::GaussDbConnection;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<GaussDbConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<GaussDbConnection>>;

pub fn establish_connection() -> GaussDbConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL 环境变量必须设置");
    
    GaussDbConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("无法连接到数据库 {}", database_url))
}

pub fn create_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL 环境变量必须设置");
    
    let manager = ConnectionManager::<GaussDbConnection>::new(database_url);
    
    r2d2::Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("创建数据库连接池失败")
}
```

### 3. 实现 CRUD 操作

创建 `src/operations.rs`：

```rust
use diesel::prelude::*;
use crate::models::{User, NewUser, UpdateUser};
use crate::schema::users;
use diesel_gaussdb::GaussDbConnection;

pub fn create_user(conn: &mut GaussDbConnection, new_user: &NewUser) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(conn)
}

pub fn get_user_by_id(conn: &mut GaussDbConnection, user_id: i32) -> QueryResult<User> {
    users::table
        .find(user_id)
        .select(User::as_select())
        .first(conn)
}

pub fn get_all_users(conn: &mut GaussDbConnection) -> QueryResult<Vec<User>> {
    users::table
        .select(User::as_select())
        .order(users::created_at.desc())
        .load(conn)
}

pub fn update_user(
    conn: &mut GaussDbConnection, 
    user_id: i32, 
    update_data: &UpdateUser
) -> QueryResult<User> {
    diesel::update(users::table.find(user_id))
        .set(update_data)
        .returning(User::as_returning())
        .get_result(conn)
}

pub fn delete_user(conn: &mut GaussDbConnection, user_id: i32) -> QueryResult<usize> {
    diesel::delete(users::table.find(user_id))
        .execute(conn)
}

pub fn find_user_by_email(conn: &mut GaussDbConnection, email: &str) -> QueryResult<User> {
    users::table
        .filter(users::email.eq(email))
        .select(User::as_select())
        .first(conn)
}
```

### 4. 主程序

更新 `src/main.rs`：

```rust
mod database;
mod models;
mod operations;
mod schema;

use database::establish_connection;
use models::NewUser;
use operations::*;
use chrono::Utc;

fn main() {
    // 加载环境变量
    dotenv::dotenv().ok();
    
    // 建立数据库连接
    let mut connection = establish_connection();
    
    println!("🚀 diesel-gaussdb 快速开始示例");
    
    // 创建新用户
    let new_user = NewUser {
        name: "张三".to_string(),
        email: "zhangsan@example.com".to_string(),
    };
    
    match create_user(&mut connection, &new_user) {
        Ok(user) => {
            println!("✅ 创建用户成功: {:?}", user);
            
            // 查询用户
            match get_user_by_id(&mut connection, user.id) {
                Ok(found_user) => println!("🔍 查询用户: {:?}", found_user),
                Err(e) => println!("❌ 查询用户失败: {}", e),
            }
            
            // 更新用户
            let update_data = models::UpdateUser {
                name: Some("张三丰".to_string()),
                email: None,
                updated_at: Utc::now().naive_utc(),
            };
            
            match update_user(&mut connection, user.id, &update_data) {
                Ok(updated_user) => println!("📝 更新用户成功: {:?}", updated_user),
                Err(e) => println!("❌ 更新用户失败: {}", e),
            }
            
            // 查询所有用户
            match get_all_users(&mut connection) {
                Ok(users) => {
                    println!("📋 所有用户 ({} 个):", users.len());
                    for user in users {
                        println!("  - {}: {} ({})", user.id, user.name, user.email);
                    }
                },
                Err(e) => println!("❌ 查询所有用户失败: {}", e),
            }
        },
        Err(e) => println!("❌ 创建用户失败: {}", e),
    }
}
```

### 5. 添加必要的模块声明

更新 `src/lib.rs`：

```rust
pub mod database;
pub mod models;
pub mod operations;
pub mod schema;

pub use database::*;
pub use models::*;
pub use operations::*;
```

## ▶️ 第五步：运行应用

### 1. 添加环境变量支持

在 `Cargo.toml` 中添加：

```toml
[dependencies]
dotenv = "0.15"
```

### 2. 运行应用

```bash
cargo run
```

您应该看到类似以下的输出：

```
🚀 diesel-gaussdb 快速开始示例
✅ 创建用户成功: User { id: 1, name: "张三", email: "zhangsan@example.com", ... }
🔍 查询用户: User { id: 1, name: "张三", email: "zhangsan@example.com", ... }
📝 更新用户成功: User { id: 1, name: "张三丰", email: "zhangsan@example.com", ... }
📋 所有用户 (1 个):
  - 1: 张三丰 (zhangsan@example.com)
```

## 🎉 恭喜！

您已经成功创建了第一个使用 diesel-gaussdb 的应用程序！

## 📚 下一步

现在您已经掌握了基础知识，可以探索更多高级功能：

- [配置指南](configuration.md) - 了解更多配置选项
- [最佳实践](best-practices.md) - 学习推荐的使用模式
- [API 参考](../api/) - 查看完整的 API 文档
- [示例项目](../examples/) - 查看更多实际示例

## 🆘 需要帮助？

如果遇到问题，请查看：

- [故障排除指南](troubleshooting.md)
- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues)
- [华为云 GaussDB 技术支持论坛](https://bbs.huaweicloud.com/forum/forum-1131-1.html)

---

**祝您使用 diesel-gaussdb 开发愉快！** 🚀
