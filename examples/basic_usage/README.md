# GaussDB Diesel 基础使用示例

这个示例展示了如何使用 diesel-gaussdb 进行基本的数据库操作。

## 功能演示

- 数据库连接
- 表创建
- 数据插入 (单个和批量)
- 数据查询 (全部和条件查询)
- 数据更新
- 数据删除
- 事务处理
- 复杂查询

## 运行示例

### 1. 准备数据库

确保您有一个运行中的 GaussDB 或 OpenGauss 实例：

```bash
# 使用 Docker 运行 OpenGauss
docker run --name opengauss \
  -e GS_PASSWORD=Gaussdb@123 \
  -p 5432:5432 \
  -d opengauss/opengauss:7.0.0-RC1.B023
```

### 2. 设置环境变量

```bash
export GAUSSDB_URL="host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres"
```

### 3. 运行示例

```bash
# 从项目根目录运行
cargo run --example basic_usage --features gaussdb

# 或者进入示例目录运行
cd examples/basic_usage
cargo run
```

## 预期输出

```
连接到数据库: host=localhost port=5432 user=gaussdb password=Gaussdb@123 dbname=postgres
✅ 数据库连接成功！
✅ 用户表创建成功！
✅ 成功插入 3 个用户

📋 所有用户列表:
  ID: 1, 姓名: 张三, 邮箱: zhangsan@example.com, 创建时间: 2023-12-01 10:00:00
  ID: 2, 姓名: 李四, 邮箱: lisi@example.com, 创建时间: 2023-12-01 10:00:01
  ID: 3, 姓名: 王五, 邮箱: wangwu@example.com, 创建时间: 2023-12-01 10:00:02

🔍 姓名包含'张'的用户:
  ID: 1, 姓名: 张三, 邮箱: zhangsan@example.com

✏️  成功更新 1 个用户的邮箱
  更新后的用户: 张三, 新邮箱: zhangsan_new@example.com

🔄 开始事务操作...
  ✅ 事务中的操作完成
✅ 事务提交成功！

📊 最终用户列表 (共 4 个用户):
  ID: 1, 姓名: 张三, 邮箱: zhangsan_new@example.com
  ID: 2, 姓名: 李四, 邮箱: lisi_updated@example.com
  ID: 3, 姓名: 王五, 邮箱: wangwu@example.com
  ID: 4, 姓名: 赵六, 邮箱: zhaoliu@example.com

🔍 复杂查询示例:
  用户总数: 4

🗑️  成功删除 1 个用户

📈 操作完成！最终用户数量: 3

🎉 GaussDB Diesel 基础操作演示完成！
```

## 代码说明

### 数据模型定义

```rust
// 表结构定义
diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        created_at -> Timestamp,
    }
}

// 查询模型
#[derive(Queryable, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

// 插入模型
#[derive(Insertable)]
struct NewUser<'a> {
    name: &'a str,
    email: &'a str,
}
```

### 基本操作

1. **连接数据库**
   ```rust
   let mut connection = GaussDBConnection::establish(&database_url)?;
   ```

2. **插入数据**
   ```rust
   diesel::insert_into(users::table)
       .values(&new_users)
       .execute(&mut connection)?;
   ```

3. **查询数据**
   ```rust
   let results = users::table
       .filter(users::name.like("%张%"))
       .load::<User>(&mut connection)?;
   ```

4. **更新数据**
   ```rust
   diesel::update(users::table.filter(users::name.eq("张三")))
       .set(users::email.eq("new_email@example.com"))
       .execute(&mut connection)?;
   ```

5. **删除数据**
   ```rust
   diesel::delete(users::table.filter(users::name.eq("王五")))
       .execute(&mut connection)?;
   ```

6. **事务处理**
   ```rust
   connection.transaction::<_, diesel::result::Error, _>(|conn| {
       // 事务中的操作
       Ok(())
   })?;
   ```

## 故障排除

### 连接问题

如果遇到连接问题，请检查：

1. 数据库是否正在运行
2. 连接字符串是否正确
3. 用户权限是否足够
4. 防火墙设置

### 编译问题

确保启用了正确的功能：

```toml
[dependencies]
diesel-gaussdb = { version = "0.1.0", features = ["gaussdb"] }
```

### 运行时错误

常见错误及解决方案：

- **表不存在**: 示例会自动创建表
- **权限不足**: 确保数据库用户有创建表的权限
- **类型不匹配**: 检查数据模型定义是否与数据库表结构匹配

## 下一步

- 查看 [高级功能示例](../advanced_features/)
- 了解 [Web 应用集成](../web_application/)
- 阅读 [完整文档](../../README.md)
