//! CTE（公共表表达式）支持
//!
//! 这个模块提供了对 PostgreSQL 风格的 WITH 子句和 CTE 的完整支持，
//! 包括递归 CTE 和多个 CTE 的组合使用。

use crate::backend::GaussDB;
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;

/// CTE（公共表表达式）定义
/// 
/// 表示一个 WITH 子句中的单个 CTE 定义
#[derive(Debug, Clone, QueryId)]
pub struct CteDefinition<N, Q> {
    /// CTE 名称
    name: N,
    /// CTE 查询
    query: Q,
    /// 是否为递归 CTE
    recursive: bool,
    /// 列名列表（可选）
    column_names: Option<Vec<String>>,
}

impl<N, Q> CteDefinition<N, Q> {
    /// 创建新的 CTE 定义
    /// 
    /// # 参数
    /// 
    /// * `name` - CTE 名称
    /// * `query` - CTE 查询
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::cte::CteDefinition;
    /// 
    /// // WITH regional_sales AS (SELECT ...)
    /// let cte = CteDefinition::new("regional_sales", my_query);
    /// ```
    pub fn new(name: N, query: Q) -> Self {
        CteDefinition {
            name,
            query,
            recursive: false,
            column_names: None,
        }
    }

    /// 标记为递归 CTE
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::cte::CteDefinition;
    /// 
    /// // WITH RECURSIVE employee_hierarchy AS (...)
    /// let cte = CteDefinition::new("employee_hierarchy", my_query)
    ///     .recursive();
    /// ```
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// 指定列名
    /// 
    /// # 参数
    /// 
    /// * `columns` - 列名列表
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel_gaussdb::query_builder::cte::CteDefinition;
    /// 
    /// // WITH sales(region, total) AS (...)
    /// let cte = CteDefinition::new("sales", my_query)
    ///     .with_columns(vec!["region".to_string(), "total".to_string()]);
    /// ```
    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.column_names = Some(columns);
        self
    }
}

impl<N, Q> QueryFragment<GaussDB> for CteDefinition<N, Q>
where
    N: QueryFragment<GaussDB>,
    Q: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        self.name.walk_ast(pass.reborrow())?;
        
        // 添加列名（如果指定）
        if let Some(ref columns) = self.column_names {
            pass.push_sql("(");
            for (i, column) in columns.iter().enumerate() {
                if i > 0 {
                    pass.push_sql(", ");
                }
                pass.push_sql(column);
            }
            pass.push_sql(")");
        }
        
        pass.push_sql(" AS (");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        
        Ok(())
    }
}

/// WITH 子句构建器
/// 
/// 用于构建包含一个或多个 CTE 的 WITH 子句
#[derive(Debug, Clone, QueryId)]
pub struct WithClause<C> {
    /// CTE 定义列表
    ctes: C,
    /// 是否包含递归 CTE
    has_recursive: bool,
}

impl<C> WithClause<C> {
    /// 创建新的 WITH 子句
    /// 
    /// # 参数
    /// 
    /// * `ctes` - CTE 定义
    pub fn new(ctes: C) -> Self {
        WithClause {
            ctes,
            has_recursive: false,
        }
    }

    /// 标记包含递归 CTE
    pub fn recursive(mut self) -> Self {
        self.has_recursive = true;
        self
    }
}

/// 单个 CTE 的 WITH 子句
pub type SingleWithClause<C> = WithClause<C>;

/// 两个 CTE 的 WITH 子句
pub type DoubleWithClause<C1, C2> = WithClause<(C1, C2)>;

/// 三个 CTE 的 WITH 子句
pub type TripleWithClause<C1, C2, C3> = WithClause<(C1, C2, C3)>;

// 单个 CTE 的实现
impl<C> QueryFragment<GaussDB> for WithClause<C>
where
    C: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.push_sql("WITH ");

        if self.has_recursive {
            pass.push_sql("RECURSIVE ");
        }

        self.ctes.walk_ast(pass.reborrow())?;

        Ok(())
    }
}

/// CTE DSL 扩展 trait
/// 
/// 这个 trait 为查询添加了 `with` 方法支持
pub trait WithDsl<Cte> {
    /// 查询的输出类型
    type Output;

    /// 添加 WITH 子句到查询
    /// 
    /// # 参数
    /// 
    /// * `cte` - CTE 定义
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use diesel::prelude::*;
    /// use diesel_gaussdb::prelude::*;
    /// 
    /// // WITH regional_sales AS (SELECT region, SUM(amount) FROM sales GROUP BY region)
    /// // SELECT * FROM regional_sales WHERE total > 1000
    /// let cte = CteDefinition::new("regional_sales", 
    ///     sales::table
    ///         .select((sales::region, sales::amount.sum()))
    ///         .group_by(sales::region)
    /// );
    /// 
    /// let results = diesel::sql_query("SELECT * FROM regional_sales WHERE total > 1000")
    ///     .with(cte)
    ///     .load::<MyResult>(&mut connection)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn with(self, cte: Cte) -> Self::Output;
}

/// 便捷函数：创建 CTE 定义
/// 
/// # 参数
/// 
/// * `name` - CTE 名称
/// * `query` - CTE 查询
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::cte::*;
/// 
/// let cte = cte("my_cte", my_query);
/// ```
pub fn cte<N, Q>(name: N, query: Q) -> CteDefinition<N, Q> {
    CteDefinition::new(name, query)
}

/// 便捷函数：创建递归 CTE 定义
/// 
/// # 参数
/// 
/// * `name` - CTE 名称
/// * `query` - CTE 查询
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::cte::*;
/// 
/// let recursive_cte = recursive_cte("employee_hierarchy", my_recursive_query);
/// ```
pub fn recursive_cte<N, Q>(name: N, query: Q) -> CteDefinition<N, Q> {
    CteDefinition::new(name, query).recursive()
}

/// 便捷函数：创建 WITH 子句
/// 
/// # 参数
/// 
/// * `ctes` - CTE 定义
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use diesel_gaussdb::query_builder::cte::*;
/// 
/// let with_clause = with(cte("my_cte", my_query));
/// ```
pub fn with<C>(ctes: C) -> WithClause<C> {
    WithClause::new(ctes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cte_definition_creation() {
        // 测试 CTE 定义的创建
        let cte_def = CteDefinition::new("test_cte", "SELECT 1");
        
        assert!(!cte_def.recursive);
        assert!(cte_def.column_names.is_none());
        
        println!("✅ CTE 定义创建测试通过");
    }

    #[test]
    fn test_recursive_cte() {
        // 测试递归 CTE
        let cte_def = CteDefinition::new("recursive_cte", "SELECT 1")
            .recursive();
        
        assert!(cte_def.recursive);
        
        println!("✅ 递归 CTE 测试通过");
    }

    #[test]
    fn test_cte_with_columns() {
        // 测试带列名的 CTE
        let cte_def = CteDefinition::new("named_cte", "SELECT 1, 2")
            .with_columns(vec!["col1".to_string(), "col2".to_string()]);
        
        assert!(cte_def.column_names.is_some());
        assert_eq!(cte_def.column_names.unwrap().len(), 2);
        
        println!("✅ 带列名的 CTE 测试通过");
    }

    #[test]
    fn test_with_clause_creation() {
        // 测试 WITH 子句创建
        let cte_def = CteDefinition::new("test_cte", "SELECT 1");
        let with_clause = WithClause::new(cte_def);
        
        assert!(!with_clause.has_recursive);
        
        println!("✅ WITH 子句创建测试通过");
    }

    #[test]
    fn test_with_clause_recursive() {
        // 测试递归 WITH 子句
        let cte_def = CteDefinition::new("recursive_cte", "SELECT 1");
        let with_clause = WithClause::new(cte_def).recursive();
        
        assert!(with_clause.has_recursive);
        
        println!("✅ 递归 WITH 子句测试通过");
    }

    #[test]
    fn test_convenience_functions() {
        // 测试便捷函数
        let cte_def = cte("test", "SELECT 1");
        let recursive_cte_def = recursive_cte("recursive_test", "SELECT 1");
        let with_clause = with(cte_def);

        assert!(recursive_cte_def.recursive); // 递归 CTE 应该为 true
        assert!(!with_clause.has_recursive); // WITH 子句默认不是递归的

        println!("✅ 便捷函数测试通过");
    }

    #[test]
    fn test_cte_debug() {
        // 测试 Debug 实现
        let cte_def = CteDefinition::new("debug_test", "SELECT 1");
        let debug_str = format!("{:?}", cte_def);
        
        assert!(debug_str.contains("CteDefinition"));
        
        println!("✅ CTE Debug 实现测试通过");
    }
}
