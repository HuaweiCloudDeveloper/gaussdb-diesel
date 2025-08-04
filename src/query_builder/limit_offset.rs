//! LIMIT and OFFSET clause implementation for GaussDB
//!
//! This module provides support for PostgreSQL-style LIMIT and OFFSET clauses,
//! which are also supported by GaussDB.

use crate::backend::GaussDB;
use diesel::query_builder::{QueryFragment, AstPass};
use diesel::result::QueryResult;

/// Represents a LIMIT clause in a SELECT statement
#[derive(Debug, Clone)]
pub struct LimitClause<T> {
    limit: T,
}

impl<T> LimitClause<T> {
    /// Create a new LIMIT clause with the given limit value
    pub fn new(limit: T) -> Self {
        Self { limit }
    }
}

impl<T> QueryFragment<GaussDB> for LimitClause<T>
where
    T: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql(" LIMIT ");
        self.limit.walk_ast(out.reborrow())?;
        Ok(())
    }
}

/// Represents an OFFSET clause in a SELECT statement
#[derive(Debug, Clone)]
pub struct OffsetClause<T> {
    offset: T,
}

impl<T> OffsetClause<T> {
    /// Create a new OFFSET clause with the given offset value
    pub fn new(offset: T) -> Self {
        Self { offset }
    }
}

impl<T> QueryFragment<GaussDB> for OffsetClause<T>
where
    T: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql(" OFFSET ");
        self.offset.walk_ast(out.reborrow())?;
        Ok(())
    }
}

/// Combined LIMIT and OFFSET clause
#[derive(Debug, Clone)]
pub struct LimitOffsetClause<L, O> {
    limit: Option<L>,
    offset: Option<O>,
}

impl<L, O> LimitOffsetClause<L, O> {
    /// Create a new LIMIT OFFSET clause
    pub fn new(limit: Option<L>, offset: Option<O>) -> Self {
        Self { limit, offset }
    }

    /// Create a clause with only LIMIT
    pub fn limit_only(limit: L) -> Self {
        Self {
            limit: Some(limit),
            offset: None,
        }
    }

    /// Create a clause with only OFFSET
    pub fn offset_only(offset: O) -> Self {
        Self {
            limit: None,
            offset: Some(offset),
        }
    }

    /// Create a clause with both LIMIT and OFFSET
    pub fn limit_and_offset(limit: L, offset: O) -> Self {
        Self {
            limit: Some(limit),
            offset: Some(offset),
        }
    }
}

impl<L, O> QueryFragment<GaussDB> for LimitOffsetClause<L, O>
where
    L: QueryFragment<GaussDB>,
    O: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        if let Some(ref limit) = self.limit {
            out.push_sql(" LIMIT ");
            limit.walk_ast(out.reborrow())?;
        }
        
        if let Some(ref offset) = self.offset {
            out.push_sql(" OFFSET ");
            offset.walk_ast(out.reborrow())?;
        }
        
        Ok(())
    }
}

/// A trait for adding LIMIT support to query builders
pub trait LimitDsl<Expr> {
    /// The type returned by `.limit()`
    type Output;

    /// Add a LIMIT clause to the query
    fn limit(self, limit: Expr) -> Self::Output;
}

/// A trait for adding OFFSET support to query builders
pub trait OffsetDsl<Expr> {
    /// The type returned by `.offset()`
    type Output;

    /// Add an OFFSET clause to the query
    fn offset(self, offset: Expr) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_clause() {
        let clause = LimitClause::new(10i64);
        assert_eq!(clause.limit, 10i64);
    }

    #[test]
    fn test_offset_clause() {
        let clause = OffsetClause::new(20i64);
        assert_eq!(clause.offset, 20i64);
    }

    #[test]
    fn test_limit_offset_clause() {
        let clause = LimitOffsetClause::limit_and_offset(10i64, 20i64);
        assert!(clause.limit.is_some());
        assert!(clause.offset.is_some());
        assert_eq!(clause.limit.unwrap(), 10i64);
        assert_eq!(clause.offset.unwrap(), 20i64);
    }

    #[test]
    fn test_limit_only_clause() {
        let clause = LimitOffsetClause::<_, ()>::limit_only(10i64);
        assert!(clause.limit.is_some());
        assert!(clause.offset.is_none());
        assert_eq!(clause.limit.unwrap(), 10i64);
    }

    #[test]
    fn test_offset_only_clause() {
        let clause = LimitOffsetClause::<(), _>::offset_only(20i64);
        assert!(clause.limit.is_none());
        assert!(clause.offset.is_some());
        assert_eq!(clause.offset.unwrap(), 20i64);
    }
}
