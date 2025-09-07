//! GaussDB Diesel 基础使用示例
//!
//! 这个示例展示了如何使用 diesel-gaussdb 进行基本的数据库操作

use diesel::prelude::*;
use diesel_gaussdb::GaussDBConnection;
use std::env;

// 定义表结构
diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        created_at -> Timestamp,
    }
}

// 定义数据模型
#[derive(Queryable, Debug)]
#[diesel(table_name = users)]
struct User {
    id: i32,
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    name: &'a str,
    email: &'a str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取数据库连接字符串
    let database_url = env::var("GAUSSDB_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string()
        });

    println!("连接到数据库: {}", database_url);

    // 建立数据库连接
    let mut connection = GaussDBConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    println!("✅ 数据库连接成功！");

    // 创建表（如果不存在）
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&mut connection)
    .expect("Failed to create users table");

    println!("✅ 用户表创建成功！");

    // 清理现有数据
    diesel::delete(users::table)
        .execute(&mut connection)
        .expect("Failed to clean existing data");

    // 插入新用户
    let new_users = vec![
        NewUser {
            name: "张三",
            email: "zhangsan@example.com",
        },
        NewUser {
            name: "李四",
            email: "lisi@example.com",
        },
        NewUser {
            name: "王五",
            email: "wangwu@example.com",
        },
    ];

    let inserted_count = diesel::insert_into(users::table)
        .values(&new_users)
        .execute(&mut connection)
        .expect("Error saving new users");

    println!("✅ 成功插入 {} 个用户", inserted_count);

    // 查询所有用户
    let results = users::table
        .load::<User>(&mut connection)
        .expect("Error loading users");

    println!("\n📋 所有用户列表:");
    for user in &results {
        println!("  ID: {}, 姓名: {}, 邮箱: {}, 创建时间: {}", 
                 user.id, user.name, user.email, user.created_at);
    }

    // 查询特定用户
    let zhang_users = users::table
        .filter(users::name.like("%张%"))
        .load::<User>(&mut connection)
        .expect("Error loading zhang users");

    println!("\n🔍 姓名包含'张'的用户:");
    for user in &zhang_users {
        println!("  ID: {}, 姓名: {}, 邮箱: {}", user.id, user.name, user.email);
    }

    // 更新用户信息
    let updated_count = diesel::update(users::table.filter(users::name.eq("张三")))
        .set(users::email.eq("zhangsan_new@example.com"))
        .execute(&mut connection)
        .expect("Error updating user");

    println!("\n✏️  成功更新 {} 个用户的邮箱", updated_count);

    // 查询更新后的用户
    let updated_user = users::table
        .filter(users::name.eq("张三"))
        .first::<User>(&mut connection)
        .expect("Error loading updated user");

    println!("  更新后的用户: {}, 新邮箱: {}", updated_user.name, updated_user.email);

    // 使用事务进行批量操作
    println!("\n🔄 开始事务操作...");
    
    connection.transaction::<_, diesel::result::Error, _>(|conn| {
        // 在事务中插入新用户
        let transaction_user = NewUser {
            name: "赵六",
            email: "zhaoliu@example.com",
        };

        diesel::insert_into(users::table)
            .values(&transaction_user)
            .execute(conn)?;

        // 更新另一个用户
        diesel::update(users::table.filter(users::name.eq("李四")))
            .set(users::email.eq("lisi_updated@example.com"))
            .execute(conn)?;

        println!("  ✅ 事务中的操作完成");
        Ok(())
    }).expect("Transaction failed");

    println!("✅ 事务提交成功！");

    // 查询最终结果
    let final_results = users::table
        .order(users::id.asc())
        .load::<User>(&mut connection)
        .expect("Error loading final results");

    println!("\n📊 最终用户列表 (共 {} 个用户):", final_results.len());
    for user in &final_results {
        println!("  ID: {}, 姓名: {}, 邮箱: {}", user.id, user.name, user.email);
    }

    // 演示复杂查询
    println!("\n🔍 复杂查询示例:");
    
    // 使用原生 SQL 查询
    let user_count: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result::<(i64,)>(&mut connection)
        .expect("Failed to count users")
        .0;

    println!("  用户总数: {}", user_count);

    // 删除一个用户
    let deleted_count = diesel::delete(users::table.filter(users::name.eq("王五")))
        .execute(&mut connection)
        .expect("Error deleting user");

    println!("\n🗑️  成功删除 {} 个用户", deleted_count);

    // 最终统计
    let final_count = users::table
        .count()
        .get_result::<i64>(&mut connection)
        .expect("Error counting final users");

    println!("\n📈 操作完成！最终用户数量: {}", final_count);

    println!("\n🎉 GaussDB Diesel 基础操作演示完成！");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_model() {
        let user = User {
            id: 1,
            name: "测试用户".to_string(),
            email: "test@example.com".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "测试用户");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_new_user() {
        let new_user = NewUser {
            name: "新用户",
            email: "new@example.com",
        };

        assert_eq!(new_user.name, "新用户");
        assert_eq!(new_user.email, "new@example.com");
    }
}
