//! GaussDB数组比较操作实现
//! 
//! 提供ANY和ALL操作符的支持，用于数组比较操作。
//! 这些操作符在PostgreSQL兼容的数据库中用于处理数组和子查询的比较。

use crate::backend::GaussDB;
use diesel::expression::{AsExpression, Expression, TypedExpressionType, ValidGrouping};
use diesel::query_builder::*;
use diesel::result::QueryResult;
use diesel::sql_types::{Array, SqlType};
use diesel::QueryId;

/// 创建GaussDB `ANY` 表达式
/// 
/// ANY操作符用于检查左侧值是否等于右侧数组中的任何一个值。
/// 
/// # 示例
/// 
/// ```rust
/// # use diesel::prelude::*;
/// # use diesel_gaussdb::prelude::*;
/// # use diesel_gaussdb::expression::array_comparison::any;
/// 
/// // 查找名字在指定列表中的用户
/// // users.filter(name.eq(any(vec!["Sean", "Jim"])))
/// ```
pub fn any<ST, T>(vals: T) -> Any<T::Expression>
where
    T: AsArrayExpression<ST>,
{
    Any::new(vals.as_expression())
}

/// 创建GaussDB `ALL` 表达式
/// 
/// ALL操作符用于检查左侧值是否等于右侧数组中的所有值。
/// 
/// # 示例
/// 
/// ```rust
/// # use diesel::prelude::*;
/// # use diesel_gaussdb::prelude::*;
/// # use diesel_gaussdb::expression::array_comparison::all;
/// 
/// // 查找价格大于所有指定值的产品
/// // products.filter(price.gt(all(vec![100, 200, 300])))
/// ```
pub fn all<ST, T>(vals: T) -> All<T::Expression>
where
    T: AsArrayExpression<ST>,
{
    All::new(vals.as_expression())
}

/// ANY表达式结构体
/// 
/// 表示SQL中的 `ANY(array)` 操作
#[derive(Debug, Copy, Clone, QueryId, ValidGrouping)]
pub struct Any<Expr> {
    expr: Expr,
}

impl<Expr> Any<Expr> {
    /// 创建新的ANY表达式
    pub fn new(expr: Expr) -> Self {
        Any { expr }
    }
}

impl<Expr, ST> Expression for Any<Expr>
where
    Expr: Expression<SqlType = Array<ST>>,
    ST: SqlType + TypedExpressionType,
{
    type SqlType = ST;
}

impl<Expr> QueryFragment<GaussDB> for Any<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("ANY(");
        self.expr.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

/// ALL表达式结构体
/// 
/// 表示SQL中的 `ALL(array)` 操作
#[derive(Debug, Copy, Clone, QueryId, ValidGrouping)]
pub struct All<Expr> {
    expr: Expr,
}

impl<Expr> All<Expr> {
    /// 创建新的ALL表达式
    pub fn new(expr: Expr) -> Self {
        All { expr }
    }
}

impl<Expr, ST> Expression for All<Expr>
where
    Expr: Expression<SqlType = Array<ST>>,
    ST: SqlType + TypedExpressionType,
{
    type SqlType = ST;
}

impl<Expr> QueryFragment<GaussDB> for All<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("ALL(");
        self.expr.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

/// 数组表达式转换trait
///
/// 用于将各种类型转换为数组表达式，支持ANY和ALL操作
pub trait AsArrayExpression<ST: 'static> {
    /// 转换后的表达式类型
    type Expression: Expression<SqlType = Array<ST>>;

    /// 将值转换为数组表达式
    fn as_expression(self) -> Self::Expression;
}

impl<ST, T> AsArrayExpression<ST> for T
where
    ST: 'static,
    T: AsExpression<Array<ST>>,
{
    type Expression = <T as AsExpression<Array<ST>>>::Expression;

    fn as_expression(self) -> Self::Expression {
        <T as AsExpression<Array<ST>>>::as_expression(self)
    }
}

// 注意：子查询支持需要访问diesel的私有模块，暂时跳过
// 这些实现在实际使用中可能需要特殊处理

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::GaussDB;
    use crate::query_builder::GaussDBQueryBuilder;
    use diesel::query_builder::QueryBuilder;
    use diesel::sql_types::{Array, Integer, Text};

    /// 测试辅助函数：生成QueryFragment的SQL
    fn generate_sql<T>(fragment: T) -> String
    where
        T: QueryFragment<GaussDB>
    {
        let mut query_builder = GaussDBQueryBuilder::new();
        fragment.to_sql(&mut query_builder, &GaussDB).unwrap();
        query_builder.finish()
    }

    #[test]
    fn test_any_expression_structure() {
        // 测试ANY表达式结构体可以正确创建
        let any_expr = Any::new(diesel::dsl::sql::<Array<Integer>>("ARRAY[1,2,3]"));
        
        // 测试debug格式化
        let debug_str = format!("{:?}", any_expr);
        assert!(debug_str.contains("Any"));
    }

    #[test]
    fn test_all_expression_structure() {
        // 测试ALL表达式结构体可以正确创建
        let all_expr = All::new(diesel::dsl::sql::<Array<Text>>("ARRAY['a','b','c']"));
        
        // 测试debug格式化
        let debug_str = format!("{:?}", all_expr);
        assert!(debug_str.contains("All"));
    }

    #[test]
    fn test_any_sql_generation() {
        // 测试ANY表达式的SQL生成
        let any_expr = Any::new(diesel::dsl::sql::<Array<Integer>>("ARRAY[1,2,3]"));
        let sql = generate_sql(any_expr);
        
        assert_eq!(sql, "ANY(ARRAY[1,2,3])");
    }

    #[test]
    fn test_all_sql_generation() {
        // 测试ALL表达式的SQL生成
        let all_expr = All::new(diesel::dsl::sql::<Array<Text>>("ARRAY['a','b','c']"));
        let sql = generate_sql(all_expr);
        
        assert_eq!(sql, "ALL(ARRAY['a','b','c'])");
    }

    #[test]
    fn test_any_function() {
        // 测试any函数
        let array_expr = diesel::dsl::sql::<Array<Integer>>("ARRAY[1,2,3]");
        let any_expr = any(array_expr);
        let sql = generate_sql(any_expr);
        
        assert_eq!(sql, "ANY(ARRAY[1,2,3])");
    }

    #[test]
    fn test_all_function() {
        // 测试all函数
        let array_expr = diesel::dsl::sql::<Array<Integer>>("ARRAY[1,2,3]");
        let all_expr = all(array_expr);
        let sql = generate_sql(all_expr);
        
        assert_eq!(sql, "ALL(ARRAY[1,2,3])");
    }
}
