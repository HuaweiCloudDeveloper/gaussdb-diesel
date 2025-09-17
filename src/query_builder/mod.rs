//! Query builder implementation for GaussDB
//!
//! This module provides the query builder that constructs SQL queries
//! compatible with GaussDB's PostgreSQL-like syntax.

use diesel::query_builder::QueryBuilder;
use diesel::result::QueryResult;
use crate::backend::GaussDB;

pub mod distinct_on;
pub mod limit_offset;
pub mod limit_offset_impl;
pub mod on_constraint;
pub mod copy;
pub mod window_functions;
pub mod cte;
pub mod subquery;
pub mod query_fragment_impls;
pub mod returning;

pub use self::distinct_on::DistinctOnClause;
pub use self::limit_offset::LimitOffsetClause;
pub use self::on_constraint::{OnConstraint, ConflictTarget, on_constraint};
pub use self::copy::{CopyFormat, CopyTarget, CopyOperation};
pub use self::window_functions::{WindowFunction, OverClause, over};
pub use self::cte::{CteDefinition, WithClause, cte, recursive_cte, with};
pub use self::subquery::{
    ScalarSubquery, ExistsSubquery, NotExistsSubquery, InSubquery, NotInSubquery,
    scalar_subquery, exists, not_exists, in_subquery, not_in_subquery
};

/// The GaussDB query builder
///
/// This query builder generates PostgreSQL-compatible SQL that works with GaussDB.
/// It uses PostgreSQL-style parameter binding ($1, $2, etc.) and identifier quoting.
#[derive(Default, Debug)]
pub struct GaussDBQueryBuilder {
    sql: String,
    bind_idx: u32,
}

impl GaussDBQueryBuilder {
    /// Constructs a new query builder with an empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// Get cxzAX从v不那么， the current SQL string
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Get the current bind parameter index
    pub fn bind_idx(&self) -> u32 {
        self.bind_idx
    }


}

impl QueryBuilder<GaussDB> for GaussDBQueryBuilder {
    fn push_sql(&mut self, sql: &str) {
        self.sql.push_str(sql);
    }

    fn push_identifier(&mut self, identifier: &str) -> QueryResult<()> {
        self.push_sql("\"");
        // Escape any existing quotes by doubling them
        self.push_sql(&identifier.replace('"', "\"\""));
        self.push_sql("\"");
        Ok(())
    }

    fn push_bind_param(&mut self) {
        self.push_bind_param_value_only();
        self.sql.push('$');
        let mut buffer = itoa::Buffer::new();
        self.sql.push_str(buffer.format(self.bind_idx));
    }

    fn push_bind_param_value_only(&mut self) {
        self.bind_idx += 1;
    }

    fn finish(self) -> String {
        self.sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_query_builder_creation() {
        let builder = GaussDBQueryBuilder::new();
        assert_eq!(builder.sql(), "");
        assert_eq!(builder.bind_idx(), 0);
    }

    #[test]
    fn test_push_sql() {
        let mut builder = GaussDBQueryBuilder::new();
        builder.push_sql("SELECT * FROM users");
        assert_eq!(builder.sql(), "SELECT * FROM users");
    }

    #[test]
    fn test_push_identifier() {
        let mut builder = GaussDBQueryBuilder::new();
        builder.push_identifier("table_name").unwrap();
        assert_eq!(builder.sql(), "\"table_name\"");
    }

    #[test]
    fn test_push_identifier_with_quotes() {
        let mut builder = GaussDBQueryBuilder::new();
        builder.push_identifier("table\"name").unwrap();
        assert_eq!(builder.sql(), "\"table\"\"name\"");
    }

    #[test]
    fn test_push_bind_param() {
        let mut builder = GaussDBQueryBuilder::new();
        builder.push_bind_param();
        builder.push_bind_param();
        assert_eq!(builder.sql(), "$1$2");
        assert_eq!(builder.bind_idx(), 2);
    }

    #[test]
    fn test_complex_query() {
        let mut builder = GaussDBQueryBuilder::new();
        builder.push_sql("SELECT ");
        builder.push_identifier("name").unwrap();
        builder.push_sql(" FROM ");
        builder.push_identifier("users").unwrap();
        builder.push_sql(" WHERE ");
        builder.push_identifier("id").unwrap();
        builder.push_sql(" = ");
        builder.push_bind_param();
        
        assert_eq!(builder.sql(), "SELECT \"name\" FROM \"users\" WHERE \"id\" = $1");
        assert_eq!(builder.bind_idx(), 1);
    }

    #[test]
    fn test_finish() {
        let mut builder = GaussDBQueryBuilder::new();
        builder.push_sql("SELECT 1");
        let sql = builder.finish();
        assert_eq!(sql, "SELECT 1");
    }
}
