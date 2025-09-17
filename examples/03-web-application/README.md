# Web 应用示例

这个示例展示了如何在 Web 应用中使用 diesel-gaussdb 构建 REST API。

## 功能特性

- ✅ REST API 设计
- ✅ 用户 CRUD 操作
- ✅ JSON 序列化/反序列化
- ✅ 错误处理
- ✅ 数据库连接管理
- ✅ 统计查询

## 技术栈

- **Web 框架**: Axum
- **数据库**: GaussDB/OpenGauss
- **ORM**: Diesel-GaussDB
- **序列化**: Serde
- **异步运行时**: Tokio

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
cd examples/03-web-application
cargo run
```

### 3. 测试 API

服务器启动后，访问：http://localhost:8080

## API 接口

### 健康检查

```bash
GET /health
```

响应：
```json
{
  "success": true,
  "data": "服务运行正常",
  "message": "操作成功"
}
```

### 用户管理

#### 获取所有用户

```bash
GET /api/users
```

响应：
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "张三",
      "email": "zhangsan@example.com",
      "age": 25
    }
  ],
  "message": "操作成功"
}
```

#### 获取单个用户

```bash
GET /api/users/{id}
```

#### 创建用户

```bash
POST /api/users
Content-Type: application/json

{
  "name": "李四",
  "email": "lisi@example.com",
  "age": 30
}
```

#### 更新用户

```bash
POST /api/users/{id}
Content-Type: application/json

{
  "name": "李四（更新）",
  "email": "lisi_updated@example.com",
  "age": 31
}
```

#### 删除用户

```bash
DELETE /api/users/{id}
```

### 统计信息

```bash
GET /api/stats
```

响应：
```json
{
  "success": true,
  "data": {
    "total_users": 5,
    "average_age": 28.5
  },
  "message": "操作成功"
}
```

## 测试示例

### 使用 curl 测试

```bash
# 健康检查
curl http://localhost:8080/health

# 创建用户
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"测试用户","email":"test@example.com","age":25}'

# 获取所有用户
curl http://localhost:8080/api/users

# 获取特定用户
curl http://localhost:8080/api/users/1

# 更新用户
curl -X POST http://localhost:8080/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name":"更新用户","email":"updated@example.com","age":26}'

# 删除用户
curl -X DELETE http://localhost:8080/api/users/1

# 获取统计信息
curl http://localhost:8080/api/stats
```

### 使用 HTTPie 测试

```bash
# 安装 HTTPie
pip install httpie

# 创建用户
http POST localhost:8080/api/users name="测试用户" email="test@example.com" age:=25

# 获取用户
http GET localhost:8080/api/users

# 更新用户
http POST localhost:8080/api/users/1 name="更新用户" email="updated@example.com" age:=26

# 删除用户
http DELETE localhost:8080/api/users/1
```

## 代码结构

```
src/
├── main.rs              # 主应用文件
│   ├── User             # 用户数据结构
│   ├── NewUser          # 新用户数据结构
│   ├── ApiResponse      # API 响应结构
│   ├── 数据库连接管理    # establish_connection()
│   ├── API 处理器       # get_users, create_user 等
│   └── 路由配置         # create_routes()
```

## 特色功能

### 1. 类型安全的 API

```rust
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
```

### 2. 统一的响应格式

```rust
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}
```

### 3. 错误处理

```rust
async fn get_user(Path(user_id): Path<i32>) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let mut conn = establish_connection()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 查询逻辑...
    
    match users.into_iter().next() {
        Some(user) => Ok(Json(ApiResponse::success(user))),
        None => Err(StatusCode::NOT_FOUND),
    }
}
```

### 4. 参数绑定

```rust
let result = diesel::sql_query(
    "INSERT INTO users (name, email, age) VALUES ($1, $2, $3)"
)
.bind::<diesel::sql_types::Text, _>(&new_user.name)
.bind::<diesel::sql_types::Text, _>(&new_user.email)
.bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(new_user.age)
.execute(&mut conn);
```

## 扩展建议

### 1. 添加认证

```rust
// 添加 JWT 认证中间件
use axum_extra::headers::{Authorization, Bearer};

async fn protected_handler(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // 验证 JWT token
    // ...
}
```

### 2. 添加连接池

```rust
use diesel::r2d2::{ConnectionManager, Pool};

type DbPool = Pool<ConnectionManager<GaussDBConnection>>;

// 在应用状态中共享连接池
#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
}
```

### 3. 添加中间件

```rust
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/api/users", get(get_users))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());
```

## 性能优化

1. **连接池**: 使用 r2d2 连接池管理数据库连接
2. **异步处理**: 充分利用 Tokio 异步运行时
3. **错误处理**: 优雅的错误处理和响应
4. **日志记录**: 完整的请求日志记录

---

**🎯 这个 Web 应用示例展示了 diesel-gaussdb 在现代 Web 开发中的实际应用！**
