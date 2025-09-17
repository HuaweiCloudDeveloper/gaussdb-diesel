//! RETURNING clause implementation for GaussDB
//!
//! This module provides support for PostgreSQL-style RETURNING clauses,
//! which are also supported by GaussDB. RETURNING clauses allow INSERT, UPDATE,
//! and DELETE statements to return values from the affected rows.

use crate::backend::{GaussDB, GaussDBReturningClause};
use diesel::query_builder::{QueryFragment, AstPass};
use diesel::result::QueryResult;

/// Represents a RETURNING clause in an INSERT, UPDATE, or DELETE statement
#[derive(Debug, Clone)]
pub struct ReturningClause<T> {
    returning: T,
}

impl<T> ReturningClause<T> {
    /// Create a new RETURNING clause with the given expression
    pub fn new(returning: T) -> Self {
        Self { returning }
    }
}

impl<T> QueryFragment<GaussDB> for ReturningClause<T>
where
    T: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql(" RETURNING ");
        self.returning.walk_ast(out.reborrow())?;
        Ok(())
    }
}

// 为GaussDBReturningClause实现QueryFragment
impl<T> QueryFragment<GaussDB, GaussDBReturningClause> for ReturningClause<T>
where
    T: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql(" RETURNING ");
        self.returning.walk_ast(out.reborrow())?;
        Ok(())
    }
}

/// A trait for adding RETURNING support to query builders
pub trait ReturningDsl<Expr> {
    /// The type returned by `.returning()`
    type Output;

    /// Add a RETURNING clause to the query
    fn returning(self, expr: Expr) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_builder::GaussDBQueryBuilder;
    use diesel::query_builder::QueryBuilder;

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
    fn test_returning_clause_creation() {
        let clause = ReturningClause::new("id");
        // 测试结构创建是否正确
        assert_eq!(clause.returning, "id");
    }

    #[test]
    fn test_returning_clause_sql_generation() {
        // 使用diesel::dsl::sql来创建有效的SQL片段
        use diesel::dsl::sql;
        use diesel::sql_types::Text;

        let clause = ReturningClause::new(sql::<Text>("id"));
        let sql_result = generate_sql(clause);
        assert_eq!(sql_result, " RETURNING id");
    }

    #[test]
    fn test_returning_multiple_columns() {
        use diesel::dsl::sql;
        use diesel::sql_types::Text;

        let clause = ReturningClause::new(sql::<Text>("id, name, created_at"));
        let sql_result = generate_sql(clause);
        assert_eq!(sql_result, " RETURNING id, name, created_at");
    }

    #[test]
    fn test_returning_star() {
        use diesel::dsl::sql;
        use diesel::sql_types::Text;

        let clause = ReturningClause::new(sql::<Text>("*"));
        let sql_result = generate_sql(clause);
        assert_eq!(sql_result, " RETURNING *");
    }
}
