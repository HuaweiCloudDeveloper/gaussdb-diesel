# Getting Started Guide

This guide will help you get started with diesel-gaussdb quickly, from installation to running your first query.

## 📋 Prerequisites

Before you begin, make sure you have:

- **Rust 1.70.0+**: [Install Rust](https://rustup.rs/)
- **GaussDB or OpenGauss**: Database server
- **Git**: For cloning example projects

## 🚀 Step 1: Project Setup

### 1. Create a New Project

```bash
cargo new my-gaussdb-app
cd my-gaussdb-app
```

### 2. Add Dependencies

Edit your `Cargo.toml` file:

```toml
[dependencies]
diesel = { version = "2.2", features = ["postgres", "r2d2", "chrono"] }
diesel-gaussdb = "1.0"
gaussdb = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 3. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

## 🗄️ Step 2: Database Setup

### 1. Start OpenGauss Database

Using Docker:

```bash
docker run --name opengauss \
  -e GS_PASSWORD=Gaussdb@123 \
  -e GS_DB=my_app \
  -e GS_USER=gaussdb \
  -p 5432:5432 \
  -d opengauss/opengauss:7.0.0-RC1.B023
```

### 2. Configure Environment Variables

Create a `.env` file:

```env
DATABASE_URL=host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=my_app
```

### 3. Initialize Diesel

```bash
diesel setup
```

## 📊 Step 3: Create Data Models

### 1. Create Migration

```bash
diesel migration generate create_users
```

### 2. Write Migration SQL

Edit `migrations/*/up.sql`:

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

Edit `migrations/*/down.sql`:

```sql
DROP TABLE users;
```

### 3. Run Migration

```bash
diesel migration run
```

## 🦀 Step 4: Write Rust Code

### 1. Define Models

Create `src/models.rs`:

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

### 2. Create Database Connection

Create `src/database.rs`:

```rust
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_gaussdb::GaussDbConnection;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<GaussDbConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<GaussDbConnection>>;

pub fn establish_connection() -> GaussDbConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    GaussDbConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let manager = ConnectionManager::<GaussDbConnection>::new(database_url);
    
    r2d2::Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create pool")
}
```

### 3. Implement CRUD Operations

Create `src/operations.rs`:

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

### 4. Main Program

Update `src/main.rs`:

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
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Establish database connection
    let mut connection = establish_connection();
    
    println!("🚀 diesel-gaussdb Getting Started Example");
    
    // Create a new user
    let new_user = NewUser {
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
    };
    
    match create_user(&mut connection, &new_user) {
        Ok(user) => {
            println!("✅ User created successfully: {:?}", user);
            
            // Query the user
            match get_user_by_id(&mut connection, user.id) {
                Ok(found_user) => println!("🔍 Found user: {:?}", found_user),
                Err(e) => println!("❌ Failed to find user: {}", e),
            }
            
            // Update the user
            let update_data = models::UpdateUser {
                name: Some("John Smith".to_string()),
                email: None,
                updated_at: Utc::now().naive_utc(),
            };
            
            match update_user(&mut connection, user.id, &update_data) {
                Ok(updated_user) => println!("📝 User updated successfully: {:?}", updated_user),
                Err(e) => println!("❌ Failed to update user: {}", e),
            }
            
            // Query all users
            match get_all_users(&mut connection) {
                Ok(users) => {
                    println!("📋 All users ({} total):", users.len());
                    for user in users {
                        println!("  - {}: {} ({})", user.id, user.name, user.email);
                    }
                },
                Err(e) => println!("❌ Failed to load users: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to create user: {}", e),
    }
}
```

### 5. Add Module Declarations

Update `src/lib.rs`:

```rust
pub mod database;
pub mod models;
pub mod operations;
pub mod schema;

pub use database::*;
pub use models::*;
pub use operations::*;
```

## ▶️ Step 5: Run the Application

### 1. Add Environment Variable Support

Add to `Cargo.toml`:

```toml
[dependencies]
dotenv = "0.15"
```

### 2. Run the Application

```bash
cargo run
```

You should see output similar to:

```
🚀 diesel-gaussdb Getting Started Example
✅ User created successfully: User { id: 1, name: "John Doe", email: "john.doe@example.com", ... }
🔍 Found user: User { id: 1, name: "John Doe", email: "john.doe@example.com", ... }
📝 User updated successfully: User { id: 1, name: "John Smith", email: "john.doe@example.com", ... }
📋 All users (1 total):
  - 1: John Smith (john.doe@example.com)
```

## 🎉 Congratulations!

You've successfully created your first application using diesel-gaussdb!

## 📚 Next Steps

Now that you have the basics down, explore more advanced features:

- [Configuration Guide](configuration.md) - Learn about configuration options
- [Best Practices](best-practices.md) - Learn recommended usage patterns
- [API Reference](../api/) - Browse the complete API documentation
- [Examples](../examples/) - See more real-world examples

## 🆘 Need Help?

If you run into issues, check out:

- [Troubleshooting Guide](troubleshooting.md)
- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues)
- [Huawei Cloud GaussDB Technical Support Forum](https://bbs.huaweicloud.com/forum/forum-1131-1.html)

---

**Happy coding with diesel-gaussdb!** 🚀
