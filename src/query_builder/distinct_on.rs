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



/// Helper trait for ordering with DISTINCT ON
///
/// When using DISTINCT ON, PostgreSQL requires that the ORDER BY clause
/// starts with the same expressions used in DISTINCT ON.
pub trait OrderDecorator<T> {
    /// Apply ordering that's compatible with DISTINCT ON
    fn then_order_by(self, expr: T) -> Self;
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
}
