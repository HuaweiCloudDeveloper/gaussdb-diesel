//! DISTINCT ON clause implementation for GaussDB
//!
//! This module provides support for PostgreSQL-style DISTINCT ON clauses,
//! which are also supported by GaussDB.

use crate::backend::GaussDB;
use diesel::query_builder::{QueryFragment, AstPass};
use diesel::result::QueryResult;

/// Represents a DISTINCT ON clause in a SELECT statement
///
/// This is a PostgreSQL/GaussDB specific feature that allows you to specify
/// which columns should be used for determining uniqueness.
///
/// # Example
///
/// ```sql
/// SELECT DISTINCT ON (user_id) user_id, created_at, message
/// FROM messages
/// ORDER BY user_id, created_at DESC;
/// ```
#[derive(Debug, Clone)]
pub struct DistinctOnClause<T> {
    expr: T,
}

impl<T> DistinctOnClause<T> {
    /// Create a new DISTINCT ON clause with the given expression
    pub fn new(expr: T) -> Self {
        Self { expr }
    }
}

impl<T> QueryFragment<GaussDB> for DistinctOnClause<T>
where
    T: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("DISTINCT ON (");
        self.expr.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}



/// 多个表达式的 DISTINCT ON 支持
///
/// 这个结构体支持在 DISTINCT ON 子句中使用多个表达式
#[derive(Debug, Clone)]
pub struct MultiDistinctOnClause<T> {
    exprs: T,
}

impl<T> MultiDistinctOnClause<T> {
    /// 创建新的多表达式 DISTINCT ON 子句
    pub fn new(exprs: T) -> Self {
        Self { exprs }
    }
}

impl<T> QueryFragment<GaussDB> for MultiDistinctOnClause<(T,)>
where
    T: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("DISTINCT ON (");
        self.exprs.0.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<T, U> QueryFragment<GaussDB> for MultiDistinctOnClause<(T, U)>
where
    T: QueryFragment<GaussDB>,
    U: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("DISTINCT ON (");
        self.exprs.0.walk_ast(out.reborrow())?;
        out.push_sql(", ");
        self.exprs.1.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<T, U, V> QueryFragment<GaussDB> for MultiDistinctOnClause<(T, U, V)>
where
    T: QueryFragment<GaussDB>,
    U: QueryFragment<GaussDB>,
    V: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("DISTINCT ON (");
        self.exprs.0.walk_ast(out.reborrow())?;
        out.push_sql(", ");
        self.exprs.1.walk_ast(out.reborrow())?;
        out.push_sql(", ");
        self.exprs.2.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

/// Helper trait for ordering with DISTINCT ON
///
/// When using DISTINCT ON, PostgreSQL requires that the ORDER BY clause
/// starts with the same expressions used in DISTINCT ON.
pub trait OrderDecorator<T> {
    /// Apply ordering that's compatible with DISTINCT ON
    fn then_order_by(self, expr: T) -> Self;
}

/// DISTINCT ON DSL 扩展 trait
///
/// 这个 trait 为查询添加了 `distinct_on` 方法支持
pub trait DistinctOnDsl<Expr> {
    /// 查询的输出类型
    type Output;

    /// 添加 DISTINCT ON 子句到查询
    ///
    /// # 参数
    ///
    /// * `expr` - 用于去重的表达式
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use diesel::prelude::*;
    /// use diesel_gaussdb::prelude::*;
    ///
    /// // SELECT DISTINCT ON (user_id) * FROM posts ORDER BY user_id, created_at DESC
    /// let results = posts::table
    ///     .distinct_on(posts::user_id)
    ///     .order((posts::user_id, posts::created_at.desc()))
    ///     .load::<Post>(&mut connection)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn distinct_on(self, expr: Expr) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distinct_on_clause() {
        // Test that the clause can be created and has the expected structure
        let clause = DistinctOnClause::new("test_column");
        assert_eq!(clause.expr, "test_column");
    }

    #[test]
    fn test_distinct_on_creation() {
        let clause = DistinctOnClause::new("test_expr");
        assert_eq!(clause.expr, "test_expr");
    }

    #[test]
    fn test_distinct_on_structure() {
        // 测试 DISTINCT ON 结构体的基本功能
        let clause = DistinctOnClause::new("user_id");
        assert_eq!(clause.expr, "user_id");

        println!("✅ DISTINCT ON 结构体测试通过");
    }

    #[test]
    fn test_multi_distinct_on_structure() {
        // 测试多表达式 DISTINCT ON 结构体
        let clause = MultiDistinctOnClause::new(("user_id", "category"));

        // 验证结构体可以正确创建
        let debug_str = format!("{:?}", clause);
        assert!(debug_str.contains("MultiDistinctOnClause"));

        println!("✅ 多表达式 DISTINCT ON 结构体测试通过");
    }

    #[test]
    fn test_distinct_on_clone() {
        // 测试 Clone 实现
        let clause1 = DistinctOnClause::new("test_column");
        let clause2 = clause1.clone();

        assert_eq!(clause1.expr, clause2.expr);

        println!("✅ DISTINCT ON Clone 实现测试通过");
    }

    #[test]
    fn test_multi_distinct_on_clone() {
        // 测试多表达式 Clone 实现
        let clause1 = MultiDistinctOnClause::new((1, 2, 3));
        let clause2 = clause1.clone();

        // 验证克隆成功
        let debug1 = format!("{:?}", clause1);
        let debug2 = format!("{:?}", clause2);
        assert_eq!(debug1, debug2);

        println!("✅ 多表达式 DISTINCT ON Clone 实现测试通过");
    }

    #[test]
    fn test_multi_distinct_on_debug() {
        // 测试多表达式 Debug 实现
        let clause = MultiDistinctOnClause::new((1, 2, 3));
        let debug_str = format!("{:?}", clause);
        assert!(debug_str.contains("MultiDistinctOnClause"));

        println!("✅ 多表达式 DISTINCT ON Debug 实现测试通过");
    }
}
