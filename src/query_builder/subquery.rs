//! 子查询支持
//!
//! 这个模块提供了对 PostgreSQL 风格子查询的完整支持，
//! 包括标量子查询、EXISTS 子查询、IN 子查询等。

use crate::backend::GaussDB;
use diesel::expression::Expression;
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;

/// 标量子查询表达式
/// 
/// 表示一个返回单个值的子查询，可以在 SELECT、WHERE 等子句中使用
#[derive(Debug, Clone, QueryId)]
pub struct ScalarSubquery<Q> {
    /// 子查询
    query: Q,
}

impl<Q> ScalarSubquery<Q> {
    /// 创建新的标量子查询
    /// 
    /// # 参数
    /// 
    /// * `query` - 子查询表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::subquery::ScalarSubquery;
    /// 
    /// // SELECT (SELECT COUNT(*) FROM orders WHERE user_id = users.id) as order_count
    /// let subquery = ScalarSubquery::new(
    ///     orders::table
    ///         .filter(orders::user_id.eq(users::id))
    ///         .count()
    /// );
    /// ```
    pub fn new(query: Q) -> Self {
        ScalarSubquery { query }
    }
}

impl<Q> QueryFragment<GaussDB> for ScalarSubquery<Q>
where
    Q: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.push_sql("(");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        Ok(())
    }
}

impl<Q> Expression for ScalarSubquery<Q>
where
    Q: Expression,
{
    type SqlType = Q::SqlType;
}

/// EXISTS 子查询表达式
/// 
/// 表示一个 EXISTS 子查询，用于检查子查询是否返回任何行
#[derive(Debug, Clone, QueryId)]
pub struct ExistsSubquery<Q> {
    /// 子查询
    query: Q,
}

impl<Q> ExistsSubquery<Q> {
    /// 创建新的 EXISTS 子查询
    /// 
    /// # 参数
    /// 
    /// * `query` - 子查询表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::subquery::ExistsSubquery;
    /// 
    /// // WHERE EXISTS (SELECT 1 FROM orders WHERE user_id = users.id)
    /// let exists_query = ExistsSubquery::new(
    ///     orders::table
    ///         .filter(orders::user_id.eq(users::id))
    ///         .select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
    /// );
    /// ```
    pub fn new(query: Q) -> Self {
        ExistsSubquery { query }
    }
}

impl<Q> QueryFragment<GaussDB> for ExistsSubquery<Q>
where
    Q: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.push_sql("EXISTS (");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        Ok(())
    }
}

impl<Q> Expression for ExistsSubquery<Q>
where
    Q: QueryFragment<GaussDB>,
{
    type SqlType = diesel::sql_types::Bool;
}

/// NOT EXISTS 子查询表达式
/// 
/// 表示一个 NOT EXISTS 子查询，用于检查子查询是否不返回任何行
#[derive(Debug, Clone, QueryId)]
pub struct NotExistsSubquery<Q> {
    /// 子查询
    query: Q,
}

impl<Q> NotExistsSubquery<Q> {
    /// 创建新的 NOT EXISTS 子查询
    /// 
    /// # 参数
    /// 
    /// * `query` - 子查询表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::subquery::NotExistsSubquery;
    /// 
    /// // WHERE NOT EXISTS (SELECT 1 FROM orders WHERE user_id = users.id)
    /// let not_exists_query = NotExistsSubquery::new(
    ///     orders::table
    ///         .filter(orders::user_id.eq(users::id))
    ///         .select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
    /// );
    /// ```
    pub fn new(query: Q) -> Self {
        NotExistsSubquery { query }
    }
}

impl<Q> QueryFragment<GaussDB> for NotExistsSubquery<Q>
where
    Q: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.push_sql("NOT EXISTS (");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        Ok(())
    }
}

impl<Q> Expression for NotExistsSubquery<Q>
where
    Q: QueryFragment<GaussDB>,
{
    type SqlType = diesel::sql_types::Bool;
}

/// IN 子查询表达式
/// 
/// 表示一个 IN 子查询，用于检查值是否在子查询结果中
#[derive(Debug, Clone, QueryId)]
pub struct InSubquery<E, Q> {
    /// 左侧表达式
    expr: E,
    /// 子查询
    query: Q,
}

impl<E, Q> InSubquery<E, Q> {
    /// 创建新的 IN 子查询
    /// 
    /// # 参数
    /// 
    /// * `expr` - 左侧表达式
    /// * `query` - 子查询表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::subquery::InSubquery;
    /// 
    /// // WHERE user_id IN (SELECT user_id FROM active_users)
    /// let in_query = InSubquery::new(
    ///     users::id,
    ///     active_users::table.select(active_users::user_id)
    /// );
    /// ```
    pub fn new(expr: E, query: Q) -> Self {
        InSubquery { expr, query }
    }
}

impl<E, Q> QueryFragment<GaussDB> for InSubquery<E, Q>
where
    E: QueryFragment<GaussDB>,
    Q: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        self.expr.walk_ast(pass.reborrow())?;
        pass.push_sql(" IN (");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        Ok(())
    }
}

impl<E, Q> Expression for InSubquery<E, Q>
where
    E: Expression,
    Q: QueryFragment<GaussDB>,
{
    type SqlType = diesel::sql_types::Bool;
}

/// NOT IN 子查询表达式
/// 
/// 表示一个 NOT IN 子查询，用于检查值是否不在子查询结果中
#[derive(Debug, Clone, QueryId)]
pub struct NotInSubquery<E, Q> {
    /// 左侧表达式
    expr: E,
    /// 子查询
    query: Q,
}

impl<E, Q> NotInSubquery<E, Q> {
    /// 创建新的 NOT IN 子查询
    /// 
    /// # 参数
    /// 
    /// * `expr` - 左侧表达式
    /// * `query` - 子查询表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::subquery::NotInSubquery;
    /// 
    /// // WHERE user_id NOT IN (SELECT user_id FROM banned_users)
    /// let not_in_query = NotInSubquery::new(
    ///     users::id,
    ///     banned_users::table.select(banned_users::user_id)
    /// );
    /// ```
    pub fn new(expr: E, query: Q) -> Self {
        NotInSubquery { expr, query }
    }
}

impl<E, Q> QueryFragment<GaussDB> for NotInSubquery<E, Q>
where
    E: QueryFragment<GaussDB>,
    Q: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        self.expr.walk_ast(pass.reborrow())?;
        pass.push_sql(" NOT IN (");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        Ok(())
    }
}

impl<E, Q> Expression for NotInSubquery<E, Q>
where
    E: Expression,
    Q: QueryFragment<GaussDB>,
{
    type SqlType = diesel::sql_types::Bool;
}

/// 子查询 DSL 扩展 trait
/// 
/// 这个 trait 为表达式添加了子查询方法支持
pub trait SubqueryDsl<Q> {
    /// 创建 EXISTS 子查询
    fn exists(query: Q) -> ExistsSubquery<Q>;
    
    /// 创建 NOT EXISTS 子查询
    fn not_exists(query: Q) -> NotExistsSubquery<Q>;
}

impl<Q> SubqueryDsl<Q> for Q {
    fn exists(query: Q) -> ExistsSubquery<Q> {
        ExistsSubquery::new(query)
    }
    
    fn not_exists(query: Q) -> NotExistsSubquery<Q> {
        NotExistsSubquery::new(query)
    }
}

/// 便捷函数：创建标量子查询
/// 
/// # 参数
/// 
/// * `query` - 子查询表达式
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::subquery::*;
/// 
/// let scalar = scalar_subquery(my_query);
/// ```
pub fn scalar_subquery<Q>(query: Q) -> ScalarSubquery<Q> {
    ScalarSubquery::new(query)
}

/// 便捷函数：创建 EXISTS 子查询
/// 
/// # 参数
/// 
/// * `query` - 子查询表达式
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::subquery::*;
/// 
/// let exists_query = exists(my_query);
/// ```
pub fn exists<Q>(query: Q) -> ExistsSubquery<Q> {
    ExistsSubquery::new(query)
}

/// 便捷函数：创建 NOT EXISTS 子查询
/// 
/// # 参数
/// 
/// * `query` - 子查询表达式
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::subquery::*;
/// 
/// let not_exists_query = not_exists(my_query);
/// ```
pub fn not_exists<Q>(query: Q) -> NotExistsSubquery<Q> {
    NotExistsSubquery::new(query)
}

/// 便捷函数：创建 IN 子查询
/// 
/// # 参数
/// 
/// * `expr` - 左侧表达式
/// * `query` - 子查询表达式
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::subquery::*;
/// 
/// let in_query = in_subquery(my_expr, my_query);
/// ```
pub fn in_subquery<E, Q>(expr: E, query: Q) -> InSubquery<E, Q> {
    InSubquery::new(expr, query)
}

/// 便捷函数：创建 NOT IN 子查询
/// 
/// # 参数
/// 
/// * `expr` - 左侧表达式
/// * `query` - 子查询表达式
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::subquery::*;
/// 
/// let not_in_query = not_in_subquery(my_expr, my_query);
/// ```
pub fn not_in_subquery<E, Q>(expr: E, query: Q) -> NotInSubquery<E, Q> {
    NotInSubquery::new(expr, query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_subquery_creation() {
        // 测试标量子查询的创建
        let subquery = ScalarSubquery::new("SELECT COUNT(*) FROM users");
        
        let debug_str = format!("{:?}", subquery);
        assert!(debug_str.contains("ScalarSubquery"));
        
        // Test passed
    }

    #[test]
    fn test_exists_subquery_creation() {
        // 测试 EXISTS 子查询的创建
        let exists_query = ExistsSubquery::new("SELECT 1 FROM orders WHERE user_id = 1");
        
        let debug_str = format!("{:?}", exists_query);
        assert!(debug_str.contains("ExistsSubquery"));
        
        // Test passed
    }

    #[test]
    fn test_not_exists_subquery_creation() {
        // 测试 NOT EXISTS 子查询的创建
        let not_exists_query = NotExistsSubquery::new("SELECT 1 FROM banned_users WHERE user_id = 1");
        
        let debug_str = format!("{:?}", not_exists_query);
        assert!(debug_str.contains("NotExistsSubquery"));
        
        // Test passed
    }

    #[test]
    fn test_in_subquery_creation() {
        // 测试 IN 子查询的创建
        let in_query = InSubquery::new("user_id", "SELECT user_id FROM active_users");
        
        let debug_str = format!("{:?}", in_query);
        assert!(debug_str.contains("InSubquery"));
        
        // Test passed
    }

    #[test]
    fn test_not_in_subquery_creation() {
        // 测试 NOT IN 子查询的创建
        let not_in_query = NotInSubquery::new("user_id", "SELECT user_id FROM banned_users");
        
        let debug_str = format!("{:?}", not_in_query);
        assert!(debug_str.contains("NotInSubquery"));
        
        // Test passed
    }

    #[test]
    fn test_convenience_functions() {
        // 测试便捷函数
        let scalar = scalar_subquery("SELECT 1");
        let exists_query = exists("SELECT 1");
        let not_exists_query = not_exists("SELECT 1");
        let in_query = in_subquery("id", "SELECT id FROM table");
        let not_in_query = not_in_subquery("id", "SELECT id FROM table");
        
        // 验证所有函数都能正确创建对象
        assert!(format!("{:?}", scalar).contains("ScalarSubquery"));
        assert!(format!("{:?}", exists_query).contains("ExistsSubquery"));
        assert!(format!("{:?}", not_exists_query).contains("NotExistsSubquery"));
        assert!(format!("{:?}", in_query).contains("InSubquery"));
        assert!(format!("{:?}", not_in_query).contains("NotInSubquery"));
        
        // Test passed
    }

    #[test]
    fn test_subquery_dsl() {
        // 测试 SubqueryDsl trait
        let query = "SELECT 1";
        
        let exists_query = <&str as SubqueryDsl<&str>>::exists(query);
        let not_exists_query = <&str as SubqueryDsl<&str>>::not_exists(query);
        
        assert!(format!("{:?}", exists_query).contains("ExistsSubquery"));
        assert!(format!("{:?}", not_exists_query).contains("NotExistsSubquery"));
        
        // Test passed
    }
}
