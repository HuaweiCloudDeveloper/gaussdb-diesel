# 高级查询示例

这个示例展示了 diesel-gaussdb 的高级查询功能，包括窗口函数、CTE、子查询等高级 SQL 特性。

## 功能特性

### 高级 SQL 功能
- ✅ 窗口函数 (ROW_NUMBER, RANK, DENSE_RANK)
- ✅ CTE (公共表表达式)
- ✅ 复杂子查询 (EXISTS, IN, 标量子查询)
- ✅ 聚合查询和分组统计
- ✅ 多表联接查询

### 查询类型
- **窗口函数**: 数据排序和分析
- **CTE 查询**: 复杂数据处理
- **子查询**: 嵌套查询逻辑
- **聚合统计**: 数据汇总分析

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

### 2. 运行示例

```bash
cd examples/02-advanced-queries
cargo run
```

### 3. 预期输出

```
🚀 启动 Diesel-GaussDB 高级查询示例
✅ 数据库连接成功！
✅ 示例数据设置完成

🪟 === 窗口函数演示 ===
1. ROW_NUMBER - 用户文章编号...
  张三: Rust 编程入门 (第1篇)
  张三: Diesel ORM 指南 (第2篇)
  李四: GaussDB 使用技巧 (第1篇)

2. RANK - 文章评论数排名...
  排名1: 《Rust 编程入门》 - 2 条评论
  排名2: 《GaussDB 使用技巧》 - 1 条评论

🔄 === CTE (公共表表达式) 演示 ===
1. 简单 CTE - 活跃用户统计...
  活跃用户: 张三 - 2 篇文章
  活跃用户: 李四 - 2 篇文章

🔍 === 子查询演示 ===
1. EXISTS 子查询 - 有文章的用户...
  作者: 张三
  作者: 李四

📊 === 聚合查询演示 ===
1. 基础统计信息...
  总用户数: 5
  总文章数: 7
  已发布文章数: 5
```

## 代码结构

```
src/
└── main.rs              # 主程序文件
    ├── establish_connection()    # 数据库连接
    ├── create_tables()          # 表结构创建
    ├── setup_sample_data()      # 示例数据
    ├── demo_window_functions()  # 窗口函数演示
    ├── demo_cte_queries()       # CTE 查询演示
    ├── demo_subqueries()        # 子查询演示
    └── demo_aggregation_queries() # 聚合查询演示
```

## 高级功能详解

### 1. 窗口函数

```rust
// ROW_NUMBER 示例
let results: Vec<UserPostStats> = diesel::sql_query(
    "SELECT u.name as author, p.title, 
     ROW_NUMBER() OVER (PARTITION BY u.name ORDER BY p.created_at) as row_num
     FROM posts p 
     JOIN users u ON p.author_id = u.id 
     WHERE p.published = true
     ORDER BY u.name, row_num"
).load(conn)?;
```

### 2. CTE (公共表表达式)

```rust
// 多个 CTE 示例
let comprehensive_stats: Vec<UserActivity> = diesel::sql_query(
    "WITH user_posts AS (
       SELECT u.id, u.name, COUNT(p.id) as post_count
       FROM users u
       LEFT JOIN posts p ON u.id = p.author_id
       GROUP BY u.id, u.name
     ),
     user_comments AS (
       SELECT u.id, COUNT(c.id) as comment_count
       FROM users u
       LEFT JOIN comments c ON u.id = c.author_id
       GROUP BY u.id
     )
     SELECT up.name, up.post_count, 
            COALESCE(uc.comment_count, 0) as comment_count
     FROM user_posts up
     LEFT JOIN user_comments uc ON up.id = uc.id
     ORDER BY (up.post_count + COALESCE(uc.comment_count, 0)) DESC"
).load(conn)?;
```

### 3. 子查询

```rust
// EXISTS 子查询示例
let authors: Vec<UserActivity> = diesel::sql_query(
    "SELECT u.name, 0 as post_count, 0 as comment_count
     FROM users u
     WHERE EXISTS (
       SELECT 1 FROM posts p WHERE p.author_id = u.id
     )
     ORDER BY u.name"
).load(conn)?;
```

### 4. 聚合查询

```rust
// 复杂聚合统计
let user_post_stats: Vec<UserActivity> = diesel::sql_query(
    "SELECT u.name, COUNT(p.id) as post_count, 0 as comment_count
     FROM users u
     LEFT JOIN posts p ON u.id = p.author_id
     GROUP BY u.id, u.name
     ORDER BY post_count DESC"
).load(conn)?;
```

## 数据模型

### 查询结果结构体

```rust
#[derive(Debug, diesel::QueryableByName)]
struct UserPostStats {
    #[diesel(sql_type = diesel::sql_types::Text)]
    author: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    row_num: i32,
}

#[derive(Debug, diesel::QueryableByName)]
struct UserActivity {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    post_count: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    comment_count: i64,
}
```

## 学习要点

### 1. 窗口函数应用场景
- 数据排序和编号
- 分组内排名
- 累计统计
- 移动平均

### 2. CTE 使用技巧
- 简化复杂查询
- 提高代码可读性
- 递归查询
- 多步骤数据处理

### 3. 子查询优化
- 选择合适的子查询类型
- 避免相关子查询的性能问题
- 使用 EXISTS 替代 IN (某些情况下)
- 考虑改写为 JOIN

### 4. 聚合查询最佳实践
- 合理使用 GROUP BY
- 注意 HAVING 和 WHERE 的区别
- 使用索引优化聚合性能
- 避免在大表上进行全表聚合

## 性能优化建议

### 1. 索引优化

```sql
-- 为常用查询字段创建索引
CREATE INDEX idx_posts_author_published ON posts(author_id, published);
CREATE INDEX idx_posts_created_at ON posts(created_at);
CREATE INDEX idx_comments_post_author ON comments(post_id, author_id);
```

### 2. 查询优化

- 使用 LIMIT 限制结果集大小
- 避免 SELECT * 查询
- 合理使用 WHERE 条件过滤
- 考虑查询执行计划

### 3. 数据库配置

- 调整 work_mem 参数
- 优化 shared_buffers 设置
- 启用查询统计

## 故障排除

### 常见问题

1. **类型不匹配错误**
   ```
   解决方案: 检查 QueryableByName 结构体的 sql_type 注解
   ```

2. **查询超时**
   ```
   解决方案: 添加适当的索引，优化查询条件
   ```

3. **内存不足**
   ```
   解决方案: 使用 LIMIT 分页查询，避免一次性加载大量数据
   ```

---

**🎯 这个高级查询示例展示了 diesel-gaussdb 在复杂数据分析场景中的强大能力！**
