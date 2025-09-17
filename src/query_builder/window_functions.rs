//! 窗口函数支持
//!
//! 这个模块提供了对 PostgreSQL 风格窗口函数的完整支持，
//! 包括 OVER 子句、PARTITION BY、ORDER BY 等功能。

use crate::backend::GaussDB;
use diesel::expression::Expression;
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;

/// 窗口函数表达式
/// 
/// 表示一个完整的窗口函数调用，包括函数本身和 OVER 子句
#[derive(Debug, Clone, QueryId)]
pub struct WindowFunction<F, W> {
    /// 窗口函数
    function: F,
    /// OVER 子句
    over_clause: W,
}

impl<F, W> WindowFunction<F, W> {
    /// 创建新的窗口函数表达式
    /// 
    /// # 参数
    /// 
    /// * `function` - 窗口函数（如 ROW_NUMBER(), RANK() 等）
    /// * `over_clause` - OVER 子句定义
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::window_functions::*;
    /// 
    /// // ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC)
    /// let window_fn = WindowFunction::new(
    ///     row_number(),
    ///     over().partition_by(users::department).order_by(users::salary.desc())
    /// );
    /// ```
    pub fn new(function: F, over_clause: W) -> Self {
        WindowFunction {
            function,
            over_clause,
        }
    }
}

impl<F, W> QueryFragment<GaussDB> for WindowFunction<F, W>
where
    F: QueryFragment<GaussDB>,
    W: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        self.function.walk_ast(pass.reborrow())?;
        pass.push_sql(" OVER ");
        self.over_clause.walk_ast(pass.reborrow())?;
        Ok(())
    }
}

impl<F, W> Expression for WindowFunction<F, W>
where
    F: Expression,
    W: QueryFragment<GaussDB>,
{
    type SqlType = F::SqlType;
}

/// OVER 子句构建器
/// 
/// 用于构建窗口函数的 OVER 子句，支持 PARTITION BY 和 ORDER BY
#[derive(Debug, Clone, QueryId)]
pub struct OverClause<P, O> {
    /// PARTITION BY 表达式
    partition_by: Option<P>,
    /// ORDER BY 表达式
    order_by: Option<O>,
}

impl OverClause<(), ()> {
    /// 创建空的 OVER 子句
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::window_functions::*;
    /// 
    /// // OVER ()
    /// let over = OverClause::new();
    /// ```
    pub fn new() -> Self {
        OverClause {
            partition_by: None,
            order_by: None,
        }
    }
}

impl<P, O> OverClause<P, O> {
    /// 添加 PARTITION BY 子句
    /// 
    /// # 参数
    /// 
    /// * `expr` - 分区表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::window_functions::*;
    /// 
    /// // OVER (PARTITION BY department)
    /// let over = OverClause::new().partition_by(users::department);
    /// ```
    pub fn partition_by<E>(self, expr: E) -> OverClause<E, O> {
        OverClause {
            partition_by: Some(expr),
            order_by: self.order_by,
        }
    }

    /// 添加 ORDER BY 子句
    /// 
    /// # 参数
    /// 
    /// * `expr` - 排序表达式
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::window_functions::*;
    /// 
    /// // OVER (ORDER BY salary DESC)
    /// let over = OverClause::new().order_by(users::salary.desc());
    /// ```
    pub fn order_by<E>(self, expr: E) -> OverClause<P, E> {
        OverClause {
            partition_by: self.partition_by,
            order_by: Some(expr),
        }
    }
}

impl<P, O> QueryFragment<GaussDB> for OverClause<P, O>
where
    P: QueryFragment<GaussDB>,
    O: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.push_sql("(");
        
        if let Some(ref partition) = self.partition_by {
            pass.push_sql("PARTITION BY ");
            partition.walk_ast(pass.reborrow())?;
            
            if self.order_by.is_some() {
                pass.push_sql(" ");
            }
        }
        
        if let Some(ref order) = self.order_by {
            pass.push_sql("ORDER BY ");
            order.walk_ast(pass.reborrow())?;
        }
        
        pass.push_sql(")");
        Ok(())
    }
}

/// 常用窗口函数定义
pub mod functions {
    use super::*;
    use diesel::sql_types::BigInt;

    /// ROW_NUMBER() 窗口函数
    /// 
    /// 为结果集中的每一行分配一个唯一的序号
    #[derive(Debug, Clone, Copy, QueryId)]
    pub struct RowNumber;

    impl QueryFragment<GaussDB> for RowNumber {
        fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            pass.push_sql("ROW_NUMBER()");
            Ok(())
        }
    }

    impl Expression for RowNumber {
        type SqlType = BigInt;
    }

    /// 创建 ROW_NUMBER() 函数
    pub fn row_number() -> RowNumber {
        RowNumber
    }

    /// RANK() 窗口函数
    /// 
    /// 为结果集中的每一行分配一个排名，相同值的行具有相同排名
    #[derive(Debug, Clone, Copy, QueryId)]
    pub struct Rank;

    impl QueryFragment<GaussDB> for Rank {
        fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            pass.push_sql("RANK()");
            Ok(())
        }
    }

    impl Expression for Rank {
        type SqlType = BigInt;
    }

    /// 创建 RANK() 函数
    pub fn rank() -> Rank {
        Rank
    }

    /// DENSE_RANK() 窗口函数
    /// 
    /// 类似 RANK()，但排名是连续的
    #[derive(Debug, Clone, Copy, QueryId)]
    pub struct DenseRank;

    impl QueryFragment<GaussDB> for DenseRank {
        fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            pass.push_sql("DENSE_RANK()");
            Ok(())
        }
    }

    impl Expression for DenseRank {
        type SqlType = BigInt;
    }

    /// 创建 DENSE_RANK() 函数
    pub fn dense_rank() -> DenseRank {
        DenseRank
    }

    /// COUNT() 窗口函数
    /// 
    /// 计算窗口内的行数
    #[derive(Debug, Clone, QueryId)]
    pub struct WindowCount<E> {
        expr: E,
    }

    impl<E> WindowCount<E> {
        /// 创建新的 COUNT 窗口函数
        pub fn new(expr: E) -> Self {
            WindowCount { expr }
        }
    }

    impl<E> QueryFragment<GaussDB> for WindowCount<E>
    where
        E: QueryFragment<GaussDB>,
    {
        fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            pass.push_sql("COUNT(");
            self.expr.walk_ast(pass.reborrow())?;
            pass.push_sql(")");
            Ok(())
        }
    }

    impl<E> Expression for WindowCount<E>
    where
        E: Expression,
    {
        type SqlType = BigInt;
    }

    /// 创建 COUNT() 窗口函数
    pub fn count<E>(expr: E) -> WindowCount<E> {
        WindowCount::new(expr)
    }
}

/// 便捷函数：创建空的 OVER 子句
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::window_functions::*;
/// 
/// // ROW_NUMBER() OVER ()
/// let window_fn = WindowFunction::new(functions::row_number(), over());
/// ```
pub fn over() -> OverClause<(), ()> {
    OverClause::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::functions::*;

    #[test]
    fn test_window_function_creation() {
        // 测试窗口函数的创建
        let window_fn = WindowFunction::new(row_number(), over());
        
        // 验证结构体可以正确创建
        let debug_str = format!("{:?}", window_fn);
        assert!(debug_str.contains("WindowFunction"));
        
        // 窗口函数创建测试通过
    }

    #[test]
    fn test_over_clause_creation() {
        // 测试 OVER 子句的创建
        let over_clause = over();
        
        let debug_str = format!("{:?}", over_clause);
        assert!(debug_str.contains("OverClause"));
        
        // OVER 子句创建测试通过
    }

    #[test]
    fn test_window_functions() {
        // 测试各种窗口函数
        let row_num = row_number();
        let rank_fn = rank();
        let dense_rank_fn = dense_rank();

        // 验证函数可以正确创建
        assert!(format!("{:?}", row_num).contains("RowNumber"));
        assert!(format!("{:?}", rank_fn).contains("Rank"));
        assert!(format!("{:?}", dense_rank_fn).contains("DenseRank"));

        // 窗口函数类型测试通过
    }

    #[test]
    fn test_over_clause_builder() {
        // 测试 OVER 子句构建器
        let over_with_partition = over().partition_by("department");
        let over_with_order = over().order_by("salary");
        let over_with_both = over()
            .partition_by("department")
            .order_by("salary");
        
        // 验证构建器模式工作正常
        assert!(format!("{:?}", over_with_partition).contains("partition_by"));
        assert!(format!("{:?}", over_with_order).contains("order_by"));
        assert!(format!("{:?}", over_with_both).contains("partition_by"));
        assert!(format!("{:?}", over_with_both).contains("order_by"));
        
        // OVER 子句构建器测试通过
    }

    #[test]
    fn test_window_count() {
        // 测试 COUNT 窗口函数
        let count_fn = count("*");

        let debug_str = format!("{:?}", count_fn);
        assert!(debug_str.contains("WindowCount"));

        // COUNT 窗口函数测试通过
    }
}
